import LeanMode.ds_red_black_tree.DsRedBlackTree
import Mathlib

open Aeneas Aeneas.Std Result
open ds_red_black_tree

set_option linter.unusedVariables false

/-!
# Specification: Red-Black Tree

`rbt_insert` / `rbt_delete` operate on a persistent red-black tree.  Correctness
is stated in terms of `view` (the set of stored keys), `is_bst` (the search-tree
ordering) and the red-black color invariants: the root is black, no red node has
a red child, every root-to-leaf path has the same number of black nodes, and no
transient double-black state survives.
-/

/-- The set of keys stored in the tree. -/
def view : Tree → Set Int
  | Tree.Empty => ∅
  | Tree.DoubleBlackEmpty => ∅
  | Tree.Node _ l k r => view l ∪ view r ∪ {k}

/-- The color of a tree's root (`Empty` is the black NIL leaf). -/
def root_color : Tree → Color
  | Tree.Empty => Color.Black
  | Tree.DoubleBlackEmpty => Color.DoubleBlack
  | Tree.Node c _ _ _ => c

/-- The search-tree invariant: every key in the left subtree is smaller and
    every key in the right subtree is larger. -/
def is_bst : Tree → Prop
  | Tree.Empty => True
  | Tree.DoubleBlackEmpty => True
  | Tree.Node _ l k r =>
      (∀ x ∈ view l, x < k) ∧ is_bst l ∧
      (∀ x ∈ view r, x > k) ∧ is_bst r

/-- No red node has a red child. -/
def no_red_red : Tree → Prop
  | Tree.Empty => True
  | Tree.DoubleBlackEmpty => False
  | Tree.Node c l _ r =>
      (c = Color.Red → root_color l = Color.Black ∧ root_color r = Color.Black) ∧
      no_red_red l ∧ no_red_red r

/-- No transient double-black state remains in a finished tree. -/
def no_db : Tree → Prop
  | Tree.Empty => True
  | Tree.DoubleBlackEmpty => False
  | Tree.Node c l _ r => c ≠ Color.DoubleBlack ∧ no_db l ∧ no_db r

/-- The black height along the left spine (for a balanced tree this is the
    common black height of every root-to-leaf path). -/
def black_height : Tree → Nat
  | Tree.Empty => 0
  | Tree.DoubleBlackEmpty => 0
  | Tree.Node c l _ _ =>
      black_height l + (match c with | Color.Black => 1 | _ => 0)

/-- Every node's left and right subtrees have equal black height. -/
def bh_balanced : Tree → Prop
  | Tree.Empty => True
  | Tree.DoubleBlackEmpty => False
  | Tree.Node _ l _ r => black_height l = black_height r ∧ bh_balanced l ∧ bh_balanced r

/-- A valid red-black tree: search-tree ordered, root black, no red-red,
    no double-black, and black-height balanced. -/
def is_rbtree (t : Tree) : Prop :=
  is_bst t ∧ no_red_red t ∧ no_db t ∧ bh_balanced t ∧ root_color t = Color.Black

/-- Correctness of `rbt_insert`: if the input is a valid red-black tree, the
    result is still valid and stores exactly the old keys plus the inserted
    key (`∪` absorbs an existing duplicate). -/
theorem rbt_insert_correct (t : Tree) (v : Int) (t' : Tree)
    (h_pre : is_rbtree t)
    (h_insert : rbt_insert t v = ok t') :
    is_rbtree t' ∧ view t' = view t ∪ {v} := by
  sorry

/-- Correctness of `rbt_delete`: if the input is a valid red-black tree, the
    result is still valid and stores exactly the old keys minus the deleted
    key. -/
theorem rbt_delete_correct (t : Tree) (v : Int) (t' : Tree)
    (h_pre : is_rbtree t)
    (h_delete : rbt_delete t v = ok t') :
    is_rbtree t' ∧ view t' = view t \ {v} := by
  sorry

/-- Correctness of `rbt_find`: membership in the tree coincides with
    membership in the set of stored keys. -/
theorem rbt_find_correct (t : Tree) (v : Int) (b : Bool)
    (h_pre : is_rbtree t)
    (h_find : rbt_find t v = ok b) :
    (b = true ↔ v ∈ view t) := by
  sorry

/-- The number of keys stored in the tree. -/
def size : Tree → Nat
  | Tree.Empty => 0
  | Tree.DoubleBlackEmpty => 0
  | Tree.Node _ l _ r => size l + size r + 1

/-- The (computed) height of the tree. -/
def tree_height : Tree → Nat
  | Tree.Empty => 0
  | Tree.DoubleBlackEmpty => 0
  | Tree.Node _ l _ r => 1 + max (tree_height l) (tree_height r)

/-- Height ≤ 2·black-height + 1: with no red-red, every root-to-leaf path has
    at least half its length black. -/
theorem rbt_height_le_2bh (t : Tree) (h_pre : is_rbtree t) :
    tree_height t ≤ 2 * black_height t + 1 := by
  sorry

/-- Size ≥ 2^black-height − 1: black-height balance forces at least a perfect
    black tree of that height. -/
theorem rbt_size_ge_2bh (t : Tree) (h_pre : is_rbtree t) :
    2 ^ black_height t ≤ size t + 1 := by
  sorry

/-- Combined height bound: height ≤ 2·log₂(size+1) + 2. -/
theorem rbt_height_log_bound (t : Tree) (h_pre : is_rbtree t) :
    tree_height t ≤ 2 * Nat.log2 (size t + 1) + 2 := by
  sorry
