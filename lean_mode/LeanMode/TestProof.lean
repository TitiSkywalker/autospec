import LeanMode.TestRust
open Aeneas Aeneas.Std Result

-- ============================================================================
-- Correctness specification for matmul
--
--   Preconditions (must hold before calling matmul):
--     1. Matrix sizes match the declared dimensions
--     2. The coordinates (i, j) being verified are in range
--     3. All computed indices are within slice bounds (follows from 1+2)
--
--   Postcondition (guaranteed when matmul returns ok):
--     c[i + j\*m] = Σ_{t=0}^{k-1} a[i + t\*m] \* b[t + j\*k]
--
--   Column-major storage: element (row, col) is at index row + col * rows.
--     a[m×k]: a[i + t*m] = row i, column t
--     b[k×n]: b[t + j*k] = row t, column j
--     c[m×n]: c[i + j*m] = row i, column j
-- ============================================================================

theorem matmul_correct

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Input: matrices and dimensions                          ║
    -- ╚══════════════════════════════════════════════════════════╝

    (a b c_result : Slice Rat)              -- a[m×k], b[k×n], c[m×n] (column-major)
    (m k n : Usize)                         -- dimensions

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 1: |slice| = dimension product             ║
    -- ╚══════════════════════════════════════════════════════════╝

    (h_len_a : a.length = m.val * k.val)
    (h_len_b : b.length = k.val * n.val)
    (h_len_c : c_result.length = m.val * n.val)

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 2: coordinates in range                    ║
    -- ╚══════════════════════════════════════════════════════════╝

    (i j : Nat)                               -- coordinates (row, col)
    (hi : i < m.val)                          -- row < #rows
    (hj : j < n.val)                          -- col < #cols

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Precondition 3: index bounds (follows from Pre 1+2)     ║
    -- ╚══════════════════════════════════════════════════════════╝

    (hc : i + j * m.val < c_result.length)            -- c index is safe
    (ha : ∀ t, t < k.val → i + t * m.val < a.length)  -- a index is safe
    (hb : ∀ t, t < k.val → t + j * k.val < b.length)  -- b index is safe

    -- ╔══════════════════════════════════════════════════════════╗
    -- ║  Postcondition: if matmul returns ok, the result         ║
    -- ║  element equals the dot product                          ║
    -- ╚══════════════════════════════════════════════════════════╝

    (h_result : test_rust.matmul c_result a b m k n = ok c_result) :
    --          ^^^^^^^^^ if matmul succeeds (returns ok) ...

    c_result[i + j * m.val]'hc =            -- ... then c at (i,j)
    Finset.sum (Finset.attach (Finset.range k.val)) (λ ⟨t, ht⟩ =>
      let ht' : t < k.val := Finset.mem_range.1 ht
      a[i + t * m.val]'(ha t ht') * b[t + j * k.val]'(hb t ht'))
    -- = Σ_{t=0}^{k-1} a at (i,t) * b at (t,j)

    := by
  sorry
