import LeanMode.array_permutations.ArrayPermutations
import Mathlib

open Aeneas Aeneas.Std Result
open array_permutations

set_option linter.unusedVariables false

/-!
# Specification: generate all permutations of `0..n-1`.

`permutations n` returns a flat list of `n! * n` elements: block `b` holds the
`b`-th permutation of `List.range n`. Every block is a permutation of
`List.range n`, and the blocks are pairwise distinct — so they are exactly the
`n!` permutations of `0..n-1`.
-/

/-- Convert a `Slice Std.Usize` to `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- The `b`-th block of `n` elements from a flat list. -/
def block (l : List Nat) (b n : Nat) : List Nat := (l.drop (b * n)).take n

/-- Correctness of `permutations`: every block is a permutation of
    `List.range n`, and all blocks are pairwise distinct. -/
theorem permutations_correct (n : Std.Usize) (result : alloc.vec.Vec Std.Usize)
    (h : permutations n = ok result) :
    (alloc.vec.Vec.deref result).length = Nat.factorial n.val * n.val ∧
    (∀ b, b < Nat.factorial n.val →
       List.Perm (block (toNatList (alloc.vec.Vec.deref result)) b n.val)
         (List.range n.val)) ∧
    List.Nodup ((List.range (Nat.factorial n.val)).map
       (fun b => block (toNatList (alloc.vec.Vec.deref result)) b n.val)) := by
  sorry
