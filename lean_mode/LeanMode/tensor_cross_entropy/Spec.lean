import LeanMode.tensor_cross_entropy.TensorCrossEntropy
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_cross_entropy

set_option linter.unusedVariables false

/-!
# Specification: 2D Cross-Entropy Loss

`cross_entropy pred target rows cols` computes

  loss = -Σ_r Σ_c target[r][c] · log(pred[r][c])

over two H×W row-major matrices. The inputs are probability-like: targets
are non-negative and predictions strictly positive; terms with `target = 0`
contribute 0 (the convention `0 · log p = 0`), matching the implementation's
skip of non-positive-target terms.
-/

/-- View a flat row-major `Slice ℝ` as a matrix: `Fin rows → Fin cols → ℝ`. -/
def toMatrix (s : Slice ℝ) (rows cols : Nat) (h : s.length = rows * cols) :
    Fin rows → Fin cols → ℝ :=
  λ i j =>
    have h_bound : i.val * cols + j.val < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val * cols + j.val < i.val * cols + cols := Nat.add_lt_add_left hj _
        _ = (i.val + 1) * cols := by ring
        _ ≤ rows * cols := Nat.mul_le_mul_right _ (Nat.succ_le_of_lt hi)
    s[i.val * cols + j.val]'h_bound

/-- `cross_entropy` equals the negative sum of `target · log pred`.

Requires the inputs to be valid probability vectors/matrices: targets are
non-negative (so the impl's `target > 0` guard skips exactly the zero terms,
which contribute `0 · log p = 0`) and predictions are strictly positive (so
`Real.log p` is a finite real matching the impl's `f64::ln`). -/
theorem cross_entropy_correct
    (pred target : Slice ℝ) (rows cols : Usize) (result : ℝ)
    (h_ce : cross_entropy pred target rows cols = ok result)
    (h_len_p : pred.length = rows.val * cols.val)
    (h_len_t : target.length = rows.val * cols.val)
    (htarget_nonneg : ∀ (r : Fin rows.val) (c : Fin cols.val),
        toMatrix target rows.val cols.val h_len_t r c ≥ 0)
    (hpred_pos : ∀ (r : Fin rows.val) (c : Fin cols.val),
        toMatrix pred rows.val cols.val h_len_p r c > 0) :
    let H := rows.val; let W := cols.val
    let p := toMatrix pred H W h_len_p
    let t := toMatrix target H W h_len_t
    result = -(∑ r : Fin H, ∑ c : Fin W, t r c * Real.log (p r c)) :=
by
  sorry
