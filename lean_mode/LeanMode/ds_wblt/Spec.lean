import LeanMode.ds_wblt.DsWblt
import Mathlib

open Aeneas Aeneas.Std Result
open ds_wblt

set_option linter.unusedVariables false

/-!
# Specification: Persistent Weight-Balanced Leaf Tree (WBLT)

`wblt_insert` / `wblt_delete` build a persistent, self-balancing binary search
tree where keys live only in the leaves.  Each internal node stores the maximum
key of its subtree and the subtree leaf count (`size`); balance is maintained by
the *weight* (subtree leaf count) via single/double rotations so that every
internal node satisfies `3·min(lsz, rsz) ≥ max(lsz, rsz)` (the α = 1/4 case of
α-balance).

The deep insight is the **weight-balance** invariant: the proof that updates
preserve `is_wblt` needs the global *weight* (leaf-count) construction, not just
the local control flow of a rotation.
-/

/-- The actual number of leaves in a (sub)tree. -/
def leaf_count : Option Node → Nat
  | none => 0
  | some (Node.mk _ _ l r) =>
      (if l = none then 1 else 0) + leaf_count l + leaf_count r

/-- The stored `value` field (`0` for the empty tree). -/
def stored_value : Option Node → Int
  | none => 0
  | some (Node.mk v _ _ _) => v

/-- The stored `size` field (`0` for the empty tree). -/
def node_size : Option Node → Int
  | none => 0
  | some (Node.mk _ sz _ _) => sz

/-- The set of keys stored in the leaves of the tree.  (An internal node's
    `value` is a cached maximum, not a key, so it is not counted.) -/
def view : Option Node → Set Int
  | none => ∅
  | some (Node.mk v _ l r) => (if l = none then {v} else ∅) ∪ view l ∪ view r

/-- Canonical shape: a leaf has `size = 1`; an internal node has two children,
    `size` equal to the total number of leaves, and `value` equal to the maximum
    key of its right subtree. -/
def is_canonical : Option Node → Prop
  | none => True
  | some (Node.mk v sz l r) =>
      (if l = none then sz = 1
       else sz = Int.ofNat (leaf_count l + leaf_count r) ∧ v = stored_value r) ∧
      is_canonical l ∧ is_canonical r

/-- Search-tree order: every key in the left subtree is smaller than every key
    in the right subtree.  (Vacuous for a leaf.) -/
def is_bst : Option Node → Prop
  | none => True
  | some (Node.mk _ _ l r) =>
      (∀ x ∈ view l, ∀ y ∈ view r, x < y) ∧ is_bst l ∧ is_bst r

/-- Weight-balance invariant: every internal node satisfies
    `3·min(lsz, rsz) ≥ max(lsz, rsz)`.  (Vacuous for a leaf.) -/
def is_wblt : Option Node → Prop
  | none => True
  | some (Node.mk _ _ l r) =>
      (3 * node_size l ≥ node_size r) ∧ (3 * node_size r ≥ node_size l) ∧
      is_wblt l ∧ is_wblt r

/-- A valid WBLT: canonical shape, search-tree ordered, and weight-balanced. -/
def is_wblt_tree (t : Option Node) : Prop := is_canonical t ∧ is_bst t ∧ is_wblt t

/-- Correctness of `wblt_insert`: inserting `v` into a valid WBLT keeps it
    valid and adds exactly that key (`∪` absorbs a duplicate). -/
theorem wblt_insert_correct (t : Option Node) (v : Int) (t' : Option Node)
    (h_t : is_wblt_tree t)
    (h : wblt_insert t v = ok t') :
    is_wblt_tree t' ∧ view t' = view t ∪ {v} := by
  sorry

/-- Correctness of `wblt_delete`: deleting `v` from a valid WBLT keeps it
    valid, removes exactly that key, and reports whether it was present. -/
theorem wblt_delete_correct (t : Option Node) (v : Int) (t' : Option Node) (b : Bool)
    (h_t : is_wblt_tree t)
    (h : wblt_delete t v = ok (t', b)) :
    is_wblt_tree t' ∧ view t' = view t \ {v} ∧ (b = true ↔ v ∈ view t) := by
  sorry

/-- Correctness of `wblt_find`: membership coincides with the key set. -/
theorem wblt_find_correct (t : Option Node) (v : Int) (b : Bool)
    (h_t : is_wblt_tree t)
    (h : wblt_find t v = ok b) :
    (b = true ↔ v ∈ view t) := by
  sorry

/-- Correctness of `wblt_kth`: for `k` in range, it returns the unique key
    with exactly `k − 1` smaller keys in the tree; out of range, it returns
    `-1`. -/
theorem wblt_kth_correct (t : Option Node) (k x : Int)
    (h_t : is_wblt_tree t)
    (h : wblt_kth t k = ok x) :
    ((1 ≤ k ∧ k ≤ (Set.ncard (view t) : Int)) →
       x ∈ view t ∧
       (Set.ncard {y : Int | y ∈ view t ∧ y < x} : Int) = k - 1) ∧
    ((k < 1 ∨ (Set.ncard (view t) : Int) < k) → x = -1) := by
  sorry

/-- Correctness of `wblt_rank`: it counts the keys strictly smaller than `v`. -/
theorem wblt_rank_correct (t : Option Node) (v r : Int)
    (h_t : is_wblt_tree t)
    (h : wblt_rank t v = ok r) :
    r = (Set.ncard {x : Int | x ∈ view t ∧ x < v} : Int) := by
  sorry

/-- The (computed) height of the tree. -/
def tree_height : Option Node → Nat
  | none => 0
  | some (Node.mk _ _ l r) => 1 + max (tree_height l) (tree_height r)

/-- Height bound: weight-balance (`3·min ≥ max`) forces the leaf count to grow
    by at least a factor of `4/3` per level, so the height is `O(log n)`. -/
theorem wblt_height_log_bound (t : Option Node) (h_t : is_wblt_tree t) :
    tree_height t ≤ 3 * Nat.log2 (leaf_count t + 1) + 3 := by
  sorry

/-- `rank` and `kth` are inverses: if `rank t v = k` (there are `k` keys below
    `v`), then the `(k+1)`-th smallest key is exactly `v`. -/
theorem wblt_kth_rank_inverse (t : Option Node) (v x k : Int)
    (h_t : is_wblt_tree t) (hv : v ∈ view t)
    (h_rank : wblt_rank t v = ok k)
    (h_kth : wblt_kth t (k + 1) = ok x) :
    x = v := by
  sorry
