import LeanMode.string_fm_index.StringFmIndex
import Mathlib

open Aeneas Aeneas.Std Result
open string_fm_index

set_option linter.unusedVariables false

/-!
# Specification: FM-index (Ferragina–Manzini) backward search

Alphabet convention: input characters are `usize` byte values `1..=255`, and
`0` is a reserved **sentinel** strictly smaller than every character that never
occurs in the input.

`fm_build s` computes the Burrows–Wheeler transform of `t = s ++ [0]` (the
selection sort of the cyclic rotations of `t`, exactly as `string_bwt_inverse`
does), the first-column table `c` and the flat occurrence table `occ`, so that
it returns an index `idx` with

* `idx.bwt[i]` = the last column of the sorted rotation matrix of `t`,
* `idx.c[ch]` = the number of BWT characters strictly smaller than `ch`, and
* `idx.occ[i * 256 + ch]` = the number of occurrences of `ch` in `bwt[0 .. i]`.

`fm_backward_search idx p` walks the pattern right to left and returns the
**half-open** interval `[l, r)` of the sorted order of the suffixes of `t` that
start with `p`.  Because `p` is built from characters `≥ 1` while `t` ends in
the unique sentinel `0`, such a suffix is a suffix of `s` and the interval is
exactly the set of occurrences of `p` in `s`, so `r - l` counts them.

**Empty pattern.**  The suffix array of `t` has one row more than the suffix
array of `s` (the extra row is the bare sentinel suffix `[0]`), so the raw
backward-search interval for the empty pattern would be `[0, |s| + 1)` and
would over-count by one.  The implementation therefore reports `[0, |s|)` for
the empty pattern — the `|s|` suffixes of `s`, which is what
`suffixesPrefixedBy` counts (`fm_empty_pattern`).  For every non-empty pattern
with characters in `1..=255` the extra row is never selected, so the interval
is the ordinary FM-index interval.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`): the 256-wide tables are in range. -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- Every element is a positive byte value: the sentinel `0` never occurs. -/
def CharsPos (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), 1 ≤ x

/-! ## Words, prefixes and the reference sorted order -/

/-- Computable "`p` is a prefix of `w`". -/
def isPrefB : List Nat → List Nat → Bool
  | [], _ => true
  | _ :: _, [] => false
  | a :: xs, b :: ys => if a = b then isPrefB xs ys else false

/-- `p` is a prefix of `w` (the `Prop` version of `isPrefB`). -/
def IsPref : List Nat → List Nat → Prop
  | [], _ => True
  | _ :: _, [] => False
  | a :: xs, b :: ys => a = b ∧ IsPref xs ys

/-- The starting positions of the suffixes of `s` that begin with `p`, in
    increasing order.  For the empty pattern this is all of `0 .. |s| - 1`,
    i.e. it has `s.length` elements — the number of suffixes of `s`. -/
def suffixesPrefixedBy (s p : List Nat) : List Nat :=
  (List.range s.length).filter (fun i => isPrefB p (s.drop i))

/-- Computable strict lexicographic order on words (the empty word is least). -/
def lexLtW : List Nat → List Nat → Bool
  | [], [] => false
  | [], _ :: _ => true
  | _ :: _, [] => false
  | a :: xs, b :: ys =>
      if a < b then true else if b < a then false else lexLtW xs ys

/-- Insert the index `i` into the list `l` of indices that is already sorted by
    the suffixes of `w`, preserving the order. -/
def insertIdxW (w : List Nat) (i : Nat) (l : List Nat) : List Nat :=
  match l with
  | [] => [i]
  | j :: js =>
      if lexLtW (w.drop i) (w.drop j) then i :: j :: js
      else j :: insertIdxW w i js

/-- The indices `0 .. |w| - 1` sorted by the lexicographic order of the
    suffixes `w.drop ·` (all distinct, since suffixes of a word have distinct
    lengths). -/
def sortedIdxW (w : List Nat) : List Nat :=
  (List.range w.length).foldr (fun i acc => insertIdxW w i acc) []

/-- The suffix array of `t = s ++ [0]`, i.e. the **global sorted order** that
    the FM-index of `s` indexes into.

    Sorting the cyclic rotations of `t` (as `fm_build` does) and sorting the
    suffixes of `t` give the same order here: because `t` ends in the unique
    minimum `0`, a suffix of `t` is never a proper prefix of another one, so
    the first differing position of two rotations is already a position of the
    two corresponding suffixes. -/
def saRefT (s : List Nat) : List Nat := sortedIdxW (s ++ [0])

/-- The `k`-th suffix in the global sorted order of the suffixes of `s ++ [0]`
    (the word the FM-index of `s` ranks). -/
def suffixAt (s : List Nat) (k : Nat) : List Nat :=
  (s ++ [0]).drop ((saRefT s).getD k 0)

/-! ## Specification theorems -/

/-- **Counting.**  The width of the returned half-open interval is exactly the
    number of occurrences of the pattern in the text, i.e. the number of
    suffixes of `s` that start with `p`.

    The hypotheses are the ones the implementation really needs: `s` and `p`
    consist of characters `1..=255` (`CharsPos` keeps the reserved sentinel `0`
    out of the input — otherwise `p = [0]` would match the extra sentinel
    suffix of `t` and the count would be off by one — and `AllBytes` keeps
    every table index inside the 256-wide `c`/`occ` tables).  The interval is
    stated half-open so that `r.val - l.val` is the count and never underflows;
    `l.val = r.val` is the "no occurrence" case. -/
theorem fm_count_correct
    (s p : Slice Std.Usize) (idx : FmIndex) (l r : Std.Usize)
    (h_pos_s : CharsPos s) (h_bytes_s : AllBytes s)
    (h_pos_p : CharsPos p) (h_bytes_p : AllBytes p)
    (h_build : fm_build s = ok idx)
    (h_search : fm_backward_search idx p = ok (l, r)) :
    r.val - l.val =
      (suffixesPrefixedBy (toNatList (s : List Std.Usize))
        (toNatList (p : List Std.Usize))).length := by
  sorry

/-- **Interval (soundness).**  Every row `i` of the returned interval, viewed
    as a rank in the global sorted order `saRefT` of the suffixes of `s ++ [0]`,
    is a suffix that starts with `p`.

    The empty pattern is the one case where the interval is *not* the full set
    of `saRefT`-rows starting with `p`: `[0, |s|)` deliberately drops the extra
    sentinel suffix (see the module comment).  The statement below is still
    true there — every row of `[0, |s|)` is a genuine suffix of `s ++ [0]` and
    every word starts with the empty pattern.  For the converse (no occurrence
    is missed) see `fm_interval_exact`. -/
theorem fm_interval_correct
    (s p : Slice Std.Usize) (idx : FmIndex) (l r : Std.Usize)
    (h_pos_s : CharsPos s) (h_bytes_s : AllBytes s)
    (h_pos_p : CharsPos p) (h_bytes_p : AllBytes p)
    (h_build : fm_build s = ok idx)
    (h_search : fm_backward_search idx p = ok (l, r)) :
    ∀ i, l.val ≤ i → i < r.val →
      i < (saRefT (toNatList (s : List Std.Usize))).length ∧
      IsPref (toNatList (p : List Std.Usize))
        (suffixAt (toNatList (s : List Std.Usize)) i) := by
  sorry

/-- **Interval (completeness / exactness).**  For a *non-empty* pattern the
    returned interval is exactly the set of ranks whose suffix starts with `p`:
    `k ∈ [l, r)` iff the `k`-th suffix in the sorted order of the suffixes of
    `s ++ [0]` starts with `p`.  So no occurrence of `p` in `s` is missed.

    Non-emptiness is required: for the empty pattern every one of the `|s| + 1`
    rows starts with `p`, while the implementation deliberately reports
    `[0, |s|)` (`fm_empty_pattern`).  `CharsPos p` is required as well, since a
    pattern containing the sentinel `0` would also match the sentinel suffix. -/
theorem fm_interval_exact
    (s p : Slice Std.Usize) (idx : FmIndex) (l r : Std.Usize)
    (h_pos_s : CharsPos s) (h_bytes_s : AllBytes s)
    (h_pos_p : CharsPos p) (h_bytes_p : AllBytes p)
    (h_nonempty : 0 < (p : List Std.Usize).length)
    (h_build : fm_build s = ok idx)
    (h_search : fm_backward_search idx p = ok (l, r)) :
    ∀ k, k < (saRefT (toNatList (s : List Std.Usize))).length →
      (l.val ≤ k ∧ k < r.val ↔
        IsPref (toNatList (p : List Std.Usize))
          (suffixAt (toNatList (s : List Std.Usize)) k)) := by
  sorry

/-- **Empty pattern.**  The empty pattern is reported as `[0, s.len())`: the
    whole suffix array of `s` (the extra sentinel suffix of `s ++ [0]` is
    dropped), which is exactly the `s.len()` suffixes of `s`. -/
theorem fm_empty_pattern
    (idx : FmIndex) (p : Slice Std.Usize) (l r : Std.Usize)
    (h_len : (p : List Std.Usize).length = 0)
    (h_search : fm_backward_search idx p = ok (l, r)) :
    l.val = 0 ∧ r.val = idx.n.val := by
  sorry

/-- **Well-formedness.**  The returned interval is ordered (`l ≤ r`, so the
    count `r - l` never underflows) and lies inside the suffix array of
    `s ++ [0]`, whose length is `s.len() + 1 = idx.n + 1`. -/
theorem fm_range_ordered
    (s p : Slice Std.Usize) (idx : FmIndex) (l r : Std.Usize)
    (h_build : fm_build s = ok idx)
    (h_search : fm_backward_search idx p = ok (l, r)) :
    l.val ≤ r.val ∧ r.val ≤ idx.n.val + 1 := by
  sorry
