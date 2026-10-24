import LeanMode.dp_optimal_bst.DpOptimalBst
import Mathlib

open Aeneas Aeneas.Std Result
open dp_optimal_bst

set_option linter.unusedVariables false

/-!
# Specification: optimal binary search tree

`optimal_bst freq` runs the interval DP and returns the minimum expected number
of nodes visited by a successful search in a binary search tree on the keys
`0, …, n-1`, where key `k` is looked up with frequency `freq[k]`.

The specification below characterises that number *declaratively*, as a
minimum over the set of all binary search trees on those keys — with no mention
of the DP array, of interval bounds, or of the order in which the DP fills its
table.  The cost of a tree is its weighted internal path length
`Σ_k freq[k] * (depth(k) + 1)` (the root has depth `0` and costs one visit).
-/

namespace OptimalBst

/-- A binary tree over natural-number keys. -/
inductive BTree where
  | leaf : BTree
  | node : BTree → Nat → BTree → BTree
deriving DecidableEq

namespace BTree

/-- The keys of a tree, listed in inorder. -/
def keys : BTree → List Nat
  | leaf => []
  | node l k r => keys l ++ [k] ++ keys r

/-- Standard binary-search-tree insertion (used only to *enumerate* trees). -/
def insert : BTree → Nat → BTree
  | leaf, k => node leaf k leaf
  | node l v r, k =>
      if k < v then node (insert l k) v r
      else if v < k then node l v (insert r k)
      else node l v r

/-- Weighted internal path length of a tree whose root sits at depth `d`:
`Σ_{k ∈ keys} f k * (depth k + 1)`. -/
def costAt (f : Nat → Nat) : Nat → BTree → Nat
  | _, leaf => 0
  | d, node l k r => f k * (d + 1) + costAt f (d + 1) l + costAt f (d + 1) r

/-- Expected search cost of a tree (its root has depth `0`). -/
def cost (f : Nat → Nat) (t : BTree) : Nat := costAt f 0 t

end BTree

/-- `t` is a binary search tree over exactly the keys `0, …, n-1`: its inorder
traversal is the ascending list `0, 1, …, n-1` (so every key occurs once and the
left/right ordering constraints hold). -/
abbrev IsSearchTree (t : BTree) (n : Nat) : Prop := t.keys = List.range n

/-- All binary search trees over the keys `0, …, n-1`, enumerated by inserting
the keys in every possible order (each order builds a BST containing exactly
those keys, and every BST arises from its preorder traversal). -/
def allTrees (n : Nat) : Finset BTree :=
  ((List.range n).permutations.map (fun order => order.foldl BTree.insert BTree.leaf)).toFinset

/-- The set of valid search trees is non-empty (the fold from any insertion
order yields one). -/
theorem allTrees_nonempty (n : Nat) : (allTrees n).Nonempty := by
  refine ⟨(List.range n).foldl BTree.insert BTree.leaf, ?_⟩
  rw [allTrees, List.mem_toFinset, List.mem_map]
  exact ⟨List.range n, List.mem_permutations.mpr (List.Perm.refl _), rfl⟩

/-- Every enumerated tree really is a search tree on `0, …, n-1`. -/
theorem allTrees_isSearchTree (n : Nat) (t : BTree) (ht : t ∈ allTrees n) :
    IsSearchTree t n := by
  sorry

/-- The minimum expected search cost over all binary search trees on `n` keys,
where key `k` is looked up with frequency `freq[k]`. -/
def minExpectedSearchCost (freq : List Nat) : Nat :=
  ((allTrees freq.length).image (BTree.cost (fun k => freq.getD k 0))).min'
    ((allTrees_nonempty freq.length).image (BTree.cost (fun k => freq.getD k 0)))

end OptimalBst

open OptimalBst

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- Correctness of the optimal-BST interval DP: for any frequency vector, the
returned value is the minimum, over all binary search trees on the keys
`0, …, freq.length - 1`, of the weighted internal path length
`Σ_k freq[k] * (depth(k) + 1)`. — no `∃`, and no reference to the DP table. -/
theorem optimal_bst_correct (freq : Slice Std.Usize) (c : Std.Usize)
    (h : optimal_bst freq = ok c) :
    c.val = minExpectedSearchCost (toNatList freq) := by
  sorry
