import LeanMode.ds_treap.DsTreap
import Mathlib

open Aeneas Aeneas.Std Result
open ds_treap

set_option linter.unusedVariables false

/-!
# Specification: Treap

A treap is a binary search tree on `key` whose `priority` values form a
max-heap.  `treap_merge` / `treap_split` are the structural primitives, and
`treap_insert` / `treap_delete` are expressed in terms of them.  Correctness is
stated via `view` (the key set), `is_bst` (search-tree order) and `is_heap`
(the heap invariant).
-/

/-- The set of keys stored in the tree. -/
def view : Option Node → Set Int
  | none => ∅
  | some (Node.mk k _ l r) => view l ∪ view r ∪ {k}

/-- The set of priorities stored in the tree. -/
def priorities : Option Node → Set Int
  | none => ∅
  | some (Node.mk _ p l r) => priorities l ∪ priorities r ∪ {p}

/-- The search-tree invariant: every key in the left subtree is smaller and
    every key in the right subtree is larger. -/
def is_bst : Option Node → Prop
  | none => True
  | some (Node.mk k _ l r) =>
      (∀ x ∈ view l, x < k) ∧ is_bst l ∧
      (∀ x ∈ view r, x > k) ∧ is_bst r

/-- The heap invariant: every node's priority is at least every priority in
    its subtrees. -/
def is_heap : Option Node → Prop
  | none => True
  | some (Node.mk _ p l r) =>
      (∀ q ∈ priorities l, q ≤ p) ∧ is_heap l ∧
      (∀ q ∈ priorities r, q ≤ p) ∧ is_heap r

/-- A valid treap: search-tree ordered and heap ordered. -/
def is_treap (t : Option Node) : Prop := is_bst t ∧ is_heap t

/-- Correctness of `treap_merge`: merging two treaps whose key ranges are
    disjoint (everything in `a` below everything in `b`) yields a treap with
    the union of the two key sets. -/
theorem treap_merge_correct (a b : Option Node) (t' : Option Node)
    (h_a : is_treap a) (h_b : is_treap b)
    (h_order : ∀ x ∈ view a, ∀ y ∈ view b, x < y)
    (h_merge : treap_merge a b = ok t') :
    is_treap t' ∧ view t' = view a ∪ view b := by
  sorry

/-- Correctness of `treap_split`: the left result holds exactly the keys
    below `key` and the right result the keys at or above `key`; both are
    valid treaps. -/
theorem treap_split_correct (t : Option Node) (key : Int) (l r : Option Node)
    (h_t : is_treap t)
    (h_split : treap_split t key = ok (l, r)) :
    is_treap l ∧ is_treap r ∧
    view l = {x : Int | x ∈ view t ∧ x < key} ∧
    view r = {x : Int | x ∈ view t ∧ key ≤ x} := by
  sorry

/-- Correctness of `treap_insert`: inserting `(key, priority)` into a valid
    treap keeps it valid and adds exactly that key (`∪` absorbs a duplicate). -/
theorem treap_insert_correct (t : Option Node) (key priority : Int) (t' : Option Node)
    (h_t : is_treap t)
    (h_insert : treap_insert t key priority = ok t') :
    is_treap t' ∧ view t' = view t ∪ {key} := by
  sorry

/-- Correctness of `treap_delete`: deleting `key` from a valid treap keeps it
    valid and removes exactly that key. -/
theorem treap_delete_correct (t : Option Node) (key : Int) (t' : Option Node)
    (h_t : is_treap t)
    (h_delete : treap_delete t key = ok t') :
    is_treap t' ∧ view t' = view t \ {key} := by
  sorry

/-- Correctness of `treap_find`: membership coincides with the key set. -/
theorem treap_find_correct (t : Option Node) (key : Int) (b : Bool)
    (h_t : is_treap t)
    (h_find : treap_find t key = ok b) :
    (b = true ↔ key ∈ view t) := by
  sorry

/-- All priorities in the tree are pairwise distinct. -/
def distinct_priorities : Option Node → Prop
  | none => True
  | some (Node.mk _ p l r) =>
      (∀ q ∈ priorities l, q ≠ p) ∧ (∀ q ∈ priorities r, q ≠ p) ∧
      distinct_priorities l ∧ distinct_priorities r

/-- Roundtrip: splitting at `key` and merging the halves recovers the original
    tree exactly (structural equality, not merely the same `view`), provided the
    priorities are pairwise distinct. -/
theorem treap_split_merge_roundtrip (t : Option Node) (key : Int)
    (h_pre : is_treap t) (h_distinct : distinct_priorities t) :
    ∃ l r : Option Node, treap_split t key = ok (l, r) ∧ treap_merge l r = ok t := by
  sorry
