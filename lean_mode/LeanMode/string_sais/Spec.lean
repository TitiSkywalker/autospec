import LeanMode.string_sais.StringSais
import Mathlib

open Aeneas Aeneas.Std Result
open string_sais

set_option linter.unusedVariables false

/-!
# Specification: suffix array by SA-IS (induced sorting)

`sais s` returns the suffix array of `s`: the permutation of `0..n-1` that lists
the starting indices of the suffixes of `s` in lexicographic order.  Internally
SA-IS appends a sentinel `0` and classifies every position as S- or L-type, but
that machinery is invisible here — the specification only talks about the
concrete returned buffer.

Correctness is stated as three facts about that buffer:

* `sais_perm`    — the returned array is a permutation of `0..n-1`;
* `sais_sorted`  — its entries are in non-decreasing suffix order;
* `sais_correct` — together, that it *is* the suffix array;

plus `sais_unique`, that the suffix array so characterised is unique, so the
returned buffer is forced.

The alphabet convention is a hypothesis: every character is a byte value
(`AllBytes`: `≤ 255`) and none is the reserved sentinel (`CharsPos`: `≥ 1`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element of a byte string is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- Every element of the string is a real character, i.e. none is the reserved
    sentinel `0`. -/
def CharsPos (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), 1 ≤ x

/-- A `Vec Std.Usize` as a `List Nat`. -/
def vecNat (v : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize)

/-- `sa` is a permutation of `0..n-1`. -/
def IsPerm (sa : List Nat) (n : Nat) : Prop :=
  sa.length = n ∧ sa.Nodup ∧ ∀ i, i < n → i ∈ sa

/-- Strict lexicographic order on `List Nat` (the empty word is smallest). -/
def lexLt : List Nat → List Nat → Prop
  | [], [] => False
  | [], _ :: _ => True
  | _ :: _, [] => False
  | a :: xs, b :: ys => a < b ∨ (a = b ∧ lexLt xs ys)

/-- Non-strict lexicographic order. -/
def lexLe (a b : List Nat) : Prop := ¬ lexLt b a

/-- The suffix of `s` starting at `a` is lexicographically ≤ the one at `b`. -/
def leSuffix (s : List Nat) (a b : Nat) : Prop := lexLe (s.drop a) (s.drop b)

/-- `sa` is the suffix array of `s`: a permutation of `0..n-1` listing the
    starting indices of the suffixes in lexicographic order. -/
def IsSuffixArray (s : List Nat) (sa : List Nat) : Prop :=
  IsPerm sa s.length ∧ ∀ i j, i < j → j < sa.length → leSuffix s (sa.getD i 0) (sa.getD j 0)

/-! ## Specification theorems -/

/-- `sais` returns a permutation of the starting indices `0..n-1`. -/
theorem sais_perm (s : Slice Std.Usize) (sa : alloc.vec.Vec Std.Usize)
    (h : sais s = ok sa) (hb : AllBytes s) (hp : CharsPos s) :
    IsPerm (vecNat sa) ((s : List Std.Usize)).length := by
  sorry

/-- The returned indices list the suffixes of `s` in lexicographic order. -/
theorem sais_sorted (s : Slice Std.Usize) (sa : alloc.vec.Vec Std.Usize)
    (h : sais s = ok sa) (hb : AllBytes s) (hp : CharsPos s) :
    ∀ i j, i < j → j < (vecNat sa).length →
      leSuffix (toNatList (s : List Std.Usize)) ((vecNat sa).getD i 0) ((vecNat sa).getD j 0) := by
  sorry

/-- Together, the previous two facts say that `sais` computes the suffix array. -/
theorem sais_correct (s : Slice Std.Usize) (sa : alloc.vec.Vec Std.Usize)
    (h : sais s = ok sa) (hb : AllBytes s) (hp : CharsPos s) :
    IsSuffixArray (toNatList (s : List Std.Usize)) (vecNat sa) := by
  sorry

/-- The suffix array is unique: any buffer satisfying `IsSuffixArray` for `s`
    is the one `sais` returns.  `IsPerm` is needed for uniqueness — without it a
    list could satisfy the sortedness clause vacuously (e.g. by being empty). -/
theorem sais_unique (s : Slice Std.Usize) (sa sa' : alloc.vec.Vec Std.Usize)
    (h : sais s = ok sa) (hb : AllBytes s) (hp : CharsPos s)
    (h' : IsSuffixArray (toNatList (s : List Std.Usize)) (vecNat sa')) :
    vecNat sa' = vecNat sa := by
  sorry
