import LeanMode.string_manacher.StringManacher
import Mathlib

open Aeneas Aeneas.Std Result
open string_manacher

set_option linter.unusedVariables false

/-!
# Specification: Manacher's algorithm (odd-length palindromes)

`manacher s` returns `d[i]` such that the longest odd-length palindrome centered
at `i` is `s[i-(d[i]-1) ..= i+(d[i]-1)]` (length `2*d[i]-1`).
-/

/-- A list is a palindrome. -/
def isPalindrome (l : List Nat) : Prop := l = l.reverse

/-- `s[i-(k-1) ..= i+(k-1)]` is a palindrome, for `k ≥ 1` (length `2k-1`). -/
def isCenteredPal (s : List Nat) (i k : Nat) : Prop :=
  0 < k ∧ k ≤ i + 1 ∧ i + k ≤ s.length ∧
    isPalindrome ((s.drop (i - (k - 1))).take (2 * k - 1))

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-! ## Specification theorems -/

/-- `manacher` computes, for each center `i`, the maximal odd-palindrome radius `d[i]`: the radius is maximal unless it reaches either end of the string. -/
theorem manacher_correct (s : Slice Nat) (result : alloc.vec.Vec Std.Usize)
    (h : manacher s = ok result) :
    (alloc.vec.Vec.deref result).length = (s : List Nat).length ∧
    (∀ i, i < (s : List Nat).length →
      let k := (toNatList (alloc.vec.Vec.deref result)).getD i 0
      isCenteredPal (s : List Nat) i k ∧
      (k = i + 1 ∨ i + k = (s : List Nat).length ∨
         ¬ isCenteredPal (s : List Nat) i (k + 1))) := by
  sorry
