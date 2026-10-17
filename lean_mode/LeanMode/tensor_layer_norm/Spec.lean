import LeanMode.tensor_layer_norm.TensorLayerNorm
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_layer_norm

set_option linter.unusedVariables false

/-!
# Specification: 2D Layer Normalization

`layer_norm input rows cols gamma beta eps` normalizes each row of an H×W
matrix (row-major) and applies a per-feature affine transform:

  μ_r  = (1/W) · Σ_j x[r][j]
  σ²_r = (1/W) · Σ_j (x[r][j] - μ_r)²
  y[r][j] = γ[j] · (x[r][j] - μ_r) / √(σ²_r + ε) + β[j]

For γ = 1, β = 0 the output row is centered (mean 0).

The correctness theorems assume `eps > 0` (so `√(σ² + ε)` is a positive real).
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

/-- View a `Slice ℝ` of known length as a vector `Fin len → ℝ`. -/
def toVec (s : Slice ℝ) (len : Nat) (h : s.length = len) : Fin len → ℝ :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound

/-- Per-row mean: μ = (1/W) · Σ_j x[j]. -/
noncomputable def rowMean (W : Nat) (x : Fin W → ℝ) : ℝ :=
  (∑ j : Fin W, x j) / (W : ℝ)

/-- Per-row variance: σ² = (1/W) · Σ_j (x[j] - μ)². -/
noncomputable def rowVar (W : Nat) (x : Fin W → ℝ) : ℝ :=
  (∑ j : Fin W, (x j - rowMean W x) * (x j - rowMean W x)) / (W : ℝ)

/-- Output length equals input length (for non-empty input). -/
theorem layer_norm_length
    (input gamma beta : Slice ℝ) (rows cols : Usize) (eps : ℝ)
    (result : alloc.vec.Vec ℝ)
    (h_ln : layer_norm input rows cols gamma beta eps = ok result)
    (hr : rows > 0#usize) (hc : cols > 0#usize) :
    result.length = rows.val * cols.val := by
  sorry

/-- `layer_norm` computes the normalized + affine-transformed output
    (requires `eps > 0` so that `√(σ² + ε)` is a positive real, matching the
    impl's `f64::sqrt`). -/
theorem layer_norm_correct
    (input gamma beta : Slice ℝ) (rows cols : Usize) (eps : ℝ)
    (result : alloc.vec.Vec ℝ)
    (h_ln : layer_norm input rows cols gamma beta eps = ok result)
    (h_len_in : input.length = rows.val * cols.val)
    (h_len_g : gamma.length = cols.val)
    (h_len_b : beta.length = cols.val)
    (h_len_out : (alloc.vec.Vec.deref result).length = rows.val * cols.val)
    (heps : eps > 0) :
    let H := rows.val; let W := cols.val
    let inp := toMatrix input H W h_len_in
    let g := toVec gamma W h_len_g
    let b := toVec beta W h_len_b
    let out := toMatrix (alloc.vec.Vec.deref result) H W h_len_out
    ∀ (r : Fin H) (j : Fin W),
      out r j =
        g j * (inp r j - rowMean W (fun k => inp r k)) /
          Real.sqrt (rowVar W (fun k => inp r k) + eps) + b j :=
by
  sorry

/-- With γ = 1 and β = 0, each normalized output row is centered (mean 0)
    (requires `eps > 0`). -/
theorem layer_norm_zero_mean
    (input gamma beta : Slice ℝ) (rows cols : Usize) (eps : ℝ)
    (result : alloc.vec.Vec ℝ)
    (h_ln : layer_norm input rows cols gamma beta eps = ok result)
    (h_len_in : input.length = rows.val * cols.val)
    (h_len_g : gamma.length = cols.val)
    (h_len_b : beta.length = cols.val)
    (h_len_out : (alloc.vec.Vec.deref result).length = rows.val * cols.val)
    (heps : eps > 0)
    (hg_one : toVec gamma cols.val h_len_g = fun _ => 1)
    (hb_zero : toVec beta cols.val h_len_b = fun _ => 0) :
    let H := rows.val; let W := cols.val
    let out := toMatrix (alloc.vec.Vec.deref result) H W h_len_out
    ∀ (r : Fin H), (∑ j : Fin W, out r j) = 0 :=
by
  sorry
