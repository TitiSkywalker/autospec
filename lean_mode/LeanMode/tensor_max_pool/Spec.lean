import LeanMode.tensor_max_pool.TensorMaxPool
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_max_pool

set_option linter.unusedVariables false

/-!
# Specification: 2D Max Pooling (valid, stride 1)

`max_pool2d input h w kh kw` computes the 2D max pooling of an H×W input
with a Kh×Kw window (stride 1, no padding), both row-major.

Output dimensions: (H - Kh + 1) × (W - Kw + 1).

For each output position (i, j):
  out[i][j] = max_{m < Kh, n < Kw} input[i + m][j + n]

spec: `out[i][j]` upper-bounds every window element and is attained:
  (∀ m n, out[i][j] ≥ input[i+m][j+n]) ∧
  (∃ m n, out[i][j] = input[i+m][j+n])
-/

/-- View a flat row-major `Slice Rat` as a matrix: `Fin rows → Fin cols → Rat`. -/
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
        _ ≤ rows * cols := Nat.mul_le_mul_right cols (Nat.succ_le_of_lt hi)
    s[i.val * cols + j.val]'h_bound

/-- A Kh-wide window at output row `i` stays below `H`:
    `i < H - Kh + 1` and `m < Kh` give `i + m < H` (needs `Kh ≤ H`). -/
lemma window_row_lt (H Kh : Nat) (hKh : Kh ≤ H) {i m : Nat}
    (hi : i < H - Kh + 1) (hm : m < Kh) : i + m < H := by
  omega

/-- A Kw-wide window at output column `j` stays below `W`. -/
lemma window_col_lt (W Kw : Nat) (hKw : Kw ≤ W) {j n : Nat}
    (hj : j < W - Kw + 1) (hn : n < Kw) : j + n < W := by
  omega

/-- The (m,n)-th element of the window anchored at output (i,j), i.e.
    `inp[i+m][j+n]`, with the index bounds discharged here. -/
def window (H W Kh Kw : Nat) (inp : Fin H → Fin W → Rat)
    (hkh_le : Kh ≤ H) (hkw_le : Kw ≤ W)
    (i : Fin (H - Kh + 1)) (j : Fin (W - Kw + 1)) (m : Fin Kh) (n : Fin Kw) : Rat :=
  inp ⟨i.val + m.val, window_row_lt H Kh hkh_le i.is_lt m.is_lt⟩
      ⟨j.val + n.val, window_col_lt W Kw hkw_le j.is_lt n.is_lt⟩

/-- Output length: `(H - Kh + 1) * (W - Kw + 1)`. -/
theorem max_pool2d_length
    (input : Slice Rat) (h w kh kw : Usize) (result : alloc.vec.Vec Rat)
    (h_pool : max_pool2d input h w kh kw = ok result)
    (hh : h > 0#usize) (hw : w > 0#usize)
    (hkh : kh > 0#usize) (hkw : kw > 0#usize)
    (hkh_le : kh ≤ h) (hkw_le : kw ≤ w) :
    result.length = (h.val - kh.val + 1) * (w.val - kw.val + 1) := by
  sorry

/-- Correctness: `out[i][j]` is the maximum of its window. -/
theorem max_pool2d_eq
    (input : Slice Rat) (h w kh kw : Usize) (result : alloc.vec.Vec Rat)
    (h_pool : max_pool2d input h w kh kw = ok result)
    (h_len_in : input.length = h.val * w.val)
    (hkh_le : kh.val ≤ h.val) (hkw_le : kw.val ≤ w.val)
    (h_len_out : (alloc.vec.Vec.deref result).length = (h.val - kh.val + 1) * (w.val - kw.val + 1)) :
    let H := h.val; let W := w.val
    let Kh := kh.val; let Kw := kw.val
    let inp := toMatrix input H W h_len_in
    let out := toMatrix (alloc.vec.Vec.deref result) (H - Kh + 1) (W - Kw + 1) h_len_out
    ∀ (i : Fin (H - Kh + 1)) (j : Fin (W - Kw + 1)),
      (∀ (m : Fin Kh) (n : Fin Kw),
        out i j ≥ window H W Kh Kw inp hkh_le hkw_le i j m n) ∧
      (∃ (m : Fin Kh) (n : Fin Kw),
        out i j = window H W Kh Kw inp hkh_le hkw_le i j m n) := by
  sorry