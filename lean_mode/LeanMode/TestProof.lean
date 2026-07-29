import LeanMode.TestRust
open Aeneas Aeneas.Std Result

-- =============================================================================
-- matmul 正确性规范
--
--   Precondition  (调用 matmul 之前必须满足):
--     1. 输入矩阵大小与维度一致
--     2. 待验证的坐标 (i, j) 在矩阵范围内
--     3. 索引不越界 (由 1,2 推出)
--
--   Postcondition (matmul 返回 ok 时保证):
--     c_result[i + j*m] = Σ_{t=0}^{k-1} a[i + t*m] * b[t + j*k]
-- =============================================================================

theorem matmul_correct

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  输入: 矩阵和维度                                        ║
    -- ╚══════════════════════════════════════════════════════════╝

    (a b c_result : Slice Rat)               -- a[m×k], b[k×n], c[m×n] (列主序)
    (m k n : Usize)                          -- 维度参数

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 1: 矩阵大小 = 维度乘积                     ║
    -- ╚══════════════════════════════════════════════════════════╝

    (h_len_a : a.length = m.val * k.val)
    (h_len_b : b.length = k.val * n.val)
    (h_len_c : c_result.length = m.val * n.val)

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 2: 合法坐标                                ║
    -- ╚══════════════════════════════════════════════════════════╝

    (i j : Nat)                              -- 坐标 (行号, 列号)
    (hi : i < m.val)                          --  行号 < 行数
    (hj : j < n.val)                           --  列号 < 列数

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 3: 索引不越界 (由 Pre 1+2 推出)             ║
    -- ╚══════════════════════════════════════════════════════════╝

    (hc : i + j * m.val < c_result.length)    -- c 的索引安全
    (ha : ∀ t, t < k.val → i + t * m.val < a.length)  -- a 的索引安全
    (hb : ∀ t, t < k.val → t + j * k.val < b.length)  -- b 的索引安全

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Postcondition: matmul 返回 ok 时，结果等于点积           ║
    -- ╚══════════════════════════════════════════════════════════╝

    (h_result : test_rust.matmul c_result a b m k n = ok c_result) :
    --          ^^^^^^^^ 如果 matmul 不失败 (返回 ok) ...

    c_result[i + j * m.val]'hc =             -- ... 则 c 在 (i,j) 的值
    Finset.sum (Finset.attach (Finset.range k.val)) (λ ⟨t, ht⟩ =>
      let ht' : t < k.val := Finset.mem_range.1 ht
      a[i + t * m.val]'(ha t ht') * b[t + j * k.val]'(hb t ht'))
    -- = Σ_{t=0}^{k-1} a 在 (i,t) 的值 * b 在 (t,j) 的值

    := by
  sorry
