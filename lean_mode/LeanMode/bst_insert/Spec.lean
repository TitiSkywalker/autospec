import LeanMode.bst_insert.BstInsert
import Mathlib

open Aeneas Aeneas.Std Result
open bst_insert

set_option linter.unusedVariables false

/-!
# Specification: BST Insert

`bst_insert t v` inserts `v` into the BST rooted at `t` and returns the new root.
`bst_find t v` returns `true` iff `v` is present in the tree.

The correctness is stated in terms of a `view` (the set of values in the tree) and `is_bst` (the BST structural invariant).
-/

mutual
  /-- The set of all values stored in the tree. -/
  def view : Option Node → Set Int
    | none => ∅
    | some n => view_node n
  /-- Helper: view of a Node (not wrapped in Option). -/
  def view_node : Node → Set Int
    | Node.mk v l r => view l ∪ view r ∪ {v}
end

mutual
  /-- The BST structural invariant:
      For every node, all values in the left subtree are strictly less,
      and all values in the right subtree are strictly greater. -/
  def is_bst : Option Node → Prop
    | none => True
    | some n => is_bst_node n
  /-- Helper: is_bst for a Node (not wrapped in Option). -/
  def is_bst_node : Node → Prop
    | Node.mk v l r =>
      (∀ x ∈ view l, x < v) ∧
      is_bst l ∧
      (∀ x ∈ view r, x > v) ∧
      is_bst r
end

/-- Correctness of `bst_insert`:
    If the input tree is a valid BST, then after insertion:
    1. The result is still a valid BST.
    2. The set of values in the result equals the original set plus the
       inserted value (`∪` handles duplicates naturally). -/
theorem bst_insert_correct (t : Option Node) (v : Int) (t' : Option Node)
    (h_pre : is_bst t)
    (h_insert : bst_insert t v = ok t') :
    is_bst t' ∧ view t' = view t ∪ {v} := by
  sorry
