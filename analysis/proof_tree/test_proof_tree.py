"""Pure unit tests for proof_tree taxonomy, stats, and bucket logic.

Covers:
- _tactic_bucket: one representative per bucket + unknown name
- _premise_bucket: the three def_path shapes
- _case_summary: two-theorem fixture including a stale 3-bucket cache entry

Run with:
    python3 -m pytest analysis/proof_tree/test_proof_tree.py -q
"""

from __future__ import annotations

from collections import Counter

import pytest

from analysis.proof_tree.taxonomy import (
    _premise_bucket,
    _tactic_bucket,
)
from analysis.proof_tree.stats import _case_summary


# ---------------------------------------------------------------------------
# _tactic_bucket
# ---------------------------------------------------------------------------

@pytest.mark.parametrize("name,expected", [
    # one representative from each bucket
    ("step",              "aeneas"),
    ("omega",             "math_automation"),
    ("simp",              "simp_family"),
    ("rw",                "rewrite"),
    ("cases",             "decompose"),
    ("intro",             "intro_bind"),
    ("exact",             "close"),
    ("«tactic_<;>_»", "combinator"),
    # residual
    ("totally_unknown",   "other"),
    # decide must land in math_automation (not close)
    ("decide",            "math_automation"),
    # internal AST names map to the same bucket as their source keyword
    ("tacticLet__",       "intro_bind"),
    ("tacticHave__",      "intro_bind"),
    ("tacticRfl",         "close"),
    ("tacticExact_mod_cast_", "close"),
    ("tacticSimp_rw___",  "simp_family"),
    ("finCases",          "decompose"),
    ("tacticFunext___",   "decompose"),
    # math-specific tactics added to the taxonomy
    ("dsimp",             "simp_family"),
    ("ext",               "decompose"),
    ("congr",             "decompose"),
    ("byContra",          "decompose"),
    ("convert!",          "close"),
    ("filterUpwards",     "close"),
    # Treant's internal spellings (previously fell through to "other")
    ("renameI",           "decompose"),
    ("splitIfs",          "decompose"),
    ("simpAll",           "simp_family"),
    ("ringNF",            "math_automation"),
    ("tacticExfalso",     "close"),
    ("tacticRwa__",       "rewrite"),
    ("setTactic",         "intro_bind"),
    ("first",             "combinator"),
    ("allGoals",          "combinator"),
])
def test_tactic_bucket(name, expected):
    assert _tactic_bucket(name) == expected


# ---------------------------------------------------------------------------
# _premise_bucket
# ---------------------------------------------------------------------------

@pytest.mark.parametrize("def_path,expected", [
    ("/home/user/.elan/toolchains/aeneas/backends/lean/AeneasLib/Std.lean", "aeneas"),
    ("/home/user/project/RustVerif/Spec.lean",                              "program"),
    ("/home/user/.elan/toolchains/mathlib4/Mathlib/Algebra.lean",           "math"),
])
def test_premise_bucket(def_path, expected):
    assert _premise_bucket(def_path) == expected


# ---------------------------------------------------------------------------
# _case_summary
# ---------------------------------------------------------------------------

def _make_theorem(
    tactic_histogram: dict,
    premise_buckets: dict,
    premises: list[str],
    *,
    tactic_buckets: dict | None = None,
    tactics: int = 10,
    depth: int = 5,
    open_goals: int = 0,
):
    """Build a minimal per-theorem stat dict as stored in the cache."""
    entry = {
        "tactics": tactics,
        "depth": depth,
        "premises": premises,          # old cache key: list[str]
        "tactic_histogram": tactic_histogram,
        "premise_buckets": premise_buckets,
        "open_goals": open_goals,
    }
    if tactic_buckets is not None:
        entry["tactic_buckets"] = tactic_buckets
    return entry


def test_case_summary_basic():
    """Two-theorem fixture: aggregate tactics/depth/premises correctly."""
    th1 = _make_theorem(
        tactic_histogram={"simp": 3, "omega": 2},
        premise_buckets={"aeneas": 1, "math": 2},
        premises=["Mathlib.Foo", "Aeneas.Bar"],
        tactics=5,
        depth=4,
    )
    th2 = _make_theorem(
        tactic_histogram={"exact": 4, "intro": 1},
        premise_buckets={"program": 3},
        premises=["RustVerif.Spec.helper"],  # overlapping with nothing above
        tactics=5,
        depth=6,
    )
    result = {"theorems": [th1, th2]}
    s = _case_summary(result)
    assert s is not None
    assert s["n_theorems"] == 2
    assert s["tactics"] == 10
    assert s["depth"] == 6       # max of 4, 6
    assert s["premises"] == 3    # 3 distinct names
    assert s["open_goals"] == 0

    # tactic buckets recomputed from histogram (7-bucket model)
    assert s["tactic_buckets"]["simp_family"] == 3
    assert s["tactic_buckets"]["math_automation"] == 2
    assert s["tactic_buckets"]["close"] == 4
    assert s["tactic_buckets"]["intro_bind"] == 1

    # premise buckets aggregated
    pb = Counter(s["premise_buckets"])
    assert pb["aeneas"] == 1
    assert pb["math"] == 2
    assert pb["program"] == 3


def test_case_summary_stale_3bucket_cache():
    """Old cache entries with a stale tactic_buckets are upgraded from histogram."""
    # Stale entry has 3-bucket split (structural / aeneas / math); histogram
    # is the source of truth for the 7-bucket recomputation.
    th = _make_theorem(
        tactic_histogram={"simp": 5, "rw": 2, "exact": 3},
        premise_buckets={},
        premises=[],
        tactic_buckets={"structural": 10},   # stale 3-bucket entry
        tactics=10,
        depth=3,
    )
    result = {"theorems": [th]}
    s = _case_summary(result)
    assert s is not None
    # Must produce 7-bucket result, not pass through the stale 3-bucket value
    assert "structural" not in s["tactic_buckets"]
    assert s["tactic_buckets"]["simp_family"] == 5
    assert s["tactic_buckets"]["rewrite"] == 2
    assert s["tactic_buckets"]["close"] == 3


def test_case_summary_error_result():
    """Error dicts (no 'theorems' key) return None."""
    assert _case_summary({"name": "foo", "error": "build_fail"}) is None


def test_case_summary_empty_theorems():
    """Empty theorem list returns a zero-valued summary, not None."""
    s = _case_summary({"theorems": []})
    assert s is not None
    assert s["n_theorems"] == 0
    assert s["tactics"] == 0
