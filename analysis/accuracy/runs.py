"""Which evaluation runs a report compares.

A *run* is one verified results directory (``[prover] output_dir`` after
``python -m evaluation.verifier``): one sub-directory per testcase holding the
agent's ``Spec.lean``, ``verdict.txt`` and, optionally, its ``trace/``.

Runs are given on the command line, either one by one (``--run LABEL=DIR``)
or as a parent directory whose sub-directories are runs (``--runs-dir DIR``,
labelled by directory name).
"""

from __future__ import annotations

import tomllib
from dataclasses import dataclass, field
from pathlib import Path


@dataclass
class Context:
    """Everything a report needs: the runs to compare and where to write."""

    runs: dict[str, Path]           # display label -> results directory
    out_dir: Path                   # figures land here
    benchmark: Path                 # lean_mode/LeanMode
    prices: dict[str, dict] = field(default_factory=dict)  # label -> USD / Mtok


def _looks_like_run(d: Path) -> bool:
    return any((c / "verdict.txt").is_file() or (c / "Spec.lean").is_file()
               for c in d.iterdir() if c.is_dir())


def parse_runs(specs: list[str], runs_dir: str | None) -> dict[str, Path]:
    """``--run LABEL=DIR`` specs (``DIR`` alone uses its basename as label)
    plus every run under ``runs_dir``, in that order."""
    runs: dict[str, Path] = {}
    for spec in specs:
        label, sep, path = spec.partition("=")
        if not sep:
            label, path = Path(spec).name, spec
        runs[label] = Path(path).resolve()
    if runs_dir:
        for d in sorted(Path(runs_dir).iterdir()):
            if d.is_dir() and _looks_like_run(d):
                runs.setdefault(d.name, d.resolve())
    for label, path in runs.items():
        if not path.is_dir():
            raise SystemExit(f"run {label!r}: {path} is not a directory")
    if not runs:
        raise SystemExit("no runs given: pass --run LABEL=DIR and/or --runs-dir DIR")
    return runs


def load_prices(path: str | None) -> dict[str, dict]:
    """Price table (USD per million tokens) keyed by run label, from TOML."""
    if not path:
        return {}
    return tomllib.loads(Path(path).read_text())
