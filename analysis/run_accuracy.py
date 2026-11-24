"""Accuracy reports over one or more verified evaluation runs.

A run is a results directory produced by ``python -m evaluation.manager`` and
checked by ``python -m evaluation.verifier`` (one sub-directory per testcase
with a ``verdict.txt``). Each report is a positional name; with none given, all
reports run in order.

Usage::

    python3 -m analysis --run Opus=results/opus --run GPT=results/gpt
    python3 -m analysis --runs-dir results/              # every run under results/
    python3 -m analysis summary category --runs-dir results/
    python3 -m analysis tokens --runs-dir results/ --prices analysis/prices.example.toml
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))  # make `analysis.*` importable regardless of cwd

from analysis.accuracy import by_category, summary, tier_radar, tiers, token_stats  # noqa: E402
from analysis.accuracy.runs import Context, load_prices, parse_runs  # noqa: E402

# name -> (run function, one-line description); run order is this order.
REPORTS = {
    "summary": (summary.main, "verdict counts (PROVED / UNPROVED / BUILD_FAIL / CHEAT / ...) per run"),
    "category": (by_category.main, "per-category PROVED rate + bar chart"),
    "tiers": (tiers.main, "accuracy on the beginner / advanced difficulty tiers"),
    "radar": (tier_radar.main, "per-tier, per-category radar charts"),
    "tokens": (token_stats.main, "token usage, dollar cost and cost-accuracy curves"),
}


def main() -> None:
    ap = argparse.ArgumentParser(
        prog="python3 -m analysis",
        description=__doc__,
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("reports", nargs="*", metavar="REPORT",
                    help=f"reports to run: {', '.join(REPORTS)} (default: all)")
    ap.add_argument("--run", action="append", default=[], metavar="LABEL=DIR",
                    help="a verified results directory (repeatable)")
    ap.add_argument("--runs-dir", metavar="DIR",
                    help="a directory whose sub-directories are runs (labelled by name)")
    ap.add_argument("--out", default=str(ROOT / "analysis" / "out"),
                    help="where figures are written (default: %(default)s)")
    ap.add_argument("--benchmark", default=str(ROOT / "lean_mode" / "LeanMode"),
                    help="benchmark directory (default: %(default)s)")
    ap.add_argument("--prices", metavar="TOML",
                    help="USD-per-million-token price table keyed by run label "
                         "(see analysis/prices.example.toml)")
    ap.add_argument("--list", action="store_true", help="list the reports and exit")
    args = ap.parse_args()

    if args.list:
        for name, (_, desc) in REPORTS.items():
            print(f"{name:<10} {desc}")
        return

    unknown = [r for r in args.reports if r not in REPORTS]
    if unknown:
        ap.error(f"unknown report(s): {', '.join(unknown)}; choose from {', '.join(REPORTS)}")

    ctx = Context(
        runs=parse_runs(args.run, args.runs_dir),
        out_dir=Path(args.out),
        benchmark=Path(args.benchmark),
        prices=load_prices(args.prices),
    )
    for name in args.reports or list(REPORTS):
        run, desc = REPORTS[name]
        print(f"\n{'#' * 70}\n### {name}: {desc}\n{'#' * 70}")
        run(ctx)


if __name__ == "__main__":
    main()
