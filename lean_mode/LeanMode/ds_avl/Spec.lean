import LeanMode.ds_avl.DsAvl
import Mathlib

open Aeneas Aeneas.Std Result
open ds_avl

set_option linter.unusedVariables false

/-!
# Specification: AVL Tree

An AVL tree is a binary search tree in which the heights of any node's two
children differ by at most one.  `avl_insert` / `avl_delete` rebalance with
single/double rotations; correctness is stated via `view` (the key set),
`is_bst` (search-tree order) and `is_avl` (the balance invariant).
-/

/-- The set of keys stored in the tree. -/
def view : Option Node → Set Int
  | none => ∅
  | some (Node.mk k _ l r) => view l ∪ view r ∪ {k}

/-- The search-tree invariant: every key in the left subtree is smaller and
    every key in the right subtree is larger. -/
def is_bst : Option Node → Prop
  | none => True
  | some (Node.mk k _ l r) =>
      (∀ x ∈ view l, x < k) ∧ is_bst l ∧
      (∀ x ∈ view r, x > k) ∧ is_bst r

/-- The (computed) height of a tree. -/
def tree_height : Option Node → Nat
  | none => 0
  | some (Node.mk _ _ l r) => 1 + max (tree_height l) (tree_height r)

/-- The AVL invariant: every node's stored `height` equals its computed height
    and the two child heights differ by at most one.  (The stored-height clause
    is essential — `rebalance` reads the stored field, not the computed one.) -/
def is_avl : Option Node → Prop
  | none => True
  | some (Node.mk _ h l r) =>
      h = 1 + max (tree_height l) (tree_height r) ∧
      tree_height l ≤ tree_height r + 1 ∧
      tree_height r ≤ tree_height l + 1 ∧
      is_avl l ∧ is_avl r

/-- A valid AVL tree: search-tree ordered, height balanced, with correct
    stored heights. -/
def is_avl_tree (t : Option Node) : Prop := is_bst t ∧ is_avl t

/-- Correctness of `avl_insert`: inserting `key` into a valid AVL tree keeps
    it valid and adds exactly that key (`∪` absorbs a duplicate). -/
theorem avl_insert_correct (t : Option Node) (key : Int) (t' : Option Node)
    (h_pre : is_avl_tree t)
    (h_insert : avl_insert t key = ok t') :
    is_avl_tree t' ∧ view t' = view t ∪ {key} := by
  sorry

/-- Correctness of `avl_delete`: deleting `key` from a valid AVL tree keeps it
    valid and removes exactly that key. -/
theorem avl_delete_correct (t : Option Node) (key : Int) (t' : Option Node)
    (h_pre : is_avl_tree t)
    (h_delete : avl_delete t key = ok t') :
    is_avl_tree t' ∧ view t' = view t \ {key} := by
  sorry

/-- Correctness of `avl_find`: membership coincides with the key set. -/
theorem avl_find_correct (t : Option Node) (key : Int) (b : Bool)
    (h_pre : is_avl_tree t)
    (h_find : avl_find t key = ok b) :
    (b = true ↔ key ∈ view t) := by
  sorry

/-- The number of keys stored in the tree. -/
def size : Option Node → Nat
  | none => 0
  | some (Node.mk _ _ l r) => size l + size r + 1

/-- Height bound: an AVL tree of height `h` has at least `2 ^ (h / 2)` keys
    (counting the empty tree's implicit slot) — the balance invariant forces
    exponential growth every two levels. -/
theorem avl_height_log_bound (t : Option Node) (h_pre : is_avl t) :
    2 ^ ((tree_height t) / 2) ≤ size t + 1 := by
  sorry
