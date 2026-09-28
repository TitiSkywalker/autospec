import LeanMode.ds_fenwick.DsFenwick
import Mathlib

open Aeneas Aeneas.Std Result
open ds_fenwick

set_option linter.unusedVariables false

/-!
# Specification: Fenwick Tree (binary-indexed tree)

`bit_new` / `update` / `query` maintain a Fenwick tree over `0 .. n`.
`update (i, delta)` adds `delta` to position `i`; `query (j)` returns the
prefix sum `a[0..=j]`.  Queries are assumed in range (`j < n`).
-/


/-! ## Specification theorems -/

/-- An empty tree answers every in-range prefix query with `0`. -/
theorem bit_new_query_zero (n i : Std.Usize) (b : BIT) (s : Int)
    (h_new : bit_new n = ok b)
    (h_lt : i.val < n.val)
    (h_q : query b i = ok s) :
    s = 0 := by
  sorry

/-- A point update at `i` shifts the prefix sum at every `j ≥ i` by exactly `delta`. -/
theorem update_query_ge (b : BIT) (i j : Std.Usize) (delta : Int) (b' : BIT)
    (h : update b i delta = ok b')
    (h_ge : i.val ≤ j.val)
    (h_lt : j.val < b.n.val)
    (s0 s1 : Int)
    (hq0 : query b j = ok s0)
    (hq1 : query b' j = ok s1) :
    s1 = s0 + delta := by
  sorry

/-- A point update at `i` leaves the prefix sum at every `j < i` unchanged. -/
theorem update_query_lt (b : BIT) (i j : Std.Usize) (delta : Int) (b' : BIT)
    (h : update b i delta = ok b')
    (h_lt : j.val < i.val)
    (h_lt_i : i.val < b.n.val)
    (s0 s1 : Int)
    (hq0 : query b j = ok s0)
    (hq1 : query b' j = ok s1) :
    s1 = s0 := by
  sorry
