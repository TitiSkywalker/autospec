"""Runnable facade for all proof-tree measurement experiments.

Four stages; three corpora are measured with the same per-theorem statistics (7 tactic
buckets + 3 premise-provenance buckets) so results are directly comparable:

* **vericoding**    — agent-written Lean proofs of Rust programs via Aeneas.
* **navier_stokes** — OpenAI NavierStokes & Euler blowup formalization.
* **minif2f**       — control: LLM-solved miniF2F competition-math proofs.
* **comparison**    — side-by-side tactic/premise mix of the corpora.

Usage::

    python3 -m analysis.proof_tree.experiments                     # all (cached)
    python3 -m analysis.proof_tree.experiments --only vericoding
    python3 -m analysis.proof_tree.experiments --only navier_stokes
    python3 -m analysis.proof_tree.experiments --only minif2f
    python3 -m analysis.proof_tree.experiments --only comparison
    python3 -m analysis.proof_tree.experiments --rebuild            # vericoding bypass
    python3 -m analysis.proof_tree.experiments --no-cache           # NS re-extract
"""

from __future__ import annotations

import argparse
import json
import sys
from collections import Counter
from pathlib import Path
from typing import Optional

from analysis.proof_tree import minif2f
from analysis.proof_tree.config import DEFAULT_VERDICTS, RESULTS_DIR, ROOT, cache_dir_for
from analysis.proof_tree.report import report
from analysis.proof_tree.stats import _case_summary, _pct
from analysis.proof_tree.taxonomy import ALL_TACTIC_BUCKETS, PREMISE_BUCKETS
from analysis.proof_tree.workspace import discover_cases, ensure_workspace, process_case


# ── helpers ────────────────────────────────────────────────────────────────────

def _load_ns_module():
    """Return the measure_navier_stokes module, ensuring treant is on sys.path."""
    # The module itself adds the treant path as a side-effect of being imported.
    from analysis.proof_tree.local import measure_navier_stokes as _ns
    return _ns


def _aggregate_corpus(results: dict[str, dict]) -> dict:
    """Compute corpus-level totals across per-case results (any corpus with ``theorems`` lists)."""
    tac_total: Counter = Counter()
    prem_total: Counter = Counter()
    all_premises: set[str] = set()
    tactics_sum = 0
    depth_max = 0
    open_goals_sum = 0
    n_theorems_sum = 0

    for result in results.values():
        s = _case_summary(result)
        if s is None:
            continue
        tac_total.update(s.get("tactic_buckets", {}))
        prem_total.update(s.get("premise_buckets", {}))
        tactics_sum += s["tactics"]
        depth_max = max(depth_max, s["depth"])
        open_goals_sum += s["open_goals"]
        n_theorems_sum += s["n_theorems"]
        for th in result.get("theorems", []):
            all_premises.update(th.get("premise_names", th.get("premises", [])))

    return {
        "n_cases": len(results),
        "n_theorems": n_theorems_sum,
        "tactics": tactics_sum,
        "depth": depth_max,
        "premises": len(all_premises),
        "open_goals": open_goals_sum,
        "tactic_bucket_totals": dict(tac_total),
        "premise_bucket_totals": dict(prem_total),
    }


# ── Main class ─────────────────────────────────────────────────────────────────

class ProofTreeExperiments:
    """One entry point for all proof-tree measurement experiments.

    Three corpora, all measured with the same per-theorem statistics
    (7 tactic buckets + 3 premise-provenance buckets) so they are directly
    comparable:

    * **vericoding**     — agent-written Lean proofs of Rust programs via Aeneas
                           (Aeneas plumbing + program-specific lemmas + Mathlib).
    * **navier_stokes**  — the OpenAI NavierStokes & Euler blowup formalization,
                           a large pure-Mathlib proof (no Aeneas, no Rust).
    * **minif2f**        — control: LLM-solved miniF2F competition-math proofs
                           (short, pure Mathlib, same kind of prover as vericoding).
    * **comparison**     — side-by-side tactic/premise mix of the corpora.

    Each ``run_*`` method loads/extracts its corpus, computes statistics via the
    shared stats/taxonomy modules, saves a JSON summary + figures to
    ``self.local_dir``, prints full numbers to the terminal, and returns the
    summary dict.  ``run_all`` orchestrates the three.
    """

    def __init__(self, local_dir: "Path | str | None" = None) -> None:
        if local_dir is None:
            local_dir = ROOT / "analysis/proof_tree/local"
        self.local_dir = Path(local_dir)
        self.figs_dir = self.local_dir / "figs"
        self.local_dir.mkdir(parents=True, exist_ok=True)
        self.figs_dir.mkdir(parents=True, exist_ok=True)

    # ── vericoding ─────────────────────────────────────────────────────────────

    def run_vericoding(
        self,
        *,
        rebuild: bool = False,
        results_dir: str = RESULTS_DIR,
        cache_key: Optional[str] = None,
    ) -> dict:
        """Load or rebuild the vericoding corpus, print tables, save figures, return summary.

        cache_key: subdirectory key within results cache, typically the model name.
        """
        _results_dir = ROOT / results_dir
        cache_dir = cache_dir_for(_results_dir, cache_key)

        results: dict[str, dict] = {}

        if rebuild:
            ws = ensure_workspace()
            cache_dir.mkdir(parents=True, exist_ok=True)
            cases = discover_cases(_results_dir)
            proved_cases = [(n, v) for n, v in cases if v in DEFAULT_VERDICTS]
            for i, (name, verdict) in enumerate(proved_cases, 1):
                print(
                    f"[{i}/{len(proved_cases)}] {name}: building + extracting...",
                    file=sys.stderr,
                )
                result = process_case(name, _results_dir / name, ws)
                result["verdict"] = verdict
                cache_file = cache_dir / f"{name}.json"
                cache_file.write_text(json.dumps(result))
                results[name] = result
        else:
            if cache_dir.is_dir():
                for f in sorted(cache_dir.glob("*.json")):
                    try:
                        results[f.stem] = json.loads(f.read_text())
                    except (json.JSONDecodeError, OSError) as exc:
                        print(f"warning: skipping {f.name}: {exc}", file=sys.stderr)
            else:
                print(
                    f"warning: cache directory not found: {cache_dir}",
                    file=sys.stderr,
                )

        # Filter to DEFAULT_VERDICTS before reporting.
        proved_results = {
            n: r for n, r in results.items()
            if r.get("verdict") in DEFAULT_VERDICTS
        }

        report(proved_results, verdict_scope=DEFAULT_VERDICTS, out_dir=self.figs_dir)

        try:
            from analysis.proof_tree.distribution import run as dist_run
            dist_run(
                results_dir=results_dir,
                model=cache_key,
                out=str(self.figs_dir / "proof_tree_size_by_category.png"),
            )
        except ImportError:
            print("matplotlib not installed, skipping box-plot", file=sys.stderr)

        summary = _aggregate_corpus(proved_results)
        summary_path = self.local_dir / "vericoding_summary.json"
        summary_path.write_text(json.dumps(summary, indent=2))

        print(
            f"\nvericoding aggregate: {summary['n_cases']} cases, "
            f"{summary['n_theorems']} theorems, "
            f"{summary['tactics']} tactics total, "
            f"max depth {summary['depth']}, "
            f"{summary['premises']} distinct premises"
        )
        return summary

    # ── navier_stokes ──────────────────────────────────────────────────────────

    def run_navier_stokes(self, *, no_cache: bool = False) -> dict:
        """Load or extract the NS/Euler corpus, print tables, save JSON, return summary.

        The 25-minute extraction is NEVER run by default — load ``ns_euler_raw.jsonl``
        and rebuild the import-graph roots via ``ImportGraph`` instead.  Only the
        ``--no-cache`` flag triggers a full re-extraction.
        """
        ns = _load_ns_module()
        raw_jsonl = ns.RAW_JSONL

        if no_cache or not raw_jsonl.is_file():
            print("extracting NavierStokes & Euler (this can take ~25 min)...", file=sys.stderr)
            traces, roots = ns.extract(raw_jsonl)
        else:
            print(f"loading cached extraction: {raw_jsonl}", file=sys.stderr)
            from treant.lean.extracted_data import ExtractedData
            from treant.lean.import_graph import ImportGraph
            traces = ExtractedData.from_jsonl(str(raw_jsonl))
            roots = ImportGraph.build_project(ns.TREANT_PROJECT).roots()

        print(f"building proof trees + measuring {len(traces)} traces...", file=sys.stderr)
        result = ns.build_and_measure(traces, roots)
        theorems = result["theorems"]

        theorems_path = self.local_dir / "ns_euler_theorems.json"
        theorems_path.write_text(json.dumps(result, indent=2))

        summary = ns.summarize(theorems)
        summary_path = self.local_dir / "ns_euler_summary.json"
        summary_path.write_text(json.dumps(summary, indent=2))

        self._print_ns_report(summary)
        return summary

    def _print_ns_report(self, summary: dict) -> None:
        """Print the full NS bucket tables to the terminal."""
        n_thm = summary.get("n_theorems", 0)
        print(f"\n=== NavierStokes & Euler ({n_thm} theorems) ===")
        print(f"tactics total:      {summary['tactics']}")
        print(f"avg tactics/thm:    {summary['tactics'] / max(n_thm, 1):.1f}")
        print(f"max depth:          {summary['depth']}")
        print(f"distinct premises:  {summary['premises']}")
        print(f"open goals:         {summary['open_goals']}")

        # OO-017: use a single Counter as the source for both count and pct.
        tb = Counter(summary.get("tactic_bucket_totals", {}))
        tp = _pct(tb, ALL_TACTIC_BUCKETS)
        print("\ntactic buckets:")
        print(f"{'bucket':<18} {'count':>8} {'%':>7}")
        print("-" * 35)
        for k in ALL_TACTIC_BUCKETS:
            print(f"{k:<18} {tb[k]:>8} {tp[k]:>6.1f}%")

        pb = Counter(summary.get("premise_bucket_totals", {}))
        pp = _pct(pb, PREMISE_BUCKETS)
        print("\npremise provenance:")
        print(f"{'bucket':<18} {'count':>8} {'%':>7}")
        print("-" * 35)
        for k in PREMISE_BUCKETS:
            print(f"{k:<18} {pb[k]:>8} {pp[k]:>6.1f}%")

    # ── minif2f ────────────────────────────────────────────────────────────────

    def run_minif2f(self, *, rebuild: bool = False) -> dict:
        """Measure the miniF2F control corpus, save JSON, print tables, return summary."""
        results = minif2f.load_or_build(rebuild=rebuild)
        ok = {n: r for n, r in results.items() if "error" not in r}
        failed = Counter(r["error"] for r in results.values() if "error" in r)
        if failed:
            print(f"minif2f samples skipped: {dict(failed)}", file=sys.stderr)

        summary = _aggregate_corpus(ok)
        summary["n_cases"] = len(ok)
        (self.local_dir / "minif2f_summary.json").write_text(json.dumps(summary, indent=2))

        print(
            f"\nminif2f aggregate: {summary['n_cases']}/{len(results)} samples, "
            f"{summary['n_theorems']} theorems, {summary['tactics']} tactics total, "
            f"max depth {summary['depth']}, {summary['premises']} distinct premises"
        )
        return summary

    # ── comparison ─────────────────────────────────────────────────────────────

    def run_comparison(
        self,
        *,
        results_dir: str = RESULTS_DIR,
        cache_key: Optional[str] = None,
    ) -> dict:
        """Build and save comparison figures/JSON from the two saved summaries.

        Loads ``vericoding_summary.json`` and ``ns_euler_summary.json`` from
        ``self.local_dir``.  If either is missing the corresponding ``run_*``
        method is called first — note that a missing NS summary triggers the
        ~25-min extraction.

        Args:
            results_dir: passed to :meth:`run_vericoding` on auto-fallback.
            cache_key:   cache namespace, passed to :meth:`run_vericoding` on
                         auto-fallback (typically the model name).
        """
        paths = {
            "vericoding": self.local_dir / "vericoding_summary.json",
            "navier_stokes": self.local_dir / "ns_euler_summary.json",
            "minif2f": self.local_dir / "minif2f_summary.json",
        }
        if not paths["vericoding"].is_file():
            print("warning: vericoding_summary.json missing — running vericoding first...", file=sys.stderr)
            self.run_vericoding(results_dir=results_dir, cache_key=cache_key)
        if not paths["navier_stokes"].is_file():
            print(
                "warning: ns_euler_summary.json missing — running navier_stokes first "
                "(this can take ~25 min)...",
                file=sys.stderr,
            )
            self.run_navier_stokes()
        if not paths["minif2f"].is_file():
            print("warning: minif2f_summary.json missing — running minif2f first...", file=sys.stderr)
            self.run_minif2f()

        tactic_pct: dict[str, dict] = {}
        premise_pct: dict[str, dict] = {}
        for label, path in paths.items():
            summary = json.loads(path.read_text())
            tactic_pct[label] = _pct(Counter(summary.get("tactic_bucket_totals", {})), ALL_TACTIC_BUCKETS)
            premise_pct[label] = _pct(Counter(summary.get("premise_bucket_totals", {})), PREMISE_BUCKETS)

        comparison = {"tactic_pct": tactic_pct, "premise_pct": premise_pct}
        (self.local_dir / "comparison.json").write_text(json.dumps(comparison, indent=2))

        from analysis.proof_tree import plots
        plots.plot_comparison(tactic_pct, premise_pct, self.figs_dir)

        self._print_comparison("Tactic composition", ALL_TACTIC_BUCKETS, tactic_pct)
        self._print_comparison("Premise provenance", PREMISE_BUCKETS, premise_pct)
        return comparison

    @staticmethod
    def _print_comparison(title: str, buckets: tuple, pct: dict[str, dict]) -> None:
        """Print one side-by-side table: a row per bucket, a column per corpus."""
        print(f"\n=== {title} comparison (%) ===\n")
        print(f"{'bucket':<18}" + "".join(f"{label:>15}" for label in pct))
        print("-" * (18 + 15 * len(pct)))
        for k in buckets:
            print(f"{k:<18}" + "".join(f"{p[k]:>14.1f}%" for p in pct.values()))

    # ── orchestrator ───────────────────────────────────────────────────────────

    def run_all(
        self,
        *,
        vericoding: bool = True,
        navier_stokes: bool = True,
        minif2f: bool = True,
        comparison: bool = True,
        vericoding_rebuild: bool = False,
        navier_stokes_no_cache: bool = False,
        minif2f_rebuild: bool = False,
        results_dir: str = RESULTS_DIR,
        cache_key: Optional[str] = None,
    ) -> dict:
        """Run selected experiments in order, using caches by default.

        Args:
            vericoding:           run the vericoding experiment.
            navier_stokes:        run the NavierStokes experiment.
            minif2f:              run the miniF2F control experiment.
            comparison:           run the comparison experiment.
            vericoding_rebuild:   bypass the vericoding per-case cache and rebuild from scratch.
            navier_stokes_no_cache: re-extract NavierStokes raw JSONL (~25 min).
            minif2f_rebuild:      bypass the miniF2F per-sample cache.
            results_dir:          vericoding results directory.
            cache_key:            subdirectory key within results cache, typically the model name.

        Returns:
            dict with keys "vericoding", "navier_stokes", "comparison" (each the
            summary dict from the corresponding run, or None if not run).
        """
        _sep = "=" * 60
        results: dict = {"vericoding": None, "navier_stokes": None, "minif2f": None, "comparison": None}

        if vericoding:
            print(f"\n{_sep}")
            print("EXPERIMENT: vericoding")
            print(_sep)
            results["vericoding"] = self.run_vericoding(
                rebuild=vericoding_rebuild,
                results_dir=results_dir,
                cache_key=cache_key,
            )

        if navier_stokes:
            print(f"\n{_sep}")
            print("EXPERIMENT: navier_stokes")
            print(_sep)
            results["navier_stokes"] = self.run_navier_stokes(no_cache=navier_stokes_no_cache)

        if minif2f:
            print(f"\n{_sep}")
            print("EXPERIMENT: minif2f")
            print(_sep)
            results["minif2f"] = self.run_minif2f(rebuild=minif2f_rebuild)

        if comparison:
            print(f"\n{_sep}")
            print("EXPERIMENT: comparison")
            print(_sep)
            # Ensure summary files exist; if a corpus wasn't run this session,
            # check disk — the method itself will run any that are still missing.
            results["comparison"] = self.run_comparison(
                results_dir=results_dir,
                cache_key=cache_key,
            )

        return results


EXPERIMENTS = ["vericoding", "navier_stokes", "minif2f", "comparison"]


# ── CLI ────────────────────────────────────────────────────────────────────────

def main() -> None:
    ap = argparse.ArgumentParser(
        description="Run all proof-tree measurement experiments (vericoding, NavierStokes, miniF2F, comparison)"
    )
    ap.add_argument("--rebuild", action="store_true", help="rebuild vericoding and miniF2F caches from scratch")
    ap.add_argument("--no-cache", action="store_true", dest="no_cache",
                    help="re-extract NavierStokes raw JSONL (triggers ~25 min extraction)")
    ap.add_argument("--results-dir", default=RESULTS_DIR, dest="results_dir",
                    help="vericoding results directory (default: %(default)s)")
    ap.add_argument("--model", default=None,
                    help="cache namespace override (default: basename of --results-dir)")
    ap.add_argument("--only", action="append", dest="only",
                    choices=EXPERIMENTS,
                    metavar="{" + ",".join(EXPERIMENTS) + "}",
                    help="run only this experiment (repeatable; default: all)")
    args = ap.parse_args()

    # Bootstrap sys.path for sub-packages.
    if str(ROOT) not in sys.path:
        sys.path.insert(0, str(ROOT))

    run_set = set(args.only or EXPERIMENTS)

    expt = ProofTreeExperiments()
    expt.run_all(
        vericoding="vericoding" in run_set,
        navier_stokes="navier_stokes" in run_set,
        minif2f="minif2f" in run_set,
        comparison="comparison" in run_set,
        vericoding_rebuild=args.rebuild,
        navier_stokes_no_cache=args.no_cache,
        minif2f_rebuild=args.rebuild,
        results_dir=args.results_dir,
        cache_key=args.model,
    )


if __name__ == "__main__":
    main()
