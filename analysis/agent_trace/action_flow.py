"""Action sequencing from raw tool-call symbols (no latent phases).

Collapses each tool call to one of three roles — R = read, W = write,
V = verify (mcp and build, i.e. any Lean toolchain feedback) — and measures how
agents sequence them. ``other`` calls are dropped.

Outputs
* ``analysis/agent_trace/local/action_flow/action_flow.json``  per-agent statistics
* ``figs``-style PNG at paper text width: one 3x3 transition matrix per agent
* a LaTeX table body printed to stdout

The transition matrices count every call transition in the agent's traces and
are shown with the self-loop removed on the off-diagonal (rows sum to 100%) and
the stay probability on the grey diagonal, matching the HMM figures.

Usage::

    python3 analysis/agent_trace/action_flow.py
"""

from __future__ import annotations

import json
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
import numpy as np  # noqa: E402
from matplotlib.patches import Rectangle  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: E402
    RUNS,
    _category_of,
    _iter_case_tool_calls,
    _vibe_category_of,
    discover_cases,
)
from analysis.agent_trace.hmm.hmm_heatmaps import (  # noqa: E402
    _INK,
    _INK_2,
    _NEUTRAL_CELL,
    _SURFACE,
    _ink_on,
    _pct,
)

OUT_DIR = ROOT / "analysis" / "agent_trace" / "local" / "action_flow"
FIG_W = 6.5  # inches: paper text width
ROLES = ("R", "W", "V")
ROLE_OF = {"read": "R", "write": "W", "mcp": "V", "build": "V"}
CATEGORY_OF = {"R": "read", "W": "write", "V": "mcp+build"}
ROLE_NAMES = {"R": "read", "W": "write", "V": "verify"}
WINDOW = 5  # a read is "used" if a write follows within this many calls


def roles_of(case_dir: Path, kind: str) -> list[str]:
    f = _vibe_category_of if kind == "vibe" else _category_of
    out = []
    for name, cmd, _fp in _iter_case_tool_calls(case_dir, kind):
        role = ROLE_OF.get(f(name, cmd))
        if role:
            out.append(role)
    return out


def verdict(case_dir: Path) -> str:
    p = case_dir / "verdict.txt"
    return p.read_text().strip().split()[0] if p.exists() else "?"


def _transitions(seq: list[str]) -> np.ndarray:
    idx = {r: i for i, r in enumerate(ROLES)}
    t = np.zeros((3, 3))
    for a, b in zip(seq[:-1], seq[1:]):
        t[idx[a], idx[b]] += 1
    return t


def collect() -> dict:
    agents = {}
    for name, (rel, kind) in RUNS.items():
        counts = np.zeros((3, 3))
        cases, n_calls = [], 0
        for _case, d in discover_cases(ROOT / rel).items():
            seq = roles_of(d, kind)
            if len(seq) < 5:
                continue
            t = _transitions(seq)
            counts += t
            n_calls += len(seq)
            cases.append({"roles": seq, "verdict": verdict(d), "calls": len(seq)})
        agents[name] = {"counts": counts, "n_calls": n_calls, "cases": cases}
    return agents


def per_case_metrics(seq: list[str]) -> dict:
    arr = np.array(seq)
    t = _transitions(seq)
    w_out = t[1].sum() - t[1, 1]
    v_out = t[2].sum() - t[2, 2]
    r_out = t[0].sum() - t[0, 0]
    used = sum("W" in seq[i + 1:i + 1 + WINDOW] for i, c in enumerate(seq) if c == "R")
    n_reads = int((arr == "R").sum())
    r_all = t[0].sum()  # every call spent in R, including the self-loop
    return {
        "share_R": float((arr == "R").mean()),
        "share_V": float((arr == "V").mean()),
        "W->V": t[1, 2] / w_out if w_out else np.nan,
        "V->W": t[2, 1] / v_out if v_out else np.nan,
        "V->R": t[2, 0] / v_out if v_out else np.nan,
        "R->W": t[0, 1] / r_out if r_out else np.nan,
        "R->R": t[0, 0] / r_all if r_all else np.nan,
        "read_used": used / n_reads if n_reads else np.nan,
        "first_write_at": (float(np.argmax(arr == "W")) if (arr == "W").any() else len(arr)) / len(arr),
    }


def summarise(agents: dict) -> dict:
    out = {}
    for name, d in agents.items():
        T = d["counts"]
        P = T / T.sum(1, keepdims=True)
        J = T.copy()
        np.fill_diagonal(J, 0)
        J = J / J.sum(1, keepdims=True)
        rows = [{**per_case_metrics(c["roles"]), "verdict": c["verdict"]} for c in d["cases"]]
        P_rows = [r for r in rows if r["verdict"] == "PROVED"]
        U_rows = [r for r in rows if r["verdict"] != "PROVED"]
        med = lambda rows_, k: float(np.nanmedian([r[k] for r in rows_])) if rows_ else np.nan  # noqa: E731
        out[name] = {
            "n_calls": d["n_calls"],
            "n_cases": len(d["cases"]),
            "n_proved": len(P_rows),
            "A": P.tolist(),
            "exit": J.tolist(),
            "share_R": float(T[0].sum() / T.sum()),
            "share_W": float(T[1].sum() / T.sum()),
            "share_V": float(T[2].sum() / T.sum()),
            "R->W_med": med(rows, "R->W"),
            "R->R_med": med(rows, "R->R"),
            "W->V_med": med(rows, "W->V"),
            "V->R_med": med(rows, "V->R"),
            "read_used": float(np.nanmean([r["read_used"] for r in rows])),
            "proved": {
                "n": len(P_rows),
                "share_R": med(P_rows, "share_R"),
                "share_V": med(P_rows, "share_V"),
                "R->W": med(P_rows, "R->W"),
                "read_used": float(np.nanmean([r["read_used"] for r in P_rows])) if P_rows else np.nan,
            },
            "unproved": {
                "n": len(U_rows),
                "share_R": med(U_rows, "share_R"),
                "share_V": med(U_rows, "share_V"),
                "R->W": med(U_rows, "R->W"),
                "read_used": float(np.nanmean([r["read_used"] for r in U_rows])) if U_rows else np.nan,
            },
        }
    return out


def render(stats: dict, out: Path) -> None:
    names = list(stats)
    ncols, nrows = 4, 2
    cell_w, gap = 1.30, 0.80
    fig_h = 0.35 + 0.70 + nrows * cell_w + (nrows - 1) * gap
    fig, axes = plt.subplots(nrows, ncols, figsize=(FIG_W, fig_h), squeeze=False)
    labels = [f"{ROLE_NAMES[r]}" for r in ROLES]
    for idx, name in enumerate(names):
        ax = axes[idx // ncols][idx % ncols]
        A = np.array(stats[name]["A"])
        ax.imshow(A, cmap="Blues", vmin=0, vmax=1, aspect="equal", interpolation="nearest")
        for r in range(3):
            for c in range(3):
                v = float(A[r, c])
                ax.text(c, r, _pct(v), ha="center", va="center", fontsize=7.5,
                        color=_ink_on(v), fontweight="bold" if .5 <= v < .995 else "normal")
        ax.set_xticks(range(3))
        ax.set_xticklabels(labels if idx // ncols == nrows - 1 else [], fontsize=7.5, color=_INK_2)
        ax.set_yticks(range(3))
        ax.set_yticklabels(labels if idx % ncols == 0 else [], fontsize=7.5, color=_INK_2)
        ax.set_xticks(np.arange(-.5, 3, 1), minor=True)
        ax.set_yticks(np.arange(-.5, 3, 1), minor=True)
        ax.grid(which="minor", color=_SURFACE, linewidth=2)
        ax.tick_params(which="both", length=0)
        for s in ax.spines.values():
            s.set_visible(False)
        n = stats[name]["n_calls"]
        ax.set_title(f"{name}\n{n / 1000:.1f}k calls", fontsize=7.5, fontweight="bold",
                     color=_INK, loc="left", pad=3, linespacing=1.2)
        if idx % ncols == 0:
            ax.set_ylabel("from", fontsize=8, color=_INK_2)
        if idx // ncols == nrows - 1:
            ax.set_xlabel("to", fontsize=8, color=_INK_2, labelpad=2)
    fig.subplots_adjust(left=0.10, right=0.995, top=1 - 0.30 / fig_h, bottom=0.60 / fig_h,
                        wspace=0.10, hspace=gap / cell_w)
    out.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out, dpi=300)
    plt.close(fig)
    print(f"-> {out}")


def main() -> None:
    agents = collect()
    stats = summarise(agents)
    OUT_DIR.mkdir(parents=True, exist_ok=True)
    (OUT_DIR / "action_flow.json").write_text(json.dumps(stats, indent=2), encoding="utf-8")
    render(stats, OUT_DIR / "action_flow.png")

    print("\n=== per agent (call-weighted transitions, per-case medians) ===")
    print(f"{'agent':20s} {'calls':>6} {'proved':>7} {'R%':>4} {'W%':>4} {'V%':>4} "
          f"{'W->V':>6} {'V->R':>6} {'R->W':>6} {'R->R':>6} {'read used':>9}")
    for name, s in stats.items():
        print(f"{name:20s} {s['n_calls']:6d} {s['n_proved']:3d}/{s['n_cases']:<3d} "
              f"{100 * s['share_R']:3.0f} {100 * s['share_W']:3.0f} {100 * s['share_V']:3.0f} "
              f"{s['W->V_med']:6.2f} {s['V->R_med']:6.2f} {s['R->W_med']:6.2f} "
              f"{s['R->R_med']:6.2f} {100 * s['read_used']:8.0f}%")

    print("\n=== within agent: proved vs unproved (medians) ===")
    print(f"{'agent':20s} {'nP/nU':>9} {'R%_P':>6} {'R%_U':>6} {'used_P':>7} {'used_U':>7}")
    for name, s in stats.items():
        P, U = s["proved"], s["unproved"]
        if P["n"] < 5 or U["n"] < 5:
            continue
        print(f"{name:20s} {P['n']:4d}/{U['n']:<4d} {100 * P['share_R']:6.1f} {100 * U['share_R']:6.1f} "
              f"{100 * P['read_used']:6.0f}% {100 * U['read_used']:6.0f}%")

    print("\n=== LaTeX table body ===")
    for name, s in stats.items():
        print(f"        {name:15s} & {s['n_calls']:6d} & {100 * s['share_R']:4.1f} & "
              f"{100 * s['share_V']:4.1f} & {s['W->V_med']:4.2f} & {s['V->R_med']:4.2f} & "
              f"{s['R->W_med']:4.2f} & {100 * s['read_used']:4.0f} \\\\")


if __name__ == "__main__":
    main()
