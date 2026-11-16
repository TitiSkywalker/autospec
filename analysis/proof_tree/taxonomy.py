"""Classification tables for the eight-bucket proof structure.

Proofs are split across eight tactic buckets (incl. ``combinator``) plus an "other" residual, and
three premise-provenance buckets.  This module owns all three levels:

* **tactics** — which of the eight buckets a tactic name belongs to
  (:func:`_tactic_bucket`), used when walking a proof tree and again
  (losslessly) when upgrading cached per-theorem statistics via
  :func:`_tactic_buckets_from_histogram`.
* **premises** — which bucket a cited lemma belongs to, decided by the file it
  is defined in (:func:`_premise_bucket`).
* **column spec** — :data:`BUCKET_COLUMNS` gives ``(key, header, width)`` for
  every tactic column in the terminal report, in the canonical display order.

The sets are the single source of truth for bucket membership; nothing else in
the package enumerates tactic names.
"""

from __future__ import annotations

from collections import Counter

# Aeneas's own symbolic-execution tactics over the Result monad.
_AENEAS_TACTICS: frozenset[str] = frozenset({
    "tacticScalar_tac_",
    "tacticScalar_decr_tac",
    "step",
    "step?",
    "stepStar",
    "tacticBstep__",
    "«tacticBstep_,_»",
    "«tacticBstep_As_,_»",
})
# Automated math decision procedures that close a goal purely by computation.
_MATH_AUTOMATION_TACTICS: frozenset[str] = frozenset({
    "omega", "ring", "ring_nf", "linarith", "nlinarith", "decide", "normNum",
    "positivity", "grind", "linearCombination", "fieldSimp", "pushCast",
    # Treant's internal / alternate spellings of the same decision procedures.
    "ringNF", "ring1", "abel", "abelNF", "noncomm_ring", "nativeDecide",
    "finiteness", "funPropTacStx", "measurability", "tacticModule",
    "tacticMatch_scalars", "agrindTactic",
})
# ---- Structural sub-buckets ----
# simp_family: rewriting-by-simplification (automated, but domain-agnostic).
# Includes internal AST names: tacticSimp_rw___ (simp_rw).
_SIMP_FAMILY: frozenset[str] = frozenset({
    "simp", "simp_all", "simp_rw", "tacticSimp_rw___", "aesop", "norm_cast", "push_cast",
    "simp_arith", "simpa", "gcongr", "dsimp",
    # internal names: simp_all, simpa!, aesop, norm_cast, push_cast, beta_reduce, push_neg
    "simpAll", "simpaUsingBang", "aesopTactic", "tacticNorm_cast__",
    "pushStx", "betaReduceStx", "push_neg", "zify",
    # agent-defined simp macros (ds_lct: lct_simp at; graph_bigraph_weight_match: ksimp at)
    "tacticLct_simpAt_", "tacticKsimpAt_",
})
# rewrite: explicit, directed rewriting steps.
_REWRITE_TACTICS: frozenset[str] = frozenset({
    "rwSeq", "rw", "conv", "conv_lhs", "conv_rhs", "rw_mod_cast",
    # internal names: rwa, erw, nth_rw, conv_lhs, conv_rhs; symm/trans reorient the goal
    "tacticRwa__", "tacticErw___", "tacticNth_rw_____", "convLHS", "convRHS",
    "symm", "tacticTrans___",
})
# decompose: goal/term decomposition, case splits, and focusing.
# Includes internal AST names: tacticFunext___ (funext), finCases (fin_cases).
_DECOMPOSE_TACTICS: frozenset[str] = frozenset({
    "cases", "rcases", "obtain", "induction", "«tacticBy_cases_:_»",
    "fin_cases", "finCases", "interval_cases", "split", "constructor", "use",
    "cdot", "·", "next", "case", "rename_i",
    "ext", "tacticFunext___", "congr", "byContra",
    # internal names: rename_i, split_ifs, left/right, interval_cases, use, choose,
    # match, next, rcases', injection, by_contra!, contrapose!, induction', wlog
    "renameI", "splitIfs", "left", "right", "intervalCases", "useSyntax", "choose",
    "match", "«tacticNext_=>_»", "tacticRcs_", "injection", "byContra!",
    "contrapose!", "induction'", "wlog", "casesM",
    # agent-defined macro: split_ifs, case-split, then close each branch (matrix_smith_normal_form)
    "tacticFin_ifs",
})
# intro_bind: hypothesis introduction and local definition.
# Includes internal AST names: tacticHave__ (have), tacticLet__ (let).
_INTRO_BIND_TACTICS: frozenset[str] = frozenset({
    "intro", "intros", "rintro", "tacticHave__", "have", "tacticLet__", "let", "set",
    "subst", "clear", "generalize", "revert", "suffices", "unfold",
    "show", "change",
    # internal names: set, suffices, haveI, subst_vars, replace
    "setTactic", "tacticSuffices_", "tacticHaveI__", "substVars", "replace", "delta", "clearExceptTactic",
})
# close: goal-closing tactics (produce a complete proof term).
# Includes internal AST names: tacticRfl (rfl), tacticExact_mod_cast_ (exact_mod_cast).
# NOTE: "decide" was here in the old 3-bucket model but belongs to
# math_automation (checked first in _tactic_bucket).
_CLOSE_TACTICS: frozenset[str] = frozenset({
    "exact", "apply", "refine", "rfl", "tacticRfl", "trivial", "assumption",
    "contradiction", "exfalso", "absurd", "tauto",
    "tacticExact_mod_cast_", "convert!", "filterUpwards",
    # internal names: exfalso, trivial; calc proofs; convert; infer_instance; ac_rfl
    "tacticExfalso", "tacticTrivial", "calcTactic", "convert", "tacticInfer_instance",
    "acRfl", "exacts",
    # agent-defined admissibility macros: apply the admissibility lemmas to close a fixpoint-induction goal
    "tacticPadm", "tacticPc_adm", "tacticOkimp_adm", "tacticProve_adm",
})
# combinator: tactics that only sequence / scope / repeat other tactics. The
# wrapped tactics are counted in their own buckets; this isolates the glue.
# Includes internal names: t1 <;> t2, all_goals, try, repeat, repeat', parenthesised
# blocks, tactic sequences, `first | ..`, focus/`swap`/`rotate_left`, `classical`.
_COMBINATOR_TACTICS: frozenset[str] = frozenset({
    "«tactic_<;>_»", "first", "allGoals", "paren", "tacticSeq", "tacticTry_",
    "tacticRepeat_", "repeat'", "seq_focus", "choice", "classical", "rotateLeft",
    "tacticSwap", "open",
})

# --- Disjointness invariant -------------------------------------------------
# Each tactic name must appear in at most one bucket so that counts are
# unambiguous.  This assertion is checked at import time so membership
# regressions are caught immediately.
_ALL_SETS = (
    _AENEAS_TACTICS, _MATH_AUTOMATION_TACTICS, _SIMP_FAMILY,
    _REWRITE_TACTICS, _DECOMPOSE_TACTICS, _INTRO_BIND_TACTICS, _CLOSE_TACTICS,
    _COMBINATOR_TACTICS,
)
for _i, _a in enumerate(_ALL_SETS):
    for _b in _ALL_SETS[_i + 1:]:
        assert not (_a & _b), f"tactic bucket overlap: {_a & _b}"
del _i, _a, _b


# Ordered bucket keys used throughout the package.
TACTIC_BUCKETS = (
    "aeneas", "math_automation",
    "simp_family", "rewrite", "decompose", "intro_bind", "close", "combinator",
)

# All tactic buckets including the residual catch-all.
ALL_TACTIC_BUCKETS: tuple[str, ...] = TACTIC_BUCKETS + ("other",)

# Premise-provenance bucket keys (aeneas plumbing / program RustVerif / math).
PREMISE_BUCKETS: tuple[str, ...] = ("aeneas", "program", "math")

# Report column spec for premise provenance: (bucket_key, header_label, total_width).
PREMISE_BUCKET_COLUMNS: tuple[tuple[str, str, int], ...] = (
    ("aeneas",  "prem:aeneas",  12),
    ("program", "prem:program", 13),
    ("math",    "prem:math",    10),
)

# Report column spec: (tactic_key, header_label, total_width).
# total_width covers the label (header) or the "XX.X%" value (value_width = total_width - 1).
BUCKET_COLUMNS: tuple[tuple[str, str, int], ...] = (
    ("aeneas",          "aeneas",     7),
    ("math_automation", "math_auto",  10),
    ("simp_family",     "simp",       6),
    ("rewrite",         "rewrite",    8),
    ("decompose",       "decompose",  10),
    ("intro_bind",      "intro_bind", 11),
    ("close",           "close",      6),
    ("combinator",      "combin.",    8),
    ("other",           "other",      6),
)

# Chart colors keyed by bucket name (dict lookup raises on missing key so a
# new bucket is never silently dropped from a chart).
TACTIC_BUCKET_COLORS: dict[str, str] = {
    "aeneas":          "#e15759",  # red
    "math_automation": "#76b7b2",  # teal
    "simp_family":     "#59a14f",  # green
    "rewrite":         "#f1ce63",  # yellow
    "decompose":       "#b07aa1",  # purple
    "intro_bind":      "#ff9da7",  # pink
    "close":           "#9c755f",  # brown
    "combinator":      "#d4a6c8",  # mauve
    "other":           "#bab0ac",  # gray
}

PREMISE_BUCKET_COLORS: dict[str, str] = {
    "aeneas":  "#4e79a7",  # blue
    "program": "#f28e2b",  # orange
    "math":    "#59a14f",  # green
}


def _tactic_bucket(name: str) -> str:
    """Fine-grained proof-part bucket for one tactic name.

    Returns one of: aeneas / math_automation / simp_family / rewrite /
    decompose / intro_bind / close / combinator / other  (last is the residual
    catch-all).

    The checks are ordered: a name is assigned to the first matching set.
    The bucket frozensets are pairwise disjoint (asserted at import time), so
    ordering only matters for the final ``other`` fallback.
    """
    if name in _AENEAS_TACTICS:
        return "aeneas"
    if name in _MATH_AUTOMATION_TACTICS:
        return "math_automation"
    if name in _SIMP_FAMILY:
        return "simp_family"
    if name in _REWRITE_TACTICS:
        return "rewrite"
    if name in _DECOMPOSE_TACTICS:
        return "decompose"
    if name in _INTRO_BIND_TACTICS:
        return "intro_bind"
    if name in _CLOSE_TACTICS:
        return "close"
    if name in _COMBINATOR_TACTICS:
        return "combinator"
    return "other"


def _tactic_buckets_from_histogram(histogram: dict) -> Counter:
    """Recompute tactic bucket counts from a raw tactic-name → count histogram.

    Used to upgrade cached per-theorem stats that stored the old 3-bucket
    ``tactic_buckets`` (or none at all) without requiring a lake rebuild:
    ``tactic_histogram`` is always written to the cache, so this is lossless.
    """
    buckets: Counter = Counter()
    for tactic_name, count in histogram.items():
        buckets[_tactic_bucket(tactic_name)] += count
    return buckets


def _premise_bucket(def_path: str) -> str:
    """Proof-part bucket for one premise reference, determined by where it is defined.

    * ``aeneas``  — Aeneas Std/Tactic library (monadic plumbing).
    * ``program`` — the testcase's own RustVerif files (translated function +
                    loop invariants / bridging lemmas the agent wrote).
    * ``math``    — everything else (Mathlib + Lean core).
    """
    p = def_path.replace("\\", "/")
    if "/aeneas/backends/lean/" in p:
        return "aeneas"
    if "/RustVerif/" in p:
        return "program"
    return "math"
