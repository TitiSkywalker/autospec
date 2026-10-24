import LeanMode.dp_chromatic_number.DpChromaticNumber
import Mathlib

open Aeneas Aeneas.Std Result
open dp_chromatic_number

open Classical

set_option linter.unusedVariables false

/-!
# Specification: the chromatic number via independent-set subset DP

`chromatic_number n adj` runs a subset dynamic program over the `2^n` vertex
masks of the graph encoded by the flat row-major `n × n` adjacency matrix
`adj` (`adj[u * n + v] ≠ 0` iff `u` and `v` are adjacent), and returns the
chromatic number.

The specification below characterises that number *declaratively*, with no
mention of masks, colour classes, or the DP: a **proper `k`-colouring** assigns
to every vertex a colour in `Fin k` so that adjacent vertices differ, and the
chromatic number is the least `k` admitting one.  (`k = n` always admits one
when the graph is irreflexive — colour each vertex by its own index — so the
search over `0 .. n` suffices.)
-/

namespace ChromaticNumber

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The `(u, v)` entry of a flat row-major `n × n` matrix (element `(row, col)`
lives at `row * n + col`), defaulting to `0` out of range. -/
def entry (adj : List Nat) (n u v : Nat) : Nat :=
  adj.getD (u * n + v) 0

/-- `u` and `v` are adjacent: exactly the implementation's test
`adj[u * n + v] ≠ 0`.  The relation is read off the *implementation's*
comparison (`≠ 0`), so no `0/1` hypothesis is needed; the diagonal entry must
still vanish (irreflexivity, hypothesis `h_diag` below). -/
def Adj (adj : List Nat) (n : Nat) (u v : Fin n) : Prop :=
  entry adj n u.val v.val ≠ 0

/-- A proper `k`-colouring: adjacent vertices receive different colours. -/
abbrev IsColoring (adj : List Nat) (n k : Nat) (c : Fin n → Fin k) : Prop :=
  ∀ u v : Fin n, Adj adj n u v → c u ≠ c v

/-- The graph admits a proper `k`-colouring.  This is *decidable* (the set of
colouring functions `Fin n → Fin k` is finite and `IsColoring` is a decidable
predicate), which is what lets `minColors` below compute the minimum. -/
noncomputable def hasColoring (adj : List Nat) (n k : Nat) : Bool :=
  decide (0 < ((Finset.univ : Finset (Fin n → Fin k)).filter
    (fun c => IsColoring adj n k c)).card)

/-- The **chromatic number**: the least `k ≤ n` admitting a proper
`k`-colouring, or `n + 1` when none does (impossible for an irreflexive graph,
where every `k = n` with an injective colouring works). -/
noncomputable def minColors (adj : List Nat) (n : Nat) : Nat :=
  ((List.range (n + 1)).filter (fun k => hasColoring adj n k)).headD (n + 1)

end ChromaticNumber

open ChromaticNumber

/-- **Correctness of the subset DP for the chromatic number.**  If
`chromatic_number` returns `ok k` for a graph whose flat `n × n` adjacency
matrix is irreflexive (`h_diag`: every diagonal entry is `0`), then `k` is the
chromatic number of the graph — the least number of colours in a proper
colouring (see `minColors`).

`h_len` is the well-formedness condition that `adj` really is an `n × n`
matrix; `h_n` matches the implementation's `2^n`-states bound. -/
theorem chromatic_number_correct
    (n : Std.Usize) (adj : Slice Std.Usize) (k : Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_diag : ∀ u : Fin n.val, entry (toNatList adj) n.val u.val u.val = 0)
    (h_n : n.val ≤ 20)
    (hres : chromatic_number n adj = ok k) :
    k.val = minColors (toNatList adj) n.val := by
  sorry
