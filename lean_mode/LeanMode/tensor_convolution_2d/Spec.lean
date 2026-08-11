import LeanMode.tensor_convolution_2d.TensorConvolution2d
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_convolution_2d

set_option linter.unusedVariables false

/-!
# Specification: 2D Convolution (full)

`conv2d input h w kernel kh kw` computes the full 2D convolution of an
H×W input matrix with a Kh×Kw kernel, both in row-major order.

Output dimensions: (H + Kh - 1) × (W + Kw - 1).

For each output position (i, j):
  out[i][j] = Σ_{m=0}^{Kh-1} Σ_{n=0}^{Kw-1}
    (if i ≥ m ∧ i-m < H ∧ j ≥ n ∧ j-n < W
     then input[i-m][j-n] * kernel[m][n]
     else 0)
-/

/-- View a flat row-major vector as a matrix: `Fin rows → Fin cols → Rat`. -/
def toMatrix (s : Slice Rat) (rows cols : Nat) (h : s.length = rows * cols) :
    Fin rows → Fin cols → Rat :=
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

/-- Output length: (H + Kh - 1) * (W + Kw - 1). -/
theorem conv2d_length
    (input kernel : Slice Rat) (h w kh kw : Usize) (result : alloc.vec.Vec Rat)
    (h_conv : conv2d input h w kernel kh kw = ok result)
    (hh : h > 0#usize) (hw : w > 0#usize)
    (hkh : kh > 0#usize) (hkw : kw > 0#usize) :
    result.length = (h.val + kh.val - 1) * (w.val + kw.val - 1) := by
  sorry

/-- Correctness: `conv2d` satisfies the full 2D convolution definition. -/
theorem conv2d_eq
    (input kernel : Slice Rat) (h w kh kw : Usize) (result : alloc.vec.Vec Rat)
    (h_conv : conv2d input h w kernel kh kw = ok result)
    (h_len_in : input.length = h.val * w.val)
    (h_len_ker : kernel.length = kh.val * kw.val)
    (h_len_out : result.length = (h.val + kh.val - 1) * (w.val + kw.val - 1)) :
    let H := h.val; let W := w.val
    let Kh := kh.val; let Kw := kw.val
    let out_h := H + Kh - 1; let out_w := W + Kw - 1
    let inp := toMatrix input H W h_len_in
    let ker := toMatrix kernel Kh Kw h_len_ker
    let out := toMatrix (alloc.vec.Vec.deref result) out_h out_w h_len_out
    ∀ (i : Fin out_h) (j : Fin out_w),
      out i j =
        ∑ m : Fin Kh,
          ∑ n : Fin Kw,
            if hi : i.val ≥ m.val ∧ i.val - m.val < H ∧
                     j.val ≥ n.val ∧ j.val - n.val < W then
              inp ⟨i.val - m.val, hi.2.1⟩ ⟨j.val - n.val, hi.2.2.2⟩ * ker m n
            else 0 := by
  sorry
