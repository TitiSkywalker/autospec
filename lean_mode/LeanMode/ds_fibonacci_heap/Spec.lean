import LeanMode.ds_fibonacci_heap.DsFibonacciHeap
import Mathlib

open Aeneas Aeneas.Std Result
open ds_fibonacci_heap

set_option linter.unusedVariables false

/-!
# Specification: Fibonacci heap

`fib_insert`, `fib_find_min`, `fib_merge` and `fib_extract_min` operate on the
flat array representation of `src/lib.rs`: the nodes live at the slots
`1, 2, …` of the six parallel arrays `key`, `degree`, `parent`, `child`, `next`
and `marked`, slot `0` is the null sentinel, `min` is the index of a minimum
root (`0` when the heap is empty) and `used` counts the slots handed out.

The root list and every child list are *circular* singly linked lists threaded
through `next`, which is what the predicates below follow:

* `IsChild h p c`    — `c` lies on the child list of `p`;
* `Desc h v w`       — `w` is a proper descendant of `v` (transitive closure);
* `InHeap h v`       — `v` is a node of the heap (reachable from `min`).

`nodes`, `keys`, `childSet` and `subtree` turn those relations into `Finset`s /
a key `Multiset`, so that the theorems can speak about the stored keys.

`IsFibHeap` is the well-formedness invariant of the structure: the sentinel
discipline, heap order, the two parent/child link directions, `degree` counting
the children, the **linking invariant** (the children of a node of degree `k`
have degrees exactly `k-1, k-2, …, 0`) and acyclicity of the child structure.

Every operation of `src/lib.rs` preserves that invariant (checked by brute force
on randomized inputs in the `cfg(test)` module of `src/lib.rs`, where the
linking invariant and the conclusion of the rank bound are asserted after every
step).  The linking invariant is what makes the rank bound true: heap order
alone does not bound a subtree by its degree — a heap-ordered forest may have a
root of degree `3` whose three children are all leaves, and then only `4` nodes
sit below it, while `F₅ = 5`.

The centrepiece is `fib_heap_rank_bound`, the Fredman–Tarjan rank bound: a node
of degree `k` has at least `F_{k+2}` nodes in its subtree, hence a heap of `n`
nodes has a root of degree `O(log n)`.
-/

namespace FibHeapSpec

/-! ## Reading the flat representation -/

/-- Slot `i` of a `Vec Std.Usize`, as a `Nat` (`0` when out of range). -/
def usizeAt (v : alloc.vec.Vec Std.Usize) (i : Nat) : Nat :=
  (((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize).getD i 0#usize).val

/-- Slot `i` of a `Vec Int` (`0` when out of range). -/
def intAt (v : alloc.vec.Vec Int) (i : Nat) : Int :=
  (((alloc.vec.Vec.deref v : Slice Int) : List Int).getD i 0)

/-- The number of slots of the arena. -/
def slots (h : FibHeap) : Nat := (alloc.vec.Vec.deref h.key).length

/-- The key of slot `v`. -/
def kOf (h : FibHeap) (v : Nat) : Int := intAt h.key v

/-- `degree[v]`: the number of children of `v`. -/
def degOf (h : FibHeap) (v : Nat) : Nat := usizeAt h.degree v

/-- `parent[v]` (`0` for a root). -/
def parOf (h : FibHeap) (v : Nat) : Nat := usizeAt h.parent v

/-- `child[v]`: the first child of `v` (`0` when `v` has no children). -/
def childOf (h : FibHeap) (v : Nat) : Nat := usizeAt h.child v

/-- `next[v]`: the next sibling (or, for a root, the next root). -/
def nextOf (h : FibHeap) (v : Nat) : Nat := usizeAt h.next v

/-- `marked[v]`: the loss-of-one-child flag (always `0` here: no `decrease_key`). -/
def markOf (h : FibHeap) (v : Nat) : Nat := usizeAt h.marked v

/-- The `min` handle (`0` when the heap is empty). -/
def minOf (h : FibHeap) : Nat := h.min.val

/-- The allocation counter: the number of slots handed out so far. -/
def usedOf (h : FibHeap) : Nat := h.used.val

/-! ## The structure of the heap -/

/-- `c` is a child of `p`, i.e. `c` lies on the circular child list that starts
    at `child p`.  (This is exactly what `fib_link` maintains.) -/
inductive IsChild (h : FibHeap) : Nat → Nat → Prop
  | head (p c : Nat) : childOf h p = c → c ≠ 0 → IsChild h p c
  | step (p c c' : Nat) :
      IsChild h p c → nextOf h c = c' → c' ≠ 0 → IsChild h p c'

/-- `w` is a proper descendant of `v`: a child, or a descendant of a child. -/
inductive Desc (h : FibHeap) : Nat → Nat → Prop
  | child (v c : Nat) : IsChild h v c → Desc h v c
  | step (v c w : Nat) : IsChild h v c → Desc h c w → Desc h v w

/-- `v` is a node of the heap: it is reachable from the `min` handle either
    through the circular root list or through child lists. -/
inductive InHeap (h : FibHeap) : Nat → Prop
  | root (hne : minOf h ≠ 0) : InHeap h (minOf h)
  | next (x y : Nat) : InHeap h x → nextOf h x = y → y ≠ 0 → InHeap h y
  | child (v c : Nat) : InHeap h v → IsChild h v c → InHeap h c

/-- `y` is reached from `x` by following `next` (`0` steps or more): the
    root list is the `next`-cycle starting at `min`, so every root is reached
    from `min` this way. -/
inductive NextChain (h : FibHeap) : Nat → Nat → Prop
  | refl (x : Nat) : NextChain h x x
  | step (x y z : Nat) : NextChain h x y → nextOf h y = z → z ≠ 0 → NextChain h x z

/-- The children of `v`. -/
noncomputable def childSet (h : FibHeap) (v : Nat) : Finset Nat := by
  classical exact (Finset.range (slots h)).filter (fun c => IsChild h v c)

/-- The nodes of the heap. -/
noncomputable def nodes (h : FibHeap) : Finset Nat := by
  classical exact (Finset.range (slots h)).filter (fun v => InHeap h v)

/-- The subtree of `v`: `v` together with its descendants. -/
noncomputable def subtree (h : FibHeap) (v : Nat) : Finset Nat := by
  classical exact (Finset.range (slots h)).filter (fun w => w = v ∨ Desc h v w)

/-- The multiset of stored keys (one occurrence per node). -/
noncomputable def keys (h : FibHeap) : Multiset Int :=
  (nodes h).val.map (kOf h)

/-- Well-formedness of a Fibonacci heap:

    * if the heap is non-empty, `min` is a node of the heap and a root, and its
      key is a minimum *among the roots* (heap order then makes it a minimum of
      the whole heap, which is what `fib_find_min` reports);
    * the root list is the single `next`-cycle through `min`: every root is
      reached from `min` by following `next`;
    * the *arena discipline*: every node occupies a slot that has actually been
      handed out (`v ≤ used`) and lies inside the arena (`1 ≤ v < slots`).
      This is load-bearing, not decorative: `fib_insert` writes the new node to
      slot `used + 1` and `fib_merge` relocates only the slots `1..used` of its
      right argument, so on a heap whose live nodes stray above `used` those two
      operations would clobber or drop a live node;
    * the parent of a node of the heap is again a node of the heap;
    * heap order: a parent's key is `≤` the keys of its children;
    * parent links agree with the child lists, in both directions;
    * `degree` counts the children;
    * the *linking invariant*: the children of a node of degree `k` have degrees
      `k-1, k-2, …, 0` (equivalently, their degrees are pairwise distinct and
      all `< k`);
    * the child structure is acyclic (no node is its own proper descendant);
    * no node is marked: `fib_link` clears the mark of the root it attaches, and
      nothing ever sets one, because no `decrease_key` (hence no cascading cut)
      is implemented.

    The rank bound is a pure "global magnitude" induction over this invariant:
    it needs the closure of the node set under children, the parent/child link
    agreement (which makes the parent link unique, so that the subtrees of two
    distinct children are disjoint), `degree` counting the children, the linking
    invariant, and acyclicity.  Without those, the sum
    `Σ_children card (subtree c)` could exceed `card (subtree v)` and the bound
    would fail. -/
def IsFibHeap (h : FibHeap) : Prop :=
  (minOf h ≠ 0 → InHeap h (minOf h) ∧ parOf h (minOf h) = 0) ∧
  (∀ v, InHeap h v → parOf h v = 0 → kOf h (minOf h) ≤ kOf h v) ∧
  (∀ v, InHeap h v → parOf h v = 0 → NextChain h (minOf h) v) ∧
  (∀ v, InHeap h v → 1 ≤ v ∧ v < slots h) ∧
  (∀ v, InHeap h v → v ≤ usedOf h) ∧
  (∀ c, InHeap h c → parOf h c ≠ 0 → InHeap h (parOf h c)) ∧
  (∀ v c, InHeap h v → IsChild h v c → InHeap h c) ∧
  (∀ v, InHeap h v → ∀ c, IsChild h v c → kOf h v ≤ kOf h c) ∧
  (∀ v c, InHeap h v → IsChild h v c → parOf h c = v) ∧
  (∀ c, InHeap h c → parOf h c ≠ 0 → IsChild h (parOf h c) c) ∧
  (∀ v, InHeap h v → degOf h v = (childSet h v).card) ∧
  (∀ v, InHeap h v → (childSet h v).image (degOf h) = Finset.range (degOf h v)) ∧
  (∀ v, InHeap h v → markOf h v = 0) ∧
  (∀ v, InHeap h v → ¬ Desc h v v)

end FibHeapSpec

open FibHeapSpec

/-! ## Specification theorems -/

/-- `fib_new` returns an empty heap that satisfies the invariant: no nodes, no
    keys (so the invariant is inhabited by the heaps the implementation builds). -/
theorem fib_new_is_fib_heap (n : Std.Usize) (h : FibHeap)
    (h_new : fib_new n = ok h) :
    IsFibHeap h ∧ nodes h = ∅ ∧ keys h = 0 := by
  sorry

/-- `fib_insert` adds exactly the key `k` (as one more occurrence) and preserves
    the invariant.  The capacity hypothesis is the side condition of the flat
    arena: the new node is written to slot `used + 1`, so the arena must have
    room for it (writing past the end is an out-of-bounds access). -/
theorem fib_insert_correct (h h' : FibHeap) (k : Int)
    (h_inv : IsFibHeap h)
    (h_cap : usedOf h + 2 ≤ slots h)
    (h_ins : fib_insert h k = ok h') :
    IsFibHeap h' ∧ keys h' = keys h + {k} := by
  sorry

/-- `fib_find_min` returns a minimum of the stored key multiset, and `none`
    exactly when the heap is empty.  (It is the root named by the `min` handle,
    which heap order forces to be a global minimum.) -/
theorem fib_find_min_correct (h : FibHeap) (r : Option Int)
    (h_inv : IsFibHeap h)
    (h_find : fib_find_min h = ok r) :
    (r = none ↔ keys h = 0) ∧
    (∀ x, r = some x → x ∈ keys h ∧ ∀ y, y ∈ keys h → x ≤ y) := by
  sorry

/-- `fib_merge` melds two heaps: the result stores exactly the keys of both
    (sum of the multisets) and satisfies the invariant.  The nodes of `b` are
    renumbered into the arena of `a` (that is what the flat representation
    costs: the tree structure and the root lists are spliced untouched, but the
    slots are relocated), whence the capacity hypothesis. -/
theorem fib_merge_correct (a b c : FibHeap)
    (h_a : IsFibHeap a) (h_b : IsFibHeap b)
    (h_cap : usedOf a + usedOf b + 1 ≤ slots a)
    (h_merge : fib_merge a b = ok c) :
    IsFibHeap c ∧ keys c = keys a + keys b := by
  sorry

/-- `fib_extract_min` removes the minimum: it returns `none` and the unchanged
    heap when the heap is empty, and otherwise a key `x` that is a minimum of the
    stored keys together with a well-formed heap whose key multiset is the
    original one with exactly one occurrence of `x` erased. -/
theorem fib_extract_min_correct (h h' : FibHeap) (r : Option Int)
    (h_inv : IsFibHeap h)
    (h_ex : fib_extract_min h = ok (h', r)) :
    IsFibHeap h' ∧
    (r = none → h' = h ∧ keys h = 0) ∧
    (keys h = 0 → r = none) ∧
    (∀ x, r = some x →
      keys h' = (keys h).erase x ∧ x ∈ keys h ∧ (∀ y, y ∈ keys h → x ≤ y)) := by
  sorry

/-- The Fredman–Tarjan **rank bound**: in a well-formed Fibonacci heap, a node
    of degree `k` has at least `F_{k+2}` nodes in its subtree (`F₀ = 0`,
    `F₁ = F₂ = 1`).  The proof is by induction over the (acyclic) subtree
    relation: the children of a degree-`k` node have degrees `k-1, …, 0`, so

      `card (subtree v) ≥ 1 + Σ_{j<k} F_{j+2} = F_{k+3} - 1 ≥ F_{k+2}`.

    The linkage invariant of `IsFibHeap` is what makes this true — heap order
    alone does not bound a node's subtree by its degree. -/
theorem fib_heap_rank_bound (h : FibHeap) (v : Nat)
    (h_inv : IsFibHeap h)
    (h_in : InHeap h v) :
    Nat.fib (degOf h v + 2) ≤ (subtree h v).card := by
  sorry

/-- The global form of the rank bound: a subtree is a subset of the heap, so in
    a heap of `n` nodes every node of degree `k` satisfies `F_{k+2} ≤ n`.  This
    is the "magnitude" statement — the degree of a node (and hence the size of
    the degree table `fib_extract_min` consolidates over) is `O(log_φ n)`. -/
theorem fib_heap_degree_bound (h : FibHeap) (v : Nat)
    (h_inv : IsFibHeap h)
    (h_in : InHeap h v) :
    Nat.fib (degOf h v + 2) ≤ (nodes h).card := by
  sorry
