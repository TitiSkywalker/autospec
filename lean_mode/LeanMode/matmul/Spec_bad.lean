import LeanMode.matmul.Matmul
open Aeneas Aeneas.Std Result

-- Correctness: matmul (column-major)
--   c[i + j*m] = Σ_{t=0}^{k-1} a[i + t*m] * b[t + j*k]

theorem matmul_correct
    -- input matrices
    (a b c_result : Slice Rat)
    (m k n : Usize)
    -- slice lengths match dimensions
    (h_len_a : a.length = m.val * k.val)
    (h_len_b : b.length = k.val * n.val)
    (h_len_c : c_result.length = m.val * n.val)
    -- coordinates in range
    (i j : Nat)
    (hi : i < m.val)
    (hj : j < n.val)
    -- index bounds
    (hc : i + j * m.val < c_result.length)
    (ha : ∀ t, t < k.val → i + t * m.val < a.length)
    (hb : ∀ t, t < k.val → t + j * k.val < b.length)
    -- postcondition
    (h_result : matmul.matmul c_result a b m k n = ok c_result) :
    c_result[i + j * m.val]'hc =
    Finset.sum (Finset.attach (Finset.range k.val)) (λ ⟨t, ht⟩ =>
      let ht' : t < k.val := Finset.mem_range.1 ht
      a[i + t * m.val]'(ha t ht') * b[t + j * k.val]'(hb t ht'))
    := by
  sorry
