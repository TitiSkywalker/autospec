import LeanMode.string_two_way.StringTwoWay
import Mathlib

open Aeneas Aeneas.Std Result
open string_two_way

set_option linter.unusedVariables false

/-!
# Specification: Two-Way string matching (Crochemore–Perrin, 1991)

`two_way_find text pattern` returns, in increasing order, every starting
position at which the *non-empty* pattern occurs in the text (the empty pattern
matches nothing, by the documented convention of the implementation).

The implementation is the genuine Two-Way algorithm of Crochemore and Perrin:

* `maximal_suffix w rev` is the maximal-suffix scan (`rev` selects the
  reversed comparison order, which is what the second scan needs);
* `critical_factorization p` combines the two scans into a critical
  factorization `p = u v` with `u = p[..ell]` and `v = p[ell..]`, returning the
  period `per` of the right half `v`;
* `two_way_find` matches `v` left-to-right at each candidate position and only
  then `u` right-to-left (the `memory` variable), shifting by the period on
  every *match* so that all overlapping occurrences are reported.

Mathematically the file states the Critical Factorization Theorem of
Crochemore–Perrin (1991):

* `critical_factorization_correct` — the implementation's `ell` is a critical
  factorization position: its local period is the global minimal period of `p`,
  and `ell` is smaller than that period.  The returned `per` is a period of the
  right half which lower-bounds the global period (it is exact in the
  short-period case; `two_way_find` corrects it with `max ell (|p| - ell) + 1`
  otherwise).
* `two_way_correct` — the reported positions are exactly the occurrences.
* `two_way_occurrences_ordered` — the reported positions are strictly
  increasing and every occurrence fits in the text.
* `two_way_empty_pattern` — the empty pattern yields the empty result.

Characters are `usize`-encoded byte values in `0..=255` (`AllBytes`).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element of a byte string is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- A `Vec Std.Usize` as a `List Nat`. -/
def vecNat (v : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize)

/-- `p` occurs in `s` starting at index `i`, i.e. `s = u ++ p ++ v` with
    `u.length = i`. -/
def occursAt (s p : List Nat) (i : Nat) : Prop :=
  ∃ u v, s = u ++ p ++ v ∧ u.length = i

/-- `q` is a period of `w`: shifting by `q` leaves every in-range pair of
    characters matched.  (Every non-empty word has a period, namely its own
    length, because then the condition is vacuous.) -/
def IsPeriod (w : List Nat) (q : Nat) : Prop :=
  0 < q ∧ ∀ i (h : i + q < w.length), w[i]'(by omega) = w[i + q]'h

/-- `q` is the global minimal period of `w`. -/
def IsMinPeriod (w : List Nat) (q : Nat) : Prop :=
  IsPeriod w q ∧ ∀ q', IsPeriod w q' → q ≤ q'

/-- `q` is a *local period* of the factorization `w = w[..l] ++ w[l..]`: the
    last `q` characters of the left half agree with the first `q` characters of
    the right half, as far as both are defined.  This is the equivalent form of
    the definition used in Crochemore–Perrin (see also the documentation of
    `maximal_suffix` in `library/core/src/str/pattern.rs` of the Rust
    standard library): `w[i] = w[i + q]` for every `i` with `l - q ≤ i < l` and
    `i + q < w.length`. -/
def IsLocalPeriod (w : List Nat) (l q : Nat) : Prop :=
  0 < q ∧ l ≤ w.length ∧
    ∀ i (h1 : l - q ≤ i) (h2 : i < l) (h3 : i + q < w.length),
      w[i]'(by omega) = w[i + q]'h3

/-! ## Critical factorization -/

/-- Crochemore–Perrin critical factorization.

    For a word `w` of length `≥ 2`, the `(ell, per)` returned by
    `critical_factorization` satisfies: `ell` is the position of a *critical
    factorization* — `p` is the global minimal period of `w`, it is a local
    period of `(w[..ell], w[ell..])`, and it is the *smallest* local period of
    that factorization, i.e. the local period equals the global one.  Moreover
    `ell < p` (the left half is shorter than the period), and the returned `per`
    is a period of the right half `w[ell..]` which is a positive lower bound for
    `p` (`ell + per ≤ |w|`).

    These are exactly the properties the search relies on: `ell + per ≤ |w|`
    makes the short/long period test in `two_way_find` in bounds, and
    `per ≤ p` is what makes shifting by `per` conservative. -/
theorem critical_factorization_correct (w : Slice Std.Usize) (ell per : Std.Usize)
    (h : critical_factorization w = ok (ell, per))
    (hw : 2 ≤ (w : List Std.Usize).length) :
    let W := toNatList (w : List Std.Usize)
    ∃ p, IsMinPeriod W p ∧ IsLocalPeriod W ell.val p ∧
      (∀ r, IsLocalPeriod W ell.val r → p ≤ r) ∧ ell.val < p ∧
      0 < per.val ∧ per.val ≤ p ∧ IsPeriod (W.drop ell.val) per.val ∧
      ell.val + per.val ≤ W.length := by
  sorry

/-! ## Correctness of the search -/

/-- `two_way_find` reports exactly the occurrences of a non-empty pattern: for
    every position `i` inside the text, `i` is reported if and only if the
    pattern occurs at `i`.  The hypothesis `0 < |pattern|` is necessary — the
    empty pattern occurs at *every* position of the text but the
    implementation (by its documented convention) reports nothing for it. -/
theorem two_way_correct (text pattern : Slice Std.Usize)
    (occ : alloc.vec.Vec Std.Usize)
    (h : two_way_find text pattern = ok occ)
    (hp : 0 < (pattern : List Std.Usize).length)
    (hb : AllBytes text ∧ AllBytes pattern) :
    ∀ i, i < (text : List Std.Usize).length →
      (i ∈ vecNat occ ↔
        occursAt (toNatList (text : List Std.Usize))
                 (toNatList (pattern : List Std.Usize)) i) := by
  sorry

/-- The reported positions are strictly increasing (so there are no
    duplicates) and every reported position leaves room for the whole pattern
    inside the text. -/
theorem two_way_occurrences_ordered (text pattern : Slice Std.Usize)
    (occ : alloc.vec.Vec Std.Usize)
    (h : two_way_find text pattern = ok occ)
    (hb : AllBytes text ∧ AllBytes pattern) :
    (vecNat occ).Nodup ∧
    (∀ i ∈ vecNat occ,
        i + (pattern : List Std.Usize).length ≤ (text : List Std.Usize).length) ∧
    (∀ i j, i < j → j < (vecNat occ).length →
        (vecNat occ).getD i 0 < (vecNat occ).getD j 0) := by
  sorry

/-- The empty pattern matches nothing, so the result is the empty list. -/
theorem two_way_empty_pattern (text pattern : Slice Std.Usize)
    (occ : alloc.vec.Vec Std.Usize)
    (h : two_way_find text pattern = ok occ)
    (hp : (pattern : List Std.Usize).length = 0) :
    vecNat occ = [] := by
  sorry
