import LeanMode.graph_tree_decomposition.GraphTreeDecomposition
import Mathlib

open Aeneas Aeneas.Std Result
open graph_tree_decomposition

set_option linter.unusedVariables false

/-!
# Specification: tree decompositions from an elimination ordering

`tree_decomposition n adj pi` reads an undirected graph off a flat, row-major
`n × n` symmetric `0/1` adjacency matrix `adj` and a vertex ordering `pi`, and
returns a tree decomposition of the graph:

* `bag_data` — the concatenated bag contents (vertex ids), bag `0` first;
* `bag_len[i]` — the number of vertices in bag `i`;
* `parent[i]` — the parent of bag `i` in the bag forest, sentinel `n` for a
  root.

`IsTreeDecomposition n Adj bt bl pl` is the textbook definition, spelled out on
this encoding:

* the flat encoding is consistent (`bt.length` is the sum of the bag lengths)
  and all three output vectors have length `n` — so there are exactly `n` bags;
* every bag is non-empty, duplicate-free and holds only valid vertices;
* `pl` is a rooted **forest** on the bags: `parent i` is either the sentinel `n`
  or a strictly larger index, and every bag reaches the sentinel;
* **(cover)** every edge `{u, v}` of `Adj` lies inside some bag;
* **(running intersection)** for every vertex `v`, the bags containing `v` form
  a connected subtree: they have a topmost bag `top` (the bag whose head is `v`)
  that every other such bag reaches by walking up, and the parent of every such
  bag other than `top` still contains `v`.

The bags are built from the *fill graph* of the elimination game, and that is
essential: the naive "later neighbours in the input graph" bag is **not** a tree
decomposition.  With `A–B–C–D` ordered `(A, B, C, D)`, the 4-cycle has naive
bags `{A,B,D}, {B,C}, {C,D}, {D}`, whose parent array is the path
`bag(A) → bag(B) → bag(C) → bag(D)`; vertex `D` sits in `bag(A)` and `bag(C)`
but the path between them runs through `bag(B)`, which does not contain `D`.
The clique completion performed by the implementation is exactly what restores
the running-intersection property (`tree_decomposition_running_intersection`).
The `h01`/`h_symm`/`h_irrefl` hypotheses are the `graph_hopcroft_karp` /
`graph_chordal_mcs` conventions: the implementation tests entries with `≠ 0`
while `toAdj` reads them as `= 1`, so `h01` is what makes the two agree.

The parent array is characterised as the **elimination tree**: `parent i` is the
smallest index `j > i` whose bag head `π j` lies in bag `i`
(`tree_decomposition_parent_min`).
-/

namespace graph_tree_decomposition

/-! ## Reading the flat inputs and the flat outputs -/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Nat`. -/
def toMatrix (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) :
    Fin n → Fin n → Nat :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    (s[i.val * n + j.val]'h_bound).val

/-- View a flat, row-major `n × n` matrix as a binary relation
    `Fin n → Fin n → Prop` (`Adj i j` holds iff the entry is `1`, the value the
    implementation's `≠ 0` test sees under the `0/1` hypothesis). -/
def toAdj (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) :
    Fin n → Fin n → Prop :=
  λ i j => toMatrix s n h i j = 1

/-- View a `Slice Std.Usize` of length `n` as `Fin n → Nat` (the input vertex
    ordering `pi`). -/
def toSliceNat (s : Slice Std.Usize) (n : Nat) (h : s.length = n) :
    Fin n → Nat :=
  λ i =>
    have h_bound : i.val < s.length := by rw [h]; exact i.is_lt
    (s[i.val]'h_bound).val

/-- A `Vec Std.Usize` as a `List Nat` (the shape the encoding helpers below
    work with). -/
def vlist (v : alloc.vec.Vec Std.Usize) : List Nat :=
  ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize).map (fun x => x.val)

/-! ## Decoding the bag encoding -/

/-- The offset of bag `i` in `bag_data`: the sum of the lengths of the bags
    before it. -/
def bagStartOf (bl : List Nat) (i : Nat) : Nat :=
  (bl.take i).foldl (fun a b => a + b) 0

/-- The contents of bag `i`, decoded from the CSR-style encoding: the `bl[i]`
    entries of `bt` starting at `bagStartOf bl i`. -/
def bagOf (bt bl : List Nat) (i : Nat) : List Nat :=
  (bt.drop (bagStartOf bl i)).take (bl.getD i 0)

/-- The size of bag `i`. -/
def bagCard (bl : List Nat) (i : Nat) : Nat := bl.getD i 0

/-! ## The bag forest -/

/-- The parent function of the bag array `pl`, with the sentinel `n = nb` for
    the roots (and for anything past the end). -/
def parFnAt (pl : List Nat) (nb : Nat) : Nat → Nat := fun i => pl.getD i nb

/-- `Anc f x y`: `y` is an ancestor of `x`, i.e. `y` appears on the parent-path
    `x → f x → f (f x) → …`. -/
inductive Anc (f : Nat → Nat) : Nat → Nat → Prop where
  | refl (x : Nat) : Anc f x x
  | step {x y : Nat} : Anc f (f x) y → Anc f x y

/-- The bags form a **rooted forest** on `{0, …, nb-1}` (the sentinel `nb` is
    "no parent"): every parent is in range, a parent is either the sentinel or
    a strictly larger index (so there are no cycles and each component has
    exactly one minimal element), and every bag reaches the sentinel.  Note
    that a disconnected graph legitimately produces several roots here — one
    per connected component — so this is a forest, not a tree. -/
def IsBagForest (f : Nat → Nat) (nb : Nat) : Prop :=
  (∀ i, i < nb → f i ≤ nb) ∧
  (∀ i, i < nb → f i = nb ∨ i < f i) ∧
  (∀ i, i < nb → f^[nb] i = nb)

/-- **Running intersection**: for every vertex `v` appearing in a bag, the bags
    containing `v` form a connected subtree.  Concretely there is a topmost such
    bag `top` (in the implementation, the bag whose head is `v`) that every
    other bag containing `v` reaches by walking up the parent chain, and the
    parent of every bag containing `v` other than `top` still contains `v`. -/
def IsRunningIntersection (bt bl : List Nat) (f : Nat → Nat) (nb : Nat) : Prop :=
  ∀ v, (∃ i, i < nb ∧ v ∈ bagOf bt bl i) →
    ∃ top, top < nb ∧ v ∈ bagOf bt bl top ∧
      (∀ i, i < nb → v ∈ bagOf bt bl i → Anc f i top) ∧
      (∀ i, i < nb → v ∈ bagOf bt bl i → i ≠ top → v ∈ bagOf bt bl (f i))

/-! ## The full specification predicate -/

/-- `(bt, bl, pl)` is a tree decomposition of the `n`-vertex graph `Adj`, with
    exactly `n` bags, encoded as described at the top of this file. -/
def IsTreeDecomposition (n : Nat) (Adj : Fin n → Fin n → Prop)
    (bt bl pl : List Nat) : Prop :=
  bt.length = bl.foldl (fun a b => a + b) 0 ∧
  bl.length = n ∧
  pl.length = n ∧
  (∀ i, i < n → 1 ≤ bagCard bl i) ∧
  (∀ i, i < n → (bagOf bt bl i).Nodup) ∧
  (∀ i, i < n → ∀ v, v ∈ bagOf bt bl i → v < n) ∧
  IsBagForest (parFnAt pl n) n ∧
  (∀ u v : Fin n, Adj u v →
    ∃ i, i < n ∧ u.val ∈ bagOf bt bl i ∧ v.val ∈ bagOf bt bl i) ∧
  IsRunningIntersection bt bl (parFnAt pl n) n

end graph_tree_decomposition

open graph_tree_decomposition

/-! ## Specification theorems -/

/-- The bag encoding is well formed: `bag_data` is the concatenation of the
    bags, `bag_len` and `parent` have one entry per bag, every bag is non-empty,
    duplicate-free and holds only valid vertices. -/
theorem tree_decomposition_bags
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h : tree_decomposition n adj pi = ok td) :
    (vlist td.bag_data).length =
      (vlist td.bag_len).foldl (fun a b => a + b) 0 ∧
    (vlist td.bag_len).length = n.val ∧
    (vlist td.parent).length = n.val ∧
    (∀ i, i < n.val → 1 ≤ bagCard (vlist td.bag_len) i) ∧
    (∀ i, i < n.val → (bagOf (vlist td.bag_data) (vlist td.bag_len) i).Nodup) ∧
    (∀ i, i < n.val → ∀ v, v ∈ bagOf (vlist td.bag_data) (vlist td.bag_len) i →
      v < n.val) := by
  sorry

/-- The bags form a rooted forest: each bag's parent is either the sentinel `n`
    or a strictly larger bag index, and walking up from any bag reaches the
    sentinel.  A disconnected input graph yields one root per connected
    component, so the structure is a forest and *not* a tree in general. -/
theorem tree_decomposition_forest
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h : tree_decomposition n adj pi = ok td) :
    IsBagForest (parFnAt (vlist td.parent) n.val) n.val := by
  sorry

/-- **(cover)** Every edge of the input graph lies inside some bag. -/
theorem tree_decomposition_covers
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_symm : ∀ i j : Fin n.val,
      toAdj adj n.val h_adj i j ↔ toAdj adj n.val h_adj j i)
    (h_irrefl : ∀ i : Fin n.val, ¬ toAdj adj n.val h_adj i i)
    (h : tree_decomposition n adj pi = ok td) :
    ∀ u v : Fin n.val, toAdj adj n.val h_adj u v →
      ∃ i, i < n.val ∧
        u.val ∈ bagOf (vlist td.bag_data) (vlist td.bag_len) i ∧
        v.val ∈ bagOf (vlist td.bag_data) (vlist td.bag_len) i := by
  sorry

/-- **(running intersection)** For every vertex, the bags containing it form a
    connected subtree of the bag forest: there is a topmost such bag which every
    other one reaches by walking up, and the parent of every other such bag
    still contains the vertex. -/
theorem tree_decomposition_running_intersection
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h : tree_decomposition n adj pi = ok td) :
    IsRunningIntersection (vlist td.bag_data) (vlist td.bag_len)
      (parFnAt (vlist td.parent) n.val) n.val := by
  sorry

/-- **Correctness**: the output is a tree decomposition of the input graph.
    This is the conjunction of the four statements above, packaged as the
    textbook predicate `IsTreeDecomposition`.

    Preconditions (not checked at runtime): `adj` has length `n * n` and encodes
    a symmetric, irreflexive `0/1` matrix, and `pi` lists each of the `n`
    vertices exactly once.  The `0/1` hypothesis is what reconciles the
    implementation's `≠ 0` edge test with the `= 1` spec relation `toAdj`; the
    symmetry hypothesis is needed because the implementation only ever reads
    the entry at the earlier of the two positions in the order. -/
theorem tree_decomposition_correct
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_symm : ∀ i j : Fin n.val,
      toAdj adj n.val h_adj i j ↔ toAdj adj n.val h_adj j i)
    (h_irrefl : ∀ i : Fin n.val, ¬ toAdj adj n.val h_adj i i)
    (h : tree_decomposition n adj pi = ok td) :
    IsTreeDecomposition n.val (toAdj adj n.val h_adj)
      (vlist td.bag_data) (vlist td.bag_len) (vlist td.parent) := by
  sorry

/-- **The elimination tree.**  For every bag `i`, the parent of `i` is the
    *earliest* later bag whose head `π j` also lies in bag `i`: every such head
    is at an index `≥ parent i`, and the head `π (parent i)` itself lies in bag
    `i` whenever `parent i < n`.  (Together with `IsBagForest` this pins
    `parent i` down as `min { j > i : π j ∈ bag i }`, with the sentinel `n` when
    the set is empty.) -/
theorem tree_decomposition_parent_min
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : Slice Std.Usize)
    (td : TreeDecomposition)
    (h_adj : adj.length = n.val * n.val)
    (h_pi_len : pi.length = n.val)
    (h_pi_lt : ∀ i : Fin n.val, toSliceNat pi n.val h_pi_len i < n.val)
    (h_pi_inj : ∀ i j : Fin n.val, i ≠ j →
      toSliceNat pi n.val h_pi_len i ≠ toSliceNat pi n.val h_pi_len j)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h : tree_decomposition n adj pi = ok td) :
    (∀ i j : Fin n.val, i.val < j.val →
      parFnAt (vlist td.parent) n.val i.val = j.val →
      toSliceNat pi n.val h_pi_len j ∈
        bagOf (vlist td.bag_data) (vlist td.bag_len) i.val) ∧
    (∀ i j : Fin n.val, i.val < j.val →
      toSliceNat pi n.val h_pi_len j ∈
        bagOf (vlist td.bag_data) (vlist td.bag_len) i.val →
      parFnAt (vlist td.parent) n.val i.val ≤ j.val) := by
  sorry

/-! ## The optional decision problem: `treewidth ≤ k` -/

/-- The **width** of a decomposition: the largest bag size minus one, i.e.
    `max_i (|bag i| - 1)`.  For a decomposition whose bags are non-empty this is
    `(max_i |bag i|) - 1`, the textbook width. -/
def decWidth (n : Nat) (bl : List Nat) : Nat :=
  (List.range n).foldl (fun acc i => max acc (bagCard bl i - 1)) 0

/-- **Treewidth = minimum elimination width.**  If `min_elimination_width`
    returns `w` then

    * some tree decomposition of width `≤ w` exists (the one generated by the
      optimal elimination ordering — `decWidth ≤ w`), so `tw ≤ w`, and
    * no tree decomposition of the graph is narrower: every decomposition
      satisfying `IsTreeDecomposition` has width `≥ w`, so `tw ≥ w`.

    Hence `w` *is* the treewidth of the graph.  The second conjunct is the
    classical characterisation "treewidth = minimum elimination width"; the
    implementation was cross-checked against the independent
    `tw = min { ω(H) - 1 : H ⊇ G chordal }` formulation on **every** graph with
    at most five vertices (`src/lib.rs`, `mod tw_reference`). -/
theorem min_elimination_width_correct
    (n : Std.Usize) (adj : Slice Std.Usize) (w : Std.Usize)
    (h_adj : adj.length = n.val * n.val)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_symm : ∀ i j : Fin n.val,
      toAdj adj n.val h_adj i j ↔ toAdj adj n.val h_adj j i)
    (h_irrefl : ∀ i : Fin n.val, ¬ toAdj adj n.val h_adj i i)
    (h : min_elimination_width n adj = ok w) :
    (∃ (bt bl pl : List Nat),
      IsTreeDecomposition n.val (toAdj adj n.val h_adj) bt bl pl ∧
      decWidth n.val bl ≤ w.val) ∧
    (∀ (bt bl pl : List Nat),
      IsTreeDecomposition n.val (toAdj adj n.val h_adj) bt bl pl →
      w.val ≤ decWidth n.val bl) := by
  sorry

/-- **The `treewidth ≤ k` decision.**  With `w` the minimum elimination width
    of the graph (i.e. its treewidth, by `min_elimination_width_correct`), the
    graph admits a tree decomposition of width at most `k` exactly when
    `w ≤ k`.  This is the decision form of the partial-k-tree /
    elimination-ordering characterisation of treewidth. -/
theorem min_elimination_width_le_iff
    (n : Std.Usize) (adj : Slice Std.Usize) (w k : Std.Usize)
    (h_adj : adj.length = n.val * n.val)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_symm : ∀ i j : Fin n.val,
      toAdj adj n.val h_adj i j ↔ toAdj adj n.val h_adj j i)
    (h_irrefl : ∀ i : Fin n.val, ¬ toAdj adj n.val h_adj i i)
    (h : min_elimination_width n adj = ok w) :
    (w.val ≤ k.val) ↔
      ∃ (bt bl pl : List Nat),
        IsTreeDecomposition n.val (toAdj adj n.val h_adj) bt bl pl ∧
        decWidth n.val bl ≤ k.val := by
  sorry
