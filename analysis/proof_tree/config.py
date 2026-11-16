"""Paths and knobs for the proof-tree analysis.

Everything that points at a directory on disk lives here, so moving the results
or the on-disk cache is a one-file edit. Paths are repository-relative strings;
callers resolve them against :data:`ROOT`.

``sys.path`` manipulation is handled by the package entry points
(:mod:`.__main__` and :func:`.distribution.main`), not here, so that this
lowest-layer module has no import-time side effects.
"""

from __future__ import annotations

from pathlib import Path

# ``analysis/proof_tree/config.py`` -> ``analysis/proof_tree`` -> ``analysis`` -> repository root
ROOT = Path(__file__).resolve().parents[2]

# ======================= Configuration (edit here) =======================

RESULTS_DIR = "evaluation/local/runs/main/opus-5.5"  # Opus 5.5 full run (same as analysis/accuracy/runs.py)
BASE_DIR = "evaluation/local/base"              # prebuilt aeneas+Mathlib workspace
WORKSPACE_DIR = "analysis/local/proof_tree_ws"  # reusable scratch workspace (gitignored)
CACHE_DIR = "analysis/local/proof_tree_data"    # per-case cached stats (gitignored)
OUT_DIR = "analysis/figs"

# miniF2F control: LLM-solved competition-math Lean proofs (JSONL; each record
# has ``problem_id`` and ``trials[*].{code,passed}``).
# (label, path, how many PASSED samples to take from it).
MINIF2F_SAMPLES = [
    ("goedel", "/home/disk2/liguchan/Lean/Treant/local/dataset/goedel_minif2f.jsonl", 40),
    ("kimina", "/home/disk2/liguchan/Lean/Treant/local/dataset/kimina_minif2f.jsonl", 40),
]
MINIF2F_CACHE_TAG = "minif2f"  # cache namespace under CACHE_DIR

LIB = "RustVerif"  # matches evaluation.workspace.LIB

# Verdicts analyzed by default (the "correct" proofs). Pass --all to include
# every case regardless of verdict (UNPROVED/BUILD_FAIL/...).
DEFAULT_VERDICTS = {"PROVED"}


def cache_dir_for(results_dir: Path, model: str | None) -> Path:
    """Return the per-model cache directory.

    ``model`` overrides the default; otherwise the cache namespace is the
    basename of ``results_dir``.  This is the single source of truth for the
    cache-namespace rule; both :mod:`.cli` and :mod:`.distribution` call it.
    """
    model_tag = model or results_dir.name
    return ROOT / CACHE_DIR / model_tag
