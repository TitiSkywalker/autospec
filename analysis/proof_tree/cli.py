"""Command-line entry point: flags, cache policy, run loop.

This is the only module that decides *what* to analyze; it shells out to
:mod:`.workspace` for anything that touches Lean, and to :mod:`.report` /
:mod:`.plots` for output.

Usage::

    python3 -m analysis.proof_tree                   # analyze every case (cached)
    python3 -m analysis.proof_tree --rebuild          # ignore the cache
    python3 -m analysis.proof_tree --only crypto_rsa  # just one case, verbose
    python3 -m analysis.proof_tree --limit 10         # quick sample
    python3 -m analysis.proof_tree --dot crypto_rsa   # render that case's proof tree
"""

from __future__ import annotations

import argparse
import json
import sys

from analysis.proof_tree.config import DEFAULT_VERDICTS, OUT_DIR, RESULTS_DIR, ROOT, cache_dir_for
from analysis.proof_tree.report import report
from analysis.proof_tree.workspace import CaseError, discover_cases, ensure_workspace, process_case


def _positive_int(value: str) -> int:
    """Argparse type: reject zero and negative --limit values."""
    n = int(value)
    if n <= 0:
        raise argparse.ArgumentTypeError(f"--limit must be a positive integer, got {value!r}")
    return n


def main() -> None:
    ap = argparse.ArgumentParser(description="Analyze proof trees with treant")
    ap.add_argument("--rebuild", action="store_true", help="ignore the per-case cache")
    ap.add_argument("--only", help="process a single case (verbose)")
    ap.add_argument(
        "--limit",
        type=_positive_int,
        help="process at most N cases (must be >= 1)",
    )
    ap.add_argument(
        "--all",
        action="store_true",
        help=f"include every verdict, not just {sorted(DEFAULT_VERDICTS)}",
    )
    ap.add_argument(
        "--results-dir",
        default=RESULTS_DIR,
        help="path to the agent results directory (default: %(default)s)",
    )
    ap.add_argument(
        "--model",
        default=None,
        help="short model name used to namespace the cache (default: basename of --results-dir)",
    )
    args = ap.parse_args()

    ws = ensure_workspace()
    results_dir = ROOT / args.results_dir

    # per-model cache namespace: avoids collisions when the same case name
    # appears in multiple results directories.
    cache_dir = cache_dir_for(results_dir, args.model)
    cache_dir.mkdir(parents=True, exist_ok=True)

    cases = discover_cases(results_dir)
    verdict_filter = None if args.all else DEFAULT_VERDICTS
    if verdict_filter is not None:
        cases = [c for c in cases if c[1] in verdict_filter]
    if args.only:
        cases = [c for c in cases if c[0] == args.only]
        if not cases:
            sys.exit(f"no such case: {args.only}")
    if args.limit is not None:
        cases = cases[: args.limit]

    results: dict[str, dict] = {}
    for i, (name, verdict) in enumerate(cases, 1):
        cache_file = cache_dir / f"{name}.json"
        if cache_file.is_file() and not args.rebuild:
            results[name] = json.loads(cache_file.read_text())
            continue
        print(f"[{i}/{len(cases)}] {name} (verdict={verdict}): building + extracting...", file=sys.stderr)
        result = process_case(name, results_dir / name, ws, verbose=bool(args.only))
        result["verdict"] = verdict
        cache_file.write_text(json.dumps(result))
        results[name] = result

    report(results, verdict_scope=verdict_filter, out_dir=ROOT / OUT_DIR)

    # Also regenerate the box-plot distribution figure from the updated cache.
    try:
        from analysis.proof_tree.distribution import run as dist_run
        dist_run(results_dir=args.results_dir, model=args.model)
    except ImportError:
        pass  # matplotlib not available
