import LeanMode.string_bwt_inverse.StringBwtInverse
import Mathlib

open Aeneas Aeneas.Std Result
open string_bwt_inverse

set_option linter.unusedVariables false

/-!
# Specification: Burrows-Wheeler transform and its inverse

Alphabet convention: input characters are `usize` byte values `1..=255`, and
`0` is a reserved **sentinel** that is strictly smaller than every character
and does not occur in the input.

`bwt_encode s` appends one sentinel (`t = s ++ [0]`), sorts the `|t|` cyclic
rotations of `t` lexicographically, and returns the **last column** of the
resulting matrix.  Because the sentinel occurs exactly once and is a strict
minimum, all rotations are pairwise distinct and the sorted order is unique.

`bwt_decode` inverts it by walking the LF mapping `LF(i) = C[l[i]] + rank_i`
from row `0` of the sorted matrix (the rotation that starts with the sentinel)
and dropping the recovered leading sentinel.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- Every element is a positive byte value: the sentinel `0` never occurs. -/
def CharsPos (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), 1 ≤ x

/-! ## Reference model of the transform -/

/-- The `k`-th cyclic rotation of `t`, i.e. `t[k..] ++ t[..k]`. -/
def rot (t : List Nat) (k : Nat) : List Nat := t.drop k ++ t.take k

/-- The `|t|` cyclic rotations of `t`, in starting-index order. -/
def rotations (t : List Nat) : List (List Nat) :=
  (List.range t.length).map (fun k => rot t k)

/-- Computable lexicographic "`≤`" on words over `Nat` (the empty word is
    least), used to sort the rotation matrix. -/
def lexleB : List Nat → List Nat → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: xs, b :: ys =>
      if a < b then true else if b < a then false else lexleB xs ys

/-- Insert `w` into the lexicographically sorted list `l`, preserving order. -/
def insertSorted (w : List Nat) (l : List (List Nat)) : List (List Nat) :=
  match l with
  | [] => [w]
  | b :: bs => if lexleB w b then w :: b :: bs else b :: insertSorted w bs

/-- The lexicographically sorted list of the cyclic rotations of `t`. -/
def sortedRotations (t : List Nat) : List (List Nat) :=
  (rotations t).foldr insertSorted []

/-- The last column of a list of rotations: the last character of each row. -/
def lastColumn (l : List (List Nat)) : List Nat := l.map (fun r => r.getLastD 0)

/-- The sentinel-padded input, `s ++ [0]`. -/
def withSentinel (s : List Nat) : List Nat := s ++ [0]

/-- The last column of the sorted rotation matrix of `s ++ [0]`, i.e. the
    mathematical value returned by `bwt_encode s`. -/
def bwtReference (s : List Nat) : List Nat :=
  lastColumn (sortedRotations (withSentinel s))

/-! ## Specification theorems -/

/-- The last column has one entry per rotation, so `|bwt_encode s| = |s| + 1`. -/
theorem bwt_encode_length (s : Slice Std.Usize) (l : alloc.vec.Vec Std.Usize)
    (h_enc : bwt_encode s = ok l) :
    (alloc.vec.Vec.deref l).length = (s : List Std.Usize).length + 1 := by
  sorry

/-- The sentinel `0` occurs exactly once in the last column, hence in
    particular it occurs in the output of `bwt_encode`. -/
theorem bwt_encode_has_sentinel (s : Slice Std.Usize) (l : alloc.vec.Vec Std.Usize)
    (h_enc : bwt_encode s = ok l) :
    0 ∈ toNatList (alloc.vec.Vec.deref l) := by
  sorry

/-- `bwt_encode s` is exactly the last column of the lexicographically sorted
    matrix of cyclic rotations of `s ++ [0]` (the selection sort driven by
    `rotation_lt` realises the reference sort; all rotations are distinct
    because the sentinel `0` is unique and strictly smallest). -/
theorem bwt_encode_is_last_column (s : Slice Std.Usize) (l : alloc.vec.Vec Std.Usize)
    (h_pos : CharsPos s)
    (h_enc : bwt_encode s = ok l) :
    toNatList (alloc.vec.Vec.deref l) = bwtReference (toNatList (s : List Std.Usize)) := by
  sorry

/-- The inverse drops exactly the recovered sentinel, so it loses one element. -/
theorem bwt_decode_length (l dec : alloc.vec.Vec Std.Usize)
    (h_dec : bwt_decode (alloc.vec.Vec.deref l) = ok dec) :
    (alloc.vec.Vec.deref dec).length + 1 = (alloc.vec.Vec.deref l).length ∨
    (alloc.vec.Vec.deref l).length = 0 ∧ (alloc.vec.Vec.deref dec).length = 0 := by
  sorry

/-- Round trip: decoding the transform of a sentinel-free string recovers it. -/
theorem bwt_roundtrip (s : Slice Std.Usize) (l dec : alloc.vec.Vec Std.Usize)
    (h_pos : CharsPos s)
    (h_enc : bwt_encode s = ok l)
    (h_dec : bwt_decode (alloc.vec.Vec.deref l) = ok dec) :
    (alloc.vec.Vec.deref dec : List Std.Usize) = (s : List Std.Usize) := by
  sorry
