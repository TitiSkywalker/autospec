import LeanMode.string_lyndon_array.StringLyndonArray
import Mathlib

open Aeneas Aeneas.Std Result
open string_lyndon_array

set_option linter.unusedVariables false

/-!
# Specification: the Lyndon array

`lyndon_array s` returns `L` such that `L[i]` is the length of the *longest*
Lyndon word that is a prefix of `s[i..]`.

A word is Lyndon iff it is nonempty and strictly smaller (lexicographically)
than all of its non-trivial proper suffixes; equivalently, than all of its
proper suffixes with the empty suffix excluded.  Every single character is
Lyndon, so `L[i] ≥ 1` for `i < n`, and the output has exactly `n` entries.

The names `lexLt` / `isLyndon` / `segments` / `covers` / `nonIncreasing` /
`isLyndonFactorization` are copied verbatim from `string_lyndon`'s spec so the
two testcases share the same notion of Lyndon word.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Strict lexicographic order on `List Nat` (the empty word is smallest). -/
def lexLt : List Nat → List Nat → Prop
  | [], [] => False
  | [], _ :: _ => True
  | _ :: _, [] => False
  | a :: xs, b :: ys => a < b ∨ (a = b ∧ lexLt xs ys)

/-- A nonempty word strictly smaller than all its non-trivial proper suffixes. -/
def isLyndon (w : List Nat) : Prop :=
  w ≠ [] ∧ ∀ i, 0 < i → i < w.length → lexLt w (w.drop i)

/-- The length-`l` window starting at position `i` of `s`. -/
def sliceAt (s : List Nat) (i l : Nat) : List Nat := (s.drop i).take l

/-- A `Vec Std.Usize` viewed as a `List Nat`. -/
def vecNat (v : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize)

/-! ## Lyndon-factorization vocabulary (identical to `string_lyndon`) -/

/-- Chunk `s` into consecutive segments of the given lengths. -/
def segments : List Nat → List Nat → List (List Nat)
  | s, [] => []
  | s, l :: ls => (s.take l) :: segments (s.drop l) ls

/-- The segments cover `s` exactly: the lengths sum to `|s|`. -/
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

/-- The segment lengths obtained by repeatedly jumping `i → i + L i`.  `fuel`
    bounds the recursion; `fuel = |s|` suffices because every factor has
    length `≥ 1`. -/
def jumpLens : Nat → List Nat → List Nat → List Nat
  | 0, _, _ => []
  | fuel + 1, s, L =>
      let k := L.getD 0 0
      if k = 0 then [] else k :: jumpLens fuel (s.drop k) (L.drop k)

/-! ## Specification theorems -/

/-- The output of `lyndon_array` has one entry per start position. -/
theorem lyndon_array_length (s : Slice Std.Usize) (L : alloc.vec.Vec Std.Usize)
    (h : lyndon_array s = ok L) :
    (vecNat L).length = (s : List Std.Usize).length := by
  sorry

/-- `L[i]` is exactly the length of the longest Lyndon prefix of `s[i..]`:
    the returned window is Lyndon (so `L[i] ≥ 1`), and no strictly longer
    prefix is. -/
theorem lyndon_array_correct (s : Slice Std.Usize) (L : alloc.vec.Vec Std.Usize)
    (h : lyndon_array s = ok L) :
    ∀ i, i < (s : List Std.Usize).length →
      isLyndon (sliceAt (toNatList (s : List Std.Usize)) i ((vecNat L).getD i 0)) ∧
      (∀ l, (vecNat L).getD i 0 < l → l ≤ (s : List Std.Usize).length - i →
        ¬ isLyndon (sliceAt (toNatList (s : List Std.Usize)) i l)) := by
  sorry

/-- The classic consequence: the Lyndon array determines the Lyndon
    factorization.  Jumping `i → i + L[i]` from `0` chunks `s` into a valid
    Lyndon factorization (whose factors are lexicographically non-increasing).

    Verified in Rust (see the crate's `#[cfg(test)] mod tests`) on every string
    of length `≤ 8` over a 3-letter alphabet and on random strings, where the
    greedy segmentation coincides with Duval's factorization and satisfies all
    three clauses of `isLyndonFactorization`. -/
theorem lyndon_array_greedy_factorization
    (s : Slice Std.Usize) (L : alloc.vec.Vec Std.Usize)
    (h : lyndon_array s = ok L) :
    isLyndonFactorization (toNatList (s : List Std.Usize))
      (jumpLens (s : List Std.Usize).length
        (toNatList (s : List Std.Usize)) (vecNat L)) := by
  sorry
