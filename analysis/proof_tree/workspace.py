"""The reusable Lean workspace and per-case build/extract pipeline.

The workspace is a copy of ``evaluation/local/base`` (prebuilt aeneas +
Mathlib, ~17G) kept under :data:`~analysis.proof_tree.config.WORKSPACE_DIR` and
reused across runs: each case overwrites ``RustVerif/{Function,Spec}.lean`` and
re-runs ``lake build``, which only re-elaborates the two changed files
(~20-40s/case). Processing is therefore sequential by design (one shared
workspace) and a full run over ~90 cases takes on the order of an hour.

``treant`` and ``evaluation.workspace`` are imported lazily, inside the
functions that need them. That is deliberate: cached runs report fine on a
machine without a Lean/Treant checkout, and only the build path pays the import.
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

from analysis.proof_tree.config import BASE_DIR, LIB, ROOT, WORKSPACE_DIR
from analysis.proof_tree.stats import _tree_stats


class CaseError(Exception):
    """Raised by :func:`export_dot` when a case cannot be built or extracted."""


def _verdict(case_dir: Path) -> str:
    v = case_dir / "verdict.txt"
    return v.read_text().strip() or "UNKNOWN" if v.is_file() else "UNKNOWN"


def discover_cases(results_dir: Path) -> list[tuple[str, str]]:
    """``[(name, verdict)]`` for every case with both lean files present."""
    cases = []
    for d in sorted(results_dir.iterdir()):
        if d.is_dir() and (d / "Function.lean").is_file() and (d / "Spec.lean").is_file():
            cases.append((d.name, _verdict(d)))
    return cases


def ensure_workspace() -> Path:
    """Return the reusable workspace, creating it from BASE_DIR if needed."""
    return ensure_workspace_at(ROOT / WORKSPACE_DIR)


def ensure_workspace_at(ws: Path) -> Path:
    """Return a workspace at ``ws``, creating it from BASE_DIR if needed.

    Parameterised so parallel workers can each own an isolated workspace copy
    (the shared ``Spec.lean``/``Function.lean`` would otherwise collide). The
    default (:func:`ensure_workspace`) points at ``WORKSPACE_DIR``.

    Copies are made with a full ``cp -a`` (not hardlinks): the build rewrites
    ``RustVerif/*.lean`` and ``.lake/build/**`` in place, which would otherwise
    corrupt sibling copies sharing those inodes.
    """
    if not (ws / LIB / "Basic.lean").is_file():
        template = ROOT / WORKSPACE_DIR
        if template.is_dir():
            # Full copy of the prebuilt workspace (isolated, no shared inodes).
            print(f"copying workspace {template} -> {ws} ...", file=sys.stderr)
            ws.parent.mkdir(parents=True, exist_ok=True)
            subprocess.run(["cp", "-a", str(template), str(ws)], check=True)
        else:
            print(f"setting up workspace at {ws} (one-time copy of {BASE_DIR}, ~17G)...", file=sys.stderr)
            from evaluation.workspace import create_workspace, LIB as EVAL_LIB

            assert LIB == EVAL_LIB, (
                f"config.LIB={LIB!r} disagrees with evaluation.workspace.LIB={EVAL_LIB!r}"
            )
            ws.parent.mkdir(parents=True, exist_ok=True)
            create_workspace(str(ws), base_dir=str(ROOT / BASE_DIR))
    return ws


def _lake_build(ws: Path) -> tuple[bool, str]:
    try:
        r = subprocess.run(
            ["lake", "build", LIB], cwd=ws, capture_output=True, text=True, timeout=600
        )
    except subprocess.TimeoutExpired:
        return False, "build timed out after 600s"
    output = (r.stdout or "") + (r.stderr or "")
    return r.returncode == 0, "\n".join(output.splitlines()[-200:])


def _extract_theorems(ws: Path, filename: str = f"{LIB}/Spec.lean"):
    """Run the treant extractor + proof-tree builder on ``filename`` (default: Spec.lean)."""
    from treant.lean.extractor import Extractor
    from treant.tree.tree import ProofTreeBuilder

    tmp = ws / "_extracted.json"
    try:
        data = Extractor().extract(
            project_dir=str(ws), filename=filename, save_json=str(tmp)
        )
    finally:
        tmp.unlink(missing_ok=True)
    return ProofTreeBuilder().build(data)


def _build_and_extract(name: str, case_dir: Path, ws: Path, verbose: bool = False) -> list:
    """Copy lean files, run lake build, extract theorems; raise :exc:`CaseError` on any failure.

    Returns the raw list of theorem objects from the extractor.
    Raises :exc:`CaseError` with a message of the form ``"<name>: <error_kind>"``
    so callers can convert it to whichever error representation they need.
    """
    lib = ws / LIB
    try:
        (lib / "Function.lean").write_text((case_dir / "Function.lean").read_text())
        (lib / "Spec.lean").write_text((case_dir / "Spec.lean").read_text())
    except (OSError, UnicodeDecodeError) as exc:
        raise CaseError(f"{name}: build_fail") from exc

    ok, log = _lake_build(ws)
    if not ok:
        if verbose:
            print(log, file=sys.stderr)
        kind = "build_timeout" if "timed out" in log else "build_fail"
        raise CaseError(f"{name}: {kind}")

    try:
        theorems = _extract_theorems(ws)
    except (Exception, SystemExit) as exc:  # extractor panics via sys.exit
        if verbose:
            print(f"extraction failed: {exc!r}", file=sys.stderr)
        raise CaseError(f"{name}: extract_fail") from exc

    return theorems


def process_case(name: str, case_dir: Path, ws: Path, verbose: bool = False) -> dict:
    """Build+extract one case's proof tree; never raises (errors are returned)."""
    try:
        theorems = _build_and_extract(name, case_dir, ws, verbose=verbose)
    except CaseError as exc:
        kind = str(exc).split(": ", 1)[-1] if ": " in str(exc) else "build_fail"
        return {"name": name, "error": kind}
    return _measure(name, theorems)


def process_source(name: str, code: str, ws: Path) -> dict:
    """Extract a standalone Lean file's proof trees (no Aeneas, no ``lake build``).

    Used for corpora that are just Mathlib proofs (e.g. the miniF2F control).
    The source is written to a scratch file inside the workspace so Treant
    elaborates it against the workspace's Mathlib. Never raises.
    """
    scratch = ws / "_scratch_source.lean"
    try:
        scratch.write_text(code)
        theorems = _extract_theorems(ws, scratch.name)
    except (Exception, SystemExit):  # extractor panics via sys.exit
        return {"name": name, "error": "extract_fail"}
    finally:
        scratch.unlink(missing_ok=True)
    return _measure(name, theorems)


def _measure(name: str, theorems: list) -> dict:
    """Per-theorem :func:`_tree_stats` for every theorem that has a proof tree."""
    per_theorem = []
    for th in theorems:
        if not th.roots:
            continue
        st = _tree_stats(th.roots)
        st["theorem"] = th.full_name
        st["kind"] = th.kind
        per_theorem.append(st)
    return {"name": name, "theorems": per_theorem}
