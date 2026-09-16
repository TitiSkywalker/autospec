"""
Proving pipeline orchestration.

``Manager`` scans the benchmark, builds the base mirror once, then dispatches
the ``TestCase`` list to a pool of worker processes (``_worker``) and collects,
saves, and summarizes their results.

Usage::

    python -m evaluation.manager --config evaluation/local/config.toml
"""

import argparse
import re
import shutil
from multiprocessing import Process, Queue
from pathlib import Path

from evaluation.config import AgentConfig, Config, DEFAULT_CONFIG, McpConfig
from evaluation.logger import configure, get_logger
from evaluation.progress import StatusBoard
from evaluation.prover import Prover
from evaluation.testcase import TestCase
from evaluation.workspace import create_workspace

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")


def _worker(
    base_dir: Path,
    worker_dir: Path,
    index: int,
    input_queue: Queue,
    output_queue: Queue,
    agent: AgentConfig,
    mcp: McpConfig,
    output_dir: Path,
) -> None:
    """One worker process: prove queued test cases in its own workspace copy.

    All output is shipped to the manager over ``output_queue`` as tagged tuples
    — ``("log", text)``, ``("progress", index, text)``, ``("result", index,
    name, spec, status)`` — so a single writer owns the terminal.
    """
    configure(
        lambda m: output_queue.put(("log", m.rstrip("\n"))),
        colorize=True,
    )
    with Prover(
        base_dir,
        worker_dir,
        index,
        agent,
        mcp,
        output_dir,
        progress=lambda text: output_queue.put(("progress", index, text)),
    ) as prover:
        while True:
            case = input_queue.get()
            if case is None:
                return
            output_queue.put(("result", index, *prover.prove(case)))


class Manager:
    """Build a base mirror once, then prove every test case with N parallel workers."""

    def __init__(self, config: Config | None = None):
        self.config = config or Config.load(DEFAULT_CONFIG)
        self.input_queue: Queue = Queue()
        self.output_queue: Queue = Queue()
        self.logger = get_logger(self)

    def run(self) -> None:
        self._prepare()
        cases = TestCase.scan(self.config.input_dir)
        if not cases:
            self.logger.warning(f"No testcases found in {self.config.input_dir}")
            return
        funcs = {case.name: case.func_src for case in cases}

        workers = [
            Process(
                target=_worker,
                args=(
                    self.config.base_dir,
                    self.config.worker_dir,
                    i,
                    self.input_queue,
                    self.output_queue,
                    self.config.agent,
                    self.config.mcp,
                    self.config.output_dir,
                ),
            )
            for i in range(self.config.num_workers)
        ]
        for w in workers:
            w.start()

        board = StatusBoard([f"ws-{i}" for i in range(self.config.num_workers)])

        for case in cases:
            self.input_queue.put(case)
        for _ in workers:
            self.input_queue.put(None)

        results = []
        pending = len(cases)
        while pending:
            msg = self.output_queue.get()
            kind = msg[0]
            if kind == "log":
                board.log(msg[1])
            elif kind == "progress":
                board.update(msg[1], msg[2])
            else:  # ("result", index, name, spec, status)
                _, index, name, spec, status = msg
                results.append((name, spec, status))
                pending -= 1
                self._save(name, funcs[name], spec, status)
                board.update(index, f"{name} · {status}")
        for w in workers:
            w.join()

        self._summarize(results)

    def _prepare(self) -> None:
        """Build the base mirror once (toolchain + vibe config), then clear any
        leftover worker dirs."""
        c = self.config
        create_workspace(str(c.base_dir), vibe_dir=c.vibe_dir)
        # The early-exit path above skips vibe deployment when base already
        # exists, so (re)sync it to keep the mirror self-contained.
        if c.vibe_dir is not None:
            shutil.copytree(c.vibe_dir, c.base_dir / ".vibe", dirs_exist_ok=True)
        if c.worker_dir.exists():
            shutil.rmtree(c.worker_dir)
        c.worker_dir.mkdir(parents=True, exist_ok=True)

    def _save(
        self, name: str, func_src: str, spec_src: str | None, status: str
    ) -> None:
        out = self.config.output_dir / name
        out.mkdir(parents=True, exist_ok=True)
        (out / "Function.lean").write_text(func_src)
        (out / "status.txt").write_text(status + "\n")
        if spec_src is not None:
            (out / "Spec.lean").write_text(spec_src)

    def _summarize(self, results) -> None:
        def verdict(spec: str | None, status: str) -> str:
            if status == "build_failed":
                return "BUILD_FAIL"
            if status != "ok":
                return "ERROR"
            return "UNPROVED" if spec and _SORRY_RE.search(spec) else "PROVED"

        self.logger.info("\n=== Evaluation summary ===")
        for name, spec, status in results:
            v = verdict(spec, status)
            emit = self.logger.success if v == "PROVED" else self.logger.info
            emit(f"{name:<40} {v}")

        counts: dict[str, int] = {}
        for _, spec, status in results:
            v = verdict(spec, status)
            counts[v] = counts.get(v, 0) + 1
        self.logger.info(
            f"\n{len(results)} testcases -> "
            + ", ".join(
                f"{v}: {counts.get(v, 0)}"
                for v in ("PROVED", "UNPROVED", "BUILD_FAIL", "ERROR")
            )
        )


def main() -> None:
    parser = argparse.ArgumentParser(description="Run the agent evaluation pipeline")
    parser.add_argument(
        "--config", default=str(DEFAULT_CONFIG), help="Path to config.toml"
    )
    args = parser.parse_args()
    Manager(Config.load(Path(args.config))).run()


if __name__ == "__main__":
    main()
