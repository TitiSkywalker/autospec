import LeanMode.ds_trie.DsTrie
import Mathlib

open Aeneas Aeneas.Std Result
open ds_trie

set_option linter.unusedVariables false

/-!
# Specification: Binary Trie

`trie_insert t key idx` inserts the suffix `key[idx..]` into the trie `t`;
`trie_search t key idx` returns `true` iff that suffix is a stored key.
The set of stored keys is the `view` of the trie.  A key is a `List Nat` of
bits (`0` / `1`); the empty key `[]` is stored iff `is_end` is set.
-/

mutual
  /-- The set of keys stored in the trie (viewed as bit-lists). -/
  def view : Option TrieNode → Set (List Nat)
    | none => ∅
    | some n => view_node n
  /-- Helper: view of a TrieNode (not wrapped in Option). -/
  def view_node : TrieNode → Set (List Nat)
    | TrieNode.mk c0 c1 b =>
        (if b then {([] : List Nat)} else ∅) ∪
        (view c0).image (fun k => 0 :: k) ∪
        (view c1).image (fun k => 1 :: k)
end

/-- View a key slice as a `List Nat` of bits. -/
def sliceToList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- The suffix of `key` starting at index `idx`. -/
def keySuffix (key : Slice Std.Usize) (idx : Nat) : List Nat :=
  (sliceToList key).drop idx


/-! ## Specification theorems -/

/-- `trie_search` returns `true` iff the key suffix is stored. -/
theorem trie_search_correct
    (t : Option TrieNode) (key : Slice Std.Usize) (idx : Std.Usize) (result : Bool)
    (h_bits : ∀ x ∈ sliceToList key, x = 0 ∨ x = 1)
    (h : trie_search t key idx = ok result) :
    (result = true) ↔ keySuffix key idx.val ∈ view t := by
  sorry

/-- `trie_insert` adds exactly the key suffix to the stored keys. -/
theorem trie_insert_correct
    (t : Option TrieNode) (key : Slice Std.Usize) (idx : Std.Usize) (t' : Option TrieNode)
    (h_bits : ∀ x ∈ sliceToList key, x = 0 ∨ x = 1)
    (h : trie_insert t key idx = ok t') :
    view t' = view t ∪ {keySuffix key idx.val} := by
  sorry
