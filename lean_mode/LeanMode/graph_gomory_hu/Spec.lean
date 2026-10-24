import LeanMode.graph_gomory_hu.GraphGomoryHu
import Mathlib

open Aeneas Aeneas.Std Result
open graph_gomory_hu

set_option linter.unusedVariables false

/-!
# Specification: the Gomory–Hu cut tree (Gusfield's algorithm)

`gomory_hu n cap` returns `ok (parent, weight)`, a tree on the vertices
`0, …, n-1` rooted at vertex `0` (`parent[0] = 0`, and following `parent` from
any vertex reaches `0`), where the tree edge `(i, parent[i])` carries the
capacities `weight[i]`.

The theory is the **Gomory–Hu tree theorem** (1961): for every pair `u ≠ v`,
the minimum `u`–`v` cut of the graph equals the minimum `weight` on the tree
path from `u` to `v`.  In particular the single tree encodes all
`n (n-1) / 2` pairwise min-cut values.

`cap` is a flat, row-major, symmetric `n × n` capacity matrix
(`toCap cap n h i j` is the capacity of the edge `i — j`, `0` = no edge).

## Layout

* `toCap` — the flat-matrix view.
* `cutCapacity`, `IsMinCut`, `minCut` — cuts and their minimum.
* `walk`, `isTreeRootedAt`, `rootPath`, `minEdgeOnPath` — the returned arrays
  as a rooted tree plus the minimum edge weight on a tree path.
* `isGomoryHuTree` — "these arrays are a Gomory–Hu tree".
* theorems — the returned arrays are a rooted tree, and every pairwise min cut
  is realised by a tree path (`gomory_hu_correct`), plus the max-flow/min-cut
  statement for the inlined Edmonds–Karp helper.
-/

namespace graph_gomory_hu

/-! ## The flat capacity matrix -/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) of unsigned capacities as `Fin n → Fin n → Nat`. -/
def toCap (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Nat :=
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

/-! ## Cuts -/

/-- Capacity of the cut induced by `S`: the total capacity of the edges leaving
    `S` (from `S` to its complement).  The diagonal `cap v v` never contributes,
    because `u ∈ S` and `v ∉ S` forces `u ≠ v`. -/
def cutCapacity {n : Nat} (cap : Fin n → Fin n → Nat) (S : Finset (Fin n)) : Nat :=
  ∑ u ∈ S, ∑ v : Fin n, if v ∈ S then 0 else cap u v

/-- `val` is the minimum `u`–`v` cut value: it is *achieved* by some separating
    set `S`, and no separating set has a smaller capacity. -/
def IsMinCut {n : Nat} (cap : Fin n → Fin n → Nat) (u v : Fin n) (val : Nat) : Prop :=
  (∃ S : Finset (Fin n), u ∈ S ∧ v ∉ S ∧ cutCapacity cap S = val) ∧
  (∀ S : Finset (Fin n), u ∈ S → v ∉ S → val ≤ cutCapacity cap S)

/-- The minimum `u`–`v` cut value (the `sInf` of the capacities of the sets
    separating `u` from `v`). -/
noncomputable def minCut {n : Nat} (cap : Fin n → Fin n → Nat) (u v : Fin n) : Nat :=
  sInf { val : Nat | ∃ S : Finset (Fin n), u ∈ S ∧ v ∉ S ∧ cutCapacity cap S = val }

/-- `minCut` is an achieved and minimal cut value. -/
theorem minCut_isMinCut {n : Nat} (cap : Fin n → Fin n → Nat) {u v : Fin n}
    (huv : u ≠ v) : IsMinCut cap u v (minCut cap u v) := by
  sorry

/-! ## The returned arrays as a rooted tree -/

/-- The `k`-th iterate of `parent`. -/
def walk {n : Nat} (parent : Fin n → Fin n) : Nat → Fin n → Fin n
  | 0, i => i
  | k + 1, i => walk parent k (parent i)

/-- `parent` is a tree rooted at `r`: `r` is its own parent, and following
    `parent` from any vertex eventually reaches `r`. -/
def isTreeRootedAt {n : Nat} (r : Fin n) (parent : Fin n → Fin n) : Prop :=
  parent r = r ∧ ∀ i : Fin n, ∃ k : Nat, walk parent k i = r

/-- The first `fuel` vertices on the root path of `i` (walking `parent`). -/
def rootPath {n : Nat} (parent : Fin n → Fin n) : Nat → Fin n → List (Fin n)
  | 0, _ => []
  | k + 1, i => i :: rootPath parent k (parent i)

/-- The minimum of a list of naturals (`0` for the empty list). -/
def listMin : List Nat → Nat
  | [] => 0
  | x :: xs => xs.foldr Nat.min x

/-- The weights of the tree edges on the `u`–`v` path, each tree edge being
    represented by its child endpoint.  The two root paths share the common
    prefix up to the join of `u` and `v`, so the symmetric difference of the
    two vertex sets is exactly the set of child endpoints of the path's edges. -/
def pathEdgeWeights {n : Nat} (parent : Fin n → Fin n) (weight : Fin n → Nat)
    (N : Nat) (u v : Fin n) : List Nat :=
  let pu := rootPath parent N u
  let pv := rootPath parent N v
  List.filterMap (fun x => if x ∈ pv then none else some (weight x)) pu ++
    List.filterMap (fun x => if x ∈ pu then none else some (weight x)) pv

/-- The minimum tree-edge weight on the tree path from `u` to `v`
    (`fuel = N` bounds the length of the walked paths). -/
def minEdgeOnPath {n : Nat} (parent : Fin n → Fin n) (weight : Fin n → Nat)
    (N : Nat) (u v : Fin n) : Nat :=
  listMin (pathEdgeWeights parent weight N u v)

/-- The predicate "`parent` / `weight` is a Gomory–Hu tree of `cap` rooted at
    `r`": the parent array is a tree rooted at `r`, and the minimum weight on
    each tree path equals the corresponding minimum cut. -/
def isGomoryHuTree {n : Nat} (r : Fin n) (cap : Fin n → Fin n → Nat)
    (parent : Fin n → Fin n) (weight : Fin n → Nat) : Prop :=
  isTreeRootedAt r parent ∧
    ∀ u v : Fin n, u ≠ v → minEdgeOnPath parent weight n u v = minCut cap u v

/-! ## Extracting the returned arrays -/

/-- Entry `i` of a returned `Vec Std.Usize` of length exactly `n`. -/
def vecEntry (v : alloc.vec.Vec Std.Usize) (n : Nat)
    (h : v.length = n) (i : Fin n) : Std.Usize :=
  have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
  v[i.val]'h_bound

/-- A returned `Vec Std.Usize` as a function `Fin n → Fin n` (the range
    hypothesis says the stored values really are vertex numbers). -/
def toIdxFun (v : alloc.vec.Vec Std.Usize) (n : Nat)
    (h_len : v.length = n)
    (h_range : ∀ i : Fin n, (vecEntry v n h_len i).val < n) : Fin n → Fin n :=
  λ i => ⟨(vecEntry v n h_len i).val, h_range i⟩

/-- A returned `Vec Std.Usize` as a function `Fin n → Nat`. -/
def toNatFun (v : alloc.vec.Vec Std.Usize) (n : Nat)
    (h_len : v.length = n) : Fin n → Nat :=
  λ i => (vecEntry v n h_len i).val

/-! ## Specification theorems -/

/-- Both returned arrays have length `n`. -/
theorem gomory_hu_lengths
    (n : Std.Usize) (cap : Slice Std.Usize)
    (parent weight : alloc.vec.Vec Std.Usize)
    (h : gomory_hu n cap = ok (parent, weight)) :
    parent.length = n.val ∧
    weight.length = n.val := by
  sorry

/-- **Max-flow / min-cut for the inlined Edmonds–Karp helper.**
    On a symmetric capacity matrix, the value returned by `maxflow_mincut s t`
    is the minimum `s`–`t` cut value (a maximum flow saturates a minimum cut,
    and no flow can exceed any cut). -/
theorem maxflow_mincut_min_cut
    (n : Std.Usize) (cap : Slice Std.Usize) (s t : Std.Usize)
    (value : Std.Usize) (side : alloc.vec.Vec Std.Usize)
    (h_cap : cap.length = n.val * n.val)
    (h_sym : ∀ i j : Fin n.val, toCap cap n.val h_cap i j = toCap cap n.val h_cap j i)
    (h_s : s.val < n.val) (h_t : t.val < n.val) (h_ne : s ≠ t)
    (h : maxflow_mincut n cap s t = ok (value, side)) :
    IsMinCut (toCap cap n.val h_cap) ⟨s.val, h_s⟩ ⟨t.val, h_t⟩ value.val := by
  sorry

/-- **The returned `parent` array is a tree rooted at `0`.** -/
theorem gomory_hu_is_tree
    (n : Std.Usize) (cap : Slice Std.Usize)
    (parent weight : alloc.vec.Vec Std.Usize)
    (h_parent : parent.length = n.val)
    (h_range : ∀ i : Fin n.val, (vecEntry parent n.val h_parent i).val < n.val)
    (h_weight : weight.length = n.val)
    (h_n : 0 < n.val)
    (h : gomory_hu n cap = ok (parent, weight)) :
    isTreeRootedAt (⟨0, h_n⟩ : Fin n.val) (toIdxFun parent n.val h_parent h_range) := by
  sorry

/-- **The Gomory–Hu tree theorem** (Gomory–Hu 1961): for every pair `u ≠ v`,
    the minimum `u`–`v` cut value of `cap` equals the minimum weight on the
    `u`–`v` path of the returned tree. -/
theorem gomory_hu_correct
    (n : Std.Usize) (cap : Slice Std.Usize)
    (parent weight : alloc.vec.Vec Std.Usize)
    (h_cap : cap.length = n.val * n.val)
    (h_sym : ∀ i j : Fin n.val, toCap cap n.val h_cap i j = toCap cap n.val h_cap j i)
    (h_parent : parent.length = n.val)
    (h_range : ∀ i : Fin n.val, (vecEntry parent n.val h_parent i).val < n.val)
    (h_weight : weight.length = n.val)
    (h_n : 0 < n.val)
    (h : gomory_hu n cap = ok (parent, weight)) :
    ∀ u v : Fin n.val, u ≠ v →
      minEdgeOnPath (toIdxFun parent n.val h_parent h_range)
          (toNatFun weight n.val h_weight) n.val u v
        = minCut (toCap cap n.val h_cap) u v := by
  sorry

/-- The same statement in "achieved and minimal" form: the tree path minimum
    *is* a minimum cut value, witnessed by a separating set `S`. -/
theorem gomory_hu_path_min_is_min_cut
    (n : Std.Usize) (cap : Slice Std.Usize)
    (parent weight : alloc.vec.Vec Std.Usize)
    (h_cap : cap.length = n.val * n.val)
    (h_sym : ∀ i j : Fin n.val, toCap cap n.val h_cap i j = toCap cap n.val h_cap j i)
    (h_parent : parent.length = n.val)
    (h_range : ∀ i : Fin n.val, (vecEntry parent n.val h_parent i).val < n.val)
    (h_weight : weight.length = n.val)
    (h_n : 0 < n.val)
    (h : gomory_hu n cap = ok (parent, weight)) :
    ∀ u v : Fin n.val, u ≠ v →
      IsMinCut (toCap cap n.val h_cap) u v
        (minEdgeOnPath (toIdxFun parent n.val h_parent h_range)
          (toNatFun weight n.val h_weight) n.val u v) := by
  sorry

/-- The returned arrays form a Gomory–Hu tree of `cap` rooted at `0`. -/
theorem gomory_hu_is_gomory_hu_tree
    (n : Std.Usize) (cap : Slice Std.Usize)
    (parent weight : alloc.vec.Vec Std.Usize)
    (h_cap : cap.length = n.val * n.val)
    (h_sym : ∀ i j : Fin n.val, toCap cap n.val h_cap i j = toCap cap n.val h_cap j i)
    (h_parent : parent.length = n.val)
    (h_range : ∀ i : Fin n.val, (vecEntry parent n.val h_parent i).val < n.val)
    (h_weight : weight.length = n.val)
    (h_n : 0 < n.val)
    (h : gomory_hu n cap = ok (parent, weight)) :
    isGomoryHuTree (⟨0, h_n⟩ : Fin n.val) (toCap cap n.val h_cap)
      (toIdxFun parent n.val h_parent h_range) (toNatFun weight n.val h_weight) := by
  sorry

end graph_gomory_hu
