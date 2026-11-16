"""
Proving agent execution.

``Prover`` owns one workspace: it copies the base mirror into
``worker_dir/ws-<i>/``, starts a lean-lsp-mcp server, proves test cases by
spawning the configured agent (vibe, claude code, codex, or dsh — streaming
tool activity and turn progress), then tears down the server and its temporary
copy.
"""

import os
import random
import re
import shutil
import signal
import socket
import subprocess
import time
from pathlib import Path
from threading import Thread

from evaluation.agents import make_agent
from evaluation.config import AgentConfig, McpConfig
from evaluation.isolation import SandboxConfig, sandbox_command
from evaluation.logger import get_logger
from evaluation.workspace import create_workspace


class Prover:
    """Prove test cases inside one workspace.

    Owns the workspace copy, its lean-lsp-mcp server, and the proving agent
    subprocess. Use as a context manager so the server and temporary copy are torn down on exit.
    """

    _STOP_EVENT_RE = re.compile(r"<vibe_stop_event>(.*?)</vibe_stop_event>", re.DOTALL)

    def __init__(
        self,
        base_dir: Path,
        worker_dir: Path,
        index: int,
        agent: AgentConfig,
        mcp: McpConfig,
        output_dir: Path,
        progress=None,
        input_dir: Path | None = None,
        sandbox: SandboxConfig | None = None,
    ):
        self.base_dir = base_dir
        self.ws = worker_dir / f"ws-{index}"
        self.port = mcp.start_port + index
        self.agent = agent
        self.mcp = mcp
        self.output_dir = output_dir
        self.input_dir = input_dir
        self.sandbox = sandbox or SandboxConfig()
        self.progress = progress  # callable(text) -> reports turn progress
        self.mcp_proc: subprocess.Popen | None = None
        self._cases_started = 0
        self.logger = get_logger(self)

    def __enter__(self) -> "Prover":
        self._agent = make_agent(self.agent, self.mcp, self.port)
        try:
            self._progress("copying base mirror...")
            create_workspace(str(self.ws), base_dir=self.base_dir, mcp_port=self.port)
            if self.sandbox.enabled:
                # Trace discovery runs in the parent, so point it at the same
                # private home that the subprocess will use.
                self._agent.session_home = self.ws.resolve() / ".agent-home"
            self._progress("starting mcp server...")
            self.mcp_proc = self._start_mcp()
            self._progress("ready")
        except Exception:
            self._stop_mcp()
            shutil.rmtree(self.ws, ignore_errors=True)
            raise
        return self

    def __exit__(self, *exc_info) -> bool:
        self._stop_mcp()
        shutil.rmtree(self.ws, ignore_errors=True)
        return False

    def _stop_mcp(self) -> None:
        if self.mcp_proc is None:
            return
        if self.mcp_proc.poll() is None:
            self.mcp_proc.terminate()
        try:
            self.mcp_proc.wait(timeout=10)
        except subprocess.TimeoutExpired:
            self.mcp_proc.kill()
            self.mcp_proc.wait()
        self.mcp_proc = None

    def prove(self, case) -> tuple[str, str | None, str]:
        """Install, build, and prove ``case``; return ``(name, spec, status)``."""
        try:
            if self._cases_started and self.sandbox.enabled and self.sandbox.fresh_workspace:
                self._stop_mcp()
                shutil.rmtree(self.ws)
                create_workspace(str(self.ws), base_dir=self.base_dir, mcp_port=self.port)
                self.mcp_proc = self._start_mcp()
            self._cases_started += 1
            # Agents can create nested Git repositories or ignored files which
            # `git clean -fd` does not remove. A private home is discarded even
            # when the faster, reusable-workspace mode is selected.
            home = self.ws / ".agent-home"
            if home.is_symlink():
                home.unlink()
            elif home.exists():
                shutil.rmtree(home)
            case.install(self.ws)
            if self._build(case.name):
                spec = case.extract(
                    self.ws, self.output_dir, self._run_agent(case.name)
                )
                return case.name, spec, "ok"
            return case.name, None, "build_failed"
        except Exception as exc:  # a worker must never die silently
            session, self._failed_session = getattr(self, "_failed_session", None), None
            try:
                case.archive_trace(self.ws, self.output_dir, session)
            except Exception as archive_exc:
                self.logger.warning(f"[{case.name}] could not archive trace: {archive_exc}")
            return case.name, None, f"error: {exc}"
        finally:
            if not (self.sandbox.enabled and self.sandbox.fresh_workspace):
                case.reset(self.ws)

    def _build(self, name: str) -> bool:
        """Run ``lake build`` to confirm the installed case compiles."""
        self._progress(f"{name} building...")
        result = subprocess.run(
            ["lake", "build"], cwd=self.ws, capture_output=True, text=True
        )
        if result.returncode == 0:
            self._progress(f"{name} build ok")
            return True
        tail = "\n".join((result.stderr or result.stdout).splitlines()[-6:])
        self.logger.warning(f"[{name}] baseline build failed:\n{tail}")
        self._progress(f"{name} build failed")
        return False

    def _progress(self, text: str) -> None:
        """Route a status line to the progress board (or the log when standalone)."""
        if self.progress is not None:
            self.progress(text)
        else:
            self.logger.info(text)

    def _start_mcp(self) -> subprocess.Popen:
        """Launch a lean-lsp-mcp HTTP server and wait until it binds ``self.port``."""
        cfg = self.mcp
        cmd = cfg.command + [
                "--transport",
                cfg.transport,
                "--host",
                cfg.host,
                "--port",
                str(self.port),
                "--lean-project-path",
                str(self.ws),
            ]
        cmd, sandbox_env = sandbox_command(
            cmd, self.ws, self.sandbox,
            [p for p in (self.input_dir, self.output_dir, self.base_dir) if p is not None],
            home_name=".mcp-home",
        )
        proc = subprocess.Popen(
            cmd,
            cwd=self.ws,
            env={**os.environ, **sandbox_env},
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
        code = proc.poll()
        if code is None:
            proc.terminate()
            proc.wait(timeout=10)
        raise RuntimeError(
            f"lean-lsp-mcp on port {self.port} did not become ready "
            f"(exit code {code}); check [prover.sandbox].ro_paths and MCP dependencies"
        )

    def _run_agent(self, name: str) -> Path | None:
        """Spawn the proving agent, resume it on abnormal exit, and return the newest session trace (or None)."""
        max_turns = self.agent.max_turns
        before = {p.name for p in self._agent.sessions(self.ws)}

        header = f"[{name}] running {self._agent.kind} agent in {self.ws.name}"
        if max_turns is not None:
            header += f" (max {max_turns} turns)"
        self.logger.info(header)

        self._agent.prepare(self.ws)

        session_id = None
        attempt = 0
        failed = False
        while True:
            cmd = (
                self._agent.argv()
                if attempt == 0
                else self._agent.resume_argv(session_id)
            )
            failed = self._run_once(name, cmd)
            if not failed or attempt >= self.agent.retries:
                break
            attempt += 1
            newest = self._newest(self._agent.sessions(self.ws), before)
            session_id = self._agent.session_id(newest) if newest else None
            if session_id is None:
                self.logger.warning(
                    f"[{name}] agent died but no session id found; giving up"
                )
                break
            delay = self._retry_delay(attempt)
            self.logger.warning(
                f"[{name}] agent died; resuming session {session_id[:8]}… "
                f"in {delay:.1f}s (attempt {attempt}/{self.agent.retries})"
            )
            time.sleep(delay)

        if failed:
            # Keep the session of a failed run so the caller can still archive it.
            self._failed_session = self._newest(self._agent.sessions(self.ws), before)
            raise RuntimeError(f"{self._agent.kind} agent failed after {attempt + 1} attempt(s)")
        return self._newest(self._agent.sessions(self.ws), before)

    def _run_once(self, name: str, cmd: list[str]) -> bool:
        """Run one agent invocation; return True when it exited abnormally
        (a backend failure worth resuming), False otherwise."""
        cmd, sandbox_env = sandbox_command(
            cmd,
            self.ws,
            self.sandbox,
            [p for p in (self.input_dir, self.output_dir, self.base_dir) if p is not None],
        )
        launch_env = {**os.environ, **self._agent.env(), **sandbox_env}
        if not self._agent.is_streaming():
            try:
                result = subprocess.run(
                    cmd,
                    cwd=self.ws,
                    timeout=self.agent.timeout,
                    env=launch_env,
                    stdin=subprocess.DEVNULL,
                )
            except subprocess.TimeoutExpired:
                self.logger.warning(
                    f"[{name}] agent timed out after {self.agent.timeout}s"
                )
                return False
            if result.returncode != 0:
                self.logger.warning(
                    f"[{name}] agent failed: {self._exit_reason(result.returncode)}"
                )
                return True
            return False

        # Streaming output emits one JSON line per history entry; the agent
        # backend maps each line to a normalized event (a completed tool call
        # is the main unit of work). The session trace is written in batches,
        # so reading it lags behind the live stream.
        proc = subprocess.Popen(
            cmd,
            cwd=self.ws,
            stdin=subprocess.DEVNULL,  # agents run non-interactively: don't wait on stdin
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            encoding="utf-8",
            errors="replace",
            env={**launch_env, "PYTHONUNBUFFERED": "1"},
        )

        last_answer = ""
        stop_reason = ""
        stop_reported = False
        capped = False
        steps = 0
        last_tool = ""
        started = time.monotonic()

        def progress_line() -> str:
            line = (
                f"{name} · {steps} tools · "
                f"{self._fmt_elapsed(time.monotonic() - started)}"
            )
            if last_tool:
                line += f" · {last_tool}"
            return line

        def report_progress() -> None:
            self._progress(progress_line())

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
            nonlocal last_answer, stop_reason, steps, last_tool, capped
            event = self._agent.parse_stream_line(line.rstrip("\n"))
            if event is None:
                return
            if event.pass_through:
                self.logger.info(event.pass_through)
            if event.step:
                steps += 1
                last_tool = event.tool
                if event.tool:
                    self.logger.debug(f"[{name}] {event.tool}")
                cap = self.agent.max_turns
                if (
                    not capped
                    and self._agent.external_max_turns
                    and cap is not None
                    and steps > cap
                ):
                    # The pipeline enforces ``max_turns`` as a tool-call cap:
                    # kill the process once it exceeds the allowed tool calls.
                    capped = True
                    proc.kill()
                    self.logger.warning(
                        f"[{name}] stopped: exceeded {cap} tool calls (max_turns)"
                    )
            if event.answer:
                last_answer = event.answer
            if event.stop:
                stop_reason = event.stop
                report_stop(event.stop)
            report_progress()

        def reader() -> None:
            for line in proc.stdout:
                relay(line)
            report_progress()  # final state

        def heartbeat() -> None:
            while proc.poll() is None:
                report_progress()
                time.sleep(1.0)

        rt = Thread(target=reader, daemon=True)
        ht = Thread(target=heartbeat, daemon=True)
        rt.start()
        ht.start()
        timed_out = False
        try:
            proc.wait(timeout=self.agent.timeout)
        except subprocess.TimeoutExpired:
            timed_out = True
            proc.kill()
            self.logger.warning(f"[{name}] agent timed out after {self.agent.timeout}s")
        rt.join(timeout=5)

        # A non-zero exit that we didn't cause ourselves means the agent died on
        # its own — e.g. a backend failure or a crash — so surface it instead of
        # silently treating it as "ok". A controlled limit stop (e.g. claude's
        # ``error_max_turns``) is not a failure: it exhausted its budget cleanly,
        # so do not resume it.
        code = proc.returncode
        limit_stop = self._agent.is_limit_stop(stop_reason)
        abnormal = code not in (0, None) and not timed_out and not capped and not limit_stop
        if abnormal:
            self.logger.warning(f"[{name}] agent failed: {self._exit_reason(code)}")

        if last_answer:
            self.logger.info(f"[{name}] final answer:\n{last_answer}")

        return abnormal

    # --- session-log helpers ---

    def _retry_delay(self, attempt: int) -> float:
        """Exponential backoff with jitter for resume-retries."""
        base = self.agent.retry_backoff
        if base <= 0:
            return 0.0
        return base * (2 ** (attempt - 1)) * (0.5 + random.random())

    @staticmethod
    def _exit_reason(code: int | None) -> str:
        """Human-readable description of an agent subprocess exit status."""
        if code is None:
            return "still running"
        if code == 0:
            return "exited cleanly"
        if code < 0:
            try:
                return f"killed by {signal.Signals(-code).name}"
            except ValueError:
                return f"killed by signal {-code}"
        return f"exit code {code}"

    @staticmethod
    def _newest(sessions: list[Path], before: set[str]) -> Path | None:
        """The newest trace (file or dir) created after ``before`` was sampled."""
        new = [p for p in sessions if p.name not in before]
        return max(new, key=lambda p: p.stat().st_mtime) if new else None

    @staticmethod
    def _fmt_elapsed(seconds: float) -> str:
        seconds = int(seconds)
        if seconds < 60:
            return f"{seconds}s"
        m, s = divmod(seconds, 60)
        if m < 60:
            return f"{m}m{s:02d}s"
        h, m = divmod(m, 60)
        return f"{h}h{m:02d}m"
