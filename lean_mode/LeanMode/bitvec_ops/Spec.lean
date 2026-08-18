import LeanMode.bitvec_ops.BitvecOps
import Mathlib

open Aeneas Aeneas.Std Result
open bitvec_ops

set_option linter.unusedVariables false

/-!
# Specification: Fixed-width bitvector (bitvec int-model)

`bitvec_ops` exercises the `bitvec` int-model: `u32` maps to Lean `BitVec 32`
(fixed-width, wrapping semantics).  Specs are the roundtrip (rotate), the
wrap-modulo identities (wrapping arithmetic, shifts), bitwise identities, and
popcount.
-/

-- ── rotation ─────────────────────────────────────────────────────────────

/-- Rotating right then left by the same amount recovers the original value. -/
theorem rotate_roundtrip
    (x n mid result : BitVec 32)
    (h1 : rotate_right x n = ok mid)
    (h2 : rotate_left mid n = ok result) :
    result = x := by
  sorry

-- ── wrapping arithmetic ──────────────────────────────────────────────────

/-- `wrapping_add` adds modulo 2³² (fixed-width wrap). -/
theorem wrapping_add_mod
    (a b result : BitVec 32)
    (h : wrapping_add a b = ok result) :
    result.toNat = (a.toNat + b.toNat) % (2 ^ 32) := by
  sorry

/-- `wrapping_sub` subtracts modulo 2³² (blocks 0 `a.toNat + 2^32 - b.toNat` always). -/
theorem wrapping_sub_mod
    (a b result : BitVec 32)
    (h : wrapping_sub a b = ok result) :
    result.toNat = (a.toNat + 2 ^ 32 - b.toNat) % (2 ^ 32) := by
  sorry

/-- `wrapping_mul` multiplies modulo 2³². -/
theorem wrapping_mul_mod
    (a b result : BitVec 32)
    (h : wrapping_mul a b = ok result) :
    result.toNat = (a.toNat * b.toNat) % (2 ^ 32) := by
  sorry

/-- `wrapping_neg` is two's-complement negation modulo 2³². -/
theorem wrapping_neg_mod
    (x result : BitVec 32)
    (h : wrapping_neg x = ok result) :
    result.toNat = (2 ^ 32 - x.toNat) % (2 ^ 32) := by
  sorry

-- ── bitwise logic ────────────────────────────────────────────────────────

/-- XOR of a value with itself is zero. -/
theorem bit_xor_self (x : BitVec 32) :
    bit_xor x x = ok (0 : BitVec 32) := by
  sorry

/-- Bitwise NOT is an involution. -/
theorem bit_not_involutive
    (x y : BitVec 32)
    (h : bit_not x = ok y) :
    bit_not y = ok x := by
  sorry

/-- `x & ~x = 0` (a value AND its complement is zero). -/
theorem bit_and_complement_zero
    (x y : BitVec 32)
    (h : bit_not x = ok y) :
    bit_and x y = ok (0 : BitVec 32) := by
  sorry

/-- `x | ~x = all-ones` (a value OR its complement is the full mask). -/
theorem bit_or_complement_full
    (x y : BitVec 32)
    (h : bit_not x = ok y) :
    bit_or x y = ok ((2 ^ 32 - 1 : Nat) : BitVec 32) := by
  sorry

-- ── shifts ───────────────────────────────────────────────────────────────

/-- Logical shift left is multiplication by a power of two, modulo 2³². -/
theorem shift_left_mod
    (x n result : BitVec 32)
    (h : shift_left x n = ok result) :
    result.toNat = (x.toNat * 2 ^ n.toNat) % (2 ^ 32) := by
  sorry

/-- Logical shift right is division by a power of two. -/
theorem shift_right_div
    (x n result : BitVec 32)
    (h : shift_right x n = ok result) :
    result.toNat = x.toNat / 2 ^ n.toNat := by
  sorry

-- ── population count ─────────────────────────────────────────────────────

/-- Number of set bits of `x`. -/
def bitCount (x : BitVec 32) : Nat :=
  ∑ i : Fin 32, if x.getLsb i then 1 else 0

/-- `popcount` returns the number of set bits. -/
theorem popcount_spec
    (x result : BitVec 32)
    (h : popcount x = ok result) :
    result.toNat = bitCount x := by
  sorry