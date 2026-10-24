import LeanMode.string_generalized_sam.StringGeneralizedSam
import Mathlib

open Aeneas Aeneas.Std Result
open string_generalized_sam

set_option linter.unusedVariables false

/-!
# Specification: generalized suffix automaton (GSAM) of a set of strings

`gsam_build s lens` builds one automaton for the *set* of strings encoded in the
concatenation `s` (`lens[k]` is the length of the `k`-th string, read from
`s[Σ_{j<k} lens[j] .. Σ_{j≤k} lens[j])`).  Each string is inserted with the
ordinary suffix-automaton *extend* step, with `last` **reset to the root (`0`)
before every string**; that reset is what stops a query from matching across a
string boundary.  `gsam_contains g p` walks the transitions of `g` from the root
reading the pattern `p`.

Node `0` is the root (the empty string) and is never the target of a transition,
so the flat `256`-wide table stores `0` for "no transition" and `0` doubles as
the absent marker.

The main statements are

* `gsam_contains_correct` / `gsam_recognizes_substrings` — the automaton
  recognizes exactly the words that are substrings of *some* input string;
* `gsam_no_crossing_boundary` — every accepted word lies inside a single input
  string (it never spans a boundary);
* `gsam_endpos_classes` — for nonempty substrings, reaching the same state is
  the same as having the same *end-position set* (across all strings), i.e. the
  reachable states are the endpos equivalence classes;
* `gsam_linear` — `sz ≤ 2 * |s| + 1` states;
* `gsam_root_states` — the empty word sits at the root (state `0`), which is
  never the target of a transition.

A caveat that the statements have to respect: the *empty* word.  `walk g 0 []`
is the root, yet the end-position set of `[]` is "every position of every
string", which coincides with that of a string consisting of a single
character.  Hence `gsam_endpos_classes` is stated for nonempty patterns, and
`gsam_contains_correct` assumes the set of strings is nonempty (the empty
pattern is vacuously a substring of every string).
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- The `k`-th string of the encoding `s`/`lens`: `lens[k]` characters of `s`
    starting right after the first `lens[0] + … + lens[k-1]` characters. -/
def strAtD (s lens : List Nat) (k : Nat) : List Nat :=
  (s.drop ((lens.take k).sum)).take (lens.getD k 0)

/-- `p` occurs in `s` at index `i`: `s[i..] = p ++ _`. -/
def occursAt (s p : List Nat) (i : Nat) : Prop :=
  ∃ t, s.drop i = p ++ t

/-- `p` is a substring of `s`. -/
def isSubstring (s p : List Nat) : Prop :=
  ∃ i, occursAt s p i

/-- `p` occurs in *some* of the input strings. -/
def InSomeString (s lens : List Nat) (p : List Nat) : Prop :=
  ∃ k, k < lens.length ∧ isSubstring (strAtD s lens k) p

/-- `p` occurs in `t` *ending* at index `j` (so `j < t.length`). -/
def occursAtEnd (t p : List Nat) (j : Nat) : Prop :=
  j < t.length ∧ ∃ u, t.take (j + 1) = u ++ p

/-- The end-position set of `p`: every `(k, j)` such that `p` occurs in the
    `k`-th input string ending at index `j`. -/
def endposSet (s lens : List Nat) (p : List Nat) : Set (Nat × Nat) :=
  { q | q.1 < lens.length ∧ occursAtEnd (strAtD s lens q.1) p q.2 }

/-- The child of state `v` on character `c` (`0` = absent), read from the flat
    `256`-wide table `next[v * 256 + c]`. -/
def nextOf (g : Gsam) (v c : Nat) : Nat :=
  ((alloc.vec.Vec.deref g.next : List Std.Usize).getD (v * 256 + c) 0#usize).val

/-- Follow the transitions of `g` from state `v` while reading the word `w`;
    `none` if some transition is missing.  Because a missing transition is
    stored as `0` and `0` is never a target, `some v'` means the word is
    present. -/
def walk (g : Gsam) (v : Nat) : List Nat → Option Nat
  | [] => some v
  | c :: cs =>
      let nv := nextOf g v c
      if nv = 0 then none else walk g nv cs

/-- The state reached from the root by the word `w` (`none` = absent). -/
def stateOf (g : Gsam) (w : List Nat) : Option Nat := walk g 0 w

/-- The word `w` is spelled by a path from the root. -/
def PathExists (g : Gsam) (w : List Nat) : Prop := (walk g 0 w).isSome

/-! ## Specification theorems -/

/-- The number of states is linear in the input: the reset-`last` construction
    creates at most one `cur` and one `clone` per character, hence at most
    `2 * |s| + 1` states.  (Brute-forced over random multi-string inputs; the
    observed maximum of `sz` is `2 * |s|`.) -/
theorem gsam_linear (s lens : Slice Std.Usize) (g : Gsam)
    (h : gsam_build s lens = ok g) :
    g.sz.val ≤ 2 * (s : List Std.Usize).length + 1 := by
  sorry

/-- `gsam_contains` recognizes exactly the words that are substrings of *some*
    input string. -/
theorem gsam_contains_correct (s lens p : Slice Std.Usize) (g : Gsam) (b : Bool)
    (hb : AllBytes s ∧ AllBytes p)
    (h_lens : 0 < (lens : List Std.Usize).length)
    (h_sum : (toNatList (lens : List Std.Usize)).sum = (s : List Std.Usize).length)
    (h_build : gsam_build s lens = ok g)
    (h : gsam_contains g p = ok b) :
    b = true ↔ InSomeString (toNatList (s : List Std.Usize))
      (toNatList (lens : List Std.Usize)) (toNatList (p : List Std.Usize)) := by
  sorry

/-- Every word accepted by the automaton is a substring of one of the input
    strings: no reported match spans a string boundary.  (This is the
    soundness half of `gsam_contains_correct`, stated separately because it is
    the semantic point of the reset-`last` construction: it is *not* the case
    that an accepted word is merely a substring of the concatenation `s`.)

    The `0 < lens.length` hypothesis is needed, exactly as in
    `gsam_contains_correct`: with an *empty* set of strings the empty pattern is
    vacuously accepted (`gsam_contains g [] = ok true`, since the walk never
    takes a transition) while `InSomeString [] [] []` is false. -/
theorem gsam_no_crossing_boundary (s lens p : Slice Std.Usize) (g : Gsam)
    (hb : AllBytes s ∧ AllBytes p)
    (h_lens : 0 < (lens : List Std.Usize).length)
    (h_sum : (toNatList (lens : List Std.Usize)).sum = (s : List Std.Usize).length)
    (h_build : gsam_build s lens = ok g)
    (h : gsam_contains g p = ok true) :
    InSomeString (toNatList (s : List Std.Usize))
      (toNatList (lens : List Std.Usize)) (toNatList (p : List Std.Usize)) := by
  sorry

/-- Conversely, every substring of an input string is accepted. -/
theorem gsam_recognizes_substrings (s lens p : Slice Std.Usize) (g : Gsam)
    (hb : AllBytes s ∧ AllBytes p)
    (h_sum : (toNatList (lens : List Std.Usize)).sum = (s : List Std.Usize).length)
    (h_build : gsam_build s lens = ok g)
    (hp : InSomeString (toNatList (s : List Std.Usize))
      (toNatList (lens : List Std.Usize)) (toNatList (p : List Std.Usize))) :
    gsam_contains g p = ok true := by
  sorry

/-- The words spelled by the paths of `g` are the empty word together with the
    substrings of the input strings.  (`w`'s characters have to be byte values:
    the transition table is `256` columns wide, and a non-byte character would
    read an unrelated entry of the flat table.) -/
theorem gsam_paths_are_substrings (s lens : Slice Std.Usize) (g : Gsam) (w : List Nat)
    (hb : AllBytes s) (hw : ∀ x ∈ w, x ≤ 255)
    (h_sum : (toNatList (lens : List Std.Usize)).sum = (s : List Std.Usize).length)
    (h_build : gsam_build s lens = ok g) :
    PathExists g w ↔
      (w = [] ∨ InSomeString (toNatList (s : List Std.Usize))
        (toNatList (lens : List Std.Usize)) w) := by
  sorry

/-- Endpos equivalence classes across all the strings: two nonempty substrings
    (one of the input strings) lead to the same state iff they have the same
    end-position set.  This says that the reachable states *are* the endpos
    equivalence classes, and that distinct states carry distinct classes.

    (Nonemptiness is needed: the empty word's end-position set is "every
    position of every string", which also equals the end-position set of a
    single-character string, yet the empty word sits at the root.) -/
theorem gsam_endpos_classes (s lens p q : Slice Std.Usize) (g : Gsam)
    (hb : AllBytes s ∧ AllBytes p ∧ AllBytes q)
    (h_sum : (toNatList (lens : List Std.Usize)).sum = (s : List Std.Usize).length)
    (h_build : gsam_build s lens = ok g)
    (hpn : 0 < (p : List Std.Usize).length)
    (hqn : 0 < (q : List Std.Usize).length)
    (hp : InSomeString (toNatList (s : List Std.Usize))
      (toNatList (lens : List Std.Usize)) (toNatList (p : List Std.Usize)))
    (hq : InSomeString (toNatList (s : List Std.Usize))
      (toNatList (lens : List Std.Usize)) (toNatList (q : List Std.Usize))) :
    stateOf g (toNatList (p : List Std.Usize)) = stateOf g (toNatList (q : List Std.Usize)) ↔
      endposSet (toNatList (s : List Std.Usize)) (toNatList (lens : List Std.Usize))
          (toNatList (p : List Std.Usize)) =
        endposSet (toNatList (s : List Std.Usize)) (toNatList (lens : List Std.Usize))
          (toNatList (q : List Std.Usize)) := by
  sorry

/-- The root is the state of the empty word, and it is never the target of a
    transition — so `0` really is the "absent transition" marker.  Also: for an
    empty set of strings the automaton is the bare root. -/
theorem gsam_root_states (s lens : Slice Std.Usize) (g : Gsam)
    (h_build : gsam_build s lens = ok g) :
    stateOf g [] = some 0 ∧
    0 < g.sz.val ∧
    ((lens : List Std.Usize).length = 0 → g.sz.val = 1) := by
  sorry
