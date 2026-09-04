import LeanMode.tensor_convolution_1d.TensorConvolution1d
import Mathlib

open Aeneas Aeneas.Std Result
open tensor_convolution_1d

set_option linter.unusedVariables false

/-!
# Specification: 1D Convolution (full)

`conv1d input kernel` computes the full discrete 1D convolution:
`y[i] = Σ_{j} input[i-j] * kernel[j]`, output length `n + k - 1`.
-/

/-- View a `Slice Rat` of known length as `Fin len → Rat`. -/
def toVec (s : Slice Rat) (len : Nat) (h : s.length = len) : Fin len → Rat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    s[i.val]'h_bound


/-- Output length: `n + k - 1`. -/
theorem conv1d_length
    (input kernel : Slice Rat) (result : alloc.vec.Vec Rat)
    (h_conv : conv1d input kernel = ok result)
    (hn : Slice.len input > 0#usize)
    (hk : Slice.len kernel > 0#usize) :
    result.length = Slice.len input + Slice.len kernel - 1#usize := by
  sorry


/-- `conv1d` satisfies the full convolution definition:
    `out[i] = Σ_{j:Fin k, i≥j.val ∧ i-j.val<n} input[i - j.val] * kernel[j.val]`. -/
theorem conv1d_eq
    (input kernel : Slice Rat) (result : alloc.vec.Vec Rat)
    (h_conv : conv1d input kernel = ok result)
    (h_len_in : input.length = (Slice.len input).val)
    (h_len_ker : kernel.length = (Slice.len kernel).val)
    (h_len_out : (alloc.vec.Vec.deref result).length = (Slice.len input).val + (Slice.len kernel).val - 1) :
    let n := (Slice.len input).val
    let k := (Slice.len kernel).val
    let m := n + k - 1
    let inp := toVec input n h_len_in
    let ker := toVec kernel k h_len_ker
    let out := toVec (alloc.vec.Vec.deref result) m h_len_out
    ∀ (i : Fin m), out i =
      ∑ j : Fin k,
        if h : i.val ≥ j.val ∧ i.val - j.val < n then
          inp ⟨i.val - j.val, h.2⟩ * ker j
        else 0 :=
by
  sorry
