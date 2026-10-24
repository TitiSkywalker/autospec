import LeanMode.string_order_preserving_match.StringOrderPreservingMatch
import Mathlib

open Aeneas Aeneas.Std Result
open string_order_preserving_match

set_option linter.unusedVariables false

/-!
# Specification: Order-preserving pattern matching

A pattern `p` *order-matches* the text window `t[i..i+m]` when the two words are
**order-isomorphic** — the definition of Kubica–Kulczyński–Radoszewski–Rytter–
Waleń (2013): for all positions `a, b` inside the window,
`p[a] < p[b]` iff `t[i+a] < t[i+b]`.  Since `<` is a strict total order on
`Nat`, this single strict-order relation also pins down the equality pattern
(`p[a] = p[b] ↔ t[i+a] = t[i+b]`), so it really is "the same relative order,
ties included".

## Convention chosen: rank encoding, ties allowed

The implementation characterises order-isomorphism with the **rank encoding**
`rank(s)[k] = #{ j : s[j] < s[k] }` (ties receive equal ranks), comparing the
rank encoding of each text window with that of the pattern.  We keep **ties**
(the "tie-correct" route (b) of the task description) and prove the exact
statement for it:

> for words `w`, `p` of equal length, `rankEncoding w = rankEncoding p` iff
> `w` and `p` are order-isomorphic.

This is *exact even with repeated elements* — contrary to the caveat that the
strict-smaller count might only characterise order-isomorphism on words with
distinct entries.  The reason is that within a single word the map
`k ↦ #{ j : w[j] < w[k] }` is strictly order-preserving:
`w[a] < w[b] ↔ (#{j : w[j] < w[a]}) < (#{j : w[j] < w[b]})` (the left-to-right
filter inclusion is strict because `a` itself is counted on the right but not
the left).  Composing this with `rankEncoding w = rankEncoding p` gives
`w[a] < w[b] ↔ p[a] < p[b]`, i.e. order-isomorphism, and the converse is
immediate.  The random fuzzers in `src/lib.rs`'s `#[cfg(test)] mod tests`
(4000 pairs over alphabets of size 1–4, so ties are pervasive) confirm the
equivalence, and the reference `∀ a b` checker is the definition.  Hence no
distinctness hypothesis is needed.  (`op_match` recomputes each window's rank
encoding from scratch — the definitional route — which is why keeping ties is
harmless here; the incremental/sliding rank encoding is the version where ties
need care.)

## Conventions of the implementation

* The empty pattern matches *nowhere* (the result is empty).
* A pattern longer than the text yields the empty result.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- A `Vec Std.Usize` as a `List Nat`. -/
def vecNat (v : alloc.vec.Vec Std.Usize) : List Nat :=
  toNatList ((alloc.vec.Vec.deref v : Slice Std.Usize) : List Std.Usize)

/-- The rank encoding of a word: `rankEncoding w[k] = #{ j : w[j] < w[k] }`.
    Entries that are equal receive equal ranks. -/
def rankEncoding (w : List Nat) : List Nat :=
  w.map (fun x => (w.filter (fun y => y < x)).length)

/-- `w` and `p` are order-isomorphic: same length, and the same strict-order
    relations (which, `<` being a total order, also forces the same equalities). -/
def IsOrderIsomorphic (w p : List Nat) : Prop :=
  w.length = p.length ∧
    ∀ a b, a < p.length → b < p.length →
      (p.getD a 0 < p.getD b 0 ↔ w.getD a 0 < w.getD b 0)

/-- `p` order-matches `t` at position `i`: the window `t[i..i+|p|]` and `p` are
    order-isomorphic. -/
def OrderMatchesAt (t p : List Nat) (i : Nat) : Prop :=
  i + p.length ≤ t.length ∧ IsOrderIsomorphic ((t.drop i).take p.length) p

/-! ## Specification theorems -/

/-- The implementation's `rank_encoding` really computes `rankEncoding`. -/
theorem rank_encoding_eq (s : Slice Std.Usize) (v : alloc.vec.Vec Std.Usize)
    (h : rank_encoding s = ok v) :
    vecNat v = rankEncoding (toNatList (s : List Std.Usize)) := by
  sorry

/-- **Rank-encoding characterisation of order-isomorphism.**  Two words have the
    same rank encoding iff they are order-isomorphic.  The statement needs *no*
    distinctness hypothesis: it is exact in the presence of ties (see the module
    documentation for the proof sketch). -/
theorem rank_encoding_iff (w p : List Nat) :
    rankEncoding w = rankEncoding p ↔ IsOrderIsomorphic w p := by
  sorry

/-- **`op_match` is correct.**  For a non-empty pattern, the reported positions
    are exactly the order-matching positions: soundness and completeness. -/
theorem op_match_correct (t p : Slice Std.Usize) (occ : alloc.vec.Vec Std.Usize)
    (h : op_match t p = ok occ)
    (hp : 0 < (p : List Std.Usize).length) :
    ∀ i, i ∈ vecNat occ ↔
      OrderMatchesAt (toNatList (t : List Std.Usize)) (toNatList (p : List Std.Usize)) i := by
  sorry

/-- The reported positions are strictly increasing (hence duplicate-free) and
    every one of them leaves room for the whole pattern inside the text. -/
theorem op_match_occurrences_ordered (t p : Slice Std.Usize)
    (occ : alloc.vec.Vec Std.Usize) (h : op_match t p = ok occ) :
    (∀ i ∈ vecNat occ,
        i + (p : List Std.Usize).length ≤ (t : List Std.Usize).length) ∧
    (∀ i j, i < j → j < (vecNat occ).length →
        (vecNat occ).getD i 0 < (vecNat occ).getD j 0) := by
  sorry

/-- The empty pattern matches nowhere, so the result is the empty list (the
    documented convention of the implementation). -/
theorem op_match_empty_pattern (t p : Slice Std.Usize) (occ : alloc.vec.Vec Std.Usize)
    (h : op_match t p = ok occ)
    (hp : (p : List Std.Usize).length = 0) :
    vecNat occ = [] := by
  sorry
