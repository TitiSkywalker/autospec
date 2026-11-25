"""Accuracy and cost reports over verified evaluation runs.

A run is the output directory of an evaluation, after the verifier has written
a verdict.txt into every case. Pass runs as arguments: a run directory, a
LABEL=DIR pair to rename it, or a parent directory whose sub-directories are
runs. Every report runs unless you pick some with --report.

examples:
  python3 -m analysis results/my-agent
  python3 -m analysis results/                        every run in results/
  python3 -m analysis Opus=results/opus GPT=results/gpt
  python3 -m analysis results/ --report summary --report tiers
  python3 -m analysis results/ --report tokens --prices my-prices.toml
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
    "summary": (summary.main, "verdict counts and accuracy per run"),
    "category": (by_category.main, "accuracy per benchmark category, with a bar chart"),
    "tiers": (tiers.main, "accuracy on the beginner and advanced tiers"),
    "radar": (tier_radar.main, "radar charts of accuracy per category, one per tier"),
    "tokens": (token_stats.main, "token usage, and dollar cost when --prices is given"),
}

DEFAULT_OUT = Path("analysis/out")
DEFAULT_BENCHMARK = Path("lean_mode/LeanMode")


def _parser() -> argparse.ArgumentParser:
    reports = "\n".join(f"  {name:<10} {desc}" for name, (_, desc) in REPORTS.items())
    ap = argparse.ArgumentParser(
        prog="python3 -m analysis",
        description=__doc__.split("\n\nexamples:")[0],
        epilog=f"reports:\n{reports}\n\nexamples:{__doc__.split('examples:')[1]}",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument("runs", nargs="*", metavar="RUN",
                    help="a run directory, LABEL=DIR, or a directory of runs")
    ap.add_argument("-r", "--report", action="append", choices=list(REPORTS), metavar="NAME",
                    help="run only this report (repeatable; default: all)")
    ap.add_argument("-o", "--out", type=Path, default=DEFAULT_OUT, metavar="DIR",
                    help="where figures are saved (default: %(default)s)")
    ap.add_argument("--benchmark", type=Path, default=DEFAULT_BENCHMARK, metavar="DIR",
                    help="benchmark directory, used for tier sizes (default: %(default)s)")
    ap.add_argument("--prices", metavar="FILE",
                    help="price table for the tokens report (see analysis/prices.example.toml)")
    # Older spellings, kept so existing scripts keep working.
    ap.add_argument("--run", action="append", default=[], help=argparse.SUPPRESS)
    ap.add_argument("--runs-dir", action="append", default=[], help=argparse.SUPPRESS)
    return ap


def _resolve(path: Path) -> Path:
    """Relative defaults are relative to the repository, not the caller's cwd."""
    return path if path.is_absolute() or path.exists() else ROOT / path


def main() -> None:
    ap = _parser()
    args = ap.parse_args()

    misplaced = [r for r in args.runs if r in REPORTS and not Path(r).exists()]
    if misplaced:
        ap.error(f"{misplaced[0]!r} is a report name; select reports with --report {misplaced[0]}")
    runs = args.runs + args.run + args.runs_dir
    if not runs:
        ap.error("no runs given; pass at least one run directory (see examples below --help)")
    try:
        parsed = parse_runs(runs)
    except ValueError as e:
        ap.error(str(e))

    ctx = Context(
        runs=parsed,
        out_dir=_resolve(args.out),
        benchmark=_resolve(args.benchmark),
        prices=load_prices(args.prices),
    )
    print(f"{len(parsed)} run(s): {', '.join(parsed)}")
    for name in args.report or list(REPORTS):
        run, desc = REPORTS[name]
        print(f"\n{'#' * 70}\n### {name}: {desc}\n{'#' * 70}")
        run(ctx)


if __name__ == "__main__":
    main()
