"""Parallel BIC sweep over the number of HMM states.

``agent_behavior.fit_best`` fits K = 2..MAX_K sequentially; with BIC still falling at
the upper bound that is not enough to *select* K. This script fits every
(K, restart) pair in its own process (one numpy thread each), keeps the best
restart per K, and writes the BIC table to ``k_sweep.json`` and stdout.

Usage::

    OMP_NUM_THREADS=1 python3 analysis/agent_trace/hmm/k_sweep.py              # K = 2..8
    OMP_NUM_THREADS=1 python3 analysis/agent_trace/hmm/k_sweep.py --max-k 10 --workers 48
"""

from __future__ import annotations

import argparse
import json
import sys
from concurrent.futures import ProcessPoolExecutor, as_completed
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: E402
    CATEGORIES,
    CAT_IDX,
    CategoricalHMM,
    N_RESTARTS,
    RUNS,
    _bic,
    _to_indices,
    case_sequences,
    discover_cases,
)

OUT = ROOT / "analysis" / "agent_trace" / "local" / "hmm" / "k_sweep.json"
_OBS: list[np.ndarray] = []


def _load() -> list[np.ndarray]:
    cases_by = {n: discover_cases(ROOT / p) for n, (p, _k) in RUNS.items()}
    shared = sorted(set.intersection(*(set(c) for c in cases_by.values())))
    obs = []
    for name, (_p, kind) in RUNS.items():
        for case in shared:
            obs.extend(_to_indices(case_sequences(cases_by[name][case], kind), CAT_IDX))
    return obs


def _fit(job: tuple[int, int]) -> tuple[int, int, float]:
    k, r = job
    h = CategoricalHMM(k, len(CATEGORIES), np.random.default_rng(1000 * k + r))
    return k, r, float(h.fit(_OBS))


def main() -> None:
    ap = argparse.ArgumentParser(description="parallel BIC sweep over HMM state count")
    ap.add_argument("--min-k", type=int, default=2)
    ap.add_argument("--max-k", type=int, default=8)
    ap.add_argument("--restarts", type=int, default=N_RESTARTS)
    ap.add_argument("--workers", type=int, default=32)
    args = ap.parse_args()

    global _OBS
    _OBS = _load()
    n_obs = sum(len(s) for s in _OBS)
    print(f"{len(_OBS)} traces, {n_obs} tool calls", flush=True)

    jobs = [(k, r) for k in range(args.max_k, args.min_k - 1, -1) for r in range(args.restarts)]
    best: dict[int, float] = {}
    lls: dict[int, list[float]] = {}
    with ProcessPoolExecutor(max_workers=args.workers) as ex:  # fork: workers inherit _OBS
        for fut in as_completed([ex.submit(_fit, j) for j in jobs]):
            k, r, ll = fut.result()
            lls.setdefault(k, []).append(ll)
            best[k] = max(best.get(k, -np.inf), ll)
            print(f"  K={k} restart {r}: loglik={ll:.1f}", flush=True)

    rows = []
    print("\n  K   best loglik        BIC   dBIC vs K-1   restart spread")
    prev = None
    for k in sorted(best):
        bic = _bic(best[k], k, len(CATEGORIES), n_obs)
        d = "" if prev is None else f"{bic - prev:+.0f}"
        print(f"{k:>3} {best[k]:>13.1f} {bic:>10.1f} {d:>13} {max(lls[k]) - min(lls[k]):>14.1f}")
        rows.append({"k": k, "loglik": best[k], "bic": bic, "restart_logliks": lls[k]})
        prev = bic
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps({"n_obs": n_obs, "rows": rows}, indent=2), encoding="utf-8")
    print(f"-> {OUT}")


if __name__ == "__main__":
    main()
