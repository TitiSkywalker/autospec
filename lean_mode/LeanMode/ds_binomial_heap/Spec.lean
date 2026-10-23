import LeanMode.ds_binomial_heap.DsBinomialHeap
import Mathlib

open Aeneas Aeneas.Std Result
open ds_binomial_heap

set_option linter.unusedVariables false

/-!
# Specification: Binomial Heap

A **binomial tree** of degree (rank) `r` is a heap-ordered tree whose root has
children that are binomial trees of degrees `r-1, r-2, …, 0`.  Consequently a
degree-`r` tree contains exactly `2^r` nodes.  A **binomial heap** is a forest
of such trees in which the roots have pairwise distinct degrees — the
binary-counter invariant.

The node pool of `BinomHeap` is a set of flat arrays indexed by node number
`1 ..= n` (`0` is the null sentinel): `key`, `degree`, `parent`, `child`
(first, highest-degree child) and `sibling` (next child).  The forest itself is
the `roots` array.

The specification below is built from:

* `InForest` — the nodes reachable from the root forest along `child`/`sibling`
  links (`Descend` is the from-one-root descendancy relation);
* `IsTree` / `KidsChain` — the inductive "is a heap-ordered binomial tree of
  degree `r`" relation, an inductive relation (rather than a recursive
  function over the flat arrays) precisely to avoid the termination problem
  those functions would have;
* `treeSize` / `kidsSize` — the node count of a tree / of a sibling chain,
  structurally recursive on the degree;
* `keysOf` — the multiset of the keys of the in-forest nodes;
* `WellFormed` — the representation invariant shared as the hypothesis of the
  correctness theorems: the pool bound, the forest covering the roots with
  distinct degrees, each root a heap-ordered binomial tree of its own degree,
  and the `parent` back-pointers recorded in `KidsChain` (which forces the
  trees to be disjoint).
-/

/-- `keyAt h v` — the key stored at node `v` (`0` if out of range). -/
def keyAt (h : BinomHeap) (v : Nat) : Int :=
  (alloc.vec.Vec.deref h.key : List Int).getD v 0

/-- `degAt h v` — the degree (number of children) of node `v`. -/
def degAt (h : BinomHeap) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref h.degree : List Std.Usize).map (·.val)).getD v 0

/-- `parAt h v` — the parent of node `v` (`0` for a root). -/
def parAt (h : BinomHeap) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref h.parent : List Std.Usize).map (·.val)).getD v 0

/-- `chAt h v` — the first (highest-degree) child of node `v` (`0` if none). -/
def chAt (h : BinomHeap) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref h.child : List Std.Usize).map (·.val)).getD v 0

/-- `sibAt h v` — the next sibling of node `v` (`0` if `v` is the last). -/
def sibAt (h : BinomHeap) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref h.sibling : List Std.Usize).map (·.val)).getD v 0

/-- The root forest, as a list of node numbers (in the order the array holds
    them, i.e. by increasing degree when the heap is well formed). -/
def rootList (h : BinomHeap) : List Nat :=
  (alloc.vec.Vec.deref h.roots : List Std.Usize).map (·.val)

mutual
/-- `IsTree h v r` — node `v` roots a heap-ordered binomial tree of degree `r`:
    the root has `r` children, which are binomial trees of degrees
    `r-1, r-2, …, 0`, and every child's key is at least the root's.  The
    `parent` back-pointer is required in `KidsChain`, which is what makes the
    trees of a well-formed forest disjoint. -/
inductive IsTree (h : BinomHeap) : Nat → Nat → Prop where
  | leaf {v : Nat} :
      degAt h v = 0 → chAt h v = 0 → IsTree h v 0
  | node {v r : Nat} :
      degAt h v = r + 1 → chAt h v ≠ 0 → KidsChain h v (chAt h v) r →
      IsTree h v (r + 1)

/-- `KidsChain h p c d` — `c` heads the sibling chain of children of `p` whose
    degrees are `d, d-1, …, 0`; every child is a binomial tree, points back at
    `p`, and has key at least `keyAt h p`. -/
inductive KidsChain (h : BinomHeap) : Nat → Nat → Nat → Prop where
  | last {p c : Nat} :
      IsTree h c 0 → sibAt h c = 0 → parAt h c = p → keyAt h p ≤ keyAt h c →
      KidsChain h p c 0
  | more {p c c' r : Nat} :
      IsTree h c (r + 1) → sibAt h c = c' → c' ≠ 0 → parAt h c = p →
      keyAt h p ≤ keyAt h c → KidsChain h p c' r →
      KidsChain h p c (r + 1)
end

/-- `Descend h r w` — `w` lies in the subtree rooted at `r`, i.e. is reached
    from `r` by a path of `child` / `sibling` links. -/
inductive Descend (h : BinomHeap) (r : Nat) : Nat → Prop where
  | refl : Descend h r r
  | child {u w : Nat} : Descend h r u → chAt h u = w → w ≠ 0 → Descend h r w
  | sib {u w : Nat} : Descend h r u → sibAt h u = w → w ≠ 0 → Descend h r w

/-- `w` is stored in the forest, i.e. lies in the subtree of some root. -/
def InForest (h : BinomHeap) (w : Nat) : Prop :=
  ∃ r, r ∈ rootList h ∧ Descend h r w

/-- A well-formed binomial heap:
    * every pool vector has exactly the slots `0 ..= n` — this is not
      decorative: `binom_insert` / `splice` append with `Vec.push` and assume
      the new node lands at logical index `n + 1`, so a padded pool would
      silently attach a key to the wrong slot (and `keysOf` would disagree with
      the multiset law);
    * the roots are distinct degrees;
    * every root is parentless, siblingless and the root of a heap-ordered
      binomial tree of exactly its recorded degree;
    * the forest lies inside the pool. -/
def WellFormed (h : BinomHeap) : Prop :=
  (alloc.vec.Vec.deref h.key : List Int).length = h.n.val + 1 ∧
  (alloc.vec.Vec.deref h.degree : List Std.Usize).length = h.n.val + 1 ∧
  (alloc.vec.Vec.deref h.parent : List Std.Usize).length = h.n.val + 1 ∧
  (alloc.vec.Vec.deref h.child : List Std.Usize).length = h.n.val + 1 ∧
  (alloc.vec.Vec.deref h.sibling : List Std.Usize).length = h.n.val + 1 ∧
  ((rootList h).map (degAt h)).Nodup ∧
  (∀ r, r ∈ rootList h →
      r ≠ 0 ∧ parAt h r = 0 ∧ sibAt h r = 0 ∧ IsTree h r (degAt h r)) ∧
  (∀ w, InForest h w → w ≤ h.n.val)

mutual
/-- Node count of the sibling chain headed by `c` whose top tree has degree
    `r` (degrees `r, r-1, …, 0`), given the head node's degree. -/
def kidsSize (h : BinomHeap) (c : Nat) (r : Nat) : Nat :=
  match r with
  | 0 => 1
  | r + 1 => treeSize h c (r + 1) + kidsSize h (sibAt h c) r
termination_by 2 * r + 1

/-- Node count of the binomial tree rooted at `v` of degree `r`. -/
def treeSize (h : BinomHeap) (v : Nat) (r : Nat) : Nat :=
  match r with
  | 0 => 1
  | r + 1 => 1 + kidsSize h (chAt h v) r
termination_by 2 * r
end

/-- The multiset of keys stored in the forest. -/
noncomputable def keysOf (h : BinomHeap) : Multiset Int := by
  classical
  exact
    (Finset.filter (fun w => InForest h w) (Finset.range (h.n.val + 1))).val.map
      (keyAt h)

/-! ## Specification theorems -/

/-- **Structural "/binomial" size theorem.**  A node rooting a well-formed
    binomial tree of degree `r` has exactly `2 ^ r` nodes in its subtree — the
    defining structural property of binomial trees (and the reason a heap built
    from `n` keys has at most `log2 n + 1` trees). -/
theorem binom_tree_size (h : BinomHeap) (v r : Nat)
    (h_tree : IsTree h v r) :
    treeSize h v r = 2 ^ r := by
  sorry

/-- **Insert correctness.**  Inserting `key` into a well-formed heap yields a
    well-formed heap whose key multiset is the old one plus `{key}` (no keys
    are lost or duplicated). -/
theorem binom_insert_correct (h h' : BinomHeap) (k : Int)
    (h_wf : WellFormed h)
    (h_ok : binom_insert h k = ok h') :
    WellFormed h' ∧ keysOf h' = keysOf h + {k} := by
  sorry

/-- **Merge (union) correctness.**  Merging two well-formed heaps yields a
    well-formed heap; in particular its degrees stay distinct (the
    binary-counter invariant is restored by the consolidation pass) and its key
    multiset is the sum of the inputs'. -/
theorem binom_merge_correct (a b h : BinomHeap)
    (h_wf_a : WellFormed a)
    (h_wf_b : WellFormed b)
    (h_ok : binom_merge a b = ok h) :
    WellFormed h ∧ keysOf h = keysOf a + keysOf b := by
  sorry

/-- **Minimum read.**  On a non-empty well-formed heap, `binom_find_min`
    returns a stored key that is below every stored key — the minimum of the
    key multiset. -/
theorem binom_find_min_correct (h : BinomHeap) (m : Int)
    (h_wf : WellFormed h)
    (h_ne : rootList h ≠ [])
    (h_ok : binom_find_min h = ok m) :
    m ∈ keysOf h ∧ ∀ x, x ∈ keysOf h → m ≤ x := by
  sorry

/-- **Extract-min correctness.**  Removing the minimum from a non-empty
    well-formed heap returns a key that is stored and minimal, and yields a
    well-formed heap whose key multiset is the original *minus exactly one
    occurrence* of that minimum. -/
theorem binom_extract_min_correct (h h' : BinomHeap) (mk : Int)
    (h_wf : WellFormed h)
    (h_ne : rootList h ≠ [])
    (h_ok : binom_extract_min h = ok (mk, h')) :
    WellFormed h' ∧
    mk ∈ keysOf h ∧
    (∀ x, x ∈ keysOf h → mk ≤ x) ∧
    keysOf h' = keysOf h - {mk} := by
  sorry

/-- **Forest size.**  In a well-formed heap the number of stored keys is the
    sum, over the roots, of `2 ^ degree` — the binomial trees of the forest
    partition the stored nodes.  Together with distinct root degrees this is
    the quantitative form of the binary-counter invariant. -/
theorem binom_forest_size (h : BinomHeap)
    (h_wf : WellFormed h) :
    (keysOf h).card = ((rootList h).map (fun r => 2 ^ degAt h r)).sum := by
  sorry
