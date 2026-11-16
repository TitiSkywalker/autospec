"""Per-tier, per-category accuracy radar charts for every agent.

Each testcase is assigned a difficulty tier with the benchmark's existing
complexity proxy (:mod:`analysis.accuracy.tiers`): the ``rust + lean`` line count
of the case itself (the Rust program plus its Aeneas translation). Tier 1 is the
easier half (below the median), tier 2 the harder half. No agent output takes
part in the split.

Each case is then bucketed by the ``<category>_<name>`` prefix. For every tier
one radar chart is drawn: one polygon per agent over the category axes, height =
accuracy (PROVED / total cases of that category in the tier). A case counts as
PROVED only from ``verdict.txt`` written by the Comparator verifier; a case
without one counts as not proved.

``python3 analysis/run_accuracy.py tiers`` prints that split; this module
(``python3 analysis/run_accuracy.py radar``) prints the per-tier table and writes
``analysis/accuracy/local/radar_tier{1,2}.png``
"""

import argparse
from collections import defaultdict
from pathlib import Path

from analysis.accuracy import tiers as tier_split
from analysis.accuracy.runs import RUNS
from analysis.accuracy.verdicts import category, collect

ROOT = Path(__file__).resolve().parents[2]

OUT_DIR = "analysis/accuracy/local"  # plot output dir

# Categorical slots in the palette's fixed order (light mode). The validator
# passes this order on the adjacent pairlist; radar charts put every series on
# screen at once, so identity is carried by the legend plus the printed table.
SERIES = {
    "blue": "#2a78d6",
    "orange": "#eb6834",
    "aqua": "#1baf7a",
    "yellow": "#eda100",
    "magenta": "#e87ba4",
    "green": "#008300",
    "violet": "#4a3aa7",
    "red": "#e34948",
}
SURFACE = "#fcfcfb"
INK = "#0b0b0b"
INK_MUTED = "#52514e"
GRID = "#d9d8d4"

# ========================= (no need to edit below) =========================


def _columns(categories: list[str], total: dict[str, int]) -> list[str]:
    """The tier's axes: only categories that actually have cases in the tier.

    A category with no case in this tier would be a flat zero line on every
    polygon, which reads as "all agents failed it" instead of "not measured".
    """
    return [c for c in categories if total.get(c)]


def _matrix(runs: dict[str, dict[str, str]], tiers: dict[str, int], tier: int):
    """{agent: (proved per category, total per category)} for one tier."""
    proved: dict[str, dict[str, int]] = {}
    total: dict[str, int] = defaultdict(int)
    for name, t in tiers.items():
        if t == tier:
            total[category(name)] += 1
    for label, cases in runs.items():
        counts: dict[str, int] = defaultdict(int)
        for name, verdict in cases.items():
            if tiers.get(name) == tier and verdict == "PROVED":
                counts[category(name)] += 1
        proved[label] = counts
    return proved, dict(total)


def _acc(proved: int, total: int) -> float:
    return proved / total * 100.0 if total else 0.0


def _print_table(runs, tiers, categories) -> None:
    for tier in (1, 2):
        proved, total = _matrix(runs, tiers, tier)
        n = sum(total.values())
        cols = _columns(categories, total)
        print(f"\n=== tier {tier} ({n} testcases) ===")
        header = f"{'agent':<22}" + "".join(f"{c[:7]:>9}" for c in cols) + f"{'overall':>10}"
        print(header)
        print("-" * len(header))
        for label in runs:
            row = f"{label:<22}"
            for cat in cols:
                cells = f"{proved[label][cat]:>2}/{total.get(cat, 0):<2}"
                row += f"{cells:>9}"
            p = sum(proved[label][c] for c in cols)
            row += f"{_acc(p, n):>9.1f}%"
            print(row)


def _fmt(ax) -> None:
    ax.set_facecolor(SURFACE)
    ax.tick_params(axis="y", labelsize=8, colors=INK_MUTED, length=0)
    ax.tick_params(axis="x", labelsize=9, colors=INK, length=0, pad=8)
    ax.grid(False, axis="x")
    ax.grid(True, axis="y", color=GRID, linewidth=0.8, linestyle="-")
    ax.set_axisbelow(True)
    for side in ("top", "right"):
        ax.spines[side].set_visible(False)
    for side in ("left", "bottom"):
        ax.spines[side].set_color(GRID)
    ax.set_ylim(0, 100)
    ax.set_yticks([0, 25, 50, 75, 100])


def _strip_plot(tier, proved, total, categories, n, out_dir: Path) -> Path:
    """One small multiple per agent: its accuracy profile across categories,
    on a shared y-axis so the panels can be compared by position alone."""
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    labels = list(proved)
    ncols = 4
    nrows = (len(labels) + ncols - 1) // ncols
    fig, axes = plt.subplots(nrows, ncols, figsize=(13, 3.0 * nrows), sharey=True)
    axes = np.atleast_1d(axes).ravel()
    x = np.arange(len(categories))
    for ax, label in zip(axes, labels):
        accs = [_acc(proved[label][c], total.get(c, 0)) for c in categories]
        p = sum(proved[label][c] for c in categories)
        bars = ax.bar(x, accs, width=0.62, color=SERIES["blue"], linewidth=0)
        ax.bar_label(bars, fmt="%.0f", padding=2, fontsize=7, color=INK_MUTED)
        _fmt(ax)
        ax.set_title(f"{label}  ·  {_acc(p, n):.1f}% overall", fontsize=10, color=INK, loc="left")
        ax.set_xticks(x)
        ax.set_xticklabels(categories, rotation=35, ha="right", fontsize=8)
    for ax in axes[len(labels):]:
        ax.set_visible(False)
    fig.suptitle(
        f"Tier {tier}: accuracy by category ({n} testcases)", fontsize=12, color=INK, x=0.01, ha="left"
    )
    fig.text(0.01, 0.995, "PROVED / cases per category", fontsize=8, color=INK_MUTED, ha="left", va="top")
    fig.set_facecolor(SURFACE)
    fig.tight_layout(rect=(0, 0, 1, 0.96))
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"accuracy_tier{tier}.png"
    fig.savefig(path, dpi=150, facecolor=SURFACE)
    plt.close(fig)
    return path


def _radar_plot(tier, proved, total, categories, n, out_dir: Path) -> Path:
    """One radar per tier: every agent a closed polygon over the category axes,
    height = accuracy. All eight axes are on one 0-100% scale."""
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    angles = np.linspace(0, 2 * np.pi, len(categories), endpoint=False).tolist()
    angles += angles[:1]
    fig = plt.figure(figsize=(7.5, 7.5))
    ax = fig.add_subplot(111, polar=True)
    ax.set_theta_offset(np.pi / 2)
    ax.set_theta_direction(-1)
    ax.set_ylim(0, 100)
    ax.set_yticks([20, 40, 60, 80, 100])
    ax.set_yticklabels(["20", "40", "60", "80", "100"], fontsize=7, color=INK_MUTED)
    ax.set_rlabel_position(180 / len(categories))  # midway between two axes, clear of the axis labels
    ax.set_xticks(angles[:-1])
    ax.set_xticklabels(
        [f"{c}\n({total.get(c, 0)})" for c in categories], fontsize=10, color=INK
    )
    ax.grid(color=GRID, linewidth=0.8)
    ax.spines["polar"].set_color(GRID)

    for (label, _), hue in zip(proved.items(), SERIES.values()):
        vals = [_acc(proved[label][c], total.get(c, 0)) for c in categories]
        vals += vals[:1]
        # Two agents are exactly 0% all the way round; their polygon collapses
        # onto the centre. Draw the outline last so it is not buried under
        # later-filled polygons.
        (z,) = ax.plot(angles, vals, color=hue, linewidth=2, label=label)
        ax.fill(angles, vals, color=hue, alpha=0.05, linewidth=0)
        if max(vals) == 0:
            z.set_zorder(10)

    ax.set_title(
        f"Tier {tier} ({n} testcases)",
        fontsize=12, color=INK, pad=28,
    )
    ax.legend(
        loc="upper center", bbox_to_anchor=(0.5, -0.08), ncol=4, frameon=False,
        fontsize=9, labelcolor=INK,
    )
    fig.set_facecolor(SURFACE)
    fig.tight_layout()
    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / f"radar_tier{tier}.png"
    fig.savefig(path, dpi=150, facecolor=SURFACE)
    plt.close(fig)
    return path


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description="Per-tier accuracy radar charts")
    ap.parse_args(argv)

    runs = {label: collect(ROOT / p) for label, p in RUNS.items()}
    tiers = tier_split.assign()
    categories = sorted({category(n) for n in tiers})
    _print_table(runs, tiers, categories)

    for tier in (1, 2):
        proved, total = _matrix(runs, tiers, tier)
        n = sum(total.values())
        cols = _columns(categories, total)
        print(f"\n-> {_radar_plot(tier, proved, total, cols, n, (ROOT / OUT_DIR))}")


if __name__ == "__main__":
    main()
