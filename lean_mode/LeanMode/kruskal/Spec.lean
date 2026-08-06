import LeanMode.kruskal.Kruskal

open Aeneas Aeneas.Std Result
open kruskal

set_option linter.unusedVariables false

/-!
# Specification: Kruskal's Minimum Spanning Tree

`kruskal` computes a minimum spanning tree of a weighted undirected graph
given by a set of node identifiers and a slice of edges `(u, v, w)`.

## `is_mst` definition

The predicate `is_mst mst_edges graph_edges nodes` holds when
`mst_edges` is a minimum spanning tree for the graph `(graph_edges, nodes)`.
-/

/-- An edge `e` appears in the given edge slice. -/
def EdgeIn (e : Std.Usize × Std.Usize × Rat) (edges : Slice (Std.Usize × Std.Usize × Rat)) : Prop :=
  ∃ (i : Std.Usize), Slice.index_usize edges i = ok e

/-- The set of all edges present in the graph. -/
def EdgeSet (edges : Slice (Std.Usize × Std.Usize × Rat)) : Set (Std.Usize × Std.Usize × Rat) :=
  { e | EdgeIn e edges }

/-- Two nodes `u` and `v` are **directly connected** by a set of edges
    if there is an edge `(u, v, w)` or `(v, u, w)` in the set. -/
def DirectlyConnected (es : Set (Std.Usize × Std.Usize × Rat)) (u v : Std.Usize) : Prop :=
  ∃ (w : Rat), (u, v, w) ∈ es ∨ (v, u, w) ∈ es

/-- Inductive definition: a path is a sequence of nodes where each
    consecutive pair is directly connected by an edge in `es`. -/
inductive IsPath (es : Set (Std.Usize × Std.Usize × Rat)) : List Std.Usize → Prop
  | single (u v : Std.Usize) (h : DirectlyConnected es u v) : IsPath es [u, v]
  | cons (u v : Std.Usize) (rest : List Std.Usize)
      (h : DirectlyConnected es u v) (hrest : IsPath es (v :: rest)) :
      IsPath es (u :: v :: rest)

/-- An edge set **connects** a set of nodes if between any two
    distinct nodes there exists a path following the edge set. -/
def Connects (es : Set (Std.Usize × Std.Usize × Rat)) (nodes : Set Std.Usize) : Prop :=
  ∀ u, u ∈ nodes → ∀ v, v ∈ nodes → u ≠ v →
    ∃ (p : List Std.Usize), IsPath es p ∧ p.head? = some u ∧ p.getLast? = some v

/-- The total weight of a list of edges (sum of `w` for each `(u, v, w)`). -/
def TotalWeight (es : List (Std.Usize × Std.Usize × Rat)) : Rat :=
  es.foldl (λ sum e => sum + e.2.2) 0

/-- A list of edges is a **spanning tree** for the graph
    `(graph_edges, nodes)` if:
    1. Every edge comes from the original graph.
    2. The edges connect all nodes.
    3. `|E| = |V| - 1` — together with (2) this implies acyclicity. -/
def IsSpanningTree (mst_edges : List (Std.Usize × Std.Usize × Rat))
    (graph_edges : Set (Std.Usize × Std.Usize × Rat)) (nodes : Finset Std.Usize) : Prop :=
  (∀ e, e ∈ mst_edges → e ∈ graph_edges) ∧
  Connects ({ e | e ∈ mst_edges }) (nodes : Set Std.Usize) ∧
  mst_edges.length + 1 = nodes.card

/-- ## `is_mst`
    `mst_edges` is a **minimum spanning tree** if it is a spanning tree
    and no other spanning tree has strictly smaller total weight. -/
def is_mst (mst_edges : List (Std.Usize × Std.Usize × Rat))
    (graph_edges : Set (Std.Usize × Std.Usize × Rat)) (nodes : Finset Std.Usize) : Prop :=
  IsSpanningTree mst_edges graph_edges nodes ∧
  ∀ (other : List (Std.Usize × Std.Usize × Rat)),
    IsSpanningTree other graph_edges nodes →
    TotalWeight mst_edges ≤ TotalWeight other

/-! ## Preconditions (matching `kruskal_mst_precond` from algoveri) -/

/-- The set of nodes in a node slice (as a `Finset`). -/
def NodeSet (nodes : Slice Std.Usize) : Finset Std.Usize :=
  ((nodes : List Std.Usize)).toFinset

/-- Every edge endpoint is in the node set.
    Corresponds to `WeightedGraph.well_formed` in the adjacency-list spec. -/
def EdgesWellFormed (edges : Slice (Std.Usize × Std.Usize × Rat)) (nodes : Finset Std.Usize) : Prop :=
  ∀ (u v : Std.Usize) (w : Rat), (u, v, w) ∈ EdgeSet edges → u ∈ nodes ∧ v ∈ nodes

/-- The original graph connects all nodes (undirected).
    Corresponds to `WeightedGraph.is_connected` in the adjacency-list spec,
    but uses undirected edge semantics suitable for Kruskal's input. -/
def GraphConnects (nodes : Slice Std.Usize) (edges : Slice (Std.Usize × Std.Usize × Rat)) : Prop :=
  Connects (EdgeSet edges) (NodeSet nodes : Set Std.Usize)

/-- Combined preconditions for Kruskal's algorithm.
    Corresponds to `kruskal_mst_precond` from algoveri:
    `well_formed ∧ size > 0 ∧ is_connected ∧ weights_bounded`.
    (`weights_bounded` is omitted — not needed for mathematical correctness.) -/
def kruskal_precond (nodes : Slice Std.Usize) (edges : Slice (Std.Usize × Std.Usize × Rat)) : Prop :=
  EdgesWellFormed edges (NodeSet nodes) ∧
  (NodeSet nodes).card > 0 ∧
  GraphConnects nodes edges

/-! ## Correctness theorem (placeholder) -/

/-- **`kruskal_correct`**:
    Under the precondition that the graph is well-formed, non-empty,
    and connected, `kruskal nodes edges` returns a minimum spanning tree.

    Proof requires formalizing Kruskal's algorithm correctness:
    cut property, union-find invariants, and exchange argument. -/
theorem kruskal_correct (nodes : Slice Std.Usize)
    (edges : Slice (Std.Usize × Std.Usize × Rat))
    (h_precond : kruskal_precond nodes edges)
    (mst_vec : alloc.vec.Vec (Std.Usize × Std.Usize × Rat))
    (h : kruskal nodes edges = ok mst_vec) :
    is_mst ((alloc.vec.Vec.deref mst_vec).val) (EdgeSet edges) (NodeSet nodes) := by
  sorry
