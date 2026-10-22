import LeanMode.string_suffix_automaton.StringSuffixAutomaton
import Mathlib

open Aeneas Aeneas.Std Result
open string_suffix_automaton

set_option linter.unusedVariables false

/-!
# Specification: Suffix automaton (SAM)

The SAM of a string `s` compresses all of its substrings into a DFA
with `O(n)` states.  `sam_build` constructs it, `sam_contains` recognizes
exactly the substrings of `s`, and `sam_distinct_substrings` counts the number
of distinct nonempty substrings.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- `p` occurs in `s` at index `i`: `s[i..] = p ++ _`. -/
def occursAt (s p : List Nat) (i : Nat) : Prop :=
  ∃ t, s.drop i = p ++ t

/-- `p` is a substring of `s`. -/
def isSubstring (s p : List Nat) : Prop :=
  ∃ i, occursAt s p i

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- Build up prefixes shortest-first, threading the accumulated front. -/
def prefixesAux (acc : List Nat) : List Nat → List (List Nat)
  | [] => []
  | x :: xs => let acc' := acc ++ [x]; acc' :: prefixesAux acc' xs

/-- All nonempty prefixes of `l` (including `l` itself), shortest first. -/
def prefixes (l : List Nat) : List (List Nat) := prefixesAux [] l

/-- All nonempty substrings of `s` (with duplicates), enumerated by start index. -/
def substrings : List Nat → List (List Nat)
  | [] => []
  | l@(_ :: xs) => prefixes l ++ substrings xs

/-! ## Specification theorems -/

/-- The number of states of the SAM of `s` is linear in `|s|` (at most `2|s| + 1`). -/
theorem sam_build_linear (s : Slice Std.Usize) (sam : Sam)
    (h : sam_build s = ok sam) :
    sam.sz.val ≤ 2 * (s : List Std.Usize).length + 1 := by
  sorry

/-- `sam_contains` recognizes exactly the substrings of the input string. -/
theorem sam_contains_correct (s p : Slice Std.Usize) (sam : Sam) (b : Bool)
    (h_bytes : AllBytes s ∧ AllBytes p)
    (h_build : sam_build s = ok sam)
    (h : sam_contains sam p = ok b) :
    b = true ↔ isSubstring (toNatList (s : List Std.Usize)) (toNatList (p : List Std.Usize)) := by
  sorry

/-- `sam_distinct_substrings` counts the number of distinct nonempty substrings. -/
theorem sam_distinct_substrings_correct (s : Slice Std.Usize) (sam : Sam) (k : Std.Usize)
    (h_bytes : AllBytes s)
    (h_build : sam_build s = ok sam)
    (h : sam_distinct_substrings sam = ok k) :
    k.val = (substrings (toNatList (s : List Std.Usize))).eraseDups.length := by
  sorry
