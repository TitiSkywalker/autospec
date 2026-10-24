import LeanMode.dp_palindrome_partition.DpPalindromePartition
import Mathlib

open Aeneas Aeneas.Std Result
open dp_palindrome_partition

set_option linter.unusedVariables false

/-!
# Specification: minimum palindrome partitioning

`min_palindrome_partition s` returns the minimum number of non-empty, contiguous,
palindromic pieces into which the input string `s` can be partitioned (the pieces
must concatenate back to `s`).

The specification below characterises that number *declaratively*, as the least
number of pieces in a palindromic factorisation — it never mentions the DP array,
the order in which the DP is filled, or the two-pointer palindrome test:

* `IsPalindrome l` says `l` reads the same forwards and backwards;
* `splitsOf s k` is the (finite) set of all ways to write `s` as a concatenation
  of exactly `k` non-empty palindromic pieces;
* `minPalindromicFactorization s` is the least such `k`.

Since `k = s.length` (one piece per character) is always feasible, the set is
never empty and the answer of an empty input is `0` (the empty partition).
-/

/-- Convert a `Slice Std.Usize` to a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

namespace PalindromePartition

/-- A list is a palindrome iff it equals its own reverse. -/
abbrev IsPalindrome (l : List Nat) : Prop := l = l.reverse

/-- `splitsOf s k` is the set of all decompositions of `s` into exactly `k`
non-empty pieces, each of which is a palindrome.  The enumeration is by the
length `t` of the first piece (`1 ≤ t ≤ s.length`): the first piece must be the
palindromic prefix `s.take t`, and the remainder is a decomposition of
`s.drop t` into `k - 1` palindromic pieces. -/
def splitsOf (s : List Nat) (k : Nat) : Finset (List (List Nat)) :=
  match s, k with
  | [], 0 => {[]}
  | _ :: _, 0 => ∅
  | s', k' + 1 =>
      ((Finset.Ico 1 (s'.length + 1)).filter (fun t => IsPalindrome (s'.take t))).biUnion
        (fun t => (splitsOf (s'.drop t) k').image (fun rest => (s'.take t) :: rest))
termination_by k
decreasing_by simp_wf

/-- The minimum number of palindromic pieces in a palindromic factorisation of
`s`: the least `k` for which a decomposition into `k` non-empty palindromic
pieces exists.  (`k = s.length` is always feasible, so the initial bound
`s.length + 1` is never returned.) -/
def minPalindromicFactorization (s : List Nat) : Nat :=
  (List.range (s.length + 1)).foldl
    (fun acc k => if (splitsOf s k).Nonempty then Nat.min acc k else acc)
    (s.length + 1)

end PalindromePartition

open PalindromePartition

/-- Correctness of the minimum-palindrome-partition dynamic program: the
returned count is the least number of non-empty palindromic pieces into which
the input can be partitioned.  The empty input returns `0`. -/
theorem min_palindrome_partition_correct (s : Slice Std.Usize) (k : Std.Usize)
    (h : min_palindrome_partition s = ok k) :
    k.val = minPalindromicFactorization (toNatList s) := by
  sorry
