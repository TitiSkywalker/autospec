"""Reading per-case verdicts out of a results directory.

A case counts as PROVED only from the ``verdict.txt`` written by the Comparator
verifier. When a run predates the verifier (no ``verdict.txt``), we fall back to
the old proving heuristic: ``status.txt == ok`` and no ``sorry``/``admit`` in
``Spec.lean`` means PROVED.
"""

import re
from pathlib import Path

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")


def category(name: str) -> str:
    """The category bucket of a testcase, from its ``<category>_<name>`` prefix."""
    return name.split("_", 1)[0]


def read_verdict(case_dir: Path) -> str:
    """Verdict of one case: PROVED / UNPROVED / BUILD_FAIL / CHEAT / ... / UNKNOWN."""
    v = case_dir / "verdict.txt"
    if v.is_file():
        return v.read_text().strip() or "UNKNOWN"
    spec = case_dir / "Spec.lean"
    status = case_dir / "status.txt"
    if spec.is_file():
        if status.is_file() and status.read_text().strip() != "ok":
            return "BUILD_FAIL"
        return "PROVED" if not _SORRY_RE.search(spec.read_text()) else "UNPROVED"
    return "UNKNOWN"


def collect(results_dir: Path) -> dict[str, str]:
    """``{case_name: verdict}`` for every case in a results directory."""
    out = {}
    for d in sorted(Path(results_dir).iterdir()):
        if not d.is_dir():
            continue
        if (d / "verdict.txt").is_file() or (d / "Spec.lean").is_file():
            out[d.name] = read_verdict(d)
    return out
