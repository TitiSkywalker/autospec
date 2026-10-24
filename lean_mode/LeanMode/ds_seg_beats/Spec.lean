import LeanMode.ds_seg_beats.DsSegBeats
import Mathlib

open Aeneas Aeneas.Std Result
open ds_seg_beats

set_option linter.unusedVariables false

/-!
# Specification: Segment Tree Beats

`segbeats_chmin` applies `a[i] = min(a[i], v)` over a range using the "second
maximum" lazy propagation; `segbeats_query_max` / `segbeats_query_sum` return
the range maximum / sum of the current array.  Correctness is stated against a
direct specification of the chmin-updated array.
-/

/-- `min` on `Int` (hand-expanded). -/
def minInt (a b : Int) : Int := if a < b then a else b

/-- The array obtained from `a` by `a[i] = min(a[i], v)` for `l ≤ i ≤ r`. -/
def chminRange (a : List Int) (l r : Nat) (v : Int) : List Int :=
  List.ofFn (fun (i : Fin a.length) =>
    if l ≤ i.1 ∧ i.1 ≤ r then minInt (a.getD i.1 0) v else a.getD i.1 0)

/-- Sum of `xs[l ..= r]`. -/
def sumRange (xs : List Int) (l r : Nat) : Int :=
  ((xs.drop l).take (r - l + 1)).sum

/-- Maximum of `xs[l ..= r]`. -/
def maxRange (xs : List Int) (l r : Nat) : Int :=
  ((xs.drop l).take (r - l + 1)).foldl (fun m x => max m x) (xs.getD l 0)

/-- After a build and one `chmin`, `query_max` returns the maximum of the
    chmin-updated array. -/
theorem chmin_query_max
    (a : Slice Int) (st st' st'' : SegBeats) (l r ql qr : Std.Usize) (v m : Int)
    (h_build : segbeats_build a = ok st)
    (h_chmin : segbeats_chmin st l r v = ok st')
    (h_q : segbeats_query_max st' ql qr = ok (st'', m))
    (h_ge : ql.val ≤ qr.val) (h_lt : qr.val < (a : List Int).length) :
    m = maxRange (chminRange (a : List Int) l.val r.val v) ql.val qr.val := by
  sorry

/-- After a build and one `chmin`, `query_sum` returns the sum of the
    chmin-updated array. -/
theorem chmin_query_sum
    (a : Slice Int) (st st' st'' : SegBeats) (l r ql qr : Std.Usize) (v s : Int)
    (h_build : segbeats_build a = ok st)
    (h_chmin : segbeats_chmin st l r v = ok st')
    (h_q : segbeats_query_sum st' ql qr = ok (st'', s))
    (h_ge : ql.val ≤ qr.val) (h_lt : qr.val < (a : List Int).length) :
    s = sumRange (chminRange (a : List Int) l.val r.val v) ql.val qr.val := by
  sorry

/-- On a freshly built tree, `query_sum` returns the true range sum. -/
theorem build_query_sum
    (a : Slice Int) (st st' : SegBeats) (ql qr : Std.Usize) (s : Int)
    (h_build : segbeats_build a = ok st)
    (h_q : segbeats_query_sum st ql qr = ok (st', s))
    (h_ge : ql.val ≤ qr.val) (h_lt : qr.val < (a : List Int).length) :
    s = sumRange (a : List Int) ql.val qr.val := by
  sorry
