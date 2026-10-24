import LeanMode.ds_kdtree.DsKdtree
import Mathlib

open Aeneas Aeneas.Std Result
open ds_kdtree

set_option linter.unusedVariables false

/-!
# Specification: 2-D KD-Tree

A KD-tree stores integer points `(x, y)` with an integer value `val`; every
node carries the axis-aligned bounding box of its subtree and the subtree value
sum.  `kd_query` answers a rectangle sum query by pruning subtrees whose
bounding box is disjoint from the rectangle.  Correctness is stated via
`coords` (the point set), `bbox_ok` (the bounding-box / sum invariant) and
`rectSum` (the direct brute-force rectangle sum).
-/

/-- The set of `(x, y)` coordinates stored in the tree. -/
def coords : Option Node → Set (Int × Int)
  | none => ∅
  | some (Node.mk x y _ l r _ _ _ _ _) => coords l ∪ coords r ∪ {(x, y)}

/-- The (computed) sum of all stored values. -/
def treeSum : Option Node → Int
  | none => 0
  | some (Node.mk _ _ v l r _ _ _ _ _) => v + treeSum l + treeSum r

/-- The bounding-box / sum invariant: every node's box contains its own point
    and all points in its subtrees, and its stored `sum` equals the subtree
    value sum. -/
def bbox_ok : Option Node → Prop
  | none => True
  | some (Node.mk x y v l r minx maxx miny maxy sum) =>
      minx ≤ x ∧ x ≤ maxx ∧ miny ≤ y ∧ y ≤ maxy ∧
      (∀ p ∈ coords l, minx ≤ p.1 ∧ p.1 ≤ maxx ∧ miny ≤ p.2 ∧ p.2 ≤ maxy) ∧
      (∀ p ∈ coords r, minx ≤ p.1 ∧ p.1 ≤ maxx ∧ miny ≤ p.2 ∧ p.2 ≤ maxy) ∧
      sum = v + treeSum l + treeSum r ∧
      bbox_ok l ∧ bbox_ok r

/-- The brute-force rectangle sum over a tree. -/
def rectSum : Option Node → Int → Int → Int → Int → Int
  | none, _, _, _, _ => 0
  | some (Node.mk x y v l r _ _ _ _ _), x1, y1, x2, y2 =>
      (if x1 ≤ x ∧ x ≤ x2 ∧ y1 ≤ y ∧ y ≤ y2 then v else 0) +
      rectSum l x1 y1 x2 y2 + rectSum r x1 y1 x2 y2

/-- `kd_insert` maintains the bounding-box invariant and adds exactly the
    inserted coordinate. -/
theorem kd_insert_bbox_ok (t : Option Node) (x y v : Int) (t' : Option Node)
    (h_pre : bbox_ok t)
    (h_insert : kd_insert t x y v = ok t') :
    bbox_ok t' ∧ coords t' = coords t ∪ {(x, y)} := by
  sorry

/-- `kd_query` returns the direct rectangle sum, under the bounding-box
    invariant (which makes the pruning sound). -/
theorem kd_query_correct (t : Option Node) (x1 y1 x2 y2 s : Int)
    (h_bbox : bbox_ok t)
    (h_q : kd_query t x1 y1 x2 y2 = ok s) :
    s = rectSum t x1 y1 x2 y2 := by
  sorry
