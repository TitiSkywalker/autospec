import LeanMode.dp_domino_tiling.DpDominoTiling
import Mathlib

open Aeneas Aeneas.Std Result
open dp_domino_tiling

set_option linter.unusedVariables false

/-!
# Specification: counting domino tilings

`domino_tilings w h` runs the broken-profile DP over the frontier mask and
returns the number of ways to cover the whole `w × h` grid with `1 × 2` / `2 × 1`
dominoes.  The specification below characterises that number *declaratively*, as
the cardinality of the set of all sets of pairwise disjoint in-grid dominoes
covering every cell — no mention of masks, of the row-major cell order, or of
the DP at all.
-/

namespace DominoTiling

/-- A single `1 × 2` or `2 × 1` domino, anchored at its top-left cell
`(row, col)`: `horiz = true` covers `(row, col)` and `(row, col + 1)`, and
`horiz = false` covers `(row, col)` and `(row + 1, col)`. -/
structure Placement where
  row : Nat
  col : Nat
  horiz : Bool
deriving DecidableEq

/-- The two cells covered by a placement. -/
def Placement.cells (p : Placement) : Finset (Nat × Nat) :=
  if p.horiz then {(p.row, p.col), (p.row, p.col + 1)}
  else {(p.row, p.col), (p.row + 1, p.col)}

/-- The cells of the `w × h` grid, as `(row, col)` pairs. -/
def grid (w h : Nat) : Finset (Nat × Nat) :=
  (Finset.range h).product (Finset.range w)

/-- All candidate placements anchored inside the `w × h` grid; placements that
stick out of the grid are filtered out by `IsDominoTiling`. -/
def allPlacements (w h : Nat) : Finset Placement :=
  (((Finset.range h).product (Finset.range w)).product (Finset.univ : Finset Bool)).image
    (fun q => ⟨q.1.1, q.1.2, q.2⟩)

/-- A set of dominoes tiles the `w × h` grid: every domino lies inside the grid,
distinct dominoes do not overlap, and every cell is covered. -/
abbrev IsDominoTiling (w h : Nat) (T : Finset Placement) : Prop :=
  (∀ p ∈ T, ∀ c ∈ p.cells, c ∈ grid w h) ∧
  (∀ p ∈ T, ∀ q ∈ T, p ≠ q → p.cells ∩ q.cells = ∅) ∧
  T.biUnion Placement.cells = grid w h

/-- All domino tilings of the `w × h` grid. -/
def allDominoTilings (w h : Nat) : Finset (Finset Placement) :=
  (allPlacements w h).powerset.filter (IsDominoTiling w h)

/-- The number of domino tilings of the `w × h` grid. -/
def dominoTilingCount (w h : Nat) : Nat :=
  (allDominoTilings w h).card

end DominoTiling

open DominoTiling

/-- Correctness of the broken-profile domino-tiling DP: the returned count is the
number of ways to tile the `w × h` grid with dominoes.  The bound `w ≤ 8` is the
one the implementation documents (the frontier is a `usize` bitfield and `2^w`
states are enumerated). -/
theorem domino_tilings_correct (w h : Std.Usize) (k : Std.Usize)
    (h_w : w.val ≤ 8)
    (hres : domino_tilings w h = ok k) :
    k.val = dominoTilingCount w.val h.val := by
  sorry
