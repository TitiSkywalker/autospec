"""The three summary charts written at the end of a run.

All three are matplotlib PNGs in :data:`~analysis.proof_tree.config.OUT_DIR`,
one per table of the terminal report:

* ``proof_tree_size_by_category.png``          — avg tactic count per category.
* ``proof_structure_premise_provenance.png``   — aeneas/program/math stacked %.
* ``proof_structure_tactic_composition.png``   — the 7 tactic buckets stacked %.

matplotlib is imported inside :func:`_plot` so a headless / matplotlib-less
environment still gets the printed report; the caller turns the resulting
``ImportError`` into a one-line note. Colours, fonts, thresholds and layout are
part of the rendered output and must not be touched.
"""

from __future__ import annotations

from pathlib import Path

from analysis.proof_tree.config import ROOT
from analysis.proof_tree.stats import _avg, _category, _pct, _sum_buckets
from analysis.proof_tree.taxonomy import (
    ALL_TACTIC_BUCKETS,
    PREMISE_BUCKETS,
    PREMISE_BUCKET_COLORS,
    TACTIC_BUCKET_COLORS,
)

# Colors for the corpus-comparison charts, keyed by corpus label.  Defined at
# module level so a reader can discover the dependency on distribution.PALETTE
# without opening the function body.
from analysis.proof_tree.distribution import PALETTE as _DIST_PALETTE
_COMPARISON_COLORS = {
    "vericoding":    _DIST_PALETTE["array"],   # "#4e79a7"
    "navier_stokes": _DIST_PALETTE["crypto"],  # "#f28e2b"
    "minif2f":       _DIST_PALETTE["dp"],      # "#59a14f"
}


def _rel(path: Path) -> str:
    """Return ``path`` relative to ROOT for byte-stable printed output."""
    return str(path.relative_to(ROOT))


def _plot(ok: dict, out_dir: Path) -> None:
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    categories = sorted({_category(n) for n in ok})
    x = np.arange(len(categories))
    out_dir.mkdir(parents=True, exist_ok=True)

    # ---- chart 1: avg tactic count per category ----
    avg_tactics = [
        _avg([s["tactics"] for n, (s, v) in ok.items() if _category(n) == cat])
        for cat in categories
    ]
    fig, ax = plt.subplots(figsize=(10, 5))
    bars = ax.bar(x, avg_tactics)
    ax.bar_label(bars, fmt="%.0f", padding=2, fontsize=8)
    ax.set_xticks(x)
    ax.set_xticklabels(categories, rotation=30, ha="right")
    ax.set_ylabel("avg tactics per proof")
    ax.set_title("Proof-tree size by category")
    ax.grid(axis="y", alpha=0.3)
    fig.tight_layout()
    path = out_dir / "proof_tree_size_by_category.png"
    fig.savefig(path, dpi=150)
    plt.close(fig)
    print(f"\nbar chart -> {_rel(path)}")

    # ---- chart 2: premise provenance stacked bar (aeneas / program / math) ----
    prem_pcts: list[list[float]] = [[] for _ in PREMISE_BUCKETS]
    for cat in categories:
        agg = _sum_buckets(
            [s for n, (s, _v) in ok.items() if _category(n) == cat],
            "premise_buckets",
        )
        pp = _pct(agg, PREMISE_BUCKETS)
        for i, k in enumerate(PREMISE_BUCKETS):
            prem_pcts[i].append(pp[k])

    fig2, ax2 = plt.subplots(figsize=(10, 5))
    bottoms = np.zeros(len(categories))
    for pcts, k in zip(prem_pcts, PREMISE_BUCKETS):
        vals = np.array(pcts)
        color = PREMISE_BUCKET_COLORS[k]
        bars2 = ax2.bar(x, vals, bottom=bottoms, color=color, label=k)
        # only label segments wide enough to read
        for rect, val in zip(bars2, vals):
            if val >= 8:
                ax2.text(
                    rect.get_x() + rect.get_width() / 2,
                    rect.get_y() + rect.get_height() / 2,
                    f"{val:.0f}%",
                    ha="center", va="center", fontsize=7, color="white", fontweight="bold",
                )
        bottoms += vals
    ax2.set_xticks(x)
    ax2.set_xticklabels(categories, rotation=30, ha="right")
    ax2.set_ylabel("% of premise citations")
    ax2.set_title("Proof structure by premise provenance")
    ax2.legend(loc="upper right", fontsize=9)
    ax2.set_ylim(0, 100)
    ax2.grid(axis="y", alpha=0.3)
    fig2.tight_layout()
    path2 = out_dir / "proof_structure_premise_provenance.png"
    fig2.savefig(path2, dpi=150)
    plt.close(fig2)
    print(f"stacked provenance chart -> {_rel(path2)}")

    # ---- chart 3: tactic composition stacked bar (8 buckets + other) ----
    tac_pcts: list[list[float]] = [[] for _ in ALL_TACTIC_BUCKETS]
    for cat in categories:
        agg = _sum_buckets(
            [s for n, (s, _v) in ok.items() if _category(n) == cat],
            "tactic_buckets",
        )
        tp = _pct(agg, ALL_TACTIC_BUCKETS)
        for i, k in enumerate(ALL_TACTIC_BUCKETS):
            tac_pcts[i].append(tp[k])

    fig3, ax3 = plt.subplots(figsize=(11, 5))
    bottoms = np.zeros(len(categories))
    for pcts, k in zip(tac_pcts, ALL_TACTIC_BUCKETS):
        vals = np.array(pcts)
        color = TACTIC_BUCKET_COLORS[k]
        bars3 = ax3.bar(x, vals, bottom=bottoms, color=color, label=k)
        for rect, val in zip(bars3, vals):
            if val >= 6:
                ax3.text(
                    rect.get_x() + rect.get_width() / 2,
                    rect.get_y() + rect.get_height() / 2,
                    f"{val:.0f}%",
                    ha="center", va="center", fontsize=6, color="white", fontweight="bold",
                )
        bottoms += vals
    ax3.set_xticks(x)
    ax3.set_xticklabels(categories, rotation=30, ha="right")
    ax3.set_ylabel("% of tactic applications")
    ax3.set_title("Proof structure by tactic composition")
    ax3.legend(loc="upper right", fontsize=8, ncol=2)
    ax3.set_ylim(0, 100)
    ax3.grid(axis="y", alpha=0.3)
    fig3.tight_layout()
    path3 = out_dir / "proof_structure_tactic_composition.png"
    fig3.savefig(path3, dpi=150)
    plt.close(fig3)
    print(f"tactic composition chart -> {_rel(path3)}")


def plot_comparison(
    tactic_pct: dict[str, dict],
    premise_pct: dict[str, dict],
    out_dir: Path,
) -> None:
    """Render two grouped-bar figures comparing corpora.

    ``tactic_pct`` / ``premise_pct`` map corpus label -> bucket percentage dict
    (e.g. ``{"vericoding": {...}, "navier_stokes": {...}, "minif2f": {...}}``);
    one bar per corpus in each bucket group. Saves ``comparison_tactics.png``
    and ``comparison_premises.png`` to *out_dir*.
    """
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    def _grouped_bar(keys, series, ylabel, title, out_path):
        x = np.arange(len(keys))
        width = 0.8 / max(len(series), 1)
        fig, ax = plt.subplots(figsize=(max(8, len(keys) * 1.3), 5))
        peak = 0.0
        for j, (label, pct) in enumerate(series.items()):
            vals = [pct[k] for k in keys]
            peak = max(peak, max(vals, default=0))
            offset = (j - (len(series) - 1) / 2) * width
            bars = ax.bar(x + offset, vals, width, label=label,
                          color=_COMPARISON_COLORS.get(label, "#aaaaaa"), alpha=0.85)
            ax.bar_label(bars, fmt="%.1f", padding=2, fontsize=6)
        ax.set_xticks(x)
        ax.set_xticklabels(keys, rotation=30, ha="right", fontsize=9)
        ax.set_ylabel(ylabel)
        ax.set_title(title)
        ax.legend(fontsize=9)
        ax.set_ylim(0, peak * 1.2 + 5)
        ax.grid(axis="y", alpha=0.3)
        ax.spines[["top", "right"]].set_visible(False)
        fig.tight_layout()
        fig.savefig(out_path, dpi=150, facecolor="white")
        plt.close(fig)
        print(f"comparison figure -> {_rel(out_path)}")

    names = " vs ".join(tactic_pct)
    _grouped_bar(
        list(ALL_TACTIC_BUCKETS), tactic_pct,
        "% of tactic applications", f"Tactic composition: {names}",
        out_dir / "comparison_tactics.png",
    )
    _grouped_bar(
        list(PREMISE_BUCKETS), premise_pct,
        "% of premise citations", f"Premise provenance: {names}",
        out_dir / "comparison_premises.png",
    )
