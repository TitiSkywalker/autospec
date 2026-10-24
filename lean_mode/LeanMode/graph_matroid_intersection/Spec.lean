import LeanMode.graph_matroid_intersection.GraphMatroidIntersection
import Mathlib

open Aeneas Aeneas.Std Result
open graph_matroid_intersection

set_option linter.unusedVariables false

/-!
# Specification: Maximum Common Independent Set of Two Matroids

The algorithm `matroid_intersection nv m from to colors k` treats the `m`
directed edges of a graph on `nv` vertices as the ground set `E = Fin m`.
`from e` / `to e` are the endpoints of edge `e` and `colors e` is its colour.

Two matroids live on `E`:

* the **graphic matroid** `M₁`, where a set is independent iff it is acyclic
  (a forest) in the graph `(Fin nv, E)`;
* the **partition matroid** `M₂`, where a set is independent iff each colour is
  used at most `k` times.

The program returns a flat 0/1 indicator of length `m`; the theorems state that
its support is a *maximum* common independent set, and that its cardinality
satisfies Edmonds' 1970 **matroid intersection theorem**

`max{|I| : I ∈ I₁ ∩ I₂} = min_{X ⊆ E} (rank₁ X + rank₂ (E \ X))`.

The development below is entirely inside the crate's namespace so that its
helper names cannot collide with Mathlib's.
-/

namespace graph_matroid_intersection

open Classical

noncomputable section

/-! ## The graphic matroid (acyclicity) -/

/-- Undirected incidence: edge `e` joins the two vertices `u` and `v`
    (either order); in particular a self-loop joins a vertex to itself. -/
def EdgeConnects {m : Nat} (frm dst : Fin m → Nat) (e : Fin m) (u v : Nat) : Prop :=
  (frm e = u ∧ dst e = v) ∨ (frm e = v ∧ dst e = u)

/-- `Trail frm dst E u es w` says that `es` is a list of pairwise-distinct
    edges of `E` forming a walk from `u` to `w`, each consecutive pair of
    vertices being joined by the corresponding edge (as an undirected edge). -/
inductive Trail {m : Nat} (frm dst : Fin m → Nat) (E : Finset (Fin m)) :
    Nat → List (Fin m) → Nat → Prop
  | nil (u : Nat) : Trail frm dst E u [] u
  | cons (u v w : Nat) (e : Fin m) (es : List (Fin m))
      (he : e ∈ E) (hconn : EdgeConnects frm dst e u v)
      (hnot : e ∉ es) (htail : Trail frm dst E v es w) :
      Trail frm dst E u (e :: es) w

/-- A set `E` of edges is **acyclic** (a forest) iff it contains no cycle,
    i.e. no closed trail of pairwise-distinct edges of positive length.
    Self-loops (length-1 trails) and parallel edges (length-2 trails) count as
    cycles, as they must. -/
def Acyclic {m : Nat} (frm dst : Fin m → Nat) (E : Finset (Fin m)) : Prop :=
  ¬ ∃ (u : Nat) (es : List (Fin m)), es ≠ [] ∧ Trail frm dst E u es u

/-! ## The partition matroid (colour feasibility) -/

/-- `E` is independent in the partition matroid iff every colour class contains
    at most `k` elements of `E`. -/
def ColorIndependent {m : Nat} (colors : Fin m → Nat) (k : Nat)
    (E : Finset (Fin m)) : Prop :=
  ∀ c : Nat, (E.filter (fun e => colors e = c)).card ≤ k

/-- A set common to both matroids. -/
def CommonIndependent {m : Nat} (frm dst colors : Fin m → Nat) (k : Nat)
    (E : Finset (Fin m)) : Prop :=
  Acyclic frm dst E ∧ ColorIndependent colors k E

/-! ## Rank functions and the intersection optimum -/

/-- `rank₁ X`: the graphic matroid rank of `X`, i.e. the maximum size of an
    acyclic subset of `X`. -/
noncomputable def rank1 {m : Nat} (frm dst : Fin m → Nat) (X : Finset (Fin m)) : Nat :=
  (X.powerset.filter (fun E => Acyclic frm dst E)).sup (fun E => E.card)

/-- `rank₂ X`: the partition matroid rank of `X`, i.e. the maximum size of a
    colour-feasible subset of `X`. -/
noncomputable def rank2 {m : Nat} (colors : Fin m → Nat) (k : Nat)
    (X : Finset (Fin m)) : Nat :=
  (X.powerset.filter (fun E => ColorIndependent colors k E)).sup (fun E => E.card)

/-- The maximum cardinality of a common independent set. -/
noncomputable def maxCommonIndependentSize {m : Nat} (frm dst colors : Fin m → Nat)
    (k : Nat) : Nat :=
  ((Finset.univ : Finset (Fin m)).powerset.filter
      (fun E => CommonIndependent frm dst colors k E)).sup (fun E => E.card)

/-! ## Reading the program's output -/

/-- View a flat `Slice` of length `n` as `Fin n → Nat`. -/
def toFn {n : Nat} (s : Slice Std.Usize) (h : s.length = n) : Fin n → Nat :=
  fun i =>
    have hb : i.val < s.length := by rw [h]; exact i.is_lt
    (s[i.val]'hb).val

/-- View a flat `Vec` of length `n` as `Fin n → Nat`. -/
def vecToFn {n : Nat} (v : alloc.vec.Vec Std.Usize) (h : v.length = n) : Fin n → Nat :=
  fun i =>
    have hb : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'hb).val

/-- The support of the returned 0/1 indicator: the set of selected edges. -/
def support {n : Nat} (v : alloc.vec.Vec Std.Usize) (h : v.length = n) : Finset (Fin n) :=
  Finset.univ.filter (fun e => vecToFn v h e ≠ 0)

/-! ## Specification theorems -/

/-- Soundness: the support of the returned indicator is common independent in
    the graphic and partition matroids. -/
theorem matroid_intersection_is_common_independent
    (nv m k : Std.Usize) (frm dst cols : Slice Std.Usize)
    (sel : alloc.vec.Vec Std.Usize)
    (h_frm : frm.length = m.val) (h_dst : dst.length = m.val)
    (h_cols : cols.length = m.val) (h_sel : sel.length = m.val)
    (h : matroid_intersection nv m frm dst cols k = ok sel) :
    CommonIndependent (toFn frm h_frm) (toFn dst h_dst) (toFn cols h_cols) k.val
      (support sel h_sel) := by
  sorry

/-- Optimality: the support of the returned indicator is a **maximum** common
    independent set (and its size equals `maxCommonIndependentSize`). -/
theorem matroid_intersection_max
    (nv m k : Std.Usize) (frm dst cols : Slice Std.Usize)
    (sel : alloc.vec.Vec Std.Usize)
    (h_frm : frm.length = m.val) (h_dst : dst.length = m.val)
    (h_cols : cols.length = m.val) (h_sel : sel.length = m.val)
    (h : matroid_intersection nv m frm dst cols k = ok sel) :
    CommonIndependent (toFn frm h_frm) (toFn dst h_dst) (toFn cols h_cols) k.val
        (support sel h_sel) ∧
    (∀ J : Finset (Fin m.val),
        CommonIndependent (toFn frm h_frm) (toFn dst h_dst) (toFn cols h_cols) k.val J →
        J.card ≤ (support sel h_sel).card) ∧
    (support sel h_sel).card =
      maxCommonIndependentSize (toFn frm h_frm) (toFn dst h_dst) (toFn cols h_cols) k.val := by
  sorry

/-- **Edmonds' matroid intersection theorem** (1970) specialised to
    graphic ∩ partition: the program's output size `s` is a lower bound for
    `rank₁ X + rank₂ (E \ X)` over *every* `X ⊆ E` (the min–max inequality
    direction), and some `X` attains equality (tightness).  Together these say
    `s = min_{X ⊆ E} (rank₁ X + rank₂ (E \ X))`. -/
theorem matroid_intersection_theorem
    (nv m k : Std.Usize) (frm dst cols : Slice Std.Usize)
    (sel : alloc.vec.Vec Std.Usize)
    (h_frm : frm.length = m.val) (h_dst : dst.length = m.val)
    (h_cols : cols.length = m.val) (h_sel : sel.length = m.val)
    (h : matroid_intersection nv m frm dst cols k = ok sel) :
    (∀ X : Finset (Fin m.val),
        (support sel h_sel).card ≤
          rank1 (toFn frm h_frm) (toFn dst h_dst) X +
          rank2 (toFn cols h_cols) k.val ((Finset.univ : Finset (Fin m.val)) \ X)) ∧
    (∃ X : Finset (Fin m.val),
        (support sel h_sel).card =
          rank1 (toFn frm h_frm) (toFn dst h_dst) X +
          rank2 (toFn cols h_cols) k.val ((Finset.univ : Finset (Fin m.val)) \ X)) := by
  sorry

end

end graph_matroid_intersection
