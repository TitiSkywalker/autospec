import LeanMode.dp_count_hamiltonian.DpCountHamiltonian
import Mathlib

open Aeneas Aeneas.Std Result
open dp_count_hamiltonian

set_option linter.unusedVariables false

/-!
# Specification: counting Hamiltonian paths by inclusion–exclusion

`count_ham_paths n adj` evaluates the alternating sum
`Σ_{S ⊆ V} (−1)^{n − |S|} · W(S)` over the flat row-major adjacency matrix `adj`
and returns the number of Hamiltonian paths of the digraph.

The specification below is the declarative statement of that count: the number
of injective vertex sequences of length `n` in which every consecutive pair is
an edge — with no mention of the subset sum, of the walk counts `W(S)`, or of
the two parity buckets.
-/

namespace CountHamPaths

/-- View a flat `usize` slice as a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The entry function of the flat row-major `n × n` matrix `adj`:
`adjFn adj n i j` is the entry `i * n + j`, i.e. whether `i → j` is an edge
(`1`) or not (`0`). -/
def adjFn (adj : Slice Std.Usize) (n : Nat) : Fin n → Fin n → Nat :=
  fun i j => (toNatList adj).getD (i.val * n + j.val) 0

/-- Every consecutive pair of the list satisfies the relation `r`.  Lists of
length 0 and 1 satisfy it vacuously. -/
def ChainAdj {α : Type} (r : α → α → Prop) : List α → Prop
  | [] => True
  | [_] => True
  | a :: b :: t => r a b ∧ ChainAdj r (b :: t)

/-- `f` is a Hamiltonian path of the digraph with adjacency relation `d`: the
vertex sequence `f 0, f 1, …, f (n-1)` visits every vertex exactly once
(injectivity into `Fin n`) and every consecutive pair is an edge. -/
def IsHamPath {n : Nat} (d : Fin n → Fin n → Prop) (f : Fin n → Fin n) : Prop :=
  Function.Injective f ∧ ChainAdj d (List.ofFn f)

/-- The number of Hamiltonian paths of the digraph with adjacency relation `d`.
Bounding the sequences to the finite type `Fin n → Fin n` and requiring
injectivity makes this the count of the *orderings* of the vertices, so the
`Finset` is finite. -/
noncomputable def hamPathCount (n : Nat) (d : Fin n → Fin n → Prop) : Nat := by
  classical
  exact ((Finset.univ : Finset (Fin n → Fin n)).filter (fun f => IsHamPath d f)).card

end CountHamPaths

open CountHamPaths

/-- Correctness of the inclusion–exclusion Hamiltonian-path count: whenever
`count_ham_paths n adj` returns `c`, the value `c` is the number of Hamiltonian
paths of the digraph whose adjacency matrix is `adj`, where `i → j` is an edge
exactly when the entry `i * n + j` of the flat matrix is non-zero.

The hypotheses say that `adj` really is an `n × n` matrix (`h_len`) with `0/1`
entries (`h_adj`), so the entries are the characteristic function of a simple
digraph's edge relation.  The implementation *multiplies* the running count by
the entry `adj[u * n + v]` rather than branching on it, so an entry `> 1` would
act as an edge multiplicity and the result would no longer be a plain count;
`h_adj` (`≤ 1`) rules that out, leaving the entries as pure edge indicators and
the walk counts as edge counts. -/
theorem count_ham_paths_correct (n : Std.Usize) (adj : Slice Std.Usize) (c : Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_adj : ∀ i j : Fin n.val, adjFn adj n.val i j ≤ 1)
    (hres : count_ham_paths n adj = ok c) :
    c.val = hamPathCount n.val (fun i j => adjFn adj n.val i j ≠ 0) := by
  sorry
