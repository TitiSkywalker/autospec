import LeanMode.TestRust
open Aeneas Aeneas.Std Result

-- =============================================================================
-- matmul 正确性定理
--
-- c_result[i + j*m] = Σ_{t=0}^{k-1} a[i + t*m] * b[t + j*k]
--
-- 列主序: 元素 (row, col) → 索引 row + col * rows
--    a[m×k], b[k×n], c[m×n]
-- =============================================================================

theorem matmul_correct
    (a b c_result : Slice Rat)              -- 矩阵 (列主序)
    (m k n : Usize)                         -- 维度
    (h_len_a : a.length = m.val * k.val)    -- |a| = m*k
    (h_len_b : b.length = k.val * n.val)    -- |b| = k*n
    (h_len_c : c_result.length = m.val * n.val) -- |c| = m*n
    (h_result : test_rust.matmul c_result a b m k n = ok c_result)
    (i j : Nat)
    (hi : i < m.val)                        -- i 是合法行号
    (hj : j < n.val)                         -- j 是合法列号
    -- 索引不越界 (由 hi, hj, h_len_* 可推出)
    (hc : i + j * m.val < c_result.length)
    (ha : ∀ t, t < k.val → i + t * m.val < a.length)
    (hb : ∀ t, t < k.val → t + j * k.val < b.length)
    :
    c_result[i + j * m.val]'hc =
    -- 对 t ∈ [0, k) 求和, attach 提供 t ∈ range 的证明
    Finset.sum (Finset.attach (Finset.range k.val)) (λ ⟨t, ht⟩ =>
      let ht' : t < k.val := Finset.mem_range.1 ht
      a[i + t * m.val]'(ha t ht') * b[t + j * k.val]'(hb t ht'))
    := by
  sorry
