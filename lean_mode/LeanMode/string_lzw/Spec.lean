import LeanMode.string_lzw.StringLzw
import Mathlib

open Aeneas Aeneas.Std Result
open string_lzw

set_option linter.unusedVariables false

/-!
# Specification: LZ78 dictionary compression

`lz78 s` greedily factors the byte string `s` and simultaneously builds the LZ78
dictionary, returned as the structure `Lz78`:

* `next` is the (flat, `256`-wide) child table of the phrase trie:
  `next[v * 256 + c]` is the child of node `v` on character `c`, and `0` means
  "no such child".  Node `0` is the root (the empty phrase) and is never the
  target of a transition, so `0` doubles as the absent marker.
* `link[v]` is the node of the phrase of `v` with its last character removed,
  `ch[v]` is that last character, so the phrase of node `v ≥ 1` is
  `phrase(v) = phrase(link[v]) ++ [ch[v]]`, spelled from the root by `walk`.
* `codes[k]` is the node of the `k`-th factor's word.  The *last* factor may
  reuse an existing node (when the remaining suffix is exactly an existing
  phrase, so there is no character left to extend it by), so `codes` need not be
  duplicate-free; the node count is `1 + #distinct factor words`.
* `sz` is the number of nodes (the root included).

The greedy step at position `i` takes `P` = the longest prefix of `s[i..]` that
is already a phrase, and emits the factor `P ++ [s[i + |P|]]` (creating a fresh
node), except when that exhausts `s`, in which case the factor is `P` itself and
no node is created.

The theorems below say: every node spells a substring of `s`; the node ↔ phrase
correspondence is injective; the phrases of the trie are exactly the empty word
and the factor words; the factor words concatenate back to `s`; the node count
is `1 + #distinct factor words`; and the greedy factorization has the fewest
factors of any valid LZ78 factorization.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat`. -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- `p` occurs in `s` at index `i`: `s[i..] = p ++ _`. -/
def occursAt (s p : List Nat) (i : Nat) : Prop :=
  ∃ t, s.drop i = p ++ t

/-- `p` is a substring of `s`. -/
def isSubstring (s p : List Nat) : Prop :=
  ∃ i, occursAt s p i

/-- The child of node `v` on character `c` (`0` = absent), read from the flat
    `256`-wide table `next[v * 256 + c]`. -/
def nextOf (t : Lz78) (v c : Nat) : Nat :=
  ((alloc.vec.Vec.deref t.next : List Std.Usize).getD (v * 256 + c) 0#usize).val

/-- The parent of node `v` in the phrase trie — the node of the phrase of `v`
    with its last character removed (`0` when the index is out of range). -/
def linkOf (t : Lz78) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref t.link : List Std.Usize).getD v 0#usize).val

/-- The last character of the phrase of node `v` (`0` when the index is out of
    range). -/
def chOf (t : Lz78) (v : Nat) : Nat :=
  ((alloc.vec.Vec.deref t.ch : List Std.Usize).getD v 0#usize).val

/-- Follow the transitions of `t` from node `v` while reading the word `w`;
    `none` if some transition is missing.  Because a missing child is stored as
    `0` and `0` is never a transition target, `some v'` means the word is
    present in the trie. -/
def walk (t : Lz78) (v : Nat) : List Nat → Option Nat
  | [] => some v
  | c :: cs =>
      let nv := nextOf t v c
      if nv = 0 then none else walk t nv cs

/-- The word `w` is spelled by a path from the root. -/
def PathExists (t : Lz78) (w : List Nat) : Prop := (walk t 0 w).isSome

/-- `w` is the word of node `v`: spelling `w` from the root ends at `v`. -/
def IsPath (t : Lz78) (v : Nat) (w : List Nat) : Prop := walk t 0 w = some v

/-- `w` is a *phrase* of the dictionary: the word of some node of the phrase
    trie.  (The empty word is the word of the root, node `0`.) -/
def Phrase (t : Lz78) (w : List Nat) : Prop := ∃ v, v < t.sz.val ∧ IsPath t v w

/-- The phrase of node `v`, recovered by a fuel-bounded walk up the parent links
    (the fuel is `t.sz.val + 1`, enough for every node of the trie). -/
def phraseFuel (t : Lz78) : Nat → Nat → List Nat
  | 0, _ => []
  | f + 1, v => if v = 0 then [] else phraseFuel t f (linkOf t v) ++ [chOf t v]

/-- The phrase of node `v`. -/
def phrase (t : Lz78) (v : Nat) : List Nat := phraseFuel t (t.sz.val + 1) v

/-- The factor words of the factorization, decoded from the node codes: the
    `k`-th factor is the phrase of the node `codes[k]`. -/
def factorsOf (t : Lz78) : List (List Nat) :=
  ((alloc.vec.Vec.deref t.codes : List Std.Usize)).map (fun v => phrase t v.val)

/-- Length of the base phrase of the factor `f` when `rem` is the remaining
    input: the factor extended the base by one character, unless it exhausts the
    input. -/
def baseLen (f rem : List Nat) : Nat := if f.length < rem.length then f.length - 1 else f.length

/-- `fs` is a valid LZ78 factorization of `s`: the factors tile `s`, each factor
    is an earlier factor (or the empty word) followed by at most one character,
    and it is as long as possible — no earlier factor is a longer prefix of the
    remaining input than the base of `f`.

    The last clause is the *greedy* condition, stated from the implementation's
    walk guard: at the position where `f` starts, the base of `f` is the longest
    phrase that is a prefix of the remaining input, so no *earlier factor* of
    `fs` is a longer prefix of `rem`.  (This is not the weaker, false claim that
    no extension `P ++ [d]` of the base by an arbitrary character `d` may be a
    prefix of `rem`: for `s = "ab"` the factor `"b"` has base `[]` while `"a"` is
    an earlier factor of the same length.) -/
def IsLz78Factorization (s : List Nat) (fs : List (List Nat)) : Prop :=
  fs.flatten = s ∧
  (∀ f ∈ fs, f ≠ []) ∧
  ∀ k (hk : k < fs.length),
    let rem := s.drop ((fs.take k).flatten.length)
    let f : List Nat := fs[k]
    f <+: rem ∧
    (f.dropLast = [] ∨ f.dropLast ∈ fs.take k) ∧
    (∀ g ∈ fs.take k, g <+: rem → g.length ≤ baseLen f rem)

/-! ## Specification theorems -/

/-- Every node of the phrase trie spells a substring of `s`. -/
theorem lz78_phrases_are_substrings (s : Slice Std.Usize) (t : Lz78) (v : Nat)
    (w : List Nat)
    (h : lz78 s = ok t) (hb : AllBytes s) (hw : ∀ x ∈ w, x ≤ 255)
    (hv : v < t.sz.val) (hp : IsPath t v w) :
    isSubstring (toNatList (s : List Std.Usize)) w := by
  sorry

/-- The node ↔ phrase correspondence is injective: two nodes of the trie that
    spell the same word are the same node. -/
theorem lz78_bijection (s : Slice Std.Usize) (t : Lz78) (v v' : Nat) (w : List Nat)
    (h : lz78 s = ok t) (hb : AllBytes s)
    (hv : v < t.sz.val) (hv' : v' < t.sz.val)
    (hp : IsPath t v w) (hp' : IsPath t v' w) :
    v = v' := by
  sorry

/-- The trie recognizes exactly the empty word and the factor words: the phrases
    of the dictionary are the factor words (and the root's empty word). -/
theorem lz78_dictionary_correct (s : Slice Std.Usize) (t : Lz78) (w : List Nat)
    (h : lz78 s = ok t) (hb : AllBytes s) (hw : ∀ x ∈ w, x ≤ 255) :
    Phrase t w ↔ w = [] ∨ w ∈ factorsOf t := by
  sorry

/-- The factor words concatenate back to `s`: the factorization tiles the
    input. -/
theorem lz78_factors_concatenate (s : Slice Std.Usize) (t : Lz78)
    (h : lz78 s = ok t) (hb : AllBytes s) :
    (factorsOf t).flatten = toNatList (s : List Std.Usize) := by
  sorry

/-- The dictionary has exactly one node per distinct factor word, plus the root:
    `sz = 1 + #distinct factor words`. -/
theorem lz78_node_count (s : Slice Std.Usize) (t : Lz78)
    (h : lz78 s = ok t) (hb : AllBytes s) :
    t.sz.val = ((factorsOf t).eraseDups).length + 1 := by
  sorry

/-- The greedy factorization is optimal: its factor list is at least as short as
    any valid LZ78 factorization of the same string. -/
theorem lz78_factorization_optimal (s : Slice Std.Usize) (t : Lz78)
    (gs : List (List Nat))
    (h : lz78 s = ok t) (hb : AllBytes s)
    (hg : IsLz78Factorization (toNatList (s : List Std.Usize)) gs) :
    (factorsOf t).length ≤ gs.length := by
  sorry
