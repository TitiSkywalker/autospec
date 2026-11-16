"""
Independent proof verification.

Two verification modes are selected by ``[verifier] mode``:

``comparator`` (default) — set up a ``Challenge.lean`` (the original benchmark
spec, ``sorry`` intact) and a ``Solution.lean`` (the agent's output) in each
workspace and run the Comparator trustworthy judge. Comparator proves that the
solution (1) states the exact same theorems, (2) uses only the permitted axioms
(no leftover ``sorry``, no injected ``axiom``), and (3) is accepted by the Lean
kernel. Verdicts:

* ``PROVED``      — "Your solution is okay!"
* ``UNPROVED``    — the proof still depends on ``sorry`` (``sorryAx``).
* ``BUILD_FAIL``  — ``lake build Solution`` (or export) fails.
* ``CHEAT``       — statement mismatch / injected axiom / missing theorem.
* ``KERNEL_FAIL`` — the Lean kernel rejects the proof.

``lake`` (legacy) — compile ``Spec.lean`` with ``lake build`` and grep for
``sorry``/``admit``. Faster, but gamed by statement-weakening or axiom injection.

Usage::

    python -m evaluation.verifier --config evaluation/local/config.toml
"""

import argparse
import json
import os
import re
import shutil
import subprocess
from datetime import datetime
from multiprocessing import Process, Queue
from pathlib import Path

from evaluation.config import Config, DEFAULT_CONFIG
from evaluation.logger import configure, get_logger
from evaluation.progress import StatusBoard
from evaluation.workspace import LIB, create_workspace

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")
# Original benchmark specs import their per-case aeneas module (e.g.
# ``import LeanMode.foo.Bar``); in the workspace that translation lives at
# ``RustVerif.Function``.
_IMPORT_RE = re.compile(r"import LeanMode\.[^\n]+")
# ``theorem <name>`` declarations, any indentation (so ones nested in
# ``namespace``/``section``/``mutual`` are found too). Comments are stripped
# before this runs, so prose never matches. ``lemma``s are deliberately not
# targets: they are the challenge's own helper facts, not proof obligations.
_THEOREM_RE = re.compile(r"^[ \t]*theorem[ \t]+(\S+)", re.MULTILINE)
_OPENER_RE = re.compile(
    r"^[ \t]*(?:noncomputable[ \t]+)?(?:namespace|section|mutual)\b(?:[ \t]+(\S+))?"
)
_END_RE = re.compile(r"^[ \t]*end\b(?:[ \t]+(\S+))?")
_BLOCK_COMMENT = re.compile(r"/-.*?-/", re.DOTALL)
_LINE_COMMENT = re.compile(r"--[^\n]*")
_PERMITTED_AXIOMS = ["propext", "Quot.sound", "Classical.choice"]

_VERDICTS = ("PROVED", "UNPROVED", "BUILD_FAIL", "CHEAT", "KERNEL_FAIL")


def _classify_lake(returncode: int, spec: str) -> str:
    """Legacy verdict from ``lake build`` + a ``sorry``/``admit`` grep."""
    if returncode != 0:
        return "BUILD_FAIL"
    if _SORRY_RE.search(spec):
        return "UNPROVED"
    return "PROVED"


def _classify_comparator(returncode: int, output: str) -> str:
    """Map Comparator's output to a verdict."""
    if returncode == 0 and "Your solution is okay!" in output:
        return "PROVED"
    if "Illegal axiom detected: 'sorryAx'" in output:
        return "UNPROVED"
    if "rejects the solution" in output:
        return "KERNEL_FAIL"
    # Any of Comparator's mismatch / illegal-axiom errors mean the solution did
    # not faithfully prove the challenge's statement (weakened theorem, changed
    # a definition the statement depends on, injected an axiom, ...).
    if any(
        m in output
        for m in (
            "Illegal axiom detected",
            "statement do not match",
            "Const does not match",
            "Const not found",
            "kind don't match",
            "not a definition",
            "not a theorem",
        )
    ):
        return "CHEAT"
    return "BUILD_FAIL"


def _strip_comments(src: str) -> str:
    """Blank out comments, keeping newlines so line structure survives."""
    def blank(text: str) -> str:
        return re.sub(r"[^\n]", " ", text)

    src = _BLOCK_COMMENT.sub(lambda m: blank(m.group(0)), src)
    return _LINE_COMMENT.sub(lambda m: blank(m.group(0)), src)


def _declarations(src: str) -> list[str]:
    """Fully-qualified names of the challenge's ``theorem`` declarations.

    Comparator is handed these names verbatim, so they must be what Lean calls
    the constant: a theorem inside ``namespace Foo`` is ``Foo.bar``, not
    ``bar`` (a bare name makes ``lean4export`` panic with "Constant not found
    in environment"). Comments are stripped first so prose such as
    "theorem for general graphs" is not mistaken for a declaration.
    """
    src = _strip_comments(src)
    prefix = ""  # namespace prefix currently in effect
    scopes: list[str] = []  # prefix to restore at each `end`
    names: list[str] = []
    for line in src.splitlines():
        opener = _OPENER_RE.match(line)
        if opener:
            scopes.append(prefix)
            # `section` / `mutual` scope `end` but are not part of a name.
            if "namespace" in line.split()[0:2] and opener.group(1):
                prefix = f"{prefix}.{opener.group(1)}" if prefix else opener.group(1)
            continue
        if _END_RE.match(line):
            if scopes:
                prefix = scopes.pop()
            continue
        decl = _THEOREM_RE.match(line)
        if decl:
            names.append(f"{prefix}.{decl.group(1)}" if prefix else decl.group(1))
    return names


def _verify_lake(ws: Path, lib: Path, func: str, spec: str) -> tuple[str, str]:
    """Legacy mode: compile the spec and grep for ``sorry``."""
    (lib / "Function.lean").write_text(func)
    (lib / "Spec.lean").write_text(spec)
    result = subprocess.run(
        ["lake", "build"], cwd=ws, capture_output=True, text=True
    )
    output = (result.stdout or "") + (result.stderr or "")
    tail = "\n".join(output.splitlines()[-100:])
    return _classify_lake(result.returncode, spec), tail


def _verify_comparator(
    ws: Path, lib: Path, func: str, spec: str, challenge: str, sandbox: bool
) -> tuple[str, str]:
    """Comparator mode: prove the agent's ``Solution`` matches the ``Challenge``."""
    (lib / "Function.lean").write_text(func)
    (ws / "Challenge.lean").write_text(challenge)
    (ws / "Solution.lean").write_text(spec)
    theorem_names = _declarations(challenge)
    config = {
        "challenge_module": "Challenge",
        "solution_module": "Solution",
        "theorem_names": theorem_names,
        "permitted_axioms": _PERMITTED_AXIOMS,
        "enable_nanoda": False,
    }
    (ws / "config.json").write_text(json.dumps(config, indent=2))

    pkg = ws / ".lake" / "packages"
    vendor = ws / "comparator"  # path-dep vendored source, built in-place
    env = os.environ.copy()
    env["COMPARATOR_LEAN4EXPORT"] = str(
        pkg / "lean4export" / ".lake" / "build" / "bin" / "lean4export"
    )
    if sandbox:
        env["COMPARATOR_LANDRUN"] = "landrun"
    else:
        env["COMPARATOR_LANDRUN"] = str(vendor / "scripts" / "fake-landrun.sh")
    comparator = vendor / ".lake" / "build" / "bin" / "comparator"

    result = subprocess.run(
        ["lake", "env", str(comparator), "config.json"],
        cwd=ws,
        capture_output=True,
        text=True,
        env=env,
    )
    output = (result.stdout or "") + (result.stderr or "")
    verdict = _classify_comparator(result.returncode, output)
    tail = "\n".join(output.splitlines()[-100:])
    return verdict, tail


def _worker(
    base_dir: Path,
    worker_dir: Path,
    index: int,
    mode: str,
    sandbox: bool,
    in_q: Queue,
    out_q: Queue,
) -> None:
    """One verifier process: judge queued cases in its own workspace copy.

    All output is shipped to the manager over ``out_q`` as tagged tuples —
    ``("log", text)``, ``("progress", index, text)``, ``("result", index, name,
    verdict, tail)`` — mirroring ``manager._worker``.
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
        name, func, spec, challenge = case
        out_q.put(("progress", index, f"{name} · verifying..."))
        if mode == "comparator" and challenge is not None:
            verdict, tail = _verify_comparator(ws, lib, func, spec, challenge, sandbox)
        else:
            verdict, tail = _verify_lake(ws, lib, func, spec)
        out_q.put(("result", index, name, verdict, tail))
    shutil.rmtree(ws, ignore_errors=True)


class Verifier:
    """Independently judge every saved proof and report a verdict."""

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
                    self.config.verifier.mode,
                    self.config.verifier.sandbox,
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
        self._save_metadata(results)

    def _prepare(self) -> None:
        c = self.config.verifier
        if c.worker_dir.exists() and not c.resume:
            shutil.rmtree(c.worker_dir)
        c.worker_dir.mkdir(parents=True, exist_ok=True)

    def _scan(self) -> list[tuple[str, str, str, str | None]]:
        """Every ``<case>/`` with both a Function.lean and a Spec.lean.

        Returns ``(name, func, spec, challenge)`` where ``challenge`` is the
        original benchmark spec (``None`` if ``input_dir`` has no such case).
        """
        out = self.config.output_dir
        if not out.is_dir():
            return []
        cases = []
        for d in sorted(out.iterdir()):
            if not d.is_dir():
                continue
            if self.config.verifier.resume and (d / "verdict.txt").is_file():
                continue
            func, spec = d / "Function.lean", d / "Spec.lean"
            if func.is_file() and spec.is_file():
                cases.append((d.name, func.read_text(), spec.read_text(), self._challenge(d.name)))
        return cases

    def _challenge(self, name: str) -> str | None:
        """The original benchmark spec, with its aeneas import pointed at the
        workspace's ``RustVerif.Function`` module."""
        src = self.config.input_dir / name / "Spec.lean"
        if not src.is_file():
            return None
        return _IMPORT_RE.sub(f"import {LIB}.Function", src.read_text())

    def _save(self, name: str, verdict: str, tail: str) -> None:
        out = self.config.output_dir / name
        (out / "verdict.txt").write_text(verdict + "\n")
        if verdict != "PROVED":
            (out / "verify.log").write_text(tail + "\n")

    def _save_metadata(self, results) -> None:
        """Persist the verification outcome as one JSON file in ``output_dir``.

        ``verdict.txt``/``verify.log`` are per-case; this is the machine-readable
        aggregate (config provenance + counts + one entry per case), so a later
        analysis step doesn't have to re-scan every case directory.
        """
        c = self.config.verifier
        summary: dict[str, int] = {v: 0 for v in _VERDICTS}
        cases = []
        for name, verdict, _ in sorted(results, key=lambda r: r[0]):
            summary[verdict] = summary.get(verdict, 0) + 1
            cases.append({"name": name, "verdict": verdict})
        meta = {
            "timestamp": datetime.now().isoformat(timespec="seconds"),
            "mode": c.mode,
            "sandbox": c.sandbox,
            "permitted_axioms": _PERMITTED_AXIOMS,
            "input_dir": str(self.config.input_dir),
            "total": len(results),
            "summary": summary,
            "cases": cases,
        }
        path = self.config.output_dir / "verification_meta.json"
        path.write_text(json.dumps(meta, indent=2) + "\n")
        self.logger.info(f"metadata -> {path}")

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
            + ", ".join(f"{v}: {counts.get(v, 0)}" for v in _VERDICTS)
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
