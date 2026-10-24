import LeanMode.dp_held_karp.DpHeldKarp
import Mathlib

open Aeneas Aeneas.Std Result
open dp_held_karp

set_option linter.unusedVariables false

/-!
# Specification: Held–Karp shortest Hamiltonian cycle (TSP)

`held_karp n dist` runs the Held–Karp subset DP over the flat row-major `n × n`
distance matrix `dist` and returns the cost of the cheapest Hamiltonian cycle.
The specification below characterises that cost *declaratively* — as the minimum
over all vertex orderings of the length of the cycle through them — with no
mention of bitmasks, of the DP table, or of the order in which the states are
enumerated.
-/

namespace HeldKarp

/-- View a flat `usize` slice as a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The distance function of the complete digraph on `n` vertices, read off the
flat row-major `n × n` matrix `dist`: `distFn dist n i j` is the length of the
edge `i → j`, i.e. the entry `i * n + j`. -/
def distFn (dist : Slice Std.Usize) (n : Nat) : Fin n → Fin n → Nat :=
  fun i j => (toNatList dist).getD (i.val * n + j.val) 0

/-- The successor of `i` in the cyclic order of `Fin n` (so `n - 1` maps back to
`0`). -/
def succCyc {n : Nat} (i : Fin n) : Fin n :=
  if h : i.val + 1 < n then ⟨i.val + 1, h⟩ else ⟨0, by omega⟩

/-- Length of the Hamiltonian cycle that visits the vertices in the order given
by the permutation `π` and returns to its starting vertex: the sum of the `n`
consecutive edges `π i → π (i + 1)`, indices taken cyclically. -/
def hamCycleCost {n : Nat} (d : Fin n → Fin n → Nat) (π : Equiv.Perm (Fin n)) : Nat :=
  (Finset.univ : Finset (Fin n)).sum (fun i => d (π i) (π (succCyc i)))

/-- The set of lengths of all Hamiltonian cycles on `n` vertices, i.e. the costs
of all `n!` cyclic orderings of the vertices. -/
def allCycleCosts (n : Nat) (d : Fin n → Fin n → Nat) : Finset Nat :=
  (Finset.univ : Finset (Equiv.Perm (Fin n))).image (fun π => hamCycleCost d π)

/-- The set of cycle costs is never empty (`Equiv.refl` always supplies one). -/
theorem allCycleCosts_nonempty (n : Nat) (d : Fin n → Fin n → Nat) :
    (allCycleCosts n d).Nonempty := by
  rw [allCycleCosts]
  exact Finset.Nonempty.image Finset.univ_nonempty _

/-- The cost of a shortest Hamiltonian cycle, i.e. the answer to the travelling
salesman problem on the complete digraph with distances `d`.  For `n = 0` the
empty permutation is the only (empty) cycle, of cost `0`. -/
def minHamCycleCost (n : Nat) (d : Fin n → Fin n → Nat) : Nat :=
  (allCycleCosts n d).min' (allCycleCosts_nonempty n d)

end HeldKarp

open HeldKarp

/-- Correctness of the Held–Karp subset DP: whenever `held_karp n dist` returns
`c`, the value `c` is the cost of a shortest Hamiltonian cycle on the `n`
vertices, where the distance from `i` to `j` is the entry `i * n + j` of the flat
matrix `dist`.  The hypothesis `dist.length = n * n` says that `dist` really is a
complete `n × n` matrix, so every ordering of the vertices is an admissible tour
and the minimum is well defined. -/
theorem held_karp_correct (n : Std.Usize) (dist : Slice Std.Usize) (c : Std.Usize)
    (h_len : dist.length = n.val * n.val)
    (hres : held_karp n dist = ok c) :
    c.val = minHamCycleCost n.val (distFn dist n.val) := by
  sorry
