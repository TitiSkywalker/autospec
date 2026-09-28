import LeanMode.ds_union_find.DsUnionFind
import Mathlib

open Aeneas Aeneas.Std Result
open ds_union_find

set_option linter.unusedVariables false

/-!
# Specification: Disjoint-Set Union (union-find)

`dsu_new` / `find` / `union` / `connected` maintain a partition of `0 .. n`.
Correctness is stated via `root` (follow parent pointers to the representative)
and `connected_spec` (two elements share a root).
-/

/-- The parent array as a `List Nat`. -/
def parentList (d : DSU) : List Nat :=
  (alloc.vec.Vec.deref d.parent : List Std.Usize).map (·.val)

/-- Iterate parent pointers up to `fuel` steps; `x` if stuck. -/
def rootIter (p : List Nat) (x : Nat) (fuel : Nat) : Nat :=
  match fuel with
  | 0 => x
  | k + 1 =>
    let px := p.getD x x
    if px = x then x else rootIter p px k

/-- The representative (root) of `x` in `d`. -/
def root (d : DSU) (x : Nat) : Nat :=
  rootIter (parentList d) x ((parentList d).length + 1)

/-- `a` and `b` are connected iff they share a root. -/
def connected_spec (d : DSU) (a b : Nat) : Prop :=
  root d a = root d b


/-! ## Specification theorems -/

/-- `find` returns a representative: `y` is its own parent. -/
theorem find_root (d : DSU) (x : Std.Usize) (r : DSU) (y : Std.Usize)
    (h_lt : x.val < (parentList d).length)
    (h : find d x = ok (r, y)) :
    (parentList r).getD y.val y.val = y.val := by
  sorry

/-- `find` returns the root of `x`'s tree. -/
theorem find_representative (d : DSU) (x : Std.Usize) (r : DSU) (y : Std.Usize)
    (h_lt : x.val < (parentList d).length)
    (h : find d x = ok (r, y)) :
    root r x.val = y.val := by
  sorry

/-- `connected` is sound w.r.t. `connected_spec`. -/
theorem connected_sound (d : DSU) (a b : Std.Usize) (res : Bool)
    (h_lt_a : a.val < (parentList d).length)
    (h_lt_b : b.val < (parentList d).length)
    (h : connected d a b = ok res) :
    (res = true) ↔ connected_spec d a.val b.val := by
  sorry

/-- A true `union` merges two distinct sets. -/
theorem union_true (d : DSU) (a b : Std.Usize) (r : DSU)
    (h_lt_a : a.val < (parentList d).length)
    (h_lt_b : b.val < (parentList d).length)
    (h : union d a b = ok (r, true)) :
    root d a.val ≠ root d b.val ∧ connected_spec r a.val b.val := by
  sorry

/-- A false `union` means the two elements were already connected. -/
theorem union_false (d : DSU) (a b : Std.Usize) (r : DSU)
    (h_lt_a : a.val < (parentList d).length)
    (h_lt_b : b.val < (parentList d).length)
    (h : union d a b = ok (r, false)) :
    connected_spec d a.val b.val := by
  sorry

/-- `union` preserves pre-existing connectivity. -/
theorem union_preserves (d : DSU) (a b c e : Std.Usize) (r : DSU) (m : Bool)
    (h_lt_a : a.val < (parentList d).length)
    (h_lt_b : b.val < (parentList d).length)
    (h_lt_c : c.val < (parentList d).length)
    (h_lt_e : e.val < (parentList d).length)
    (h : union d a b = ok (r, m))
    (h_conn : connected_spec d c.val e.val) :
    connected_spec r c.val e.val := by
  sorry
