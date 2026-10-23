import LeanMode.ds_van_emde_boas.DsVanEmdeBoas
import Mathlib

open Aeneas Aeneas.Std Result
open ds_van_emde_boas

set_option linter.unusedVariables false

/-!
# Specification: van Emde Boas tree

A vEB tree stores a set of `Nat`s drawn from a universe `u = 2^(2^m)`.  A node
of universe `u ≥ 4` splits its universe into `√u` *clusters*, each a vEB tree
over the low `√u` values, plus a `summary` — itself a vEB tree over the `√u`
cluster indices — that remembers exactly the non-empty clusters.

The specification mirrors the implementation's conventions exactly:

* the value set `viewC v` is built from the node minimum, the `u = 2` leaf
  maximum, and the (shifted) cluster elements;
* the node minimum is *not* stored recursively in the clusters
  (`min ∉ clView v.clusters 0 √u`), while the maximum is (for `u ≥ 4`);
* `summary` records precisely the non-empty clusters
  (`viewC summary = nonemptyIdx v.clusters 0`);
* `veb_successor` returns the pair `(found, value)`, and the sentinel
  `value = 0` whenever `found = false`.

The structural invariant `isVeb` collects the universe well-formedness, the
min/max correctness, the "min is not in the clusters" convention and the
summary agreement; every correctness theorem carries it.
-/

namespace DsVeb

/-- Shift a set of cluster-local indices into global coordinates. -/
def shiftSet (s : Set Nat) (d : Nat) : Set Nat := (fun x => x + d) '' s

/-- `n` is a valid vEB universe: `n = 2^(2^m)`, i.e. `n ∈ {2, 4, 16, 256, …}`. -/
def IsUniv (n : Nat) : Prop := ∃ m, n = 2 ^ (2 ^ m)

mutual
  /-- The element values stored in the cluster list `c`, whose clusters are
      numbered `i, i+1, …`: cluster `j` contributes `j * h + y` for each of its
      elements `y`. -/
  def clView : Option Clusters → Nat → Nat → Set Nat
    | none, _, _ => ∅
    | some (Clusters.mk head tail), i, h =>
        shiftSet (viewC head) (i * h) ∪ clView tail (i + 1) h

  /-- The element values stored in a node.  The node `min` is always present
      (when the node is non-empty); the node `max` is present explicitly only at
      the `u = 2` base — for `u ≥ 4` it lives inside its cluster. -/
  def viewC : Veb → Set Nat
    | Veb.mk u half emp mn mx _ cl =>
        if emp then ∅
        else {mn.val} ∪ (if u.val = 2 then {mx.val} else ∅) ∪ clView cl 0 half.val
end

/-- Length of the cluster list. -/
def clLen : Option Clusters → Nat
  | none => 0
  | some (Clusters.mk _ tail) => 1 + clLen tail

/-- The indices of the non-empty clusters of `c`. -/
def nonemptyIdx : Option Clusters → Nat → Set Nat
  | none, _ => ∅
  | some (Clusters.mk head tail), i =>
      (if head.empty then ∅ else {i}) ∪ nonemptyIdx tail (i + 1)

mutual
  /-- Every cluster of `c` has universe `cu` and is itself a valid vEB tree. -/
  def allClusters : Option Clusters → Nat → Prop
    | none, _ => True
    | some (Clusters.mk head tail), cu =>
        head.u.val = cu ∧ isVeb head ∧ allClusters tail cu
  termination_by c cu => sizeOf c

  /-- The vEB structural invariant.  Besides the universe well-formedness it
      records: every stored value is `< u`; a non-empty node's `min`/`max` are
      the least/greatest stored values; the node minimum is not duplicated in
      the clusters; the `u = 2` base has no summary/clusters; and, for `u ≥ 4`,
      the node has exactly `√u` clusters of universe `√u`, all valid, with the
      summary being a valid vEB over `√u` whose element set is exactly the set
      of non-empty cluster indices. -/
  def isVeb : Veb → Prop
    | v@(Veb.mk u half emp mn mx sum cl) =>
        IsUniv u.val ∧
        (∀ y, y ∈ viewC v → y < u.val) ∧
        (emp = true ∨ (IsLeast (viewC v) mn.val ∧ IsGreatest (viewC v) mx.val)) ∧
        mn.val ∉ clView cl 0 half.val ∧
        (u.val = 2 → half.val = 1 ∧ cl = none ∧ sum = none) ∧
        (u.val ≠ 2 →
          half.val * half.val = u.val ∧ IsUniv half.val ∧
          clLen cl = half.val ∧ allClusters cl half.val ∧
          match sum with
          | none => False
          | some s => s.u.val = half.val ∧ isVeb s ∧ viewC s = nonemptyIdx cl 0)
  termination_by v => sizeOf v
end

end DsVeb

open DsVeb

/-- `veb_new u` builds the empty vEB tree over a valid universe `u`; it is a
    valid vEB tree and stores nothing. -/
theorem veb_new_correct (u : Std.Usize) (v : Veb)
    (h_u : IsUniv u.val)
    (h : veb_new u = ok v) :
    isVeb v ∧ viewC v = ∅ := by
  sorry

/-- `veb_member v x` decides membership in `viewC v`. -/
theorem veb_member_correct (v : Veb) (x : Std.Usize) (b : Bool)
    (h_pre : isVeb v)
    (h : veb_member v x = ok b) :
    (b = true ↔ x.val ∈ viewC v) := by
  sorry

/-- `veb_min v` returns the least stored value (the pair's flag is `true`), or
    reports that the tree is empty (flag `false`, empty value set, and the
    sentinel `0` in the value component). -/
theorem veb_min_correct (v : Veb) (b : Bool) (k : Std.Usize)
    (h_pre : isVeb v)
    (h : veb_min v = ok (b, k)) :
    (b = true ∧ IsLeast (viewC v) k.val) ∨ (b = false ∧ viewC v = ∅ ∧ k.val = 0) := by
  sorry

/-- `veb_max v` returns the greatest stored value (flag `true`), or reports that
    the tree is empty (flag `false`, empty value set, sentinel `0`). -/
theorem veb_max_correct (v : Veb) (b : Bool) (k : Std.Usize)
    (h_pre : isVeb v)
    (h : veb_max v = ok (b, k)) :
    (b = true ∧ IsGreatest (viewC v) k.val) ∨ (b = false ∧ viewC v = ∅ ∧ k.val = 0) := by
  sorry

/-- **Successor.**  `veb_successor v x` returns `(true, k)` with `k` the least
    stored value strictly greater than `x`, or `(false, 0)` — the documented
    sentinel `0` — when no stored value exceeds `x`. -/
theorem veb_successor_correct (v : Veb) (x : Std.Usize) (b : Bool) (k : Std.Usize)
    (h_pre : isVeb v)
    (h : veb_successor v x = ok (b, k)) :
    (b = true ↔ ∃ y, y ∈ viewC v ∧ x.val < y) ∧
    (b = true → k.val ∈ viewC v ∧ x.val < k.val ∧
                 ∀ y, y ∈ viewC v → x.val < y → k.val ≤ y) ∧
    (b = false → k.val = 0) := by
  sorry

/-- **Insert.**  Inserting an in-universe value `x` keeps the tree valid and
    adds exactly `x` to the stored set (`∪` absorbs an already-present value). -/
theorem veb_insert_correct (v v' : Veb) (x : Std.Usize)
    (h_pre : isVeb v)
    (h_lt : x.val < v.u.val)
    (h : veb_insert v x = ok v') :
    isVeb v' ∧ viewC v' = viewC v ∪ {x.val} := by
  sorry

/-- **Delete.**  Deleting `x` keeps the tree valid and removes exactly `x` from
    the stored set (deleting an absent value is a no-op). -/
theorem veb_delete_correct (v v' : Veb) (x : Std.Usize)
    (h_pre : isVeb v)
    (h : veb_delete v x = ok v') :
    isVeb v' ∧ viewC v' = viewC v \ {x.val} := by
  sorry

/-- The summary really is the indicator of the non-empty clusters — the
    recursive-global bookkeeping that makes the `√u` decomposition work.  (This
    is the `u ≠ 2` conjunct of `isVeb` itself, recorded here only as a named
    lemma; it is not an independent claim.) -/
theorem isVeb_summary (v s : Veb)
    (h_pre : isVeb v)
    (h_s : v.summary = some s) :
    viewC s = nonemptyIdx v.clusters 0 := by
  sorry
