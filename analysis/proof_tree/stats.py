"""Case-level aggregation over per-theorem proof-tree statistics.

This module is pure: it takes dictionaries (as produced by the extractor and
stored in the cache) and returns dictionaries/lists. No filesystem, no Lean, no
plotting — which is what lets the report and both charting entry points share
the same numbers.
"""

from __future__ import annotations

from collections import Counter
from typing import Optional

from analysis.proof_tree.taxonomy import (
    _premise_bucket,
    _tactic_bucket,
    _tactic_buckets_from_histogram,
)


def _category(name: str) -> str:
    return name.split("_", 1)[0]


def _pct(counts: dict, keys: tuple) -> dict:
    """Return ``{key: percentage}`` where percentages sum to 100 over ``keys``.

    The denominator is ``sum(counts.get(k, 0) for k in keys)``; if that is
    zero every key maps to 0.0 to avoid division-by-zero.
    """
    total = sum(counts.get(k, 0) for k in keys) or 1
    return {k: 100.0 * counts.get(k, 0) / total for k in keys}


def _sum_buckets(rows: list[dict], key: str) -> Counter:
    """Aggregate a bucket-count dict across a list of case-summary dicts.

    ``rows`` is a list of summary dicts as returned by :func:`_case_summary`.
    ``key`` is the dict field to aggregate (e.g. ``"tactic_buckets"`` or
    ``"premise_buckets"``).  Missing keys are silently skipped.
    """
    agg: Counter = Counter()
    for s in rows:
        agg.update(s.get(key, {}))
    return agg


def _tree_stats(roots) -> dict:
    """Walk a forest of GoalNode roots and summarize size/depth/premises.

    Tactic nodes are de-duplicated by identity (``id(tactic)``) so a shared
    subtree is only counted once; ``depth`` is still the longest chain of
    goal/tactic alternations reachable from any root.  A GoalNode with no
    tactic (``goal.tactic is None``) is an unresolved leaf (``sorry``).

    Eight-bucket structural breakdown (counts of tactic applications):
    * **aeneas**          — Aeneas plumbing (symbolic execution over Result monad).
    * **math_automation** — automated decision procedures (omega/ring/linarith/…).
    * **simp_family**     — rewriting by simplification (simp/simp_all/aesop/…).
    * **rewrite**         — explicit directed rewriting (rw/conv/…).
    * **decompose**       — goal decomposition and case splits.
    * **intro_bind**      — hypothesis introduction and local binding.
    * **close**           — goal-closing tactics (exact/apply/refine/rfl/calc/…).
    * **combinator**      — glue that only sequences/scopes other tactics (<;>/first/try/all_goals/…).

    Premise-provenance breakdown (``premise_buckets``):
    * **aeneas**   — Aeneas Std/Tactic library.
    * **program**  — testcase-specific RustVerif files.
    * **math**     — Mathlib / Lean core.
    """
    tactic_counter: Counter = Counter()
    tactic_buckets: Counter = Counter()
    premise_buckets: Counter = Counter()
    premises: set[str] = set()
    seen: set[int] = set()
    open_goals = 0
    max_depth = 0

    def visit(goal, depth: int) -> None:
        nonlocal open_goals, max_depth
        max_depth = max(max_depth, depth)
        tn = goal.tactic
        if tn is None:
            open_goals += 1
            return
        tid = id(tn.tactic)
        if tid not in seen:
            seen.add(tid)
            tactic_counter[tn.name_short] += 1
            tactic_buckets[_tactic_bucket(tn.name_short)] += 1
            for p in tn.tactic.premises:
                premises.add(p.full_name)
                premise_buckets[_premise_bucket(p.def_path)] += 1
        for child in tn.goals:
            visit(child, depth + 1)

    for r in roots:
        visit(r, 0)

    sorted_premises = sorted(premises)
    return {
        "tactics": sum(tactic_counter.values()),
        "depth": max_depth,
        # "premises" keeps the list under its original key for cache
        # compatibility; "premise_names" is the canonical theorem-level key.
        "premises": sorted_premises,
        "premise_names": sorted_premises,
        "tactic_histogram": dict(tactic_counter),
        "tactic_buckets": dict(tactic_buckets),
        "premise_buckets": dict(premise_buckets),
        "open_goals": open_goals,
    }


def _case_summary(result: dict) -> Optional[dict]:
    """Reduce one case's per-theorem stats to case-level aggregates."""
    theorems = result.get("theorems")
    if theorems is None:
        return None
    premises: set[str] = set()
    histogram: Counter = Counter()
    premise_buckets: Counter = Counter()
    tactics = depth = open_goals = 0
    for st in theorems:
        tactics += st["tactics"]
        depth = max(depth, st["depth"])
        open_goals += st["open_goals"]
        # Read premise_names (new key) with fallback to premises (old cache key).
        premises.update(st.get("premise_names", st.get("premises", [])))
        histogram.update(st["tactic_histogram"])
        premise_buckets.update(st.get("premise_buckets", {}))
    # Always recompute tactic buckets from the histogram so the 8-bucket
    # split is accurate even for old cache entries that stored the 3-bucket
    # version (or none at all).
    tactic_buckets = _tactic_buckets_from_histogram(histogram)
    return {
        "n_theorems": len(theorems),
        "tactics": tactics,
        "depth": depth,
        "premises": len(premises),
        "open_goals": open_goals,
        "tactic_histogram": dict(histogram),
        "tactic_buckets": dict(tactic_buckets),
        "premise_buckets": dict(premise_buckets),
    }


def _avg(xs: list[float]) -> float:
    return sum(xs) / len(xs) if xs else 0.0
