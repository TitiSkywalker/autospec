import LeanMode.ds_segment_tree.DsSegmentTree
import Mathlib

open Aeneas Aeneas.Std Result
open ds_segment_tree

set_option linter.unusedVariables false

/-!
# Specification: Iterative Segment Tree (range sum)

`segtree_build` / `point_update` / `range_sum` maintain a segment tree over an array `a` of length `n`.  `range_sum (l, r)` returns the inclusive-range sum `Σ a[l..=r]`; `point_update (i, v)` sets `a[i]` to `v`.  Queries are assumed in range (`l ≤ r < n`).
-/

/-- Sum of `xs[l ..= r]` (inclusive). -/
def sumRange (xs : List Int) (l r : Nat) : Int :=
  ((xs.drop l).take (r - l + 1)).sum


/-! ## Specification theorems -/

/-- `segtree_build` answers every in-range query with the true range sum. -/
theorem build_range_sum (a : Slice Int) (st : SegTree) (l r : Std.Usize) (s : Int)
    (h_build : segtree_build a = ok st)
    (h_ge : l.val ≤ r.val)
    (h_lt : r.val < (a : List Int).length)
    (h_q : range_sum st l r = ok s) :
    s = sumRange (a : List Int) l.val r.val := by
  sorry

/-- After `point_update (i, v)`, every in-range query reads the range sum of
    `a[i := v]`.  This is the full point-update correctness property. -/
theorem point_update_range_sum (a : Slice Int) (st st' : SegTree)
    (i l r : Std.Usize) (v s : Int)
    (h_build : segtree_build a = ok st)
    (h_upd : point_update st i v = ok st')
    (h_ge : l.val ≤ r.val)
    (h_lt : r.val < (a : List Int).length)
    (h_q : range_sum st' l r = ok s) :
    s = sumRange ((a : List Int).set i.val v) l.val r.val := by
  sorry

/-- A single-element range on a freshly built tree reads the input element. -/
theorem build_range_sum_single (a : Slice Int) (st : SegTree) (i : Std.Usize) (s : Int)
    (h_build : segtree_build a = ok st)
    (h_lt : i.val < (a : List Int).length)
    (h_q : range_sum st i i = ok s) :
    s = (a : List Int).getD i.val 0 := by
  sorry
