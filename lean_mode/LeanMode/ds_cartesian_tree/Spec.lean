import LeanMode.ds_cartesian_tree.DsCartesianTree
import Mathlib

open Aeneas Aeneas.Std Result
open ds_cartesian_tree

set_option linter.unusedVariables false

/-!
# Specification: Cartesian tree

`cartesian_tree a` builds the Cartesian tree of the array `a` with the
monotonic-stack construction and returns the parent array `parent`, where
`parent[i]` is the index of the parent of node `i`, or `a.len()` when `i` is
the root.  Node `i` carries the value `a[i]`.

The Cartesian tree is the unique binary tree that is simultaneously

* a **min-heap** — `a[parent[i]] ≤ a[i]` for every non-root `i`
  (`cartesian_tree_minheap`), and
* a **search tree on the indices** — the in-order traversal visits
  `0, 1, …, n-1` (`cartesian_tree_inorder`),

on a well-formed rooted tree (`cartesian_tree_wf`).  Together these say the
output is a Cartesian tree (`cartesian_tree_correct`) and, with distinct
values, that it is the *only* one (`cartesian_tree_unique`).

The reason the structure is interesting is `cartesian_tree_rmq`: the tree
encodes *every* range-minimum query, because the position of the minimum of
`a[l..=r]` is exactly the lowest common ancestor of `l` and `r`.

The parent array is represented as a function `Nat → Nat` (missing/sentinel
entries read as `n`), so that ancestors, the LCA and the traversals below are
all stated with structural / inductive definitions rather than ad-hoc fuel.
-/

/-- The parent array of a result, as a `List Nat` (each node index is the
    value of a `Std.Usize`). -/
def plist (p : alloc.vec.Vec Std.Usize) : List Nat :=
  ((alloc.vec.Vec.deref p : Slice Std.Usize) : List Std.Usize).map (fun x => x.val)

/-- The parent function of a parent array, with `n = pl.length` as the
    sentinel: the root (and anything past the end) maps to `n`. -/
def parFn (pl : List Nat) : Nat → Nat := fun i => pl.getD i pl.length

/-- `Anc f x y`: `y` is an ancestor of `x`, i.e. `y` appears on the parent-path
    `x → f x → f (f x) → …`. -/
inductive Anc (f : Nat → Nat) : Nat → Nat → Prop where
  | refl (x : Nat) : Anc f x x
  | step {x y : Nat} : Anc f (f x) y → Anc f x y

/-- `c` is the left child of `i`: its parent is `i` and its index is smaller. -/
def IsLeftChild (f : Nat → Nat) (c i : Nat) : Prop := f c = i ∧ c < i

/-- `c` is the right child of `i`: its parent is `i` and its index is larger. -/
def IsRightChild (f : Nat → Nat) (c i : Nat) : Prop := f c = i ∧ i < c

/-- `x` lies in the left subtree of `i`. -/
def InLeftSub (f : Nat → Nat) (i x : Nat) : Prop := ∃ c, IsLeftChild f c i ∧ Anc f x c

/-- `x` lies in the right subtree of `i`. -/
def InRightSub (f : Nat → Nat) (i x : Nat) : Prop := ∃ c, IsRightChild f c i ∧ Anc f x c

/-- Well-formedness of the parent array as a **rooted tree** on the node set
    `{0, …, n-1}` (the sentinel `n` is the "no parent" marker): every parent is
    in range, there is exactly one root, every node reaches the sentinel, and
    there are exactly `n - 1` edges (`n - 1` nodes with a real parent). -/
def IsTree (f : Nat → Nat) (n : Nat) : Prop :=
  (∀ i, i < n → f i ≤ n) ∧
  (∃! r, r < n ∧ f r = n) ∧
  (∀ i, i < n → f^[n] i = n) ∧
  ((List.range n).filter (fun i => f i < n)).length = n - 1

/-- The min-heap condition: the parent's value is `≤` the node's value. -/
def IsMinHeap (vs : List Int) (f : Nat → Nat) (n : Nat) : Prop :=
  ∀ i, i < n → f i < n → vs.getD (f i) 0 ≤ vs.getD i 0

/-- The search-tree-on-the-indices condition, equivalent to "the in-order
    traversal visits `0, …, n-1`": each node has at most one left child and at
    most one right child, and every descendant of `i` with a smaller index lies
    in the left subtree of `i` while every descendant with a larger index lies
    in the right subtree (and conversely). -/
def IsInOrder (f : Nat → Nat) (n : Nat) : Prop :=
  (∀ i c c', i < n → IsLeftChild f c i → IsLeftChild f c' i → c = c') ∧
  (∀ i c c', i < n → IsRightChild f c i → IsRightChild f c' i → c = c') ∧
  (∀ i x, i < n → Anc f x i →
    (x < i ↔ InLeftSub f i x) ∧ (i < x ↔ InRightSub f i x))

/-- A full Cartesian-tree predicate for the parent function `f` on
    `{0, …, n-1}` storing `vs`: a well-formed tree that is a min-heap and whose
    in-order traversal is `0, …, n-1`. -/
def IsCartesianTree (vs : List Int) (f : Nat → Nat) (n : Nat) : Prop :=
  IsTree f n ∧ IsMinHeap vs f n ∧ IsInOrder f n

/-- `w` is the lowest common ancestor of `l` and `r`: it is a common ancestor
    (an ancestor of both) and it is the deepest one — every common ancestor is
    itself an ancestor of `w`. -/
def IsLca (f : Nat → Nat) (n l r w : Nat) : Prop :=
  w < n ∧ Anc f l w ∧ Anc f r w ∧
  (∀ u, u < n → Anc f l u → Anc f r u → Anc f w u)

/-- The minimum value of `vs[l..=r]` (with `getD` default `0`). -/
def rangeMin (vs : List Int) (l r : Nat) : Int :=
  List.foldl (fun acc x => min acc x) (vs.getD l 0) ((vs.drop l).take (r + 1 - l))

/-! ## Specification theorems -/

/-- The returned parent array is a well-formed rooted tree on `{0, …, n-1}`:
    every parent is in range, there is exactly one root, every node reaches
    the sentinel, and there are exactly `n - 1` edges.

    The `0 < n` hypothesis is needed and is not part of `IsTree`: the empty
    input produces the empty parent array, which has *no* root, so `IsTree f 0`
    (with its `∃!` root clause) is false while the other clauses hold
    vacuously. -/
theorem cartesian_tree_wf (a : Slice Int) (p : alloc.vec.Vec Std.Usize)
    (h : cartesian_tree a = ok p)
    (hn : 0 < (plist p).length) :
    IsTree (parFn (plist p)) (plist p).length := by
  sorry

/-- The returned parent array is a min-heap: `a[parent[i]] ≤ a[i]` for every
    non-root `i`.  The explicit `f i < n` hypothesis excludes the root, whose
    parent is the sentinel `n`. -/
theorem cartesian_tree_minheap (a : Slice Int) (p : alloc.vec.Vec Std.Usize)
    (h : cartesian_tree a = ok p) :
    IsMinHeap (a : List Int) (parFn (plist p)) (plist p).length := by
  sorry

/-- The returned tree is a search tree on the indices: the in-order traversal
    visits `0, 1, …, n-1`. -/
theorem cartesian_tree_inorder (a : Slice Int) (p : alloc.vec.Vec Std.Usize)
    (h : cartesian_tree a = ok p) :
    IsInOrder (parFn (plist p)) (plist p).length := by
  sorry

/-- Combining the three properties: the output is a Cartesian tree of `a`
    (well-formed, min-heap, in-order = index order).  As in
    `cartesian_tree_wf` the `0 < n` hypothesis is required, because the rooted
    tree part of `IsCartesianTree` has no root in the empty case. -/
theorem cartesian_tree_correct (a : Slice Int) (p : alloc.vec.Vec Std.Usize)
    (h : cartesian_tree a = ok p)
    (hn : 0 < (plist p).length) :
    IsCartesianTree (a : List Int) (parFn (plist p)) (plist p).length := by
  sorry

/-- **The range-minimum-query correspondence.**  For every `l ≤ r`, the value
    at the lowest common ancestor `w` of `l` and `r` (in the produced tree) is
    the minimum of `a[l..=r]`.  Since `IsLca` is satisfied by a *unique* `w`
    (the tree is a genuine rooted tree), this pins the value down exactly. -/
theorem cartesian_tree_rmq (a : Slice Int) (p : alloc.vec.Vec Std.Usize)
    (h : cartesian_tree a = ok p) (l r w : Nat)
    (hlr : l ≤ r) (hr : r < (plist p).length)
    (hlca : IsLca (parFn (plist p)) (plist p).length l r w) :
    (a : List Int).getD w 0 = rangeMin (a : List Int) l r := by
  sorry

/-- **Uniqueness.**  When the values are distinct, the Cartesian tree is
    determined by its defining properties: any other parent function `g` that
    is a Cartesian tree of the same array (well-formed, min-heap, in-order =
    index order) agrees with the produced one on `{0, …, n-1}`.  The
    distinctness hypothesis is essential — with ties there are several
    in-order min-heaps (e.g. `[1, 1]` admits both roots). -/
theorem cartesian_tree_unique (a : Slice Int) (p : alloc.vec.Vec Std.Usize) (g : Nat → Nat)
    (h : cartesian_tree a = ok p)
    (hnd : (a : List Int).Nodup)
    (hg : IsCartesianTree (a : List Int) g (plist p).length) :
    ∀ i, i < (plist p).length → g i = parFn (plist p) i := by
  sorry
