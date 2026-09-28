import LeanMode.ds_sparse_table.DsSparseTable
import Mathlib

open Aeneas Aeneas.Std Result
open ds_sparse_table

set_option linter.unusedVariables false

/-!
# Specification: Sparse Table (static RMQ)

`sparse_build` / `range_min` implement range-minimum queries over a static
array `a`.  `range_min (l, r)` returns the minimum of `a[l..=r]`.  Ranges are
assumed valid (`l ≤ r < n`).
-/

/-- `m` is the minimum of `xs[l ..= r]` (inclusive). -/
def is_min_in_range (xs : List Int) (l r : Nat) (m : Int) : Prop :=
  (∀ i, l ≤ i ∧ i ≤ r → m ≤ xs.getD i 0) ∧
  (∃ i, l ≤ i ∧ i ≤ r ∧ xs.getD i 0 = m)


/-! ## Specification theorems -/

/-- `range_min` returns the minimum value of `a[l..=r]`. -/
theorem range_min_correct (a : Slice Int) (st : SparseTable) (l r : Std.Usize) (m : Int)
    (h_build : sparse_build a = ok st)
    (h_ge : l.val ≤ r.val)
    (h_lt : r.val < (a : List Int).length)
    (h_q : range_min st l r = ok m) :
    is_min_in_range (a : List Int) l.val r.val m := by
  sorry
