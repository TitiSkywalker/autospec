"""
Independent proof verification.

The proving pipeline labels a case ``PROVED`` merely when ``Spec.lean`` has no
``sorry``/``admit``; it never compiles the agent's final proof. This stage reads
the saved results (``<case>/Function.lean`` + ``<case>/Spec.lean``) from
``output_dir``, copies each into its own workspace, and actually compiles it with
``lake build``. The verdict is then grounded in real typechecking:

* ``PROVED``     — ``lake build`` succeeds and no ``sorry``/``admit`` remain.
* ``UNPROVED``   — ``lake build`` succeeds but the spec still has ``sorry``/``admit``.
* ``BUILD_FAIL`` — ``lake build`` fails (the proof is broken/inconsistent).

Usage::

    python -m evaluation.verifier --config evaluation/local/config.toml
"""

import argparse
import re
import shutil
import subprocess
from multiprocessing import Process, Queue
from pathlib import Path

from evaluation.config import Config, DEFAULT_CONFIG
from evaluation.logger import configure, get_logger
from evaluation.progress import StatusBoard
from evaluation.workspace import LIB, create_workspace

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")


def _classify(returncode: int, spec: str) -> str:
    if returncode != 0:
        return "BUILD_FAIL"
    if _SORRY_RE.search(spec):
        return "UNPROVED"
    return "PROVED"


def _worker(
    base_dir: Path, worker_dir: Path, index: int, in_q: Queue, out_q: Queue
) -> None:
    """One verifier process: compile queued cases in its own workspace copy.

    All output is shipped to the manager over ``out_q`` as tagged tuples —
    ``("log", text)``, ``("progress", index, text)``, ``("result", index, name,
    verdict, build_tail)`` — mirroring ``manager._worker``.
    """
    configure(lambda m: out_q.put(("log", m.rstrip("\n"))), colorize=True)
    ws = worker_dir / f"vs-{index}"
    out_q.put(("progress", index, "copying base mirror..."))
    create_workspace(str(ws), base_dir=base_dir)
    lib = ws / LIB

    while True:
        case = in_q.get()
        if case is None:
            break
        name, func, spec = case
        out_q.put(("progress", index, f"{name} · verifying..."))
        (lib / "Function.lean").write_text(func)
        (lib / "Spec.lean").write_text(spec)
        result = subprocess.run(
            ["lake", "build"], cwd=ws, capture_output=True, text=True
        )
        verdict = _classify(result.returncode, spec)
        # ``lake`` prints the real compile error on stdout (the "error: build
        # failed" summary line goes to stderr), so keep both.
        output = (result.stdout or "") + (result.stderr or "")
        tail = "\n".join(output.splitlines()[-100:])
        out_q.put(("result", index, name, verdict, tail))
    shutil.rmtree(ws, ignore_errors=True)


class Verifier:
    """Compile every saved proof in its own workspace and report a verdict."""

    def __init__(self, config: Config | None = None):
        self.config = config or Config.load(DEFAULT_CONFIG)
        self.in_q: Queue = Queue()
        self.out_q: Queue = Queue()
        self.logger = get_logger(self)

    def run(self) -> None:
        self._prepare()
        cases = self._scan()
        if not cases:
            self.logger.warning(
                f"No cases (Function.lean + Spec.lean) in {self.config.output_dir}"
            )
            return

        workers = [
            Process(
                target=_worker,
                args=(
                    self.config.base_dir,
                    self.config.verifier.worker_dir,
                    i,
                    self.in_q,
                    self.out_q,
                ),
            )
            for i in range(self.config.verifier.workers)
        ]
        for w in workers:
            w.start()

        board = StatusBoard([f"vs-{i}" for i in range(self.config.verifier.workers)])

        for case in cases:
            self.in_q.put(case)
        for _ in workers:
            self.in_q.put(None)

        results = []
        pending = len(cases)
        while pending:
            msg = self.out_q.get()
            kind = msg[0]
            if kind == "log":
                board.log(msg[1])
            elif kind == "progress":
                board.update(msg[1], msg[2])
            else:  # ("result", index, name, verdict, tail)
                _, index, name, verdict, tail = msg
                results.append((name, verdict, tail))
                pending -= 1
                self._save(name, verdict, tail)
                board.update(index, f"{name} · {verdict}")
        for w in workers:
            w.join()

        self._summarize(results)

    def _prepare(self) -> None:
        c = self.config.verifier
        if c.worker_dir.exists():
            shutil.rmtree(c.worker_dir)
        c.worker_dir.mkdir(parents=True, exist_ok=True)

    def _scan(self) -> list[tuple[str, str, str]]:
        """Every ``<case>/`` with both a Function.lean and a Spec.lean."""
        out = self.config.output_dir
        if not out.is_dir():
            return []
        cases = []
        for d in sorted(out.iterdir()):
            if not d.is_dir():
                continue
            func, spec = d / "Function.lean", d / "Spec.lean"
            if func.is_file() and spec.is_file():
                cases.append((d.name, func.read_text(), spec.read_text()))
        return cases

    def _save(self, name: str, verdict: str, tail: str) -> None:
        out = self.config.output_dir / name
        (out / "verdict.txt").write_text(verdict + "\n")
        if verdict == "BUILD_FAIL":
            (out / "build.log").write_text(tail + "\n")

    def _summarize(self, results) -> None:
        self.logger.info("\n=== Verification summary ===")
        for name, verdict, _ in results:
            emit = self.logger.success if verdict == "PROVED" else self.logger.info
            emit(f"{name:<40} {verdict}")

        counts: dict[str, int] = {}
        for _, verdict, _ in results:
            counts[verdict] = counts.get(verdict, 0) + 1
        self.logger.info(
            f"\n{len(results)} cases -> "
            + ", ".join(
                f"{v}: {counts.get(v, 0)}" for v in ("PROVED", "UNPROVED", "BUILD_FAIL")
            )
        )


def main() -> None:
    parser = argparse.ArgumentParser(description="Verify proved results independently")
    parser.add_argument(
        "--config", default=str(DEFAULT_CONFIG), help="Path to config.toml"
    )
    args = parser.parse_args()
    Verifier(Config.load(Path(args.config))).run()


if __name__ == "__main__":
    main()
