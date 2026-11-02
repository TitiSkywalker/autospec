"""The terminal report: per-category, per-verdict and per-bucket tables.

``report`` is the only output-producing step of a normal run. It prints five
tables (size/depth/premises by category, the same split by verdict, premise
provenance, tactic composition, and the top-tactic / largest-tree lists) and
then hands the same case summaries to :func:`~analysis.proof_tree.plots._plot`
for the summary charts.

Column widths, number formats and bucket key sets are load-bearing: the emitted
text is compared against a stored baseline, so keep them byte-stable.
"""

from __future__ import annotations

from pathlib import Path
from typing import Optional

from analysis.proof_tree.config import OUT_DIR, ROOT
from analysis.proof_tree.plots import _plot
from analysis.proof_tree.stats import _avg, _case_summary, _category, _pct, _sum_buckets
from analysis.proof_tree.taxonomy import ALL_TACTIC_BUCKETS, BUCKET_COLUMNS, PREMISE_BUCKETS, PREMISE_BUCKET_COLUMNS, TACTIC_BUCKETS


def report(
    results: dict[str, dict],
    verdict_scope: Optional[set] = None,
    out_dir: Optional[Path] = None,
) -> None:
    """Print the proof-tree analysis tables and render summary charts.

    Parameters
    ----------
    results:
        Mapping from case name to the raw result dict (as loaded from cache or
        returned by :func:`~analysis.proof_tree.workspace.process_case`).
    verdict_scope:
        Display label only — this set is shown in the section header to tell the
        reader which verdicts were pre-filtered by the caller.  It does NOT
        filter ``results``; the caller is responsible for passing only the
        desired cases.
    out_dir:
        Directory for the output PNG figures.  Defaults to ``ROOT / OUT_DIR``
        (always relative to the repository root, never the current directory).
        Pass an explicit path to redirect output.
    """
    if out_dir is None:
        out_dir = ROOT / OUT_DIR
    summaries = {name: (_case_summary(r), r.get("verdict", "UNKNOWN")) for name, r in results.items()}
    ok = {n: (s, v) for n, (s, v) in summaries.items() if s is not None}
    failed = {n: r.get("error", "unknown") for n, r in results.items() if summaries[n][0] is None}

    scope = f"verdicts={sorted(verdict_scope)}" if verdict_scope else "all verdicts"
    print(f"\n=== Proof-tree analysis ({scope}): {len(results)} cases, {len(ok)} analyzed, {len(failed)} skipped ===")
    if failed:
        print("\nskipped (build/extract failure):")
        for n, why in sorted(failed.items()):
            print(f"  {n:<40} {why}")

    print(f"\n{'category':<12} {'n':>4} {'proved':>7} {'avg tactics':>12} {'avg depth':>10} {'avg premises':>13} {'unlinked':>9}")
    print("-" * 75)
    categories = sorted({_category(n) for n in ok})
    for cat in categories:
        rows = [(s, v) for n, (s, v) in ok.items() if _category(n) == cat]
        n = len(rows)
        proved = sum(1 for _, v in rows if v == "PROVED")
        print(
            f"{cat:<12} {n:>4} {proved:>7} "
            f"{_avg([s['tactics'] for s, _ in rows]):>12.1f} "
            f"{_avg([s['depth'] for s, _ in rows]):>10.1f} "
            f"{_avg([s['premises'] for s, _ in rows]):>13.1f} "
            f"{sum(1 for s, _ in rows if s['open_goals'] > 0):>9}"
        )
    print("-" * 75)
    all_rows = list(ok.values())
    print(
        f"{'TOTAL':<12} {len(all_rows):>4} {sum(1 for _, v in all_rows if v == 'PROVED'):>7} "
        f"{_avg([s['tactics'] for s, _ in all_rows]):>12.1f} "
        f"{_avg([s['depth'] for s, _ in all_rows]):>10.1f} "
        f"{_avg([s['premises'] for s, _ in all_rows]):>13.1f} "
        f"{sum(1 for s, _ in all_rows if s['open_goals'] > 0):>9}"
    )
    print(
        "note: 'unlinked' counts goals the extractor's tactic-diff heuristic couldn't\n"
        "attach a closing tactic to (common with `<;>`/bullets/`cases` even in fully\n"
        "closed proofs) — it is NOT a sorry detector. For ground truth, use the\n"
        "verdict breakdown below (UNPROVED = actually has `sorry`)."
    )

    print(f"\n{'verdict':<14} {'n':>4} {'avg tactics':>12} {'avg depth':>10} {'avg premises':>13}")
    print("-" * 56)
    for verdict in sorted({v for _, v in ok.values()}):
        rows = [s for s, v in ok.values() if v == verdict]
        print(
            f"{verdict:<14} {len(rows):>4} "
            f"{_avg([s['tactics'] for s in rows]):>12.1f} "
            f"{_avg([s['depth'] for s in rows]):>10.1f} "
            f"{_avg([s['premises'] for s in rows]):>13.1f}"
        )

    # ---- Three-part premise provenance breakdown ----
    _prem_hdr = "".join(f" {col[1]:>{col[2]}}" for col in PREMISE_BUCKET_COLUMNS)
    print(f"\n{'category':<12} {'n':>4}{_prem_hdr}")
    print("-" * 54)
    for cat in categories:
        rows_s = [s for n, (s, v) in ok.items() if _category(n) == cat]
        agg_pb = _sum_buckets(rows_s, "premise_buckets")
        pp = _pct(agg_pb, PREMISE_BUCKETS)
        row_vals = "".join(f" {pp[col[0]]:>{col[2]-1}.1f}%" for col in PREMISE_BUCKET_COLUMNS)
        print(f"{cat:<12} {len(rows_s):>4}{row_vals}")
    print("-" * 54)
    agg_pb_all = _sum_buckets([s for s, _ in ok.values()], "premise_buckets")
    pp_all = _pct(agg_pb_all, PREMISE_BUCKETS)
    row_vals_all = "".join(f" {pp_all[col[0]]:>{col[2]-1}.1f}%" for col in PREMISE_BUCKET_COLUMNS)
    print(
        f"{'TOTAL':<12} {len(all_rows):>4}{row_vals_all}"
    )
    print(
        "note: prem:* = % of lemma references by def_path provenance.\n"
        "  aeneas = Result-monad plumbing; program = bridging lemmas/invariants in RustVerif;\n"
        "  math = Mathlib / Lean core."
    )

    # ---- Eight-bucket tactic composition breakdown ----
    _tac_hdr = "".join(f" {col[1]:>{col[2]}}" for col in BUCKET_COLUMNS)
    print(f"\n{'category':<12} {'n':>4}{_tac_hdr}")
    print("-" * 84)
    for cat in categories:
        rows_s = [s for n, (s, v) in ok.items() if _category(n) == cat]
        agg_tb = _sum_buckets(rows_s, "tactic_buckets")
        tp = _pct(agg_tb, ALL_TACTIC_BUCKETS)
        row_vals = "".join(f" {tp[col[0]]:>{col[2]-1}.1f}%" for col in BUCKET_COLUMNS)
        print(f"{cat:<12} {len(rows_s):>4}{row_vals}")
    print("-" * 84)
    agg_tb_all = _sum_buckets([s for s, _ in ok.values()], "tactic_buckets")
    tp_all = _pct(agg_tb_all, ALL_TACTIC_BUCKETS)
    row_vals_all = "".join(f" {tp_all[col[0]]:>{col[2]-1}.1f}%" for col in BUCKET_COLUMNS)
    print(f"{'TOTAL':<12} {len(all_rows):>4}{row_vals_all}")
    print(
        "note: tactic % of total tactic applications per category.\n"
        "  simp = simp/simp_all/simp_rw/aesop/gcongr; rewrite = rw/conv;\n"
        "  decompose = cases/rcases/obtain/induction/by_cases/cdot/split/use;\n"
        "  intro_bind = intro/have/let/subst/unfold/show; close = exact/apply/refine/rfl/calc;\n"
        "  combin. = <;>/first/try/all_goals/paren (glue around other tactics);\n"
        "  other = residual (tactic names not in any set above)."
    )

    top_tactics = _sum_buckets([s for s, _ in ok.values()], "tactic_histogram")
    print(f"\nmost common tactics: {', '.join(f'{t} ({c})' for t, c in top_tactics.most_common(15))}")

    biggest = sorted(ok.items(), key=lambda kv: -kv[1][0]["tactics"])[:5]
    print("\nlargest proof trees (by tactic count):")
    for name, (s, v) in biggest:
        print(f"  {name:<40} tactics={s['tactics']:<6} depth={s['depth']:<4} {v}")

    try:
        _plot(ok, out_dir)
    except ImportError:
        print("\nmatplotlib not installed, skipping chart")
