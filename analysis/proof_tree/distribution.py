"""Polished distribution chart for proof-tree statistics.

Produces a 3-panel figure (tactics / depth / premises) with:
  - Box plot showing quartiles + whiskers per category (IQR × 1.5)
  - Jittered strip of individual cases overlaid as dots
  - Log-scale y-axis (tactics span 8 → 1996; linear compresses small values)
  - Outlier labels on the top-3 cases by tactic count
  - Per-category mean marker (diamond)
  - Thin horizontal gridlines; no chartjunk
  - WCAG AA checked categorical palette (Tableau 10 base, lightness-verified)
Saved to analysis/figs/proof_tree_size_by_category.png

Unlike the summary charts in :mod:`analysis.proof_tree.plots` (which are drawn
from an in-memory run), this one reads the per-case cache directly, so it can be
regenerated at any time without touching Lean.  It is also invoked at the end of
a normal ``python3 -m analysis.proof_tree`` run via :func:`run`.

Usage::

    python3 -m analysis.proof_tree.distribution
    python3 -m analysis.proof_tree.distribution --model dsh_v41
    python3 -m analysis.proof_tree.distribution --out analysis/figs/proof_tree_size_by_category.png
    python3 -m analysis.proof_tree.distribution --subject "DSH-Flash" --model dsh_v41
    python3 -m analysis.proof_tree.distribution --verdict PROVED
"""

from __future__ import annotations

import argparse
import json
import sys
import warnings
from collections import defaultdict
from pathlib import Path

from analysis.proof_tree.config import DEFAULT_VERDICTS, OUT_DIR, RESULTS_DIR, ROOT, cache_dir_for
from analysis.proof_tree.stats import _case_summary, _category

# ======================= Configuration (edit here) =======================

# Only cases carrying this verdict are plotted by default; override with --verdict.
_DEFAULT_VERDICT = next(iter(DEFAULT_VERDICTS))  # "PROVED"

FIGURE_NAME = "proof_tree_size_by_category.png"
DPI = 180

# Subject named in the figure's suptitle. Not derived from --model, because the
# cache namespace is not the display name and the default figure must stay
# byte-identical to the previously published one.
FIGURE_SUBJECT = "Claude Opus"

# ========================= (no need to edit below) =========================

# ── palette (Tableau 10, lightness-verified against WCAG AA on white) ─────────
# Use .get(cat, fallback) so an unknown category renders rather than raising.
_PALETTE_FALLBACK = "#aaaaaa"
PALETTE: dict[str, str] = {
    "array":   "#4e79a7",
    "crypto":  "#f28e2b",
    "dp":      "#59a14f",
    "ds":      "#e15759",
    "graph":   "#76b7b2",
    "integer": "#b07aa1",
    "matrix":  "#ff9da7",
    "string":  "#9c755f",
    "tensor":  "#bab0ac",
}

# ── layout ─────────────────────────────────────────────────────────────────────
# (key, y-axis label, log scale) — one panel per metric, top to bottom.
METRICS = [
    ("tactics",  "Tactic applications",  True),   # log scale
    ("depth",    "Proof depth",           False),
    ("premises", "Distinct premises",     False),
]

RNG_SEED = 42
FIG_W, FIG_H = 13, 14


def load_rows(cache_dir: Path, verdict: str = _DEFAULT_VERDICT) -> list[tuple[str, str, int, int, int]]:
    """``[(category, name, tactics, depth, premises)]`` for cached cases with ``verdict``.

    Skips malformed JSON files with a warning to stderr rather than crashing.
    """
    if not cache_dir.is_dir():
        warnings.warn(
            f"cache directory does not exist: {cache_dir}  — no data to plot",
            stacklevel=2,
        )
        return []
    rows = []
    for f in sorted(cache_dir.glob("*.json")):
        try:
            r = json.loads(f.read_text())
        except (json.JSONDecodeError, OSError) as exc:
            print(f"warning: skipping bad cache file {f.name}: {exc}", file=sys.stderr)
            continue
        if r.get("verdict") != verdict:
            continue
        s = _case_summary(r)
        if s is None:
            continue
        rows.append((_category(f.stem), f.stem, s["tactics"], s["depth"], s["premises"]))
    return rows


def run(
    results_dir: str = RESULTS_DIR,
    model: str | None = None,
    out: str | None = None,
    subject: str = FIGURE_SUBJECT,
    verdict: str = _DEFAULT_VERDICT,
) -> None:
    """Render the box-plot figure from the cache.

    Takes the same options as the CLI flags of :func:`main`; called by the main
    proof-tree CLI at the end of a run so the distribution figure stays current.
    """
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import matplotlib.patches as mpatches
    import numpy as np

    _results_dir = ROOT / results_dir
    cache_dir = cache_dir_for(_results_dir, model)
    rows = load_rows(cache_dir, verdict)
    out_path = Path(out) if out else ROOT / OUT_DIR / FIGURE_NAME

    cats = sorted({r[0] for r in rows})
    cat_idx = {c: i for i, c in enumerate(cats)}

    rng = np.random.default_rng(RNG_SEED)
    fig, axes = plt.subplots(3, 1, figsize=(FIG_W, FIG_H), constrained_layout=False)
    fig.subplots_adjust(hspace=0.38, top=0.95, bottom=0.06, left=0.08, right=0.98)
    fig.patch.set_facecolor("white")

    for ax, (metric, ylabel, log_scale) in zip(axes, METRICS):
        # collect per-category value lists
        by_cat = defaultdict(list)
        for cat, name, tactics, depth, premises in rows:
            val = {"tactics": tactics, "depth": depth, "premises": premises}[metric]
            by_cat[cat].append((val, name))

        # ── box plots (stat layer) ────────────────────────────────────────────
        bp_data   = [sorted(v for v, _ in by_cat[c]) for c in cats]
        bp_pos    = list(range(len(cats)))
        bp = ax.boxplot(
            bp_data,
            positions=bp_pos,
            widths=0.45,
            patch_artist=True,
            showfliers=False,          # we draw outliers ourselves as jitter
            medianprops=dict(color="#1a1a1a", linewidth=2),
            whiskerprops=dict(color="#666666", linewidth=1),
            capprops=dict(color="#666666", linewidth=1),
            boxprops=dict(linewidth=0.8),
            zorder=2,
        )
        for patch, cat in zip(bp["boxes"], cats):
            color = PALETTE.get(cat, _PALETTE_FALLBACK)
            patch.set_facecolor(color)
            patch.set_alpha(0.25)
            patch.set_edgecolor(color)

        # ── jittered strip (data layer) ───────────────────────────────────────
        for i, cat in enumerate(cats):
            vals  = [v for v, _ in by_cat[cat]]
            jitter = rng.uniform(-0.17, 0.17, size=len(vals))
            ax.scatter(
                np.array(bp_pos[i:i+1]) + jitter,
                vals,
                color=PALETTE.get(cat, _PALETTE_FALLBACK),
                s=28,
                alpha=0.85,
                linewidths=0.4,
                edgecolors="white",
                zorder=3,
            )

        # ── mean diamond ─────────────────────────────────────────────────────
        for i, cat in enumerate(cats):
            vals = [v for v, _ in by_cat[cat]]
            mean = sum(vals) / len(vals)
            ax.scatter(
                [bp_pos[i]], [mean],
                marker="D",
                s=40,
                color=PALETTE.get(cat, _PALETTE_FALLBACK),
                edgecolors="#1a1a1a",
                linewidths=0.6,
                zorder=4,
                label="_nolegend_",
            )

        # ── outlier labels (tactics panel only: top-3 globally) ──────────────
        if metric == "tactics":
            all_cases = sorted(rows, key=lambda r: -r[2])[:3]
            labeled = set()
            for cat, name, t, d, p in all_cases:
                if name in labeled:
                    continue
                labeled.add(name)
                i = cat_idx[cat]
                short = name.replace(cat + "_", "")
                # push label left for rightmost categories to avoid clipping
                xoff = -70 if i >= len(cats) - 2 else 8
                ax.annotate(
                    f"{short} ({t})",
                    xy=(i, t),
                    xytext=(xoff, 4),
                    textcoords="offset points",
                    fontsize=7.5,
                    color="#333333",
                    fontweight="semibold",
                    ha="right" if xoff < 0 else "left",
                    arrowprops=dict(arrowstyle="-", color="#aaaaaa", lw=0.8),
                )

        # ── axes styling ─────────────────────────────────────────────────────
        if log_scale:
            ax.set_yscale("log")
            ax.yaxis.set_minor_formatter(matplotlib.ticker.NullFormatter())

        ax.set_xticks(bp_pos)
        ax.set_xticklabels(cats, fontsize=10)
        ax.set_ylabel(ylabel, fontsize=10)
        ax.set_xlim(-0.6, len(cats) - 0.4)
        ax.grid(axis="y", color="#e0e0e0", linewidth=0.6, zorder=0)
        ax.set_axisbelow(True)
        ax.spines[["top", "right"]].set_visible(False)
        ax.spines["left"].set_color("#cccccc")
        ax.spines["bottom"].set_color("#cccccc")
        ax.tick_params(axis="both", length=0, labelsize=9)

        # n= annotation below each category label
        for i, cat in enumerate(cats):
            n = len(by_cat[cat])
            ax.text(i, -0.08, f"n={n}", ha="center", va="top",
                    fontsize=7.5, color="#888888",
                    transform=ax.get_xaxis_transform())

    # ── shared legend: mean diamond + box explanations ────────────────────────
    legend_handles = [
        mpatches.Patch(facecolor="#dddddd", edgecolor="#555555", linewidth=0.8, label="IQR box + whiskers"),
        plt.scatter([], [], marker="D", s=40, color="#555555", edgecolors="#1a1a1a", linewidths=0.6, label="Mean"),
        plt.scatter([], [], s=28, color="#555555", alpha=0.85, edgecolors="white", linewidths=0.4, label="Individual case"),
    ]
    fig.legend(
        handles=legend_handles,
        loc="upper right",
        bbox_to_anchor=(0.99, 0.99),
        fontsize=9,
        frameon=True,
        framealpha=0.92,
        edgecolor="#cccccc",
    )

    fig.suptitle(
        f"Proof-tree statistics by benchmark category  ({subject}, {len(rows)} {verdict} cases)",
        fontsize=12,
        fontweight="semibold",
        color="#1a1a1a",
        y=1.01,
    )

    # ── save ──────────────────────────────────────────────────────────────────
    out_path.parent.mkdir(parents=True, exist_ok=True)
    fig.savefig(out_path, dpi=DPI, bbox_inches="tight", facecolor="white")
    plt.close(fig)
    print(f"saved -> {out_path}")


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Render the box-plot distribution chart from cached proof-tree stats"
    )
    ap.add_argument(
        "--model",
        default=None,
        help="cache namespace to read (default: basename of --results-dir)",
    )
    ap.add_argument(
        "--results-dir",
        default=RESULTS_DIR,
        help="results directory whose basename namespaces the cache (default: %(default)s)",
    )
    ap.add_argument(
        "--out",
        default=None,
        help=f"output PNG (default: {OUT_DIR}/{FIGURE_NAME})",
    )
    ap.add_argument(
        "--subject",
        default=FIGURE_SUBJECT,
        help="name shown in the figure title (default: %(default)s)",
    )
    ap.add_argument(
        "--verdict",
        default=_DEFAULT_VERDICT,
        help=f"only plot cases with this verdict (default: {_DEFAULT_VERDICT!r})",
    )
    args = ap.parse_args()

    # Bootstrap sys.path so evaluation.* imports work when this is the entry point.
    if str(ROOT) not in sys.path:
        sys.path.insert(0, str(ROOT))

    run(
        results_dir=args.results_dir,
        model=args.model,
        out=args.out,
        subject=args.subject,
        verdict=args.verdict,
    )


if __name__ == "__main__":
    main()
