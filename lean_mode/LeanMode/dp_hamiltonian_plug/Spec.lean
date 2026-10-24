import LeanMode.dp_hamiltonian_plug.DpHamiltonianPlug
import Mathlib

open Aeneas Aeneas.Std Result
open dp_hamiltonian_plug

set_option linter.unusedVariables false

/-!
# Specification: counting Hamiltonian cycles of a grid graph

`ham_cycles_grid w h` runs the plug DP (broken-profile / connectivity DP) over
the cells of the `w × h` grid graph and returns the number of Hamiltonian
cycles it contains.  The specification below characterises that number
*declaratively*, without any mention of contour lines, plug labels, the
row-major cell order, or the DP itself:

a Hamiltonian cycle is a set of grid edges that is a nonempty, connected,
2-regular spanning subgraph.  Each undirected cycle corresponds to exactly one
such edge set, so `hamCycleCount` counts every undirected cycle once — matching
the implementation's convention (neither direction nor starting point is
distinguished).  Degenerate grids are handled by the `Nonempty` clause: a grid
with no cells (and, more generally, any grid admitting no cycle) contributes
`0`.

Theorems below: the main correctness statement, plus the two boundary facts
that make the degenerate cases explicit (a single row/column has no cycle, and
a grid with an odd number of cells has none because it is bipartite).
-/

namespace HamPlug

/-- The cells of the `w × h` grid, as `(row, col)` pairs, `0 ≤ row < h`,
`0 ≤ col < w`. -/
def cells (w h : Nat) : Finset (Nat × Nat) :=
  (Finset.range h).product (Finset.range w)

/-- Two cells are orthogonally adjacent in the grid. -/
def adjacent (a b : Nat × Nat) : Prop :=
  (a.1 + 1 = b.1 ∧ a.2 = b.2) ∨ (b.1 + 1 = a.1 ∧ a.2 = b.2) ∨
  (a.1 = b.1 ∧ a.2 + 1 = b.2) ∨ (a.1 = b.1 ∧ b.2 + 1 = a.2)

/-- The undirected edges of the `w × h` grid, each represented as a 2-element
set of orthogonally adjacent cells.  Both orientations `(a, b)` and `(b, a)`
map to the same set, so `.image` de-duplicates them. -/
noncomputable def gridEdges (w h : Nat) : Finset (Finset (Nat × Nat)) := by
  classical
  exact
    (((cells w h).product (cells w h)).filter
        (fun p : (Nat × Nat) × (Nat × Nat) => adjacent p.1 p.2)).image
      (fun p => ({p.1, p.2} : Finset (Nat × Nat)))

/-- `u` and `v` lie in the same connected component of the subgraph with edge
set `S`. -/
inductive Reach (S : Finset (Finset (Nat × Nat))) : Nat × Nat → Nat × Nat → Prop
  | refl (u : Nat × Nat) : Reach S u u
  | step {u v w : Nat × Nat} :
      ({u, v} : Finset (Nat × Nat)) ∈ S → Reach S v w → Reach S u w

/-- The subgraph with edge set `S` is a Hamiltonian cycle of the `w × h`
grid: it has at least one cell, only uses grid edges, gives every cell degree
exactly two, and is connected.  Equivalent to "a cycle of grid edges visiting
every cell exactly once". -/
noncomputable def IsHamCycle (w h : Nat) (S : Finset (Finset (Nat × Nat))) : Prop := by
  classical
  exact
    (cells w h).Nonempty ∧
      S ⊆ gridEdges w h ∧
      (∀ v ∈ cells w h, (S.filter (fun e => v ∈ e)).card = 2) ∧
      (∀ u ∈ cells w h, ∀ v ∈ cells w h, Reach S u v)

/-- All Hamiltonian cycles of the `w × h` grid. -/
noncomputable def allHamCycles (w h : Nat) : Finset (Finset (Finset (Nat × Nat))) := by
  classical
  exact (gridEdges w h).powerset.filter (IsHamCycle w h)

/-- The number of Hamiltonian cycles of the `w × h` grid.  Each undirected
cycle is counted once. -/
noncomputable def hamCycleCount (w h : Nat) : Nat :=
  (allHamCycles w h).card

end HamPlug

open HamPlug

/-- **Correctness of the plug DP.**  If `ham_cycles_grid w h` returns `ok k`
then `k` is exactly the number of Hamiltonian cycles of the `w × h` grid graph
(`w` columns, `h` rows), each undirected cycle counted once.  The statement
needs no side conditions: the plug DP is total, and the specification agrees
with it on every grid, including the degenerate ones (`w = 0`, `h = 0`, or a
single row/column, all of which have no cycle). -/
theorem ham_cycles_grid_correct (w h : Std.Usize) (k : Std.Usize)
    (hres : ham_cycles_grid w h = ok k) :
    k.val = hamCycleCount w.val h.val := by
  sorry

/-- A grid with no columns (or no rows) has no cells, hence no cycle. -/
theorem hamCycleCount_no_columns (h : Nat) : hamCycleCount 0 h = 0 := by
  sorry

/-- A single row of `w` cells is a path, which has no cycle. -/
theorem hamCycleCount_one_row (w : Nat) : hamCycleCount w 1 = 0 := by
  sorry

/-- A grid is bipartite, so it has no Hamiltonian cycle when the number of
cells `w * h` is odd (`1 × 1`, `1 × n` with odd `n`, `3 × 3`, `5 × 5`, …). -/
theorem hamCycleCount_odd_area (w h : Nat) (hodd : w * h % 2 = 1) :
    hamCycleCount w h = 0 := by
  sorry
