import LeanMode.string_ukkonen.StringUkkonen
import Mathlib

open Aeneas Aeneas.Std Result
open string_ukkonen

set_option linter.unusedVariables false

/-!
# Specification: suffix tree (trie of all suffixes)

`suffix_tree s` builds the suffix tree of the byte string `s` with all edges
left un-compressed — i.e. the *trie of all suffixes* of `s` — together with the
suffix links of its nodes.

* node `0` is the root (the empty string) and is never the target of a
  transition, so the flat child table stores `0` for "no such child";
* node `v` corresponds to a word `w` (the string spelled by the path from the
  root to `v`), and `walk t 0 w = some v` recovers `v` from `w`;
* `link[v]` is the node of the longest proper suffix of `v`'s word that still
  occurs in `s` (i.e. is itself a node).

The three main theorems say: the reachable words are exactly the substrings of
`s`; the node count is `1 + #distinct nonempty substrings` and hence bounded by
`n(n+1)/2 + 1`; and every suffix link is the longest-proper-suffix link.
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

/-- Build up prefixes shortest-first, threading the accumulated front. -/
def prefixesAux (acc : List Nat) : List Nat → List (List Nat)
  | [] => []
  | x :: xs => let acc' := acc ++ [x]; acc' :: prefixesAux acc' xs

/-- All nonempty prefixes of `l` (including `l` itself), shortest first. -/
def prefixes (l : List Nat) : List (List Nat) := prefixesAux [] l

/-- All nonempty substrings of `s` (with duplicates), enumerated by start index. -/
def substrings : List Nat → List (List Nat)
  | [] => []
  | l@(_ :: xs) => prefixes l ++ substrings xs

/-- The distinct nonempty substrings of `s`. -/
def distinctSubstrings (s : List Nat) : List (List Nat) := (substrings s).eraseDups

/-- The child of node `v` on character `c` (`0` = absent), read from the flat
    `256`-wide table `next[v * 256 + c]`. -/
def nextOf (t : SuffixTree) (v c : Nat) : Nat :=
  ((alloc.vec.Vec.deref t.next : List Std.Usize).getD (v * 256 + c) 0#usize).val

/-- The suffix link of node `v` (`0` when the node index is out of range). -/
def linkOf (t : SuffixTree) (v : Std.Usize) : Std.Usize :=
  (alloc.vec.Vec.deref t.link : List Std.Usize).getD v.val 0#usize

/-- Follow the transitions of `t` from node `v` while reading the word `w`;
    `none` if some transition is missing.  Because a missing child is stored as
    `0` and `0` is never a target, `some v'` means the word is present. -/
def walk (t : SuffixTree) (v : Nat) : List Nat → Option Nat
  | [] => some v
  | c :: cs =>
      let nv := nextOf t v c
      if nv = 0 then none else walk t nv cs

/-- The word `w` is spelled by a path from the root (`PathExists t w`). -/
def PathExists (t : SuffixTree) (w : List Nat) : Prop := (walk t 0 w).isSome

/-- `w` is the word of node `v`: spelling `w` from the root ends at `v`. -/
def IsPath (t : SuffixTree) (v : Nat) (w : List Nat) : Prop := walk t 0 w = some v

/-- `u` is a proper suffix of `w`. -/
def isProperSuffix (w u : List Nat) : Prop := u <:+ w ∧ u.length < w.length

/-- Node `v` spells the word `w`, `link[v]` spells `u`, and `u` is the **longest
    proper suffix** of `w` that still occurs in `s`:

    * *attainment* — `u` is a proper suffix of `w` and `u` is the word of the
      node `link[v]`;
    * *maximality* — no node of the trie spells a longer proper suffix of `w`.

    The empty word is always a possible `u` (the root, node `0`): every
    nonempty `w` has `[]` as a proper suffix, and `IsPath t 0 []` holds. -/
def IsLongestProperSuffix (s : List Nat) (t : SuffixTree) (v : Std.Usize) : Prop :=
  ∃ w u : List Nat,
    IsPath t v.val w ∧
    IsPath t (linkOf t v).val u ∧
    isSubstring s w ∧
    isProperSuffix w u ∧
    (∀ (v' : Nat) (u' : List Nat), IsPath t v' u' → isProperSuffix w u' →
        u'.length ≤ u.length)

/-! ## Specification theorems -/

/-- The words reachable from the root are exactly the substrings of `s`. -/
theorem suffix_tree_substring (s : Slice Std.Usize) (t : SuffixTree) (w : List Nat)
    (h : suffix_tree s = ok t) (hb : AllBytes s)
    (hw : ∀ x ∈ w, x ≤ 255) :
    isSubstring (toNatList (s : List Std.Usize)) w ↔ PathExists t w := by
  sorry

/-- `suffix_tree` builds one node per distinct nonempty substring, plus the
    root: `sz = 1 + #distinct nonempty substrings`. -/
theorem suffix_tree_sz_distinct (s : Slice Std.Usize) (t : SuffixTree)
    (h : suffix_tree s = ok t) (hb : AllBytes s) :
    t.sz.val = (distinctSubstrings (toNatList (s : List Std.Usize))).length + 1 := by
  sorry

/-- The trie of all suffixes of `s` has at most `n(n+1)/2 + 1` nodes
    (`n = |s|`), the number of nodes of a trie holding the `n` suffixes of
    lengths `n, n-1, ..., 1` plus the root. -/
theorem suffix_tree_size_bound (s : Slice Std.Usize) (t : SuffixTree)
    (h : suffix_tree s = ok t) :
    t.sz.val ≤ (s : List Std.Usize).length * ((s : List Std.Usize).length + 1) / 2 + 1 := by
  sorry

/-- For every non-root node `v`, `link[v]` is the node of the longest proper
    suffix of `v`'s word that still occurs in `s`. -/
theorem suffix_tree_link_correct (s : Slice Std.Usize) (t : SuffixTree) (v : Std.Usize)
    (h : suffix_tree s = ok t) (hv : 0 < v.val) (hv' : v.val < t.sz.val) (hb : AllBytes s) :
    IsLongestProperSuffix (toNatList (s : List Std.Usize)) t v := by
  sorry
