import LeanMode.dp_monotone_queue.DpMonotoneQueue
import Mathlib

open Aeneas Aeneas.Std Result
open dp_monotone_queue

set_option linter.unusedVariables false

/-!
# Specification: decision-monotone DP with the monotone-queue optimization

For a **non-decreasing** array `p`, both implementations evaluate the recurrence

```
dp[0] = 0
dp[i] = min over 0 <= j < i of ( dp[j] + (p[i] - p[j])^2 )
```

and return `dp[n-1]` (`0` for the empty array).  `dp_naive` does it with the
`O(n^2)` double loop; `dp_optimized` uses the monotone-queue (1D1D) optimization,
which is correct because `(p[i] - p[j])^2` satisfies the quadrangle inequality.

The predicate below characterises the expected answer *declaratively* (the DP
table built by the recurrence, never mentioning either implementation), and the
theorems say that both implementations realise it, hence agree.
-/

namespace MonotoneQueue

/-- A flat `Slice Std.Usize` as `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The squared gap `(a - b) * (a - b)`, using `Nat` subtraction (genuine
difference on a non-decreasing array). -/
def sqGap (a b : Nat) : Nat := (a - b) * (a - b)

/-- Minimum of a list of naturals (`0` for the empty list, unused here). -/
def listMin : List Nat → Nat
  | [] => 0
  | a :: rest => rest.foldl min a

/-- Extend the DP table.  `hist` holds the pairs `(p[j], dp[j])` for the already
processed indices `0..k` and `rest` holds the not-yet-processed positions
`p[k+1], p[k+2], …`; the result is the list of DP values for `rest`. -/
def buildDp : List (Nat × Nat) → List Nat → List Nat
  | _, [] => []
  | hist, x :: xs =>
    let d := listMin (hist.map (fun (q : Nat × Nat) => q.2 + sqGap x q.1))
    d :: buildDp (hist ++ [(x, d)]) xs

/-- The full DP table `[dp[0], dp[1], …]` for the position array `pl`. -/
def dpList (pl : List Nat) : List Nat :=
  match pl with
  | [] => []
  | p0 :: rest => 0 :: buildDp [(p0, 0)] rest

/-- The value both implementations return: the last DP entry `dp[n-1]` for
`n = pl.length` (`0` for the empty array). -/
def trueVal (pl : List Nat) : Nat :=
  (dpList pl).foldl (fun _ x => x) 0

end MonotoneQueue

open MonotoneQueue

/-- Correctness of the naive `O(n^2)` DP.  Because the implementation
`assert!`s that `p` is non-decreasing, the `ok` hypothesis *implies* the
precondition, so it need not be assumed separately. -/
theorem dp_naive_correct (p : Slice Std.Usize) (a : Std.Usize)
    (h : dp_naive p = ok a) :
    a.val = trueVal (toNatList p) := by
  sorry

/-- Correctness of the monotone-queue `O(n log n)` DP. -/
theorem dp_optimized_correct (p : Slice Std.Usize) (b : Std.Usize)
    (h : dp_optimized p = ok b) :
    b.val = trueVal (toNatList p) := by
  sorry

/-- The optimized and the naive implementations agree on every input for which
both terminate successfully. -/
theorem dp_opt_equiv (p : Slice Std.Usize) (a b : Std.Usize)
    (h1 : dp_naive p = ok a) (h2 : dp_optimized p = ok b) :
    a.val = b.val := by
  sorry
