"""Emission & transition-matrix figures for the fitted HMM.

Two stages, so the (slow) fit is never repeated just to restyle a plot:

1. **fit**    — pool every agent's tool-call sequences, fit one shared HMM, refit a
   per-agent transition matrix on the shared emissions, decode each agent, and
   cache everything to ``hmm_fit_k{K}.json``.
2. **render** — read that JSON and draw four PNGs:

   * ``hmm_emission.png``          shared emission matrix ``B`` + pooled occupancy
   * ``hmm_transitions.png``       per-agent transition matrix ``A`` (self-loops outlined)
   * ``hmm_transitions_jump.png``  per-agent *exit* matrix: diagonal = probability of
     staying in the phase at the next call, off-diagonal = where the agent goes once it leaves (self-loop removed)
   * ``hmm_transition_graphs.png`` per-agent phase graph of the exit chain; node area =
     share of calls, arrow width = exit probability

States get stable, unique names (``read``, ``write``, ``write+read``, ``mcp``,
``mcp+read``) and a canonical order, so panels line up across agents
and across re-fits.

All magnitudes use the one-hue sequential blue ramp (light = 0, dark = 1).

Usage::

    python3 analysis/agent_trace/hmm/hmm_heatmaps.py                # fit (K=5) + render
    python3 analysis/agent_trace/hmm/hmm_heatmaps.py --render-only  # re-draw from cached fit
    python3 analysis/agent_trace/hmm/hmm_heatmaps.py --k 4
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import matplotlib

matplotlib.use("Agg")
import matplotlib.pyplot as plt  # noqa: E402
from matplotlib.colors import LinearSegmentedColormap  # noqa: E402
from matplotlib.patches import Circle, FancyArrowPatch, Rectangle  # noqa: E402

import numpy as np  # noqa: E402

ROOT = Path(__file__).resolve().parent.parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm.agent_behavior import (  # noqa: E402
    CATEGORIES,
    CAT_IDX,
    CategoricalHMM,
    EXTRA_RUNS,
    N_RESTARTS,
    RUNS,
    _decoded_occupancy,
    _to_indices,
    case_sequences,
    discover_cases,
    group_transition_test,
)

OUT_DIR = ROOT / "analysis" / "agent_trace" / "local" / "hmm"
EMISSION_W = 3.8  # inches: printed width (0.6 of the text width) of the emission figure
RESERVED_SLOTS = 8  # grid panels kept free for agents still being evaluated (Leanstral)
FIG_W = 6.5  # inches: the paper text width, so fonts are printed at their stated size
# Run identifier -> agent name used in the paper's tables and text.
DISPLAY_NAMES = {
    "claude-opus-5.5": "Opus 5.5",
    "claude-opus-4.5": "Opus 4.5",
    "gpt-6-sol": "GPT 6",
    "gpt-5.6-sol": "GPT 5.6",
    "deepseek-v4.1-flash": "DS V4.1",
    "glm-5.3": "GLM 5.3",
    "kimi-k3": "Kimi K3",
    "leanstral-1.5": "Leanstral 1.5",
}

_RAMP = [
    (0.00, "#fcfcfb"), (0.20, "#cde2fb"), (0.35, "#9ec5f4"), (0.50, "#6da7ec"),
    (0.65, "#3987e5"), (0.80, "#256abf"), (0.90, "#1c5cab"), (1.00, "#0d366b"),
]
_INK = "#0b0b0b"
_INK_2 = "#52514e"
_MUTED = "#8a8985"
_SURFACE = "#ffffff"
_NEUTRAL_CELL = "#ececea"  # diagonal cells in the exit matrix (not a probability)
_CMAP = LinearSegmentedColormap.from_list("seq_blue", _RAMP)
_EDGE = "#256abf"
_NODE = "#3987e5"


# --- colour / text helpers --------------------------------------------------------


def _luminance(rgb) -> float:
    def lin(c: float) -> float:
        return c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4

    return 0.2126 * lin(rgb[0]) + 0.7152 * lin(rgb[1]) + 0.0722 * lin(rgb[2])


def _ink_on(v: float) -> str:
    r, g, b, _ = _CMAP(v)
    return _INK if _luminance((r, g, b)) > 0.18 else "#ffffff"


def _pct(v: float, unit: str = "%") -> str:
    if v < 0.005:
        return "·"
    if v < 0.01:
        return f"<1{unit}"
    return f"{100 * v:.0f}{unit}"


def _short(n: float) -> str:
    return f"{n / 1000:.1f}k" if n >= 1000 else f"{n:.0f}"


# --- state naming & ordering ------------------------------------------------------


def name_states(B: np.ndarray) -> tuple[list[str], np.ndarray]:
    """Unique names and a canonical order (by dominant symbol, then runner-up)."""
    names, keys = [], []
    for row in B:
        order = np.argsort(-row)
        d, s = int(order[0]), int(order[1])
        solo = row[d] >= 0.8
        names.append(CATEGORIES[d] if solo else f"{CATEGORIES[d]}+{CATEGORIES[s]}")
        keys.append((d, -1 if solo else s))
    # Disambiguate collisions by the runner-up symbol: within a group sharing a name,
    # the purest state (largest dominant share) keeps the plain name and the others get
    # ``+runner-up``, so the suffix always marks the state that mixes in more of it.
    for n in set(names):
        group = [i for i, m in enumerate(names) if m == n]
        if len(group) < 2:
            continue
        purest = max(group, key=lambda i: B[i].max())
        for i in group:
            if i == purest:
                continue
            d, s = (int(x) for x in np.argsort(-B[i])[:2])
            names[i] = f"{CATEGORIES[d]}+{CATEGORIES[s]}"
            keys[i] = (d, s)
    return names, np.array(sorted(range(len(B)), key=lambda i: keys[i]))


def relabel(data: dict) -> dict:
    """Re-derive state names and canonical order from a cached fit's emissions.

    Cached fits store names from whatever ``name_states`` did at fit time; this keeps
    ``--render-only`` consistent with the current naming rule without refitting.
    """
    names, order = name_states(np.array(data["B"]))
    ix = np.ix_(order, order)
    data["state_names"] = [names[i] for i in order]
    data["B"] = np.array(data["B"])[order].tolist()
    data["pooled_occupancy"] = np.array(data["pooled_occupancy"])[order].tolist()
    for d in data["agents"].values():
        d["A"] = np.array(d["A"])[ix].tolist()
        d["occupancy"] = np.array(d["occupancy"])[order].tolist()
    return data


# --- fit stage --------------------------------------------------------------------


def fit_and_cache(k: int, path: Path) -> dict:
    cases_by = {n: discover_cases(ROOT / p) for n, (p, _k) in RUNS.items()}
    shared = sorted(set.intersection(*(set(c) for c in cases_by.values())))
    if not shared:
        sys.exit("no cases shared across all runs")
    agents = {}
    for name, (_p, kind) in RUNS.items():
        seqs = []
        for case in shared:
            seqs.extend(_to_indices(case_sequences(cases_by[name][case], kind), CAT_IDX))
        agents[name] = seqs
    pooled = [s for seqs in agents.values() for s in seqs]

    # The pooled fit has visibly different local optima, so use the best solution found
    # by pooled_fit.py (many restarts) when it covers exactly this pooled corpus.
    rng = np.random.default_rng(0)
    cache = path.parent / f"hmm_pooled_k{k}.json"
    n_pooled = sum(len(s) for s in pooled)
    cached = json.loads(cache.read_text()) if cache.exists() else None
    if cached and cached["n_obs"] == n_pooled:
        hmm = CategoricalHMM(k, len(CATEGORIES), rng)
        hmm.pi, hmm.A, hmm.B = (np.array(cached[x]) for x in ("pi", "A", "B"))
        best_ll = cached["loglik"]
        print(f"loaded pooled fit {cache.name}: log-lik {best_ll:.1f}", file=sys.stderr, flush=True)
    else:
        print(f"fitting pooled HMM (K={k}) ...", file=sys.stderr, flush=True)
        hmm, best_ll = None, -np.inf
        for _ in range(N_RESTARTS):
            cand = CategoricalHMM(k, len(CATEGORIES), rng)
            ll = cand.fit(pooled)
            if ll > best_ll:
                hmm, best_ll = cand, ll
        print(f"pooled log-lik {best_ll:.1f}", file=sys.stderr, flush=True)
    print("refitting per-agent transition matrices ...", file=sys.stderr, flush=True)
    lrt = group_transition_test(hmm.B, k, len(CATEGORIES), agents, init=(hmm.A, hmm.pi))

    names, order = name_states(hmm.B)
    ix = np.ix_(order, order)
    data = {
        "k": k,
        "n_cases": len(shared),
        "categories": list(CATEGORIES),
        "state_names": [names[i] for i in order],
        "B": hmm.B[order].tolist(),
        "pooled_occupancy": (_decoded_occupancy(hmm, pooled)[0][order]).tolist(),
        "agents": {},
        "lrt": {x: lrt[x] for x in ("lrt", "df", "p")},
    }
    for name, seqs in agents.items():
        model = lrt["models"][name]
        occ, _ = _decoded_occupancy(hmm, seqs)  # shared phases decode every agent
        data["agents"][name] = {
            "n_calls": int(sum(len(s) for s in seqs)),
            "n_cases": len(shared),
            "A": model.A[ix].tolist(),
            "occupancy": occ[order].tolist(),
        }
    # Held-out agents: phases and emissions stay those of the pooled fit; only the
    # agent's own transition matrix is estimated (warm-started from the shared one).
    extra_case_names = set()
    for name, (p, kind) in EXTRA_RUNS.items():
        cases = discover_cases(ROOT / p)
        used = [c for c in shared if c in cases]
        extra_case_names.update(used)
        seqs = []
        for case in used:
            seqs.extend(_to_indices(case_sequences(cases[case], kind), CAT_IDX))
        print(f"{name}: {len(used)}/{len(shared)} shared cases, "
              f"{sum(len(s) for s in seqs)} calls", file=sys.stderr, flush=True)
        model = CategoricalHMM(k, len(CATEGORIES), rng)
        model.fit_fixed_B(hmm.B, seqs, init=(lrt["h0"].A, lrt["h0"].pi))
        occ, _ = _decoded_occupancy(hmm, seqs)
        data["agents"][name] = {
            "n_calls": int(sum(len(s) for s in seqs)),
            "n_cases": len(used),
            "A": model.A[ix].tolist(),
            "occupancy": occ[order].tolist(),
        }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(data, indent=2), encoding="utf-8")
    print(f"-> {path}")
    return data


# --- shared drawing ---------------------------------------------------------------


def _style_axes(ax, rows: list[str], cols: list[str], fs: int, rot: int = 0) -> None:
    k, m = len(rows), len(cols)
    ax.set_xticks(np.arange(m))
    ax.set_xticklabels(cols, fontsize=fs, color=_INK_2, rotation=rot,
                       ha="right" if rot else "center", rotation_mode="anchor")
    ax.set_yticks(np.arange(k))
    ax.set_yticklabels(rows, fontsize=fs, color=_INK_2)
    ax.set_xticks(np.arange(-0.5, m, 1), minor=True)
    ax.set_yticks(np.arange(-0.5, k, 1), minor=True)
    ax.grid(which="minor", color=_SURFACE, linewidth=2)
    ax.tick_params(which="both", length=0)
    for s in ax.spines.values():
        s.set_visible(False)


def _prob_cells(ax, M: np.ndarray, fs: int, outline_diag: bool = False, unit: str = "%") -> None:
    ax.imshow(M, cmap=_CMAP, vmin=0, vmax=1, aspect="equal", interpolation="nearest")
    k, m = M.shape
    for r in range(k):
        for c in range(m):
            v = float(M[r, c])
            ax.text(c, r, _pct(v, unit), ha="center", va="center", fontsize=fs,
                    color=_ink_on(v), fontweight="bold" if v >= 0.5 else "normal")
    if outline_diag:
        for i in range(min(k, m)):
            ax.add_patch(Rectangle((i - .5, i - .5), 1, 1, fill=False,
                                   ec=_INK_2, lw=1.4, zorder=3))


def _colorbar(fig, axes, label: str) -> None:
    sm = plt.cm.ScalarMappable(cmap=_CMAP, norm=plt.Normalize(0, 1))
    cb = fig.colorbar(sm, ax=axes, fraction=0.018, pad=0.015, ticks=[0, .25, .5, .75, 1])
    cb.ax.set_yticklabels(["0%", "25%", "50%", "75%", "100%"])
    cb.ax.tick_params(labelsize=8, colors=_INK_2)
    cb.set_label(label, fontsize=9, color=_INK_2)
    cb.outline.set_linewidth(0.5)


def _panel_title(name: str, d: dict, data: dict, sep: str) -> str:
    """Agent name plus call count; the case count is added only when it is not the shared set."""
    extra = f", {d['n_cases']} cases" if d["n_cases"] != data["n_cases"] else ""
    return f"{DISPLAY_NAMES.get(name, name)}{sep}{_short(d['n_calls'])} calls{extra}"


def _exit_matrix(A: np.ndarray) -> tuple[np.ndarray, np.ndarray]:
    """Row-normalised off-diagonal transitions and per-state stay probability."""
    stay = np.diag(A).copy()
    J = A.copy()
    np.fill_diagonal(J, 0.0)
    tot = J.sum(1, keepdims=True)
    J = np.divide(J, tot, out=np.zeros_like(J), where=tot > 1e-12)
    return J, stay


# --- figure 1: emission -----------------------------------------------------------


def render_emission(data: dict, out: Path) -> None:
    B = np.array(data["B"])
    names = data["state_names"]
    occ = np.array(data["pooled_occupancy"])
    occ = occ / occ.sum()
    k = len(names)
    # Native width = paper text width (6.5 in), so font sizes below are the printed sizes.
    fig = plt.figure(figsize=(EMISSION_W, 0.75 + 0.35 * k))
    gs = fig.add_gridspec(1, 3, width_ratios=[5, 1.5, 0.14], wspace=0.14,
                          left=0.25, right=0.90, top=0.91, bottom=0.21)
    ax, bx, cax = fig.add_subplot(gs[0]), fig.add_subplot(gs[1]), fig.add_subplot(gs[2])
    _prob_cells(ax, B, 7.5)
    _style_axes(ax, names, list(CATEGORIES), 7.5)
    ax.set_ylabel("phase", fontsize=8, color=_INK_2)
    ax.set_xlabel("tool-call type", fontsize=8, color=_INK_2, labelpad=4)

    y = np.arange(k)
    bx.barh(y, occ, height=0.62, color=_NODE, edgecolor=_SURFACE, linewidth=1.5)
    for yi, v in zip(y, occ):
        bx.text(v + 0.03, yi, f"{100 * v:.0f}%", va="center", fontsize=7.5, color=_INK)
    bx.set_ylim(k - 0.5, -0.5)
    bx.set_xlim(0, max(occ.max() * 1.6, 0.3))
    bx.set_yticks([])
    bx.set_xticks([])
    for sp in bx.spines.values():
        sp.set_visible(False)
    bx.set_title("share of calls", fontsize=7.5, color=_INK_2, loc="left")
    sm = plt.cm.ScalarMappable(cmap=_CMAP, norm=plt.Normalize(0, 1))
    cb = fig.colorbar(sm, cax=cax, ticks=[0, .5, 1])
    cb.ax.set_yticklabels(["0%", "50%", "100%"])
    cb.ax.tick_params(labelsize=7, colors=_INK_2)
    cb.outline.set_linewidth(0.5)
    fig.savefig(out, dpi=300)
    plt.close(fig)
    print(f"-> {out}")


# --- figures 2 & 3: per-agent matrix grids ----------------------------------------


def _grid(data: dict, out: Path, kind: str) -> None:
    names = data["state_names"]
    agents = data["agents"]
    k = len(names)
    n = len(agents)
    ncols = 4
    n_slots = max(n, RESERVED_SLOTS)  # keep room for agents that are still being evaluated
    nrows = (n_slots + ncols - 1) // ncols
    # Native width = paper text width (6.5 in), so font sizes below are the printed sizes.
    cell_w, label_gap = 1.29, 0.45  # inches: panel side, space for titles + rotated labels
    # No colour bar: every cell is labelled with its value, so the scale is redundant.
    fig_h = 0.35 + 0.75 + nrows * cell_w + (nrows - 1) * label_gap
    fig, axes = plt.subplots(nrows, ncols, figsize=(FIG_W, fig_h), squeeze=False)
    for idx, (name, d) in enumerate(agents.items()):
        ax = axes[idx // ncols][idx % ncols]
        A = np.array(d["A"])
        if kind == "A":
            _prob_cells(ax, A, 6, outline_diag=True, unit="")  # caption states the unit
        else:
            J, stay = _exit_matrix(A)
            ax.imshow(J, cmap=_CMAP, vmin=0, vmax=1, aspect="equal", interpolation="nearest")
            for r in range(k):
                for c in range(k):
                    if r == c:
                        ax.add_patch(Rectangle((c - .5, r - .5), 1, 1, fc=_NEUTRAL_CELL,
                                               ec="none", zorder=2))
                        ax.text(c, r, _pct(stay[r]), ha="center", va="center",
                                fontsize=6, color=_INK, fontstyle="italic", zorder=4)
                    else:
                        v = float(J[r, c])
                        ax.text(c, r, _pct(v), ha="center", va="center", fontsize=6,
                                color=_ink_on(v), fontweight="bold" if .5 <= v < .995 else "normal",
                                zorder=4)
        _style_axes(ax, names, names, 7, rot=45)
        first_col = idx % ncols == 0
        bottom_of_column = idx + ncols >= n_slots
        if not first_col:
            ax.set_yticklabels([])
        if not bottom_of_column:
            ax.set_xticklabels([])
        ax.set_title(_panel_title(name, d, data, "\n"), fontsize=7.5, fontweight="bold",
                     color=_INK, loc="left", pad=3, linespacing=1.2)
        if first_col:
            ax.set_ylabel("from", fontsize=8, color=_INK_2)
        if bottom_of_column:
            ax.set_xlabel("to", fontsize=8, color=_INK_2, labelpad=2)
    spare = [axes[i // ncols][i % ncols] for i in range(n, n_slots)]
    for ax in spare:
        ax.axis("off")
    fig.subplots_adjust(left=0.14, right=0.995, top=1 - 0.35 / fig_h, bottom=0.75 / fig_h, wspace=0.10, hspace=label_gap / cell_w)
    fig.savefig(out, dpi=300)
    plt.close(fig)
    print(f"-> {out}")


# --- figure 4: phase graphs -------------------------------------------------------


def render_graphs(data: dict, out: Path, min_exit: float = 0.15, label_exit: float = 0.40) -> None:
    names = data["state_names"]
    k = len(names)
    agents = data["agents"]
    ncols = 4
    nrows = (len(agents) + ncols - 1) // ncols
    fig, axes = plt.subplots(nrows, ncols, figsize=(5.2 * ncols, 4.8 * nrows), squeeze=False)
    ang = np.pi / 2 - 2 * np.pi * np.arange(k) / k  # clockwise from top
    pos = np.c_[np.cos(ang), np.sin(ang)]
    R_MAX = 0.30

    for idx, (name, d) in enumerate(agents.items()):
        ax = axes[idx // ncols][idx % ncols]
        ax.set_xlim(-2.0, 2.0)
        ax.set_ylim(-1.75, 1.8)
        ax.set_aspect("equal")
        ax.axis("off")
        A = np.array(d["A"])
        J, stay = _exit_matrix(A)
        occ = np.array(d["occupancy"], float)
        occ /= max(occ.sum(), 1)
        rad = R_MAX * np.sqrt(occ / max(occ.max(), 1e-9))
        rad = np.maximum(rad, 0.05)

        edges = [(i, j, J[i, j]) for i in range(k) for j in range(k)
                 if i != j and J[i, j] >= min_exit]
        for i, j, p in sorted(edges, key=lambda e: e[2]):
            v = pos[j] - pos[i]
            u = v / np.linalg.norm(v)
            a = pos[i] + u * (rad[i] + 0.03)
            b = pos[j] - u * (rad[j] + 0.07)
            arrow = FancyArrowPatch(a, b, connectionstyle="arc3,rad=0.22",
                                    arrowstyle="-|>", mutation_scale=9 + 10 * p,
                                    lw=0.8 + 4.2 * p, color=_EDGE, alpha=0.35 + 0.6 * p,
                                    zorder=2, shrinkA=0, shrinkB=0)
            ax.add_patch(arrow)
            if p >= label_exit:
                mid = (a + b) / 2
                nrm = np.array([-u[1], u[0]])
                lp = mid - nrm * 0.17  # arc3 with +rad bends to the right of travel
                ax.text(lp[0], lp[1], f"{100 * p:.0f}%", fontsize=8.5, color=_INK,
                        ha="center", va="center", zorder=5,
                        bbox=dict(boxstyle="round,pad=0.12", fc=_SURFACE, ec="none", alpha=0.9))
        for i in range(k):
            ax.add_patch(Circle(pos[i], rad[i], fc=_NODE, ec=_SURFACE, lw=2, zorder=4))
            out_v = pos[i] / (np.linalg.norm(pos[i]) or 1)
            tp = pos[i] + out_v * (rad[i] + 0.12)
            ha = "center" if abs(out_v[0]) < 0.3 else ("left" if out_v[0] > 0 else "right")
            shift = 0.18 if out_v[1] > 0.3 else (-0.18 if out_v[1] < -0.3 else 0.0)
            ax.text(tp[0], tp[1] + shift, names[i], ha=ha, va="bottom", fontsize=9.5,
                    color=_INK, fontweight="bold", zorder=6)
            ax.text(tp[0], tp[1] + shift, f"{100 * occ[i]:.0f}% of calls\nstays {_pct(stay[i])}",
                    ha=ha, va="top", fontsize=8, color=_INK_2, linespacing=1.25, zorder=6)
        ax.set_title(_panel_title(name, d, data, "  (") + ")", fontsize=10.5,
                     fontweight="bold", color=_INK, loc="left")
    for idx in range(len(agents), nrows * ncols):
        axes[idx // ncols][idx % ncols].axis("off")
    fig.subplots_adjust(left=0.01, right=0.99, top=0.95, bottom=0.01, wspace=0.02, hspace=0.12)
    fig.savefig(out, dpi=200)
    plt.close(fig)
    print(f"-> {out}")


# --- driver -----------------------------------------------------------------------


def render_all(data: dict, out_dir: Path) -> None:
    out_dir.mkdir(parents=True, exist_ok=True)
    render_emission(data, out_dir / "hmm_emission.png")
    _grid(data, out_dir / "hmm_transitions.png", "A")
    _grid(data, out_dir / "hmm_transitions_jump.png", "J")
    render_graphs(data, out_dir / "hmm_transition_graphs.png")


def main() -> None:
    ap = argparse.ArgumentParser(description="HMM emission & transition figures")
    ap.add_argument("--k", type=int, default=5, help="number of HMM states")
    ap.add_argument("--out-dir", default=str(OUT_DIR), help="output directory")
    ap.add_argument("--render-only", action="store_true",
                    help="skip fitting; draw from the cached hmm_fit_k{K}.json")
    args = ap.parse_args()

    out_dir = Path(args.out_dir)
    cache = out_dir / f"hmm_fit_k{args.k}.json"
    if args.render_only:
        if not cache.exists():
            sys.exit(f"no cached fit at {cache}; run without --render-only first")
        data = relabel(json.loads(cache.read_text(encoding="utf-8")))
    else:
        data = fit_and_cache(args.k, cache)
    render_all(data, out_dir)


if __name__ == "__main__":
    main()
