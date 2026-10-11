import LeanMode.string_z_algorithm.StringZAlgorithm
import Mathlib

open Aeneas Aeneas.Std Result
open string_z_algorithm

set_option linter.unusedVariables false

/-!
# Specification: Z-algorithm

The Z-array of a byte string `s` is `z[i] = LCP(s, s[i..])`, the length of the
longest common prefix of `s` and its suffix starting at `i`.  In particular
`z[0] = s.length`.
-/

/-- Longest common prefix of two lists. -/
def lcp : List Nat → List Nat → Nat
  | [], _ => 0
  | _, [] => 0
  | a :: xs, b :: ys => if a = b then 1 + lcp xs ys else 0

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-! ## Specification theorems -/

/-- `z_algorithm` computes the Z-array: `z[i]` is the length of the longest common prefix of `s` and `s[i..]`. -/
theorem z_algorithm_correct (s : Slice Nat) (result : alloc.vec.Vec Std.Usize)
    (h : z_algorithm s = ok result) :
    (alloc.vec.Vec.deref result).length = (s : List Nat).length ∧
    (∀ i, i < (s : List Nat).length →
      (toNatList (alloc.vec.Vec.deref result)).getD i 0 =
        lcp (s : List Nat) ((s : List Nat).drop i)) := by
  sorry
