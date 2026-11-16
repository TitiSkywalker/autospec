#!/usr/bin/env python3
"""Run the proof-tree analysis: Opus 5.5 vericoding proofs, NavierStokes/Euler, miniF2F control.

Wraps :mod:`analysis.proof_tree.experiments`: vericoding, navier_stokes, minif2f, comparison.
Everything is cached by default.

Usage::

    python3 analysis/run_proof_tree.py                        # all four, from caches
    python3 analysis/run_proof_tree.py --only vericoding      # or navier_stokes / minif2f / comparison
    python3 analysis/run_proof_tree.py --rebuild              # re-run Lean build+extract (vericoding + minif2f)
    python3 analysis/run_proof_tree.py --no-cache             # re-extract NavierStokes (~25 min)
    python3 analysis/run_proof_tree.py --results-dir evaluation/local/other_results
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TREANT = "/home/disk2/liguchan/Lean/Treant"
OPUS_RESULTS = "evaluation/local/runs/main/opus-5.5"  # Claude-Opus-5.5 full run (116 proved)

sys.path.insert(0, str(ROOT))
sys.path.insert(0, TREANT)

from analysis.proof_tree.experiments import EXPERIMENTS, ProofTreeExperiments  # noqa: E402


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--only", action="append", choices=EXPERIMENTS,
                    help="run only this experiment (repeatable; default: all)")
    ap.add_argument("--rebuild", action="store_true", help="rebuild vericoding and miniF2F caches (runs lake / Lean)")
    ap.add_argument("--no-cache", action="store_true", help="re-extract NavierStokes raw JSONL (~25 min)")
    ap.add_argument("--results-dir", default=OPUS_RESULTS, help="vericoding results dir (default: %(default)s)")
    ap.add_argument("--model", default=None, help="cache namespace (default: basename of --results-dir)")
    args = ap.parse_args()

    run = set(args.only or EXPERIMENTS)
    ProofTreeExperiments().run_all(
        vericoding="vericoding" in run,
        navier_stokes="navier_stokes" in run,
        minif2f="minif2f" in run,
        comparison="comparison" in run,
        vericoding_rebuild=args.rebuild,
        navier_stokes_no_cache=args.no_cache,
        minif2f_rebuild=args.rebuild,
        results_dir=args.results_dir,
        cache_key=args.model,
    )


if __name__ == "__main__":
    main()
