"""Symbol-level test of the write--verify vs read-loop claim (no latent phases).

The HMM assigns phases; this script asks the same question on the raw tool-call
symbols, so the result does not depend on K or on the EM optimum.

Symbols are collapsed into three roles: R = read, W = write, V = verify
(mcp and build, both being feedback from the Lean toolchain); ``other`` calls are
skipped. For every trace we measure

* the share of calls in each role,
* P(W -> V) and P(V -> W | V leaves) / P(V -> R | V leaves), the edges of the two
  proposed loops,
* the length of read runs that end in a write (``bounded`` reads) versus read runs
  that end in another read-heavy action, i.e. an exploration stretch that returns
  a draft,
* the position of the first write, and the share of reads followed by a write
  within a small window.

All traces are then split by their final verdict (PROVED vs not), which controls
for the agent: within one agent, the same comparison can be made between cases.

Usage::

    python3 analysis/agent_trace/hmm/raw_loops.py
"""

from __future__ import annotations

import sys
from collections import defaultdict
from pathlib import Path

import numpy as np

ROOT = Path(__file__).resolve().parent.parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: E402
    RUNS,
    _iter_case_tool_calls,
    _vibe_category_of,
    _category_of,
    discover_cases,
)

ROLE = {"read": "R", "write": "W", "mcp": "V", "build": "V"}
WINDOW = 5  # calls after a read within which a write counts as "using" that read


def trace_roles(case_dir: Path, kind: str) -> list[str]:
    f = _vibe_category_of if kind == "vibe" else _category_of
    out = []
    for name, cmd, _fp in _iter_case_tool_calls(case_dir, kind):
        role = ROLE.get(f(name, cmd))
        if role:
            out.append(role)
    return out


def verdict(case_dir: Path) -> str:
    p = case_dir / "verdict.txt"
    return p.read_text().strip().split()[0] if p.exists() else "?"


def metrics(roles: list[str]) -> dict:
    n = len(roles)
    if n < 5:
        return {}
    arr = np.array(roles)
    share = {c: float((arr == c).mean()) for c in "RWV"}
    # adjacency counts, skipping nothing (roles already exclude ``other``)
    trans = defaultdict(int)
    for a, b in zip(arr[:-1], arr[1:]):
        trans[(a, b)] += 1
    w_out = sum(trans[("W", c)] for c in "RWV")
    v_out = trans[("V", "R")] + trans[("V", "W")]
    r_out = sum(trans[("R", c)] for c in "RWV")

    # a read is "used" if a write follows within WINDOW calls
    used = 0
    for i, c in enumerate(arr):
        if c == "R":
            if "W" in arr[i + 1:i + 1 + WINDOW]:
                used += 1

    first_w = int(np.argmax(arr == "W")) if (arr == "W").any() else n
    return {
        "n": n,
        **{f"share_{k}": v for k, v in share.items()},
        "W->V": trans[("W", "V")] / w_out if w_out else np.nan,
        "V->W": trans[("V", "W")] / v_out if v_out else np.nan,
        "V->R": trans[("V", "R")] / v_out if v_out else np.nan,
        "R->W": trans[("R", "W")] / r_out if r_out else np.nan,
        "R->R": trans[("R", "R")] / r_out if r_out else np.nan,
        "read_used": used / (arr == "R").sum() if (arr == "R").any() else np.nan,
        "first_write_at": first_w / n,
    }


def mean(rows: list[dict], keys: list[str]) -> dict:
    out = {}
    for k in keys:
        vals = [r[k] for r in rows if k in r and not (isinstance(r[k], float) and np.isnan(r[k]))]
        out[k] = float(np.mean(vals)) if vals else np.nan
    return out


def main() -> None:
    keys = ["n", "share_R", "share_W", "share_V", "W->V", "V->W", "V->R", "R->W",
            "R->R", "read_used", "first_write_at"]
    per_agent: dict[str, list[dict]] = {}
    for name, (p, kind) in RUNS.items():
        rows = []
        for case, d in discover_cases(ROOT / p).items():
            m = metrics(trace_roles(d, kind))
            if m:
                m["verdict"] = verdict(d)
                rows.append(m)
        per_agent[name] = rows

    print(f"\n=== raw-symbol transitions (R read, W write, V verify=mcp+build), "
          f"read 'used' if a write follows within {WINDOW} calls ===")
    hdr = (f"{'agent':20s} {'cases':>5} {'R%':>5} {'W%':>5} {'V%':>5} "
           f"{'W->V':>6} {'V->W':>6} {'V->R':>6} {'R->W':>6} {'R->R':>6} {'read used':>9} {'1stW':>5}")
    print(hdr)
    pooled = {}
    for name, rows in per_agent.items():
        m = mean(rows, keys)
        n_proved = sum(r["verdict"] == "PROVED" for r in rows)
        print(f"{name:20s} {len(rows):5d} " + " ".join(
            f"{100 * m[k]:5.0f}" if k.startswith("share_") else
            (f"{100 * m[k]:5.0f}" if k in ("W->V", "V->W", "V->R", "R->W", "R->R", "read_used", "first_write_at") else f"{m[k]:5.0f}")
            for k in ["share_R", "share_W", "share_V", "W->V", "V->W", "V->R", "R->W", "R->R", "read_used", "first_write_at"])
              + f"   ({n_proved} proved)")
        pooled[name] = m

    # within-agent contrast: proved vs not, per agent with enough of both
    print("\n=== within-agent: PROVED vs not (percentage points, proved - not) ===")
    print(f"{'agent':20s} {'nP/nU':>9} " + " ".join(f"{k:>9}" for k in
          ["share_R", "share_V", "read_used", "R->R", "first_write_at", "n"]))
    contrast = {}
    for name, rows in per_agent.items():
        P = [r for r in rows if r["verdict"] == "PROVED"]
        U = [r for r in rows if r["verdict"] != "PROVED"]
        if len(P) < 5 or len(U) < 5:
            continue
        mp, mu = mean(P, keys), mean(U, keys)
        d = {k: mp[k] - mu[k] for k in keys}
        contrast[name] = d
        print(f"{name:20s} {len(P):4d}/{len(U):<4d} " + " ".join(
            f"{100 * d[k]:9.0f}" if k not in ("n",) else f"{d[k]:9.0f}"
            for k in ["share_R", "share_V", "read_used", "R->R", "first_write_at", "n"]))

    # cross-agent correlation of the two candidate separators with proof rate
    print("\n=== cross-agent correlation with proof rate ===")
    names = [n for n in per_agent if pooled[n]["n"] > 0]
    rate = np.array([sum(r["verdict"] == "PROVED" for r in per_agent[n]) / len(per_agent[n]) for n in names])
    for k in ["R->W", "V->R", "read_used", "share_R", "share_V"]:
        x = np.array([pooled[n][k] for n in names])
        ok = ~np.isnan(x)
        r = np.corrcoef(x[ok], rate[ok])[0, 1]
        print(f"  corr(proof rate, {k:>10s}) = {r:+.2f}")


if __name__ == "__main__":
    main()
