"""
Agent evaluation pipeline.

A producer-consumer pipeline: the producer scans the benchmark and feeds ``(name, function, specification)`` tuples into a queue; each worker is bound to an independent workspace, runs the proving agent inside it, then reports the resulting ``Spec.lean`` back.

Usage::

    python -m evaluation.prover --config evaluation/local/config.toml
"""

import argparse
import re
import shutil
import subprocess
import time
from multiprocessing import Process, Queue
from pathlib import Path
from queue import Empty

from evaluation.config import DEFAULT_CONFIG, AgentConfig, Config
from evaluation.mcp_server import MCPServer
from evaluation.workspace import LIB, clear_workspace, create_workspace, setup_testcase

# Sentinel marking the end of the input queue.
_STOP = None

# Test-case directories that are not part of the benchmark.
_EXCLUDED = {"failure", "lean_mode"}

# Lean "cheating" tactics; a proved specification must be free of them.
_SORRY_RE = re.compile(r"\b(sorry|admit)\b")

# Sleep interval between non-blocking queue polls, in seconds.
_POLL_INTERVAL = 0.1

# vibe session directories are named "<prefix>_<timestamp>_<hash>"; this prefix
# also lets us skip non-session dirs (e.g. the "active" lock dir) in the same folder.
_SESSION_PREFIX = "session"


def scan_testcases(input_dir: Path) -> list[tuple[str, str, str]]:
    """Return (name, func_src, spec_src) for every test case under"""
    testcases: list[tuple[str, str, str]] = []
    for d in sorted(input_dir.iterdir()):
        if not d.is_dir() or d.name in _EXCLUDED:
            continue
        spec = d / "Spec.lean"
        funcs = [
            f for f in d.glob("*.lean") if f.stem not in {"Spec", "Test_test", "Basic"}
        ]
        assert len(funcs) == 1
        if spec.exists() and len(funcs) == 1:
            testcases.append((d.name, funcs[0].read_text(), spec.read_text()))
    return testcases


def _has_sorry(spec: str) -> bool:
    return _SORRY_RE.search(spec) is not None


def _build(workspace: Path, name: str) -> bool:
    """Run ``lake build`` to confirm the installed test case compiles."""
    result = subprocess.run(
        ["lake", "build"], cwd=workspace, capture_output=True, text=True
    )
    if result.returncode == 0:
        return True
    tail = "\n".join((result.stderr or result.stdout).splitlines()[-6:])
    print(f"[{name}] baseline build failed:\n{tail}", flush=True)
    return False


def _session_dirs(sessions_dir: Path) -> set[str]:
    """Names of vibe session directories already present under ``sessions_dir``."""
    if not sessions_dir.is_dir():
        return set()
    return {
        p.name
        for p in sessions_dir.iterdir()
        if p.is_dir() and p.name.startswith(_SESSION_PREFIX)
    }


def _save_trace(
    sessions_dir: Path,
    before: set[str],
    output_dir: Path,
    name: str,
) -> None:
    """Archive the newest vibe session created since ``before`` under
    ``output_dir/<name>/trace/``."""
    if not sessions_dir.is_dir():
        return
    new = [
        p
        for p in sessions_dir.iterdir()
        if p.is_dir()
        and p.name.startswith(_SESSION_PREFIX)
        and p.name not in before
    ]
    if not new:
        return
    latest = max(new, key=lambda p: p.stat().st_mtime)
    dest = Path(output_dir) / name / "trace"
    dest.parent.mkdir(parents=True, exist_ok=True)
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(latest, dest)


def _worker(
    workspace: Path,
    input_queue: Queue,
    output_queue: Queue,
    agent: AgentConfig,
    output_dir: Path,
) -> None:
    """Consume test cases, prove them in ``workspace``, and report ``Spec.lean``."""
    spec_path = workspace / LIB / "Spec.lean"
    sessions_dir = workspace / ".vibe" / "sessions"
    while True:
        try:
            item = input_queue.get_nowait()
        except Empty:
            time.sleep(_POLL_INTERVAL)
            continue
        if item is _STOP:
            return
        name, func_src, spec_src = item
        try:
            setup_testcase(str(workspace), func_src, spec_src)

            # The translated function + spec must compile before the agent runs.
            if not _build(workspace, name):
                output_queue.put((name, spec_src, "build_failed"))
                continue

            print(f"=== [{name}] running agent in {workspace.name} ===", flush=True)
            before = _session_dirs(sessions_dir)
            cmd = [agent.command, "-p", agent.prompt] + agent.args
            try:
                subprocess.run(cmd, cwd=workspace, timeout=agent.timeout)
            except subprocess.TimeoutExpired:
                print(
                    f"=== [{name}] agent timed out after {agent.timeout}s ===",
                    flush=True,
                )

            _save_trace(sessions_dir, before, output_dir, name)
            output_queue.put((name, spec_path.read_text(), "ok"))
        except Exception as exc:  # a worker must never die silently
            output_queue.put((name, None, f"error: {exc}"))
        finally:
            clear_workspace(str(workspace))


class LeanProverEngine:
    def __init__(self, config: Config | None = None):
        self.config = config or Config.load(DEFAULT_CONFIG)
        self.input_queue: Queue = Queue()
        self.output_queue: Queue = Queue()
        self.testcases: list = []
        self.mcp = MCPServer(
            self.config.workspaces, log_dir=self.config.output_dir / "mcp"
        )

    def setup(self) -> None:
        """Scan the benchmark and prepare the workspaces (build the base image
        once, then initialize every workspace from it)."""
        config = self.config
        if not config.workspaces:
            raise ValueError("config.toml: [workspace].candidates is empty")

        self.testcases = scan_testcases(config.input_dir)
        if not self.testcases:
            print(f"No testcases found in {config.input_dir}")

        if config.base_dir is not None:
            create_workspace(str(config.base_dir), config.vibe_dir)
        for i, ws in enumerate(config.workspaces):
            create_workspace(
                str(ws), config.vibe_dir, config.base_dir, mcp_port=self.mcp.port_for(i)
            )

    def prove(self) -> None:
        """Run the proving agent over every test case and write out results."""
        config = self.config
        testcases = self.testcases
        if not testcases:
            return
        funcs = {name: func for name, func, _ in testcases}

        # Start one worker process per workspace.
        workers = [
            Process(
                target=_worker,
                args=(
                    ws,
                    self.input_queue,
                    self.output_queue,
                    config.agent,
                    config.output_dir,
                ),
            )
            for ws in config.workspaces
        ]
        for w in workers:
            w.start()

        # Produce: feed the queue, then send one sentinel per worker.
        for tc in testcases:
            self.input_queue.put(tc)
        for _ in workers:
            self.input_queue.put(_STOP)

        # Consume: collect results and write them out.
        results = []
        while len(results) < len(testcases):
            try:
                results.append(self.output_queue.get_nowait())
            except Empty:
                time.sleep(_POLL_INTERVAL)
        for w in workers:
            w.join()

        for name, spec, status in results:
            self._save(name, funcs[name], spec, status)
        self._summarize(results)

    def start(self) -> None:
        """Convenience: run the full pipeline (setup then prove), then tear down."""
        self.setup()
        self.mcp.start()
        try:
            self.prove()
        finally:
            self.mcp.stop()

    def _save(
        self, name: str, func_src: str, spec_src: str | None, status: str
    ) -> None:
        out = self.config.output_dir / name
        out.mkdir(parents=True, exist_ok=True)
        (out / "Function.lean").write_text(func_src)
        (out / "status.txt").write_text(status + "\n")
        if spec_src is not None:
            (out / "Spec.lean").write_text(spec_src)

    @staticmethod
    def _summarize(results) -> None:
        def verdict(spec: str | None, status: str) -> str:
            if status == "build_failed":
                return "BUILD_FAIL"
            if status != "ok":
                return "ERROR"
            return "UNPROVED" if spec and _has_sorry(spec) else "PROVED"

        print("\n=== Evaluation summary ===")
        for name, spec, status in results:
            print(f"  {name:<40} {verdict(spec, status)}")

        counts: dict[str, int] = {}
        for _, spec, status in results:
            v = verdict(spec, status)
            counts[v] = counts.get(v, 0) + 1
        print(
            f"\n{len(results)} testcases -> "
            + ", ".join(
                f"{v}: {counts.get(v, 0)}"
                for v in ("PROVED", "UNPROVED", "BUILD_FAIL", "ERROR")
            )
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Run the agent evaluation pipeline")
    parser.add_argument(
        "--config", default=str(DEFAULT_CONFIG), help="Path to config.toml"
    )
    args = parser.parse_args()
    engine = LeanProverEngine(Config.load(Path(args.config)))
    engine.start()
