import LeanMode.TestRust
open Aeneas Aeneas.Std Result

-- ============================================================================
-- Run matmul on two 2×2 matrices
--
--   A = [[1, 2],    B = [[5, 6],    C = A×B = [[19, 22],
--        [3, 4]]         [7, 8]]              [43, 50]]
--
--   column-major:  a = [1, 3, 2, 4]
--                  b = [5, 7, 6, 8]
--              expect = [19, 43, 22, 50]
-- ============================================================================

def test_matmul_2x2 : Result (List Rat) := do
  let a ← lift (Array.to_slice (Array.make 4#usize [(1:Rat), 3, 2, 4]))
  let b ← lift (Array.to_slice (Array.make 4#usize [(5:Rat), 7, 6, 8]))
  let c ← lift (Array.to_slice (Array.make 4#usize [(0:Rat), 0, 0, 0]))
  let result ← test_rust.matmul c a b 2#usize 2#usize 2#usize
  ok result.val

#eval test_matmul_2x2
#eval test_rust.test_matmul
