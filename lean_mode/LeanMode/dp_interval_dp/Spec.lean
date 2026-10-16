import LeanMode.dp_interval_dp.DpIntervalDp
import Mathlib

open Aeneas Aeneas.Std Result
open dp_interval_dp

set_option linter.unusedVariables false

/-!
# Specification: matrix chain multiplication (interval DP)

`matrix_chain_order dims` returns the minimum number of scalar multiplications
needed to multiply the whole chain of matrices with dimensions
`dims[0]×dims[1]`, `dims[1]×dims[2]`, … (there are `dims.length - 1` matrices).
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- A full parenthesization of a matrix chain: a leaf is a single matrix, and
    `mul l r k` splits a chain at `k` (left covers `[i..k]`, right `[k+1..j]`). -/
inductive Paren where
  | leaf : Nat → Paren
  | mul : Paren → Paren → Nat → Paren

/-- `e` is a parenthesization covering matrices `[i..j]`. -/
def is_paren : Paren → Nat → Nat → Prop
  | Paren.leaf x, i, j => x = i ∧ i = j
  | Paren.mul l r k, i, j => is_paren l i k ∧ is_paren r (k + 1) j

/-- Cost of a parenthesization `e` covering `[i..j]`. -/
def paren_cost (dims : List Nat) : Paren → Nat → Nat → Nat
  | Paren.leaf _, _, _ => 0
  | Paren.mul l r k, i, j =>
      paren_cost dims l i k + paren_cost dims r (k + 1) j +
        dims.getD i 0 * dims.getD (k + 1) 0 * dims.getD (j + 1) 0

/-- Correctness of `matrix_chain_order`:
    (1) Optimality: every parenthesization costs at least `result`.
    (2) Attainability: some parenthesization costs exactly `result`. -/
theorem matrix_chain_order_correct (dims : Slice Std.Usize) (result : Std.Usize)
    (h_dims : 1 < (dims : List Std.Usize).length)
    (h : matrix_chain_order dims = ok result) :
    let n := (dims : List Std.Usize).length - 1
    (∀ e : Paren, is_paren e 0 (n - 1) →
       result.val ≤ paren_cost (toNatList dims) e 0 (n - 1)) ∧
    (∃ e : Paren, is_paren e 0 (n - 1) ∧
       paren_cost (toNatList dims) e 0 (n - 1) = result.val) := by
  sorry
