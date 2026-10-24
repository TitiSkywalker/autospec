import LeanMode.string_myers_bits.StringMyersBits
import Mathlib

open Aeneas Aeneas.Std Result
open string_myers_bits

set_option linter.unusedVariables false

/-!
# Specification: bit-parallel Shift-And / Myers matching

`shift_and t p` returns every start index at which the integer-encoded pattern
`p` occurs in the text `t`, using the bit-parallel Shift-And automaton

  `state = ((state << 1) | 1) & mask[c]`

where `mask[c]` has bit `j` set iff `p[j] = c`; a match ends after reading
`t[i]` iff bit `m-1` of `state` is set (`m = p.length`).

The specification has two layers:

* `shift_and_correct` — the returned occurrence list is exactly the set of
  positions where `p` occurs (soundness **and** completeness);
* `shift_and_state_correct` — the algorithm's real content: the Shift-And state
  after reading `w` has bit `j` set iff `w` ends with the length-`(j+1)` prefix
  of `p`.

Characters are integer-encoded `usize`; the mask table has `256` entries, so
the machinery needs every character `< 256` (`AllSmall`).
-/

namespace MyersBits

/-- `pattern` occurs in `text` at index `i`: `text[i..] = pattern ++ _`. -/
def occursAt (text pattern : List Nat) (i : Nat) : Prop :=
  ∃ t, text.drop i = pattern ++ t

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- All characters of `s` are `< 256` (the mask table is byte-indexed). -/
def AllSmall (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x < 256

/-- `occList occ` is the occurrence vector read back as a `Nat` list. -/
def occList (occ : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref occ : Slice Std.Usize) : List Std.Usize)

/-- Bitmask of the pattern positions holding the character `c`: bit `j` is set
    iff `j < p.length` and `p[j] = c`. -/
def maskOf (p : List Nat) (c : Nat) : Nat :=
  (List.range p.length).foldl
    (fun acc j => if p.getD j 0 = c then acc ||| (1 <<< j) else acc) 0

/-- The Shift-And automaton state after reading `w` with pattern `p`:
    `state = ((state << 1) | 1) & mask[c]` for each character `c` of `w`.
    Bit `j` ends up set iff `w` ends with the length-`(j+1)` prefix of `p`. -/
def shiftAndState (p w : List Nat) : Nat :=
  w.foldl (fun st c => ((st <<< 1) ||| 1) &&& maskOf p c) 0

/-! ## Specification theorems -/

/-- `shift_and` reports exactly the occurrence positions of `p` in `t`
    (soundness + completeness).  The hypotheses are the preconditions the
    implementation relies on: a non-empty pattern (`1 ≤ m`) fitting in one
    64-bit word (`m ≤ 64`), no longer than the text, over a byte alphabet. -/
theorem shift_and_correct (t p : Slice Std.Usize) (occ : alloc.vec.Vec Std.Usize)
    (hpos : 1 ≤ (p : List Std.Usize).length)
    (hm : (p : List Std.Usize).length ≤ 64)
    (hlen : (p : List Std.Usize).length ≤ (t : List Std.Usize).length)
    (ht : AllSmall t) (hp : AllSmall p)
    (h : shift_and t p = ok occ) :
    ∀ i, i ∈ occList occ ↔
      occursAt (toNatList (t : List Std.Usize)) (toNatList (p : List Std.Usize)) i := by
  sorry

/-- The Shift-And state invariant — the algorithm's real content: after reading
    `w`, bit `j` of the state is set iff `w` ends with `p.take (j+1)`, i.e. the
    length-`(j+1)` prefix of `p` is a suffix of `w`. -/
theorem shift_and_state_correct (p w : List Nat) (j : Nat) (hj : j < p.length) :
    (shiftAndState p w).testBit j = true ↔ (p.take (j + 1)) <:+ w := by
  sorry

/-- The occurrence list has no duplicates (positions are strictly increasing)
    and every reported position in an in-range match (`i + m ≤ n`). -/
theorem shift_and_nodup (t p : Slice Std.Usize) (occ : alloc.vec.Vec Std.Usize)
    (h : shift_and t p = ok occ) :
    (occList occ).Nodup ∧
      (∀ i ∈ occList occ, i + (p : List Std.Usize).length ≤ (t : List Std.Usize).length) := by
  sorry

end MyersBits
