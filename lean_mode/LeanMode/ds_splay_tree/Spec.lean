import LeanMode.ds_splay_tree.DsSplayTree
import Mathlib

open Aeneas Aeneas.Std Result
open ds_splay_tree

set_option linter.unusedVariables false

/-!
# Specification: Splay Tree

A splay tree is a self-adjusting BST: every access restructures the tree by
rotations so that the accessed key becomes the root.  Unlike the local
rebalancing of an AVL or red-black tree the invariant is *amortized* — the
tree is not kept balanced by any pointwise condition, only the access
*sequence* is cheap.

What is stated here is the structural half of the algorithm:

* `splay` preserves the search-tree order and the key set, and moves the
  accessed key to the root;
* `splay_insert` adds exactly the inserted key (and splays it);
* `splay_find` / `splay_contains` report membership and, when the key is
  found, splay it to the root;
* splaying a key that is already at the root is the identity — the fixed
  point that makes the zig / zig-zig / zig-zag restructuring well defined.
-/

/-- The set of keys stored in the tree. -/
def view : Option Node → Set Int
  | none => ∅
  | some (Node.mk k l r) => view l ∪ view r ∪ {k}

/-- The search-tree invariant: every key of the left subtree is smaller than
    the node key, every key of the right subtree is larger, and both subtrees
    are search trees themselves. -/
def is_bst : Option Node → Prop
  | none => True
  | some (Node.mk k l r) =>
      (∀ x, x ∈ view l → x < k) ∧ is_bst l ∧
      (∀ x, x ∈ view r → k < x) ∧ is_bst r

/-- The key stored at the root (`none` for the empty tree). -/
def rootKey : Option Node → Option Int
  | none => none
  | some n => some n.key

/-- `splay` keeps the tree a BST with the same key set, and the accessed key
    (which must be present) ends up at the root. -/
theorem splay_preserves_bst (t t' : Option Node) (key : Int)
    (h_pre : is_bst t)
    (h_mem : key ∈ view t)
    (h_splay : splay t key = ok t') :
    is_bst t' ∧ view t' = view t ∧ rootKey t' = some key := by
  sorry

/-- Splaying a key that already sits at the root leaves the tree unchanged —
    the structural fixed point of the access operation. -/
theorem splay_root_is_fixpoint (t t' : Option Node) (key : Int)
    (h_pre : is_bst t)
    (h_mem : key ∈ view t)
    (h_splay : splay t key = ok t') :
    rootKey t' = some key ∧ splay t' key = ok t' := by
  sorry

/-- `splay_insert` adds exactly the inserted key (a duplicate leaves the key
    set unchanged) and splays it to the root, keeping the tree a BST. -/
theorem splay_insert_correct (t t' : Option Node) (key : Int)
    (h_pre : is_bst t)
    (h_insert : splay_insert t key = ok t') :
    is_bst t' ∧ view t' = view t ∪ {key} ∧ rootKey t' = some key := by
  sorry

/-- `splay_contains` decides membership in the key set. -/
theorem splay_contains_correct (t : Option Node) (key : Int) (b : Bool)
    (h_pre : is_bst t)
    (h_contains : splay_contains t key = ok b) :
    (b = true ↔ key ∈ view t) := by
  sorry

/-- `splay_find` reports whether the key is present; when it is, the tree has
    the same key set with that key splayed to the root, and when it is not,
    the tree is returned unchanged. -/
theorem splay_find_correct (t t' : Option Node) (key : Int) (b : Bool)
    (h_pre : is_bst t)
    (h_find : splay_find t key = ok (t', b)) :
    (b = true ↔ key ∈ view t) ∧
    (b = true → view t' = view t ∧ is_bst t' ∧ rootKey t' = some key) ∧
    (b = false → t' = t) := by
  sorry
