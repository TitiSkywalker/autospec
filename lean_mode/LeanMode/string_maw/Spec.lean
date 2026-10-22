import LeanMode.string_maw.StringMaw
import Mathlib

open Aeneas Aeneas.Std Result
open string_maw

set_option linter.unusedVariables false

/-!
# Specification: Minimal Absent Words (Crochemore–Mignosi–Resta)

A word `w = a·U·b` (with `a`, `b` single characters) is a *minimal absent word*
(MAW) of `s` when `w` does not occur in `s`, while both `a·U` and `U·b` do
occur in `s`; the characters `a` and `b` range over the alphabet of characters
occurring in `s`.

`maw s` returns `ok (data, lens)`, where the minimal absent words are
flat-encoded: the words are concatenated in `data` and `lens[i]` is the length
of the `i`-th word.  `decodeFlat` splits `data` by `lens` back into the list of
words.  The main theorem `maw_correct` states that the decoded list is exactly
the set of minimal absent words of `s` over its own alphabet.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- `p` occurs in `s` starting at index `i`: `s[i..] = p ++ _`. -/
def occursAt (s p : List Nat) (i : Nat) : Prop :=
  ∃ t, s.drop i = p ++ t

/-- `p` is a contiguous factor (substring) of `s`. -/
def isSubstring (s p : List Nat) : Prop :=
  ∃ i, occursAt s p i

/-- Split the flat `(data, lens)` encoding back into the list of words: the
    `i`-th word is the next `lens[i]` characters of `data`. -/
def decodeFlat : List Nat → List Nat → List (List Nat)
  | _, [] => []
  | data, l :: ls => data.take l :: decodeFlat (data.drop l) ls

/-- `w` is a minimal absent word of `s` (over the alphabet of `s`): there are
    characters `a`, `b` **occurring in `s`** and a middle word `U` with
    `w = a :: U ++ [b]`, such that `w` is absent from `s` while both `a·U` and
    `U·b` occur in `s`.  Restricting `a` and `b` to the alphabet of `s` is what
    makes this a finite, meaningful notion — over a fresh character the notion
    would be unsatisfiable from the "absent" side but the "occurs" clauses would
    still exclude it. -/
def IsMaw (s w : List Nat) : Prop :=
  ∃ a U b, a ∈ s ∧ b ∈ s ∧ w = a :: U ++ [b] ∧
    ¬ isSubstring s w ∧ isSubstring s (a :: U) ∧ isSubstring s (U ++ [b])

/-! ## Specification theorems -/

/-- `maw` enumerates exactly the minimal absent words of `s`: the decoded list
    contains `w` iff `w` is a MAW of `s` over the alphabet of `s`. -/
theorem maw_correct (s : Slice Std.Usize) (data lens : alloc.vec.Vec Std.Usize)
    (h : maw s = ok (data, lens))
    (hb : AllBytes s) :
    ∀ w, w ∈ decodeFlat (toNatList ((alloc.vec.Vec.deref data) : List Std.Usize))
                        (toNatList ((alloc.vec.Vec.deref lens) : List Std.Usize)) ↔
         IsMaw (toNatList (s : List Std.Usize)) w := by
  sorry

/-- Every word emitted by `maw` is absent from `s` (it does not occur). -/
theorem maw_no_occurrence (s : Slice Std.Usize) (data lens : alloc.vec.Vec Std.Usize)
    (h : maw s = ok (data, lens))
    (hb : AllBytes s) :
    ∀ w, w ∈ decodeFlat (toNatList ((alloc.vec.Vec.deref data) : List Std.Usize))
                        (toNatList ((alloc.vec.Vec.deref lens) : List Std.Usize)) →
         ¬ isSubstring (toNatList (s : List Std.Usize)) w := by
  sorry

/-- The flat encoding is well-formed: every word has length at least two (it is
    `a·U·b`), and the lengths sum to the total size of `data`. -/
theorem maw_lens_consistent (s : Slice Std.Usize) (data lens : alloc.vec.Vec Std.Usize)
    (h : maw s = ok (data, lens)) :
    (∀ l ∈ toNatList ((alloc.vec.Vec.deref lens) : List Std.Usize), 2 ≤ l) ∧
    (toNatList ((alloc.vec.Vec.deref lens) : List Std.Usize)).sum =
      ((alloc.vec.Vec.deref data) : List Std.Usize).length := by
  sorry
