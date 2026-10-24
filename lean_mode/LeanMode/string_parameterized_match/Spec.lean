import LeanMode.string_parameterized_match.StringParameterizedMatch
import Mathlib

open Aeneas Aeneas.Std Result
open string_parameterized_match

set_option linter.unusedVariables false

/-!
# Specification: parameterized pattern matching (p-matching, Baker)

A pattern `p` *p-matches* the length-`m` text window `t[i .. i+m]`
(`m = p.length`) when some bijection `φ` between the distinct characters of
`p` and the distinct characters of the window carries `p` onto the window:
`φ (p[k]) = t[i+k]` for all `k < m`.

`pmatch t p` returns the *exact* set of start positions at which `p` p-matches
`t` — no misses and no extras.

## The predicate

`IsPMatchAt t p i` is stated with an **injective total function**
`φ : Nat → Nat` rather than a partial bijection on the two finite character
sets.  The two formulations are equivalent, and the total-injective one needs
no `Finset`/finiteness machinery:

* *the hypothesis side.*  Two pattern positions with equal characters must map
  to equal window characters (`p[k₁] = p[k₂] ⟹ φ(p[k₁]) = φ(p[k₂])`), and two
  window positions with equal characters must come from equal pattern
  characters (injectivity of `φ`).  Hence `φ` restricted to the characters
  occurring in `p` is a well-defined injection into the window's characters,
  and its image is *all* of the window's characters (`window[k] = φ(p[k])`), so
  it is a bijection between the two distinct-character sets.
* *conversely*, a bijection between the two finite sets extends to an injective
  total `φ : Nat → Nat` (map the remaining characters of the first set
  injectively into `Nat` minus the (finite) second set), so the total-`φ`
  formulation is not stronger than the bijection one.

The window is read as `(t.drop i).getD k 0 = t[i+k]`; the first conjunct
`i + p.length ≤ t.length` guarantees `i + k < t.length`, so the `getD` default
`0` never actually fires.

## The equivalence

`prevEncoding s` is the prev-occurrence encoding of `s`:

* `prevEncoding s [k] = 0` when `s[k]` does not occur in the prefix `s[..k]`;
* `prevEncoding s [k] = k - j` where `j` is the **last** index `< k` with
  `s[j] = s[k]`.

`prev_encoding_iff_pmatch` is Baker's theorem: `p` p-matches a window iff their
prev-encodings agree.  (`List.get?` does not exist in this toolchain, so the
encoding is written with an explicit `List.range`/`foldl` index scan rather
than with `get?`.)

The two conclusions are stated as:

* `prev_encoding_iff_pmatch` — the definitional predicate `IsPMatchAt` equals
  the prev-encoding equality on the window `(t.drop i).take p.length`;
* `pmatch_correct` — the occurrence list `occ` returned by `pmatch` is exactly
  the set of positions satisfying `IsPMatchAt`.

Both theorems assume `0 < p.length`: the implementation deliberately reports the
empty pattern as matching *nowhere* (`pmatch []` … returns the empty vector),
whereas `IsPMatchAt` would hold *vacuously* for the empty pattern at every
position `i ≤ t.length`.  The nonemptiness hypothesis removes exactly that
degenerate case.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- The prev-occurrence encoding of `s`: position `k` holds `0` when `s[k]`
    does not occur in `s[..k]`, and `k - j` where `j` is the last index `< k`
    with `s[j] = s[k]`.

    Computed by an explicit index scan: for each `k` the `foldl` accumulates
    `j + 1` for the largest `j < k` with `s[j] = s[k]` (`0` when none exists),
    and the distance is `k + 1 - (j + 1) = k - j`. -/
def prevEncoding (s : List Nat) : List Nat :=
  (List.range s.length).map (fun k =>
    let d := (List.range k).foldl
      (fun acc j => if s.getD j 0 = s.getD k 0 then j + 1 else acc) 0
    if d = 0 then 0 else k + 1 - d)

/-- `p` p-matches the length-`p.length` window of `t` starting at `i`: the
    window lies inside `t`, and there is an *injective* `φ : Nat → Nat` with
    `φ (p[k]) = t[i+k]` for every `k < p.length`.

    An injective total `φ` is equivalent to a bijection between the distinct
    characters of `p` and the distinct characters of the window (see the module
    doc comment). -/
def IsPMatchAt (t p : List Nat) (i : Nat) : Prop :=
  i + p.length ≤ t.length ∧
  ∃ φ : Nat → Nat,
    (∀ a b, φ a = φ b → a = b) ∧
    (∀ k, k < p.length → (t.drop i).getD k 0 = φ (p.getD k 0))

/-! ## Specification theorems -/

/-- **Baker's theorem.**  For a nonempty pattern, `p` p-matches the window of
    `t` starting at `i` if and only if the prev-occurrence encodings of `p` and
    of that window are equal.  (The window's encoding is computed window-locally
    by `prevEncoding`, so a character occurring only before the window is "new"
    inside it.) -/
theorem prev_encoding_iff_pmatch (t p : List Nat) (i : Nat) (hm : 0 < p.length) :
    IsPMatchAt t p i ↔ prevEncoding p = prevEncoding ((t.drop i).take p.length) := by
  sorry

/-- `pmatch` returns exactly the set of p-match positions: `i` is in the
    returned occurrence list iff `p` p-matches the window `t[i .. i+p.length]`.

    (The hypothesis `0 < p.length` excludes the empty pattern, which the
    implementation documents as matching nowhere.) -/
theorem pmatch_correct (t p : Slice Std.Usize) (occ : alloc.vec.Vec Std.Usize)
    (h : pmatch t p = ok occ) (hm : 0 < (p : List Std.Usize).length) :
    ∀ i, i ∈ toNatList (alloc.vec.Vec.deref occ) ↔
      IsPMatchAt (toNatList (t : List Std.Usize)) (toNatList (p : List Std.Usize)) i := by
  sorry
