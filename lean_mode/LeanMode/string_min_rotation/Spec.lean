import LeanMode.string_min_rotation.StringMinRotation
import Mathlib

open Aeneas Aeneas.Std Result
open string_min_rotation

set_option linter.unusedVariables false

/-!
# Specification: lexicographically smallest cyclic rotation (Booth's algorithm)

`min_rotation s` returns the index `k` of a cyclic rotation of `s` that is
lexicographically minimal among all `n` rotations (`n = |s|`), and `0` for the
empty string.

The rotation starting at `k` is `rotation s k = s.drop k ++ s.take k`; a
rotation is minimal when no other rotation of `s` is strictly smaller than it.

The helpers live in the `MinRotation` namespace because Mathlib already has a
root-level `rotation` (the rotation of the complex `Circle`), so a root-level
`def rotation` would be rejected as "already declared".
-/

namespace MinRotation

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- Strict lexicographic order on `List Nat` (the empty word is smallest). -/
def lexLt : List Nat → List Nat → Prop
  | [], [] => False
  | [], _ :: _ => True
  | _ :: _, [] => False
  | a :: xs, b :: ys => a < b ∨ (a = b ∧ lexLt xs ys)

/-- Non-strict lexicographic order: `a ≤ b` means "`b` is not strictly smaller
    than `a`". -/
def lexLe (a b : List Nat) : Prop := ¬ lexLt b a

/-- The cyclic rotation of `s` starting at index `k`. -/
def rotation (s : List Nat) (k : Nat) : List Nat := s.drop k ++ s.take k

end MinRotation

open MinRotation

/-! ## Specification theorems -/

/-- The returned index is a valid rotation index (in range) — or the input was
    empty, in which case the result is `0`. -/
theorem min_rotation_bound (s : Slice Std.Usize) (k : Std.Usize)
    (h : min_rotation s = ok k) :
    k.val < (s : List Std.Usize).length ∨ (s : List Std.Usize).length = 0 := by
  sorry

/-- The returned rotation is minimal: no other rotation of `s` is
    lexicographically strictly smaller. -/
theorem min_rotation_minimal (s : Slice Std.Usize) (k : Std.Usize)
    (h : min_rotation s = ok k) (hb : AllBytes s) :
    ∀ j, j < (s : List Std.Usize).length →
      lexLe (rotation (toNatList (s : List Std.Usize)) k.val)
            (rotation (toNatList (s : List Std.Usize)) j) := by
  sorry
