"""Run every accuracy-related report on the evaluation results.

Each report is a subcommand (``--only`` / repeatable to pick several); with no
argument all of them run in order. Results and figures resolve against the
repository root, so the command works from any directory.

Usage::

    python3 analysis/run_accuracy.py                 # everything
    python3 analysis/run_accuracy.py by_category     # one report
    python3 analysis/run_accuracy.py tiers tokens    # several
    python3 analysis/run_accuracy.py tiers -- --threshold 400   # args after -- go to the report
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))  # make `analysis.*` importable regardless of cwd

from analysis.accuracy import by_category, tier_radar, tiers, token_stats  # noqa: E402

# name -> (run function, one-line description); run order is this order.
REPORTS = {
    "by_category": (by_category.main, "per-category PROVED rate for every model"),
    "tiers": (tiers.main, "two-tier complexity split of the benchmark"),
    "radar": (tier_radar.main, "per-tier accuracy radar + bar charts"),
    "tokens": (token_stats.main, "token usage and cost-accuracy curves"),
}


def main() -> None:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument(
        "reports",
        nargs="*",
        default=[],
        metavar="REPORT",
        help=f"reports to run: {', '.join(REPORTS)} (default: all)",
    )
    ap.add_argument("--list", action="store_true", help="list the reports and exit")
    # Anything after `--` goes to the reports' own argument parser (none of them
    # accepts a positional, so splitting at the first `--` is exact).
    argv = sys.argv[1:]
    extra = argv[argv.index("--") + 1:] if "--" in argv else []
    args = ap.parse_args(argv[: argv.index("--")] if "--" in argv else None)

    if args.list:
        for name, (_, desc) in REPORTS.items():
            print(f"{name:<12} {desc}")
        return

    unknown = [r for r in args.reports if r not in REPORTS]
    if unknown:
        ap.error(f"unknown report(s): {', '.join(unknown)}; choose from {', '.join(REPORTS)}")

    for name in args.reports or list(REPORTS):
        run, desc = REPORTS[name]
        print(f"\n{'#' * 70}\n### {name}: {desc}\n{'#' * 70}")
        run(extra)


if __name__ == "__main__":
    main()
