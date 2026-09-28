import LeanMode.ds_linked_list.DsLinkedList
import Mathlib

open Aeneas Aeneas.Std Result
open ds_linked_list

set_option linter.unusedVariables false

/-!
# Specification: Singly-Linked List

`push_front` / `pop_front` / `append` / `reverse` / `length` operate on a
recursive cons-list ADT (`LinkedList`).  Correctness is stated via `toList`,
which views the ADT as a standard `List Int` in front-to-back order.
-/

/-- View the `LinkedList` ADT as a standard `List Int` (front-to-back). -/
def toList : LinkedList → List Int
  | LinkedList.Nil => []
  | LinkedList.Cons h t => h :: toList t


/-! ## Specification theorems -/

/-- `push_front` prepends exactly `v`. -/
theorem push_front_correct (l r : LinkedList) (v : Int)
    (h : push_front l v = ok r) :
    toList r = v :: toList l := by
  sorry

/-- `pop_front` on a non-empty list returns the head and the tail. -/
theorem pop_front_cons (l r : LinkedList) (v : Int)
    (h : pop_front l = ok (r, some v)) :
    toList l = v :: toList r := by
  sorry

/-- `pop_front` on the empty list returns `none` and leaves the list empty. -/
theorem pop_front_nil (l r : LinkedList)
    (h : pop_front l = ok (r, none)) :
    l = LinkedList.Nil ∧ r = LinkedList.Nil := by
  sorry

/-- `append` concatenates: `toList (append a b) = toList a ++ toList b`. -/
theorem append_correct (a b c : LinkedList)
    (h : append a b = ok c) :
    toList c = toList a ++ toList b := by
  sorry

/-- `reverse` reverses the list. -/
theorem reverse_correct (l r : LinkedList)
    (h : reverse l = ok r) :
    toList r = (toList l).reverse := by
  sorry

/-- `reverse` is an involution: reversing twice yields the original list. -/
theorem reverse_involutive (l r : LinkedList)
    (h : reverse l = ok r) :
    reverse r = ok l := by
  sorry

/-- `length` equals the length of the viewed list. -/
theorem length_correct (l : LinkedList) (n : Std.Usize)
    (h : length l = ok n) :
    n.val = (toList l).length := by
  sorry

/-- `length` distributes over `append`. -/
theorem length_append (a b c : LinkedList) (na nb nc : Std.Usize)
    (h_append : append a b = ok c)
    (h_na : length a = ok na)
    (h_nb : length b = ok nb)
    (h_nc : length c = ok nc) :
    nc.val = na.val + nb.val := by
  sorry
