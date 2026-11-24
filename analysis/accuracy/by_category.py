"""Per-category accuracy (PROVED rate) for each model.

``python3 -m analysis category`` prints, per category, each
model's PROVED count / total / accuracy. The category is inferred from the
``<category>_<name>`` prefix of the testcase name
(array / crypto / dp / ds / graph / integer / matrix / string / tensor).
"""

from pathlib import Path

from analysis.accuracy.runs import Context
from analysis.accuracy.verdicts import category, collect

_COL_W = 20  # column width per model


def _cell(proved: int, total: int) -> str:
    pct = proved / total if total else 0.0
    return f"{proved}/{total} ({pct:6.1%})"


def _plot(categories: list, runs, out_dir: Path) -> Path:
    """One bar group per category, one bar per model; height = accuracy (%)."""
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    import numpy as np

    x = np.arange(len(categories))
    width = 0.8 / len(runs)
    fig, ax = plt.subplots(figsize=(10, 5))
    for i, (label, cases) in enumerate(runs):
        accs = []
        for cat in categories:
            names = [n for n in cases if category(n) == cat]
            accs.append(
                sum(1 for n in names if cases[n] == "PROVED") / len(names) * 100 if names else 0.0
            )
        offset = (i - (len(runs) - 1) / 2) * width
        bars = ax.bar(x + offset, accs, width, label=label)
        ax.bar_label(bars, fmt="%.0f%%", padding=2, fontsize=8)

    ax.set_xticks(x)
    ax.set_xticklabels(categories)
    ax.set_ylabel("accuracy (%)")
    ax.set_ylim(0, 105)
    ax.set_title("Accuracy by category")
    ax.legend()
    ax.grid(axis="y", alpha=0.3)
    fig.tight_layout()

    out_dir.mkdir(parents=True, exist_ok=True)
    path = out_dir / "accuracy_by_category.png"
    fig.savefig(path, dpi=150)
    plt.close(fig)
    return path


def main(ctx: Context) -> None:
    runs = [(label, collect(p)) for label, p in ctx.runs.items()]
    categories = sorted({category(n) for _, cases in runs for n in cases})

    header = f"{'category':<10}" + "".join(f"{label:>{_COL_W}}" for label, _ in runs)
    line = "-" * len(header)
    print(header)
    print(line)

    for cat in categories:
        row = f"{cat:<10}"
        for _, cases in runs:
            names = [n for n in cases if category(n) == cat]
            proved = sum(1 for n in names if cases[n] == "PROVED")
            row += f"{_cell(proved, len(names)):>{_COL_W}}" if names else " " * _COL_W
        print(row)

    print(line)
    row = f"{'total':<10}"
    for _, cases in runs:
        proved = sum(1 for v in cases.values() if v == "PROVED")
        row += f"{_cell(proved, len(cases)):>{_COL_W}}"
    print(row)

    print(f"\n-> {_plot(categories, runs, ctx.out_dir)}")

