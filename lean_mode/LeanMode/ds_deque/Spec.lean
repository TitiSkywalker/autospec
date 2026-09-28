import LeanMode.ds_deque.DsDeque
import Mathlib

open Aeneas Aeneas.Std Result
open ds_deque

set_option linter.unusedVariables false

/-!
# Specification: Double-Ended Queue (ring buffer)

`deque_new` / `push_back` / `push_front` / `pop_front` / `pop_back` operate on a
growable ring-buffer deque.  Correctness is stated via `toList`, which unwraps
the circular `head`/`len` layout into front-to-back order.
-/

/-- View the deque as a `List Int` in front-to-back order. -/
def toList (d : Deque) : List Int :=
  let l := (alloc.vec.Vec.deref d.buf : List Int)
  (List.range d.len.val).map (fun i => l.getD ((d.head.val + i) % l.length) 0)


/-! ## Specification theorems -/

/-- `deque_new` yields an empty deque. -/
theorem deque_new_correct (d : Deque)
    (h : deque_new = ok d) :
    toList d = [] := by
  sorry

/-- `push_back` appends exactly `v`. -/
theorem push_back_correct (d r : Deque) (v : Int)
    (h : push_back d v = ok r) :
    toList r = toList d ++ [v] := by
  sorry

/-- `push_front` prepends exactly `v`. -/
theorem push_front_correct (d r : Deque) (v : Int)
    (h : push_front d v = ok r) :
    toList r = v :: toList d := by
  sorry

/-- `pop_front` removes the front element. -/
theorem pop_front_correct (d r : Deque) (v : Int)
    (h : pop_front d = ok (r, some v)) :
    toList d = v :: toList r := by
  sorry

/-- `pop_back` removes the back element. -/
theorem pop_back_correct (d r : Deque) (v : Int)
    (h : pop_back d = ok (r, some v)) :
    toList d = toList r ++ [v] := by
  sorry

/-- `pop_front` on an empty deque returns `none` and leaves it unchanged. -/
theorem pop_front_none (d r : Deque)
    (h : pop_front d = ok (r, none)) :
    toList d = [] ∧ r = d := by
  sorry

/-- `pop_back` on an empty deque returns `none` and leaves it unchanged. -/
theorem pop_back_none (d r : Deque)
    (h : pop_back d = ok (r, none)) :
    toList d = [] ∧ r = d := by
  sorry
