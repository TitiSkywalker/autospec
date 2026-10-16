import LeanMode.dp_matrix_path.DpMatrixPath
import Mathlib

open Aeneas Aeneas.Std Result
open dp_matrix_path

set_option linter.unusedVariables false

/-!
# Specification: minimum path sum in a grid

`min_path_sum grid rows cols` returns the minimum sum of a path from `(0, 0)`
to `(rows-1, cols-1)` that moves only right or down at each step.
-/

/-- Convert `Slice Int` to `List Int`. -/
def toIntList (s : Slice Int) : List Int :=
  (s : List Int)

/-- A valid monotone path is a list of flat row-major indices starting at `0`,
    ending at `rows * cols - 1`, where each step moves one cell right (same row)
    or one cell down (next row). -/
def is_path (path : List Nat) (rows cols : Nat) : Prop :=
  path.length = rows + cols - 1 ∧
  path.getD 0 0 = 0 ∧
  path.getD (path.length - 1) 0 = rows * cols - 1 ∧
  (∀ t, t + 1 < path.length →
    let cur := path.getD t 0
    let nxt := path.getD (t + 1) 0
    (nxt = cur + 1 ∧ cur / cols = (cur + 1) / cols) ∨
    (nxt = cur + cols ∧ cur / cols + 1 = (cur + cols) / cols))

/-- Sum of the grid cells visited by `path`. -/
def path_sum (grid : List Int) (path : List Nat) : Int :=
  path.foldl (fun acc idx => acc + grid.getD idx 0) 0

/-- Correctness of `min_path_sum`:
    (1) Optimality: every monotone path sums to at least `result`.
    (2) Attainability: some monotone path sums to exactly `result`. -/
theorem min_path_sum_correct
    (grid : Slice Int) (rows cols : Std.Usize) (result : Int)
    (h_rows : 0 < rows.val) (h_cols : 0 < cols.val)
    (h_len : grid.length = rows.val * cols.val)
    (h : min_path_sum grid rows cols = ok result) :
    (∀ (path : List Nat),
       is_path path rows.val cols.val →
       result ≤ path_sum (toIntList grid) path) ∧
    (∃ (path : List Nat),
       is_path path rows.val cols.val ∧
       path_sum (toIntList grid) path = result) :=
by
  sorry
