import LeanMode.string_lz77.StringLz77
import Mathlib

open Aeneas Aeneas.Std Result
open string_lz77

set_option linter.unusedVariables false

/-!
# Specification: LZ77 factorization (greedy, non-overlapping)

`lz77 s` greedily factors `s` into a list of `(offset, length)` pairs:

* `offset = 0` is a literal factor of length `1`, emitting `s[pos]`;
* `offset > 0` is a copy factor, emitting the `length` characters starting at
  `s[pos - offset]`.

The variant implemented here is *non-overlapping*: a copy factor starting at
position `pos` may only read from positions `< pos`, i.e. from the part of the
string that has already been emitted.  The factors therefore always reconstruct
`s` (`IsValidFactorization`), the greedy decomposition is optimal among valid
factorizations (`lz77_optimal`), and every copy factor is a longest possible
match at its position (`lz77_maximal`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- View the returned vector as a list of `(offset, length)` pairs. -/
def factorsOf (fs : alloc.vec.Vec LzFactor) : List (Nat × Nat) :=
  ((alloc.vec.Vec.deref fs) : List LzFactor).map
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

/-- `l` is the length of a longest allowed match for the tail `s[pos..]`:
    some earlier position `q < pos` agrees with `pos` on `l` characters without
    overlapping `pos` (`q + l ≤ pos`) or running past the end (`pos + l ≤ |s|`),
    and no earlier position allows a longer such match. -/
def IsLongestMatch (s : List Nat) (pos l : Nat) : Prop :=
  (∃ q, q < pos ∧ q + l ≤ pos ∧ pos + l ≤ s.length ∧
      (s.drop q).take l = (s.drop pos).take l) ∧
  ∀ q l', q < pos → q + l' ≤ pos → pos + l' ≤ s.length →
      (s.drop q).take l' = (s.drop pos).take l' → l' ≤ l

/-- Every factor of the list points strictly into the history emitted before
    it: `insideHistory pos fs` requires the first factor's offset to be at most
    `pos` (the number of characters emitted so far), a copy factor to stay
    **non-overlapping** (`f.2 ≤ f.1`, so the copied source
    `[pos - f.1, pos - f.1 + f.2)` lies entirely inside `[0, pos)`) and recurses
    with the position advanced by that factor's length.

    The non-overlap clause is essential: without it a copy factor could read
    characters it is itself producing (e.g. `"aaa"` would factor as
    `[(0,1), (1,2)]`), which is not what `lz77` computes and would break
    `lz77_optimal`. -/
def insideHistory : Nat → List (Nat × Nat) → Prop
  | _, [] => True
  | pos, f :: fs => f.1 ≤ pos ∧ (f.1 ≠ 0 → f.2 ≤ f.1) ∧ insideHistory (pos + f.2) fs

/-- Every factor of the list is a longest allowed match at the position where
    it starts (literals, `offset = 0`, are exempt). -/
def allMaximal (s : List Nat) : Nat → List (Nat × Nat) → Prop
  | _, [] => True
  | pos, f :: fs =>
      (f.1 ≠ 0 → IsLongestMatch s pos f.2) ∧ allMaximal s (pos + f.2) fs

/-- `IsValidFactorization s fs` holds when `fs` is a legitimate way of writing
    `s` as concatenated factors: every factor is nonempty, a literal factor has
    length exactly one, every copy factor points strictly into the part of the
    string already emitted and does not overlap the current position
    (`length ≤ offset ≤ length of history`), and replaying the factors from
    position `0` reconstructs `s` exactly. -/
def IsValidFactorization (s : List Nat) (fs : List (Nat × Nat)) : Prop :=
  (∀ f ∈ fs, 1 ≤ f.2 ∧ (f.1 = 0 → f.2 = 1)) ∧
  insideHistory 0 fs ∧
  replay s 0 fs = s

/-! ## Specification theorems -/

/-- `lz77` returns a valid factorization of `s`: the factors are nonempty, copy
    factors read only from the already emitted history, and replaying them
    reconstructs `s`. -/
theorem lz77_valid (s : Slice Std.Usize) (fs : alloc.vec.Vec LzFactor)
    (h : lz77 s = ok fs) :
    IsValidFactorization (toNatList (s : List Std.Usize)) (factorsOf fs) := by
  sorry

/-- `lz77` is optimal: its factor list is at least as short as any other valid
    factorization of the same string (greedy taking the longest match at each
    position minimises the number of factors). -/
theorem lz77_optimal (s : Slice Std.Usize) (fs : alloc.vec.Vec LzFactor)
    (gs : List (Nat × Nat))
    (h : lz77 s = ok fs)
    (hg : IsValidFactorization (toNatList (s : List Std.Usize)) gs) :
    (factorsOf fs).length ≤ gs.length := by
  sorry

/-- Every copy factor (`offset ≠ 0`) is a longest allowed match at the position
    where it starts; the start position of a factor is the total length of the
    factors preceding it (`allMaximal` threads that position through the list). -/
theorem lz77_maximal (s : Slice Std.Usize) (fs : alloc.vec.Vec LzFactor)
    (h : lz77 s = ok fs) :
    allMaximal (toNatList (s : List Std.Usize)) 0 (factorsOf fs) := by
  sorry
