"""Which evaluation runs a report compares.

A *run* is one verified results directory (``[prover] output_dir`` after
``python -m evaluation.verifier``): one sub-directory per testcase holding the
agent's ``Spec.lean``, ``verdict.txt`` and, optionally, its ``trace/``.

On the command line each run is given as a run directory, as ``LABEL=DIR`` to
choose its label, or as a parent directory whose sub-directories are runs.
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


def parse_runs(specs: list[str]) -> dict[str, Path]:
    """Map labels to run directories, in the order given.

    Each spec is ``DIR`` (labelled by its name), ``LABEL=DIR``, or a parent
    directory of runs (each sub-directory labelled by its name). Raises
    ``ValueError`` with a readable message for a bad spec.
    """
    runs: dict[str, Path] = {}
    for spec in specs:
        label, sep, path_str = spec.partition("=")
        path = Path(path_str if sep else spec)
        if not path.is_dir():
            raise ValueError(f"{path} is not a directory")
        if sep:
            runs[label] = path.resolve()
        elif _looks_like_run(path):
            runs[path.resolve().name] = path.resolve()
        else:
            found = [d for d in sorted(path.iterdir()) if d.is_dir() and _looks_like_run(d)]
            if not found:
                raise ValueError(f"{path} is neither a run nor a directory of runs "
                                 "(no case folder with a verdict.txt or Spec.lean)")
            for d in found:
                runs.setdefault(d.name, d.resolve())
    return runs


def load_prices(path: str | None) -> dict[str, dict]:
    """Price table (USD per million tokens) keyed by run label, from TOML."""
    if not path:
        return {}
    return tomllib.loads(Path(path).read_text())
