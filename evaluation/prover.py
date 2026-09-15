"""
Proving agent execution.

``Prover`` owns one workspace: it copies the base mirror into
``worker_dir/ws-<i>/``, starts a lean-lsp-mcp server, proves test cases by
spawning the vibe agent (streaming tool activity and turn progress), then tears
down the server and its temporary copy.
"""

import json
import os
import re
import shutil
import socket
import subprocess
import time
from pathlib import Path
from threading import Thread

from evaluation.config import AgentConfig, McpConfig
from evaluation.logger import get_logger
from evaluation.workspace import create_workspace


class Prover:
    """Prove test cases inside one workspace.

    Owns the workspace copy, its lean-lsp-mcp server, and the vibe agent
    subprocess. Use as a context manager so the server and temporary copy are torn down on exit.
    """

    _STOP_EVENT_RE = re.compile(r"<vibe_stop_event>(.*?)</vibe_stop_event>", re.DOTALL)
    _SESSION_PREFIX = "session"

    def __init__(
        self,
        base_dir: Path,
        worker_dir: Path,
        index: int,
        agent: AgentConfig,
        mcp: McpConfig,
        output_dir: Path,
        progress=None,
    ):
        self.base_dir = base_dir
        self.ws = worker_dir / f"ws-{index}"
        self.port = mcp.start_port + index
        self.agent = agent
        self.mcp = mcp
        self.output_dir = output_dir
        self.progress = progress  # callable(text) -> reports turn progress
        self.mcp_proc: subprocess.Popen | None = None
        self.logger = get_logger(self)

    def __enter__(self) -> "Prover":
        create_workspace(str(self.ws), base_dir=self.base_dir, mcp_port=self.port)
        self.mcp_proc = self._start_mcp()
        return self

    def __exit__(self, *exc_info) -> bool:
        self.mcp_proc.terminate()
        try:
            self.mcp_proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.mcp_proc.kill()
        shutil.rmtree(self.ws, ignore_errors=True)
        return False

    def prove(self, case) -> tuple[str, str | None, str]:
        """Install, build, and prove ``case``; return ``(name, spec, status)``."""
        try:
            case.install(self.ws)
            if self._build(case.name):
                spec = case.extract(
                    self.ws, self.output_dir, self._run_agent(case.name)
                )
                return case.name, spec, "ok"
            return case.name, None, "build_failed"
        except Exception as exc:  # a worker must never die silently
            return case.name, None, f"error: {exc}"
        finally:
            case.reset(self.ws)

    def _build(self, name: str) -> bool:
        """Run ``lake build`` to confirm the installed case compiles."""
        result = subprocess.run(
            ["lake", "build"], cwd=self.ws, capture_output=True, text=True
        )
        if result.returncode == 0:
            return True
        tail = "\n".join((result.stderr or result.stdout).splitlines()[-6:])
        self.logger.warning(f"[{name}] baseline build failed:\n{tail}")
        return False

    def _start_mcp(self) -> subprocess.Popen:
        """Launch a lean-lsp-mcp HTTP server and wait until it binds ``self.port``."""
        cfg = self.mcp
        proc = subprocess.Popen(
            cfg.command
            + [
                "--transport",
                cfg.transport,
                "--host",
                cfg.host,
                "--port",
                str(self.port),
                "--lean-project-path",
                str(self.ws),
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        deadline = time.time() + cfg.ready_timeout
        while time.time() < deadline and proc.poll() is None:
            try:
                with socket.create_connection((cfg.host, self.port), timeout=1.0):
                    return proc
            except OSError:
                time.sleep(1.0)
        self.logger.warning(f"lean-lsp-mcp on port {self.port} not ready")
        return proc

    def _run_agent(self, name: str) -> Path | None:
        """Spawn the vibe agent, stream its log + turn progress, and return the
        newest vibe session (or None)."""
        max_turns = self._max_turns(self.agent.args)
        before = {p.name for p in self._session_dirs()}

        header = f"[{name}] running agent in {self.ws.name}"
        if max_turns is not None:
            header += f" (max {max_turns} turns)"
        self.logger.info(header)

        cmd = (
            [self.agent.command, "-p", self.agent.prompt]
            + self.agent.args
            + ["--output", self.agent.output]
        )
        if self.agent.output != "streaming":
            try:
                subprocess.run(cmd, cwd=self.ws, timeout=self.agent.timeout)
            except subprocess.TimeoutExpired:
                self.logger.warning(f"[{name}] agent timed out after {self.agent.timeout}s")
            return self._newest_session(before)

        # ``--output streaming`` emits one JSON line per history entry: an "effect"
        # is a completed tool call, an assistant "message" is model text. Vibe does
        # not surface its turn counter, so turns are read from the session log.
        proc = subprocess.Popen(
            cmd,
            cwd=self.ws,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            env={**os.environ, "PYTHONUNBUFFERED": "1"},
        )

        last_answer = ""
        stop_reported = False
        session = None
        turn_count = 0

        def report_stop(text: str) -> None:
            nonlocal stop_reported
            if stop_reported:
                return
            stop_reported = True
            m = self._STOP_EVENT_RE.search(text)
            self.logger.info(
                f"[{name}] agent stopped: {m.group(1).strip() if m else text.strip()}"
            )

        def relay(line: str) -> None:
            nonlocal last_answer
            line = line.rstrip("\n")
            if not line:
                return
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                if "vibe_stop_event" in line:
                    report_stop(line)
                else:
                    self.logger.info(line)  # e.g. trust warning -> pass through
                return
            kind = entry.get("type")
            if kind == "effect":
                detail = entry.get("detail") or {}
                tool = entry.get("title") or detail.get("toolName") or "tool"
                status = (entry.get("state") or {}).get("status")
                mark = (
                    "✓"
                    if status == "completed"
                    else "✗" if status in ("failed", "cancelled") else "•"
                )
                self.logger.debug(f"[{name}] {tool} {mark}")
            elif kind == "message" and entry.get("role") == "assistant":
                for block in entry.get("content") or []:
                    if not isinstance(block, dict) or block.get("type") != "text":
                        continue
                    text = block.get("text", "").strip()
                    if not text:
                        continue
                    if "vibe_stop_event" in text:
                        report_stop(text)
                    else:
                        last_answer = text

        def poll() -> None:
            nonlocal session, turn_count
            if session is None:
                session = self._newest_session(before)
            if session is not None:
                n = self._turns(session / "messages.jsonl")
                if n != turn_count:
                    turn_count = n
                    total = f"/{max_turns}" if max_turns else ""
                    line = f"{name} {self._bar(n, max_turns)} {n}{total}"
                    if self.progress is not None:
                        self.progress(line)
                    else:
                        self.logger.info(line)

        def reader() -> None:
            for line in proc.stdout:
                relay(line)
                poll()
            poll()  # catch the final turn

        rt = Thread(target=reader, daemon=True)
        rt.start()
        try:
            proc.wait(timeout=self.agent.timeout)
        except subprocess.TimeoutExpired:
            proc.kill()
            self.logger.warning(f"[{name}] agent timed out after {self.agent.timeout}s")
        rt.join(timeout=5)

        if last_answer:
            self.logger.info(f"[{name}] final answer:\n{last_answer}")

        return self._newest_session(before)

    # --- session-log helpers ---

    def _session_dirs(self) -> list[Path]:
        sessions = self.ws / ".vibe" / "sessions"
        if not sessions.is_dir():
            return []
        return [
            p
            for p in sessions.iterdir()
            if p.is_dir() and p.name.startswith(self._SESSION_PREFIX)
        ]

    def _newest_session(self, before: set[str]) -> Path | None:
        new = [p for p in self._session_dirs() if p.name not in before]
        return max(new, key=lambda p: p.stat().st_mtime) if new else None

    @staticmethod
    def _turns(messages_file: Path) -> int:
        """Number of assistant entries in a session's messages.jsonl (one per turn)."""
        try:
            n = 0
            with messages_file.open() as f:
                for line in f:
                    try:
                        if json.loads(line).get("role") == "assistant":
                            n += 1
                    except json.JSONDecodeError:
                        pass  # trailing partial line still being written
            return n
        except OSError:
            return 0

    @staticmethod
    def _bar(done: int, total: int, width: int = 20) -> str:
        if total <= 0:
            return f"{done}"
        filled = min(width, round(done / total * width))
        return "█" * filled + "░" * (width - filled)

    @staticmethod
    def _max_turns(args: list[str]) -> int | None:
        if "--max-turns" not in args:
            return None
        try:
            return int(args[args.index("--max-turns") + 1])
        except (ValueError, IndexError):
            return None
