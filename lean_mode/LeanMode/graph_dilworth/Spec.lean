import LeanMode.graph_dilworth.GraphDilworth
import Mathlib

open Aeneas Aeneas.Std Result
open graph_dilworth

set_option linter.unusedVariables false

/-!
# Specification: Minimum Path Cover of a DAG (König / Dilworth)

`min_path_cover n from_list to_list` is given a DAG on the vertices `0..n-1`
by parallel edge lists (`from_list[k] -> to_list[k]` is a directed edge, each
edge listed once) and returns the size of a minimum *vertex-disjoint path
cover* of the DAG: the least number of directed paths, using the DAG's own
edges, whose vertex sets partition `0..n-1`.

The algorithm builds the *split bipartite graph* (a left copy `u_out` and a
right copy `v_in` of every vertex, and an edge `u_out -- v_in` for every DAG
edge `u -> v`), computes a maximum matching `ν` of it with an iterative
Kuhn/Augmenting-Path routine, and returns `n - ν`.

The formalisation distinguishes two cover notions, because they are *not* the
same object:

* `IsPathCover` — `k` vertex-disjoint directed **paths** using the DAG's edges
  (a list whose consecutive entries are edges).  This is what the algorithm
  minimises, so `min_path_cover_eq` / `min_path_cover_konig` are stated for it.
* `IsChainCover` — `k` **chains** of the reachability order `Reach` (pairwise
  comparable vertices).  This is the notion in Dilworth's theorem, stated as
  `dilworth_theorem`.

Every path is a chain (`path_cover_is_chain_cover`), so the minimum chain
cover is never larger than the minimum path cover
(`min_chain_cover_le_path_cover`); it can be strictly smaller — the DAG
`0 -> 1, 0 -> 2, 3 -> 0, 4 -> 0` is covered by the two chains `{3,0,1}` and
`{4,2}` although `4 -> 2` is not an edge, so three paths are necessary there.
-/

namespace graph_dilworth

/-! ## The DAG, its edges and its reachability relation -/

/-- A directed graph given as parallel `from_list` / `to_list` slices. -/
structure CsrGraph where
  from_list : Slice Std.Usize
  to_list : Slice Std.Usize
  h_len : from_list.length = to_list.length

/-- View an edge-list slice of length `m` as `Fin m → Nat`. -/
def toEdgeVec (s : Slice Std.Usize) (m : Nat) (h : s.length = m) : Fin m → Nat :=
  λ k =>
    have hb : k.val < s.length := by rw [h]; exact k.is_lt
    (s[k.val]'hb).val

/-- Directed edge `u → v` occurs in the concrete edge list. -/
def edge_in (g : CsrGraph) (u v : Nat) : Prop :=
  ∃ k : Fin g.from_list.length,
    toEdgeVec g.from_list g.from_list.length rfl k = u ∧
    toEdgeVec g.to_list g.from_list.length g.h_len.symm k = v

/-- The edge relation of the DAG, read off the concrete edge list. -/
def Adj (g : CsrGraph) (n : Nat) (u v : Fin n) : Prop := edge_in g u.val v.val

/-- Precondition: every edge endpoint lies in the vertex range `0..n-1`.  This
    is what makes the concrete edge list agree with the graph on `Fin n`: the
    program indexes its adjacency matrix by `idx u n v = u * n + v`, so an
    endpoint `≥ n` would either fault out of bounds (when the from-endpoint is
    `≥ n`) or write an unrelated cell of the `n × n` matrix (when the
    to-endpoint is `≥ n`), while `Adj` above simply drops such an edge. -/
def EdgesWellFormed (g : CsrGraph) (n : Nat) : Prop :=
  ∀ e : Fin g.from_list.length,
    toEdgeVec g.from_list g.from_list.length rfl e < n ∧
    toEdgeVec g.to_list g.from_list.length g.h_len.symm e < n

/-- Reachability: the reflexive-transitive closure of the edge relation. -/
def Reach (g : CsrGraph) (n : Nat) : Fin n → Fin n → Prop :=
  Relation.ReflTransGen (Adj g n)

/-- The input is a DAG: no edge of the input lies on a closed walk, i.e. the
    reachability relation is a partial order (antisymmetric). -/
def IsDAG (g : CsrGraph) (n : Nat) : Prop :=
  ∀ u v : Fin n, Adj g n u v → Reach g n v u → False

/-! ## Paths: the object the algorithm minimises -/

/-- `l` is a directed path of the DAG: a nonempty list whose consecutive
    entries are edges.  A one-element list is a trivial (single-vertex) path. -/
inductive IsDirectedPath (g : CsrGraph) (n : Nat) : List (Fin n) → Prop
  | single (v : Fin n) : IsDirectedPath g n [v]
  | cons (u v : Fin n) (l : List (Fin n)) :
      Adj g n u v → IsDirectedPath g n (v :: l) →
      IsDirectedPath g n (u :: v :: l)

/-- `k` vertex-disjoint directed paths whose vertex sets cover `Fin n`: a path
    cover with `k` paths. -/
def IsPathCover (g : CsrGraph) (n k : Nat) : Prop :=
  ∃ P : Fin k → List (Fin n),
    (∀ i, IsDirectedPath g n (P i)) ∧
    (∀ i j, i ≠ j → Disjoint (P i).toFinset (P j).toFinset) ∧
    (∀ v : Fin n, ∃ i : Fin k, v ∈ P i)

/-- `k` is the minimum size of a path cover of the DAG. -/
def MinPathCover (g : CsrGraph) (n k : Nat) : Prop :=
  IsPathCover g n k ∧ ∀ k', IsPathCover g n k' → k ≤ k'

/-- The minimum path-cover size of the DAG. -/
noncomputable def minPathCoverSize (g : CsrGraph) (n : Nat) : Nat :=
  sInf {k : Nat | IsPathCover g n k}

/-! ## Chains and antichains of the reachability order -/

/-- A chain: its vertices are pairwise comparable under `Reach`. -/
def IsChain (g : CsrGraph) (n : Nat) (S : Finset (Fin n)) : Prop :=
  ∀ u ∈ S, ∀ v ∈ S, Reach g n u v ∨ Reach g n v u

/-- An antichain: distinct vertices of `S` are pairwise incomparable, i.e. no
    distinct pair is comparable in either direction. -/
def IsAntichain (g : CsrGraph) (n : Nat) (S : Finset (Fin n)) : Prop :=
  ∀ u ∈ S, ∀ v ∈ S, u ≠ v → ¬ (Reach g n u v ∨ Reach g n v u)

/-- `k` nonempty chains partitioning `Fin n`: a chain cover with `k` chains. -/
def IsChainCover (g : CsrGraph) (n k : Nat) : Prop :=
  ∃ c : Fin k → Finset (Fin n),
    (∀ i, (c i).Nonempty) ∧
    (∀ i, IsChain g n (c i)) ∧
    (∀ i j, i ≠ j → Disjoint (c i) (c j)) ∧
    (∀ v : Fin n, ∃ i : Fin k, v ∈ c i)

/-- `k` is the minimum size of a chain cover of the reachability order. -/
def MinChainCover (g : CsrGraph) (n k : Nat) : Prop :=
  IsChainCover g n k ∧ ∀ k', IsChainCover g n k' → k ≤ k'

/-- `m` is the maximum antichain size of the reachability order. -/
def MaxAntichain (g : CsrGraph) (n m : Nat) : Prop :=
  ∃ S : Finset (Fin n),
    IsAntichain g n S ∧ S.card = m ∧
    ∀ S' : Finset (Fin n), IsAntichain g n S' → S'.card ≤ m

/-- The maximum antichain size of the reachability order. -/
noncomputable def maxAntichainSize (g : CsrGraph) (n : Nat) : Nat :=
  sSup {m : Nat | ∃ S : Finset (Fin n), IsAntichain g n S ∧ S.card = m}

/-- The minimum chain-cover size of the reachability order. -/
noncomputable def minChainCoverSize (g : CsrGraph) (n : Nat) : Nat :=
  sInf {k : Nat | IsChainCover g n k}

/-! ## The split bipartite graph and its maximum matching -/

/-- A matching of the split bipartite graph: a set of DAG edges, no two of
    which share a left (out) copy or a right (in) copy of a vertex. -/
def IsMatching (g : CsrGraph) (n : Nat) (M : Finset (Fin n × Fin n)) : Prop :=
  (∀ p ∈ M, Adj g n p.1 p.2) ∧
  (∀ p ∈ M, ∀ q ∈ M, p.1 = q.1 → p = q) ∧
  (∀ p ∈ M, ∀ q ∈ M, p.2 = q.2 → p = q)

/-- The maximum matching size of the split bipartite graph. -/
noncomputable def maxMatchingSize (g : CsrGraph) (n : Nat) : Nat :=
  sSup {m : Nat | ∃ M : Finset (Fin n × Fin n), IsMatching g n M ∧ M.card = m}

/-! ## Specification theorems -/

/-- The program's answer is the minimum path-cover size of the input DAG: it
    admits a cover by `k` vertex-disjoint directed paths, and no cover uses
    fewer paths.  Requires every edge endpoint to lie in `0..n-1`
    (`EdgesWellFormed`), otherwise the program's flat matrix indexing puts an
    out-of-range edge in the wrong cell. -/
theorem min_path_cover_eq
    (n : Std.Usize) (g : CsrGraph) (k : Std.Usize)
    (h_dag : IsDAG g n.val)
    (h_wf : EdgesWellFormed g n.val)
    (h : min_path_cover n g.from_list g.to_list = ok k) :
    MinPathCover g n.val k.val := by
  sorry

/-- Optimality, stated directly against concrete covers: the program's answer
    is at most the number of paths in any path cover of the DAG. -/
theorem min_path_cover_optimal
    (n : Std.Usize) (g : CsrGraph) (k : Std.Usize) (j : Nat)
    (h_dag : IsDAG g n.val)
    (h_wf : EdgesWellFormed g n.val)
    (h : min_path_cover n g.from_list g.to_list = ok k)
    (h_cover : IsPathCover g n.val j) :
    k.val ≤ j := by
  sorry

/-- The program's answer is exactly the minimum path-cover size. -/
theorem min_path_cover_size
    (n : Std.Usize) (g : CsrGraph) (k : Std.Usize)
    (h_dag : IsDAG g n.val)
    (h_wf : EdgesWellFormed g n.val)
    (h : min_path_cover n g.from_list g.to_list = ok k) :
    minPathCoverSize g n.val = k.val := by
  sorry

/-- **König's theorem, in path-cover form**: the minimum path cover of a DAG
    plus the maximum matching size of its split bipartite graph is `n` (this
    is the identity `k = n - ν` the program computes). -/
theorem min_path_cover_konig
    (n : Std.Usize) (g : CsrGraph) (k : Std.Usize)
    (h_dag : IsDAG g n.val)
    (h_wf : EdgesWellFormed g n.val)
    (h : min_path_cover n g.from_list g.to_list = ok k) :
    k.val + maxMatchingSize g n.val = n.val := by
  sorry

/-- Every path cover is a chain cover: the vertices of a directed path are
    pairwise reachable-comparable. -/
theorem path_cover_is_chain_cover
    (n k : Nat) (g : CsrGraph) (h : IsPathCover g n k) :
    IsChainCover g n k := by
  sorry

/-- The minimum chain cover is never larger than the minimum path cover
    (paths are chains), and may be strictly smaller. -/
theorem min_chain_cover_le_path_cover
    (n k j : Nat) (g : CsrGraph)
    (h_path : MinPathCover g n k) (h_chain : MinChainCover g n j) :
    j ≤ k := by
  sorry

/-- Every antichain needs a path of its own, so the maximum antichain size is
    a lower bound for the minimum path cover. -/
theorem max_antichain_le_min_path_cover
    (n k m : Nat) (g : CsrGraph)
    (h_path : MinPathCover g n k) (h_antichain : MaxAntichain g n m) :
    m ≤ k := by
  sorry

/-- **Dilworth's theorem** (1950): the minimum number of chains partitioning
    a finite partial order equals the maximum size of an antichain.  Here the
    partial order is the reachability order of the input DAG. -/
theorem dilworth_theorem
    (n k : Nat) (g : CsrGraph)
    (h_dag : IsDAG g n)
    (h : MinChainCover g n k) :
    MaxAntichain g n k := by
  sorry

/-- Dilworth's theorem, in the "min = max" form. -/
theorem dilworth_min_max
    (n : Nat) (g : CsrGraph) (h_dag : IsDAG g n) :
    maxAntichainSize g n = minChainCoverSize g n := by
  sorry

end graph_dilworth
