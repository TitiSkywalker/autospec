import LeanMode.string_palindromic_tree.StringPalindromicTree
import Mathlib

open Aeneas Aeneas.Std Result
open string_palindromic_tree

set_option linter.unusedVariables false

/-!
# Specification: Palindromic tree (eertree)

`eertree_build s` builds the eertree of `s`, a tree with one node per *distinct
palindromic substring* of `s` plus two roots:

* node `0` — the odd root, standing for the imaginary palindrome of length `-1`;
* node `1` — the even root, standing for the empty palindrome of length `0`.

Nodes `v ≥ 2` are the distinct palindromic substrings of `s`, created in order
of first occurrence.  To stay free of signed integers the stored length is the
*shifted* one, `L(v) = actual_len(v) + 1` (`L(0) = 0`, `L(1) = 1`), so the real
palindrome length of node `v ≥ 2` is `L(v) - 1`.  `link[v]` is the node of the
longest proper palindromic suffix of `pal(v)`, and `link[0] = link[1] = 0`.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- `w` reads the same forwards and backwards. -/
abbrev isPalindrome (w : List Nat) : Prop := w.reverse = w

/-- All nonempty palindromic substrings of `s`, enumerated by start index. -/
def palindromicSubstrings (s : List Nat) : List (List Nat) :=
  (List.range s.length).flatMap (fun i =>
    (List.range (s.length - i)).filterMap (fun k =>
      let w := (s.drop i).take (k + 1)
      if isPalindrome w then some w else none))

/-- The distinct nonempty palindromic substrings of `s`. -/
def distinctPalindromes (s : List Nat) : List (List Nat) :=
  (palindromicSubstrings s).eraseDups

/-- `u` is a proper palindromic suffix of `w`. -/
def isProperPalSuffix (w u : List Nat) : Prop :=
  isPalindrome u ∧ u <:+ w ∧ u.length < w.length

/-- Shifted length `L(v) = actual_len(v) + 1` stored for node `v`
    (`0` when the node index is out of range). -/
def shiftLen (t : Eertree) (v : Std.Usize) : Nat :=
  ((alloc.vec.Vec.deref t.len : List Std.Usize).getD v.val 0#usize).val

/-- The suffix link of node `v` (`0` when the node index is out of range). -/
def linkOf (t : Eertree) (v : Std.Usize) : Std.Usize :=
  (alloc.vec.Vec.deref t.link : List Std.Usize).getD v.val 0#usize

/-- Node `v` represents the palindrome `w` of `s` (a member of
    `distinctPalindromes s` of length `shiftLen t v - 1`), and `link[v]` is the
    node of a longest proper palindromic suffix `u` of `w`.  The predicate
    states both directions:

    * *attainment* — `u` really is a proper palindromic suffix of `w`, and its
      node is `link[v]` (`u.length + 1 = shiftLen t (linkOf t v)`);
    * *maximality* — no other proper palindromic suffix of `w` is longer.

    `u` may be the empty palindrome (`u = []`, the proper palindromic suffix of
    a single-character `w`, whose `link` is the even root, node `1`); the other
    members of the disjunction range over the nonempty palindromes of `s`. -/
def IsLongestProperPalSuffix (s : List Nat) (t : Eertree) (v : Std.Usize) : Prop :=
  ∃ w u, w ∈ distinctPalindromes s ∧
         w.length + 1 = shiftLen t v ∧
         (linkOf t v).val < v.val ∧
         (u = [] ∨ u ∈ distinctPalindromes s) ∧
         u.length + 1 = shiftLen t (linkOf t v) ∧
         isProperPalSuffix w u ∧
         (∀ u', (u' = [] ∨ u' ∈ distinctPalindromes s) → isProperPalSuffix w u' →
            u'.length ≤ u.length)

/-! ## Specification theorems -/

/-- `eertree_build` creates exactly one node per distinct nonempty palindromic
    substring of `s`, plus the two roots (`sz = #distinct + 2`). -/
theorem eertree_distinct_correct (s : Slice Std.Usize) (t : Eertree)
    (h : eertree_build s = ok t) (hb : AllBytes s) :
    t.sz.val = (distinctPalindromes (toNatList (s : List Std.Usize))).length + 2 := by
  sorry

/-- The eertree is linear in the input: at most `|s| + 2` nodes. -/
theorem eertree_size_bound (s : Slice Std.Usize) (t : Eertree)
    (h : eertree_build s = ok t) :
    t.sz.val ≤ (s : List Std.Usize).length + 2 := by
  sorry

/-- For every non-root node `v`, `link[v]` is the node of a longest proper
    palindromic suffix of the palindrome of `v`. -/
theorem eertree_link_correct (s : Slice Std.Usize) (t : Eertree) (v : Std.Usize)
    (h : eertree_build s = ok t) (hv : 2 ≤ v.val) (hv' : v.val < t.sz.val)
    (hb : AllBytes s) :
    IsLongestProperPalSuffix (toNatList (s : List Std.Usize)) t v := by
  sorry

/-- The two roots follow the documented convention: the odd root has shifted
    length `0` and the even root has shifted length `1`, and both link to `0`. -/
theorem eertree_roots (s : Slice Std.Usize) (t : Eertree)
    (h : eertree_build s = ok t) (hb : AllBytes s) :
    shiftLen t 0#usize = 0 ∧
    shiftLen t 1#usize = 1 ∧
    linkOf t 0#usize = 0#usize ∧
    linkOf t 1#usize = 0#usize := by
  sorry
