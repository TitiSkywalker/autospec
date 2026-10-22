import LeanMode.string_suffix_array.StringSuffixArray
import Mathlib

open Aeneas Aeneas.Std Result
open string_suffix_array

set_option linter.unusedVariables false

/-!
# Specification: suffix array (prefix doubling) + Kasai LCP

`build_sa s` returns the suffix array of `s` — the permutation of `0..n-1` that
lists the starting indices of the suffixes of `s` in lexicographic order — to-
gether with the LCP array of Kasai's algorithm, and `kasai_lcp s sa` recomputes
the LCP array for a given suffix array `sa`.

Correctness is stated as three facts about the concrete returned buffers:

* `suffix_array_is_perm` — the returned array is a permutation of `0..n-1`;
* `suffix_array_sorted`  — its entries are in non-decreasing suffix order;
* `suffix_array_lcp_correct` — every `lcp[i]` is exactly the length of the
  longest common prefix of the adjacent suffixes.

Characters are `usize`-encoded bytes in `0..=255` (`AllBytes`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element of a byte string is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- The suffix array carried by a result, as a `List Nat`. -/
def saList (r : SaResult) : List Nat :=
  toNatList ((alloc.vec.Vec.deref r.sa : Slice Std.Usize) : List Std.Usize)

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

/-- Length of the longest common prefix of two words. -/
def lcpLen : List Nat → List Nat → Nat
  | [], _ => 0
  | _, [] => 0
  | x :: xs, y :: ys => if x = y then 1 + lcpLen xs ys else 0

/-- `sa` is the suffix array of `s`: a permutation in non-decreasing suffix
    order. -/
def IsSuffixArray (s : List Nat) (sa : List Nat) : Prop :=
  IsPerm sa s.length ∧ ∀ i j, i < j → j < sa.length → leSuffix s (sa.getD i 0) (sa.getD j 0)

/-! ## Specification theorems -/

/-- `build_sa` returns a permutation of the starting indices `0..n-1`. -/
theorem suffix_array_is_perm (s : Slice Std.Usize) (r : SaResult)
    (h : build_sa s = ok r) (hb : AllBytes s) :
    IsPerm (saList r) ((s : List Std.Usize)).length := by
  sorry

/-- The returned indices list the suffixes of `s` in lexicographic order. -/
theorem suffix_array_sorted (s : Slice Std.Usize) (r : SaResult)
    (h : build_sa s = ok r) (hb : AllBytes s) :
    ∀ i j, i < j → j < (saList r).length →
      leSuffix (toNatList (s : List Std.Usize)) ((saList r).getD i 0) ((saList r).getD j 0) := by
  sorry

/-- Together, the previous two facts say that `build_sa` computes the suffix
    array. -/
theorem suffix_array_correct (s : Slice Std.Usize) (r : SaResult)
    (h : build_sa s = ok r) (hb : AllBytes s) :
    IsSuffixArray (toNatList (s : List Std.Usize)) (saList r) := by
  sorry

/-- Every `lcp[i]` (for `i + 1 < n`) is the length of the longest common
    prefix of the two adjacent suffixes `sa[i]` and `sa[i + 1]`.

    Both hypotheses on `sa` are needed: it must be a permutation of `0..n-1`
    (Kasai builds `rankpos` by scattering `sa`, so a repeated or out-of-range
    index silently overwrites entries and yields garbage), *and* it must be in
    non-decreasing suffix order (the `h - 1` amortisation is only valid for
    adjacent suffixes of the true suffix array).  Sortedness alone is not
    enough: on `s = [97, 98]`, `sa = [0, 0]` is trivially "sorted" but
    `kasai_lcp` returns `[0, 0]` instead of `[2, 0]`. -/
theorem suffix_array_lcp_correct (s : Slice Std.Usize) (sa : Slice Std.Usize)
    (lcp : alloc.vec.Vec Std.Usize)
    (h : kasai_lcp s sa = ok lcp)
    (hperm : IsPerm (toNatList (sa : List Std.Usize)) (s : List Std.Usize).length)
    (hsa : ∀ i, i + 1 < (sa : List Std.Usize).length →
      leSuffix (toNatList (s : List Std.Usize)) (((sa : List Std.Usize).getD i 0#usize).val)
        (((sa : List Std.Usize).getD (i + 1) 0#usize).val))
    (hb : AllBytes s) :
    ∀ i, i + 1 < (sa : List Std.Usize).length →
      (vecNat lcp).getD i 0 =
        lcpLen ((toNatList (s : List Std.Usize)).drop (((sa : List Std.Usize).getD i 0#usize).val))
               ((toNatList (s : List Std.Usize)).drop (((sa : List Std.Usize).getD (i + 1) 0#usize).val)) := by
  sorry

/-- The last entry of a nonempty LCP array is `0`. -/
theorem lcp_last_zero (s : Slice Std.Usize) (sa : Slice Std.Usize)
    (lcp : alloc.vec.Vec Std.Usize)
    (h : kasai_lcp s sa = ok lcp) (hn : 0 < (sa : List Std.Usize).length) :
    (vecNat lcp).getD ((sa : List Std.Usize).length - 1) 0 = 0 := by
  sorry
