import LeanMode.string_debruijn.StringDebruijn
import Mathlib

open Aeneas Aeneas.Std Result
open string_debruijn

set_option linter.unusedVariables false

/-!
# Specification: de Bruijn sequences (FKM construction)

`de_bruijn k n` returns a de Bruijn sequence of order `n` on the alphabet
`{0, …, k-1}`: a word `b` of length `k ^ n` whose *cyclic* word contains every
length-`n` word over the alphabet exactly once.  The implementation builds it by
concatenating, in lexicographic order, the Lyndon words whose length divides `n`
(the Fredricksen–Kessler–Maiorana construction); the specification speaks only
about the returned buffer, not about the construction.

The cyclic word is linearised as `cyclicWindow b n`: `b` is repeated and
truncated to `b.length + n - 1` characters.  Reading it in windows of length `n`
visits each of the `b.length` cyclic positions exactly once.  (For
`b.length ≥ n - 1` this is the textbook `b ++ b.take (n - 1)`; repeating `b` also
covers the short case `b.length < n - 1`, which occurs for `k = 1`.)

## Degenerate cases

* `k = 0` is excluded (`h_k : 1 ≤ k`).  The implementation returns the empty
  word (an empty alphabet has no length-`n` word for `n ≥ 1`), so `k ^ n = 0`
  holds, but the occurrence statement is vacuous anyway; we keep the hypothesis
  for clarity.
* `n = 0` is excluded (`h_n : 1 ≤ n`).  The implementation returns the single
  symbol `[0]` (length `k ^ 0 = 1`, whose one cyclic window is the empty word),
  but `countOccurrences [] …` counts the empty word at *every* position by our
  convention, so the "exactly once" statement is only meaningful for `n ≥ 1`.
* `k = 1`: the result is the single word `[0]` of length `1 = 1 ^ n`, and the
  all-zero length-`n` word occurs exactly once in its cyclic word (four copies
  of `0`) — the theorem covers this case.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- A `Vec Std.Usize` as a `List Nat`. -/
def vecNat (v : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize)

/-- Concatenate `k` copies of `b`. -/
def repConcat : List Nat → Nat → List Nat
  | _, 0 => []
  | b, k + 1 => b ++ repConcat b k

/-- The linearised cyclic word of `b`: `b` repeated, truncated to
    `b.length + n - 1` characters.  Its length-`n` windows are exactly the
    cyclic windows of `b`. -/
def cyclicWindow (b : List Nat) (n : Nat) : List Nat :=
  (repConcat b (n + 1)).take (b.length + n - 1)

/-- `w` is a prefix of `l`. -/
def matchesPrefix : List Nat → List Nat → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: xs, b :: bs => if a = b then matchesPrefix xs bs else false

/-- The number of positions of `l` at which a window equal to `w` starts.
    `countOccurrences [] l` counts the empty word at every position of `l`. -/
def countOccurrences (w : List Nat) : List Nat → Nat
  | [] => if w = [] then 1 else 0
  | x :: rest => (if matchesPrefix w (x :: rest) then 1 else 0) + countOccurrences w rest

/-- Every element of `w` is a letter of the alphabet `{0, …, k-1}`. -/
def AllSmall (k : Nat) (w : List Nat) : Prop := ∀ x ∈ w, x < k

/-! ## Specification theorems -/

/-- `de_bruijn k n` returns a de Bruijn sequence of order `n`: its length is
    `k ^ n`, and every length-`n` word over the alphabet occurs exactly once in
    its cyclic word. -/
theorem de_bruijn_correct (k n : Std.Usize) (b : alloc.vec.Vec Std.Usize)
    (h : de_bruijn k n = ok b)
    (h_k : 1 ≤ k.val) (h_n : 1 ≤ n.val) :
    (alloc.vec.Vec.deref b).length = k.val ^ n.val ∧
    ∀ w, w.length = n.val → AllSmall k.val w →
      countOccurrences w (cyclicWindow (vecNat b) n.val) = 1 := by
  sorry

/-- Every character of the returned word is a letter of the alphabet
    `{0, …, k-1}`. -/
theorem de_bruijn_range (k n : Std.Usize) (b : alloc.vec.Vec Std.Usize)
    (h : de_bruijn k n = ok b) (h_k : 1 ≤ k.val) :
    AllSmall k.val (vecNat b) := by
  sorry
