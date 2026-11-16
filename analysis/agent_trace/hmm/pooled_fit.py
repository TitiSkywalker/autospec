"""Parallel multi-restart fit of the pooled HMM; caches the best solution.

The pooled fit is the expensive part of ``hmm_heatmaps.py`` and EM has local optima
with visibly different phase structures. Fitting it once here with many restarts
in separate processes, then caching the best parameters, makes the figures
reproducible without paying for the restarts on every render.

Writes ``analysis/agent_trace/local/hmm/hmm_pooled_k{K}.json`` with
``pi``, ``A``, ``B`` and the log-likelihood of the best restart.

Usage::

    OMP_NUM_THREADS=1 python3 analysis/agent_trace/hmm/pooled_fit.py --k 5 --restarts 16 --workers 16
"""

from __future__ import annotations

import argparse
import json
import sys
from concurrent.futures import ProcessPoolExecutor
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: E402
    CATEGORIES,
    CAT_IDX,
    RUNS,
    CategoricalHMM,
    _to_indices,
    case_sequences,
    discover_cases,
)
from analysis.agent_trace.hmm.hmm_heatmaps import name_states  # noqa: E402

OUT_DIR = ROOT / "analysis" / "agent_trace" / "local" / "hmm"
_OBS: list[np.ndarray] = []


def load_pooled() -> list[np.ndarray]:
    cases_by = {n: discover_cases(ROOT / p) for n, (p, _k) in RUNS.items()}
    shared = sorted(set.intersection(*(set(c) for c in cases_by.values())))
    obs = []
    for name, (_p, kind) in RUNS.items():
        for case in shared:
            obs.extend(_to_indices(case_sequences(cases_by[name][case], kind), CAT_IDX))
    return obs


def _fit(job: tuple[int, int]) -> tuple[float, int, list, list, list]:
    k, r = job
    h = CategoricalHMM(k, len(CATEGORIES), np.random.default_rng(1000 * k + r))
    ll = float(h.fit(_OBS))
    return ll, r, h.pi.tolist(), h.A.tolist(), h.B.tolist()


def main() -> None:
    ap = argparse.ArgumentParser(description="parallel multi-restart pooled HMM fit")
    ap.add_argument("--k", type=int, default=5)
    ap.add_argument("--restarts", type=int, default=16)
    ap.add_argument("--workers", type=int, default=16)
    args = ap.parse_args()

    global _OBS
    _OBS = load_pooled()
    n_obs = sum(len(s) for s in _OBS)
    print(f"{len(_OBS)} traces, {n_obs} tool calls, K={args.k}", flush=True)

    results = []
    with ProcessPoolExecutor(max_workers=args.workers) as ex:
        for fut in [ex.submit(_fit, (args.k, r)) for r in range(args.restarts)]:
            results.append(fut.result())
    results.sort(key=lambda x: -x[0])

    for ll, r, _pi, _A, B in results:
        labels = [name for name, _ in zip(name_states(np.array(B))[0], range(5))]
        print(f"  restart {r:2d}: loglik {ll:10.1f}   phases {sorted(labels)}", flush=True)

    ll, r, pi, A, B = results[0]
    path = OUT_DIR / f"hmm_pooled_k{args.k}.json"
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps({"k": args.k, "loglik": ll, "seed": 1000 * args.k + r,
                                "n_obs": n_obs, "pi": pi, "A": A, "B": B}, indent=2),
                    encoding="utf-8")
    print(f"\nbest: restart {r}, loglik {ll:.1f}")
    print(f"B = {np.round(np.array(B), 2).tolist()}")
    print(f"-> {path}")


if __name__ == "__main__":
    main()
