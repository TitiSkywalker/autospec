import LeanMode.array_bloom_filter.ArrayBloomFilter
import Mathlib

open Aeneas Aeneas.Std Result
open array_bloom_filter

set_option linter.unusedVariables false

/-!
# Specification: Bloom filter

`bf_insert` sets the `k` bit positions of a key; `bf_contains` answers `true`
iff all of them are set.  Positions are derived by double hashing
`(x + i·(x+1)) mod size` for `i ∈ [0, k)`.  Insertion is monotone (it never
clears a bit), which yields the no-false-negative property.
-/

/-- The bits array as a `List Bool`. -/
def bitList (bf : BloomFilter) : List Bool :=
  (alloc.vec.Vec.deref bf.bits : List Bool)

/-- The `k` bit positions of key `x` in a filter of `size` bits. -/
def bit_positions (x k size : Nat) : List Nat :=
  (List.range k).map (fun i => (x + i * (x + 1)) % size)

/-! ## Specification theorems -/

/-- Inserting `x` sets every one of `x`'s bit positions. -/
theorem bf_insert_sets (bf : BloomFilter) (x : Std.Usize) (r : BloomFilter)
    (h_size : 0 < (bitList bf).length)
    (h : bf_insert bf x = ok r) :
    ∀ p, p ∈ bit_positions x.val bf.k.val (bitList bf).length →
      (bitList r).getD p false = true := by
  sorry

/-- Inserting never clears a set bit. -/
theorem bf_insert_monotone (bf : BloomFilter) (x : Std.Usize) (r : BloomFilter)
    (h_size : 0 < (bitList bf).length)
    (h : bf_insert bf x = ok r) :
    ∀ p, p < (bitList bf).length →
      (bitList bf).getD p false = true → (bitList r).getD p false = true := by
  sorry

/-- `bf_contains` answers `true` iff all of `x`'s bit positions are set. -/
theorem bf_contains_correct (bf : BloomFilter) (x : Std.Usize) (res : Bool)
    (h_size : 0 < (bitList bf).length)
    (h : bf_contains bf x = ok res) :
    res = true ↔ ∀ p, p ∈ bit_positions x.val bf.k.val (bitList bf).length →
      (bitList bf).getD p false = true := by
  sorry
