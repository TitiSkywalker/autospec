import LeanMode.string_lz_end.StringLzEnd
import Mathlib

open Aeneas Aeneas.Std Result
open string_lz_end

set_option linter.unusedVariables false

/-!
# Specification: LZ-End factorization (greedy)

`lz_end s` greedily factors `s` into a list of `(offset, length)` pairs:

* `offset = 0` is a literal factor of length `1`, emitting `s[pos]`;
* `offset > 0` is a copy factor, emitting the `length` characters starting at
  `s[pos - offset]`.

The variant implemented is the *LZ-End* parsing of Kreft & Navarro (2010).  At
position `pos` (the number of characters emitted so far) a copy source must be a
contiguous substring `s[pos - offset .. pos - offset + length]` of the parsed
prefix that **ends at a factor boundary**, i.e. at the end position of one of the
factors emitted so far.  The set of boundaries is `{0} ∪ {cumulative sums of the
emitted factor lengths}`, so the current position `pos` itself is a boundary (it
is the end of the last emitted factor).  Because the source end is `≤ pos`, a
copy factor never overlaps the position it is written to (`length ≤ offset`),
exactly as in non-overlapping LZ77; the extra LZ-End restriction is that the
source *end* must be a boundary rather than an arbitrary position `< pos`.

## What is (and is not) proved here

`lz_end_valid` says the emitted list is a valid LZ-End factorization (the factors
tile `s`, are positive, and every copy is sourced and ends at a boundary), and
`lz_end_maximal` says each factor is the longest admissible copy (literals
appear only where no copy of length `≥ 1` exists).

**There is deliberately no optimality theorem.**  Greedy LZ-End is *not* optimal
among valid LZ-End factorizations: choosing the longest factor can skip a
position that an alternative factorization would use as a boundary.  The
smallest counterexample is `s = [0,0,1,0,0,0,1,0]`, where the greedy output has
`6` factors,

  `[(0,1),(1,1),(0,1),(3,2),(4,2),(7,1)]`,

while `[(0,1),(0,1),(0,1),(0,1),(4,4)]` is a valid LZ-End factorization with only
`5` factors (its four literals put a boundary at position `4`, whose source
`s[0..4) = [0,0,1,0]` matches `s[4..8) = [0,0,1,0]`).  This was found by
brute-force enumeration of *all* valid factorizations of all binary strings up to
length `11` (and is asserted in the untranslated `#[cfg(test)] mod tests` of
`src/lib.rs`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- View the returned vector as a list of `(offset, length)` pairs. -/
def factorsOf (fs : alloc.vec.Vec LzEndFactor) : List (Nat × Nat) :=
  ((alloc.vec.Vec.deref fs) : List LzEndFactor).map
    (fun f => (f.offset.val, f.length.val))

/-- Starting from position `pos`, concatenate the segment produced by each
    factor: a literal factor `(0, m)` yields `s.take m` from `pos`, a copy
    factor `(o, m)` (with `o > 0`) yields the `m` characters at `s[pos - o]`. -/
def replay (s : List Nat) : Nat → List (Nat × Nat) → List Nat
  | _, [] => []
  | pos, f :: fs =>
      let seg := if f.1 = 0 then (s.drop pos).take f.2
                 else (s.drop (pos - f.1)).take f.2
      seg ++ replay s (pos + f.2) fs

/-- `insideLzEnd s pos bs fs` holds when `fs` is a list of factors starting at
    position `pos`, with `bs` the factor boundaries of the preceding factors
    (`bs` begins as `[0]` and gains `pos + f.2` after each factor).  Every factor
    must satisfy the implementation's inner-loop guards:

    * a literal factor has offset `0` and length exactly `1`;
    * a copy factor `(o, l)` has `o > 0`, starts inside the parsed prefix
      (`o ≤ pos`) and stays **non-overlapping** (`l ≤ o`, so the source
      `s[pos - o .. pos - o + l)` lies in `[0, pos)`), matches `s[pos .. pos + l)`,
      and its source **ends at a factor boundary**: `pos - o + l ∈ bs`.

    The last conjunct is the LZ-End restriction (mirroring `lz77`'s
    non-overlap clause `f.2 ≤ f.1`, which is the special case obtained when the
    only boundary allowed were `pos`), and it is what makes the predicate
    faithful to the algorithm's `match_at_boundary` guard. -/
def insideLzEnd (s : List Nat) : Nat → List Nat → List (Nat × Nat) → Prop
  | _, _, [] => True
  | pos, bs, f :: fs =>
      (f.1 = 0 → f.2 = 1) ∧
      (f.1 ≠ 0 → f.1 ≤ pos ∧ f.2 ≤ f.1 ∧ (pos - f.1 + f.2) ∈ bs ∧
          (s.drop (pos - f.1)).take f.2 = (s.drop pos).take f.2) ∧
      insideLzEnd s (pos + f.2) (bs ++ [pos + f.2]) fs

/-- `IsLzEndFactorization s fs` holds when `fs` is a legitimate LZ-End parsing
    of `s`: every factor is nonempty (`1 ≤ length`), every factor is a
    legitimately sourced factor (`insideLzEnd` from position `0` with the initial
    boundary list `[0]`), and replaying the factors from position `0`
    reconstructs `s` exactly. -/
def IsLzEndFactorization (s : List Nat) (fs : List (Nat × Nat)) : Prop :=
  (∀ f ∈ fs, 1 ≤ f.2) ∧
  insideLzEnd s 0 [0] fs ∧
  replay s 0 fs = s

/-- `IsLongestEndMatch s pos bs l` holds when `l` is the length of a longest
    copy admissible at `pos` with boundaries `bs`: there is a boundary `B ≤ pos`
    admitting a source `s[B - l .. B)` equal to `s[pos .. pos + l)`, and no
    boundary admits a match of any length `l' > l`.  (For `l = 0` the existence
    part holds trivially, so the predicate then says that no copy of length
    `≥ 1` exists — which is exactly when the implementation emits a literal.) -/
def IsLongestEndMatch (s : List Nat) (pos : Nat) (bs : List Nat) (l : Nat) : Prop :=
  (∃ B, B ∈ bs ∧ B ≤ pos ∧ l ≤ B ∧ pos + l ≤ s.length ∧
      (s.drop (B - l)).take l = (s.drop pos).take l) ∧
  ∀ B l', B ∈ bs → B ≤ pos → l' ≤ B → pos + l' ≤ s.length →
      (s.drop (B - l')).take l' = (s.drop pos).take l' → l' ≤ l

/-- Every factor of the list is a longest admissible copy at the position where
    it starts: a copy factor (`offset ≠ 0`) has maximal length `IsLongestEndMatch
    s pos bs f.2`, and a literal factor (`offset = 0`) is emitted only where no
    copy of length `≥ 1` exists (`IsLongestEndMatch s pos bs 0`).  `bs` threads
    the factor boundaries. -/
def allMaximal (s : List Nat) : Nat → List Nat → List (Nat × Nat) → Prop
  | _, _, [] => True
  | pos, bs, f :: fs =>
      (f.1 = 0 → IsLongestEndMatch s pos bs 0) ∧
      (f.1 ≠ 0 → IsLongestEndMatch s pos bs f.2) ∧
      allMaximal s (pos + f.2) (bs ++ [pos + f.2]) fs

/-! ## Specification theorems -/

/-- `lz_end` returns a valid LZ-End factorization of `s`: the factors are
    nonempty, every copy factor starts inside the already parsed prefix, is
    non-overlapping, and its source ends at a factor boundary, and replaying the
    factors reconstructs `s`. -/
theorem lz_end_valid (s : Slice Std.Usize) (fs : alloc.vec.Vec LzEndFactor)
    (h : lz_end s = ok fs) :
    IsLzEndFactorization (toNatList (s : List Std.Usize)) (factorsOf fs) := by
  sorry

/-- The factors produced by `lz_end` replay to `s` (the reconstruction part of
    `lz_end_valid`, stated on its own). -/
theorem lz_end_factors_reconstruct (s : Slice Std.Usize)
    (fs : alloc.vec.Vec LzEndFactor) (h : lz_end s = ok fs) :
    replay (toNatList (s : List Std.Usize)) 0 (factorsOf fs)
      = toNatList (s : List Std.Usize) := by
  sorry

/-- Every factor produced by `lz_end` is the longest admissible copy at the
    position where it starts; literals appear only where no copy of length
    `≥ 1` exists. -/
theorem lz_end_maximal (s : Slice Std.Usize) (fs : alloc.vec.Vec LzEndFactor)
    (h : lz_end s = ok fs) :
    allMaximal (toNatList (s : List Std.Usize)) 0 [0] (factorsOf fs) := by
  sorry
