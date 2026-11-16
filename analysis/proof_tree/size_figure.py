"""Proof-size figure (replaces the proof-size table in the paper).

Four panels: theorems per testcase, then tactics, depth and distinct premises per theorem.
One box per category plus "all"; box = IQR with median bar, whiskers = 5th-95th percentile,
diamond = mean, dots = individual testcases (theorems-per-testcase panel only).

Usage: python3 -m analysis.proof_tree.size_figure [cache_namespace] [out_stem]
"""

from __future__ import annotations

import sys
from pathlib import Path

import numpy as np

from analysis.proof_tree.config import ROOT
from analysis.proof_tree.loop_share import load
from analysis.proof_tree.stats import _category

CATS = ["array", "crypto", "dp", "ds", "graph", "integer", "matrix", "string", "tensor"]
COLORS = {"array": "#4e79a7", "crypto": "#f28e2b", "dp": "#59a14f", "ds": "#e15759", "graph": "#76b7b2",
          "integer": "#b07aa1", "matrix": "#ff9da7", "string": "#9c755f", "tensor": "#bab0ac", "all": "#555555"}


def collect(theorems: list[dict]) -> dict:
    cases: dict[str, list[dict]] = {}
    for t in theorems:
        cases.setdefault(t["case"], []).append(t)
    per_case = {"Theorems": lambda ths: len(ths)}
    per_thm = {"Tactic applications": lambda t: t["tactics"], "Proof depth": lambda t: t["depth"],
               "Distinct premises": lambda t: len(t.get("premise_names", t.get("premises", [])))}
    data = {"case": {}, "thm": {}}
    for name, fn in per_case.items():
        data["case"][name] = {c: [fn(v) for k, v in cases.items() if _category(k) == c] for c in CATS}
        data["case"][name]["all"] = [x for c in CATS for x in data["case"][name][c]]
    for name, fn in per_thm.items():
        data["thm"][name] = {c: [fn(t) for t in theorems if _category(t["case"]) == c] for c in CATS}
        data["thm"][name]["all"] = [x for c in CATS for x in data["thm"][name][c]]
    return data


def draw(data: dict, out_stem: Path) -> None:
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    plt.rcParams.update({"font.family": "serif", "font.size": 7.5})
    fig, axes = plt.subplots(2, 2, figsize=(6.0, 4.6))
    panels = [("case", "Theorems", "Theorems per testcase", False)] + [
        ("thm", m, f"{m} per theorem".replace("Distinct premises", "Distinct premises").replace("Tactic applications", "Tactic applications"), True)
        for m in data["thm"]]
    keys = CATS + ["all"]
    rng = np.random.default_rng(0)
    for ax, (level, metric, title, log) in zip(axes.ravel(), panels):
        groups = data[level][metric]
        vals = [np.asarray(groups[k], float) for k in keys]
        bp = ax.boxplot(vals, positions=range(len(keys)), widths=0.6, whis=(5, 95), showfliers=False,
                        patch_artist=True, medianprops=dict(color="#111", lw=1.4),
                        whiskerprops=dict(color="#777", lw=0.8), capprops=dict(color="#777", lw=0.8))
        for patch, k in zip(bp["boxes"], keys):
            patch.set(facecolor=COLORS[k], alpha=0.35, edgecolor=COLORS[k], lw=0.8)
        for i, (k, v) in enumerate(zip(keys, vals)):
            if level == "case" and k != "all":
                ax.scatter(i + rng.uniform(-0.18, 0.18, len(v)), v, s=5, color=COLORS[k], alpha=0.7, lw=0, zorder=3)
            ax.scatter([i], [v.mean()], marker="D", s=14, color=COLORS[k], edgecolor="#111", lw=0.5, zorder=4)
        if log:
            ax.set_yscale("log")
            ax.set_ylim(bottom=0.9)
        ax.set_title(title, fontsize=8)
        ax.set_xticks(range(len(keys)))
        ax.set_xticklabels(keys, rotation=60, ha="right", fontsize=6.5)
        ax.grid(axis="y", color="#e6e6e6", lw=0.5)
        ax.set_axisbelow(True)
        ax.spines[["top", "right"]].set_visible(False)
        ax.tick_params(length=2)
    fig.text(0.5, 0.005, "box: IQR with median; whiskers: 5th-95th percentile; diamond: mean; dots: testcases",
             ha="center", fontsize=6.5, color="#555")
    fig.tight_layout(rect=(0, 0.02, 1, 1))
    for ext in ("png", "pdf"):
        fig.savefig(out_stem.with_suffix(f".{ext}"), dpi=300, bbox_inches="tight")
    print("saved", out_stem)


def main() -> None:
    ns = sys.argv[1] if len(sys.argv) > 1 else "opus-5.5"
    stem = Path(sys.argv[2]) if len(sys.argv) > 2 else ROOT / "analysis/proof_tree/local/figs/proof_size"
    draw(collect(load(ns)), stem)


if __name__ == "__main__":
    main()
