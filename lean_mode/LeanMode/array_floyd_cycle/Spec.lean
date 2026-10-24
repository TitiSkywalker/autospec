import LeanMode.array_floyd_cycle.ArrayFloydCycle
import Mathlib

open Aeneas Aeneas.Std Result
open array_floyd_cycle

set_option linter.unusedVariables false

/-!
# Specification: Floyd's cycle detection (tortoise and hare)

`floyd_find_cycle f x0` returns the entry node of the cycle reached from `x0`
in the functional graph `f` (where `f[i]` is the successor of node `i`).
-/

/-- View the functional graph as a `List Nat`. -/
def floydList (f : Slice Std.Usize) : List Nat :=
  (f : List Std.Usize).map (fun x => x.val)

/-- `iter f x k` is `f` applied `k` times to `x` (the k-th successor). -/
def iter (f : List Nat) (x : Nat) : Nat → Nat
  | 0 => x
  | k + 1 => f.getD (iter f x k) 0

/-- Correctness: the result is the first node along the orbit of `x0` that
    repeats — i.e. there is a tail length `μ` and a cycle length `λ > 0` such
    that the orbit is periodic from position `μ` onward, the result sits at
    position `μ`, and no earlier position `i < μ` is periodic. -/
theorem floyd_find_cycle_correct
    (f : Slice Std.Usize) (x0 : Std.Usize) (r : Std.Usize)
    (h_valid : ∀ i : Nat, i < (floydList f).length → (floydList f).getD i 0 < (floydList f).length)
    (h_x0 : x0.val < (floydList f).length)
    (h : floyd_find_cycle f x0 = ok r) :
    ∃ μ : Nat, ∃ lam : Nat,
      0 < lam ∧
      iter (floydList f) x0.val μ = iter (floydList f) x0.val (μ + lam) ∧
      r.val = iter (floydList f) x0.val μ ∧
      (∀ i : Nat, i < μ → (∀ lam' : Nat, 0 < lam' →
        iter (floydList f) x0.val i ≠ iter (floydList f) x0.val (i + lam'))) := by
  sorry
