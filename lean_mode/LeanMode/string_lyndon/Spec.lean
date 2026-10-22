import LeanMode.string_lyndon.StringLyndon
import Mathlib

open Aeneas Aeneas.Std Result
open string_lyndon

set_option linter.unusedVariables false

/-!
# Specification: Lyndon factorization (Duval's algorithm)

`lyndon_factorization s` returns the lengths of the Lyndon factors of `s`: a
partition `s = w₁·w₂·…·w_k` where each `w_i` is a Lyndon word and
`w₁ ≥ w₂ ≥ … ≥ w_k` lexicographically.  Such a factorization exists and is
unique.
-/

/-- Strict lexicographic order on `List Nat` (the empty word is smallest). -/
def lexLt : List Nat → List Nat → Prop
  | [], [] => False
  | [], _ :: _ => True
  | _ :: _, [] => False
  | a :: xs, b :: ys => a < b ∨ (a = b ∧ lexLt xs ys)

/-- A nonempty word strictly smaller than all its non-trivial proper suffixes. -/
def isLyndon (w : List Nat) : Prop :=
  w ≠ [] ∧ ∀ i, 0 < i → i < w.length → lexLt w (w.drop i)

/-- Chunk `s` into consecutive segments of the given lengths. -/
def segments : List Nat → List Nat → List (List Nat)
  | s, [] => []
  | s, l :: ls => (s.take l) :: segments (s.drop l) ls

/-- The segments cover `s` exactly: the lengths sum to `|s|` (which forces the
    chunked segments to concatenate back to `s`). -/
def covers (s : List Nat) (lens : List Nat) : Prop :=
  lens.sum = s.length

/-- Consecutive segments are lexicographically non-increasing (`w_i ≥ w_{i+1}`). -/
def nonIncreasing (s : List Nat) (lens : List Nat) : Prop :=
  let segs := segments s lens
  ∀ i, i + 1 < segs.length → ¬ lexLt (segs.getD i []) (segs.getD (i + 1) [])

/-- `lens` is a valid Lyndon factorization of `s`. -/
def isLyndonFactorization (s : List Nat) (lens : List Nat) : Prop :=
  covers s lens ∧
  (∀ w ∈ segments s lens, isLyndon w) ∧
  nonIncreasing s lens

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-! ## Specification theorems -/

/-- `lyndon_factorization` returns the (unique) Lyndon factorization of `s`. -/
theorem lyndon_factorization_correct (s : Slice Nat) (result : alloc.vec.Vec Std.Usize)
    (h : lyndon_factorization s = ok result) :
    isLyndonFactorization (s : List Nat) (toNatList (alloc.vec.Vec.deref result)) ∧
    (∀ lens', isLyndonFactorization (s : List Nat) lens' →
        lens' = toNatList (alloc.vec.Vec.deref result)) := by
  sorry
