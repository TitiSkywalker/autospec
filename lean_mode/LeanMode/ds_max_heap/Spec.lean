import LeanMode.ds_max_heap.DsMaxHeap
import Mathlib

open Aeneas Aeneas.Std Result
open ds_max_heap

set_option linter.unusedVariables false

/-!
# Specification: Binary Max-Heap

`max_heap_push` / `max_heap_pop_max` maintain a max-heap over `Vec Int`.  The heap is viewed as a `List Int` in the standard complete-binary-tree layout: `parent(i) = (i-1)/2`, and the invariant is that every element is ≤ its parent. The multiset `view` tracks *which* elements are stored (order ignored).
-/

/-- View the heap `Vec Int` as a `List Int`. -/
def toList (v : alloc.vec.Vec Int) : List Int :=
  (alloc.vec.Vec.deref v : List Int)

/-- Safe list access with default `0`. -/
def get (l : List Int) (i : Nat) : Int :=
  l.getD i 0

/-- Parent index in the complete-binary-tree layout. -/
def parent (i : Nat) : Nat :=
  (i - 1) / 2

/-- The max-heap invariant: every element is ≤ its parent. -/
def is_heap (l : List Int) : Prop :=
  ∀ i, 0 < i ∧ i < l.length → get l i ≤ get l (parent i)

/-- Multiset of stored elements (order ignored). -/
def view (l : List Int) : Multiset Int :=
  Multiset.ofList l


/-! ## Specification theorems -/

/-- `push` preserves the heap invariant and adds exactly `v`. -/
theorem max_heap_push_correct
    (heap : alloc.vec.Vec Int) (v : Int) (result : alloc.vec.Vec Int)
    (h_pre : is_heap (toList heap))
    (h : max_heap_push heap v = ok result) :
    is_heap (toList result) ∧
    view (toList result) = view (toList heap) + {v} := by
  sorry

/-- `pop_max` returns the maximum element and removes exactly that element, preserving the heap invariant. -/
theorem max_heap_pop_max_correct
    (heap : alloc.vec.Vec Int) (result : alloc.vec.Vec Int) (max_val : Int)
    (h_pos : 0 < (toList heap).length)
    (h_pre : is_heap (toList heap))
    (h : max_heap_pop_max heap = ok (result, max_val)) :
    is_heap (toList result) ∧
    (∀ x : Int, x ∈ view (toList heap) → max_val ≥ x) ∧
    view (toList result) = view (toList heap) - {max_val} := by
  sorry
