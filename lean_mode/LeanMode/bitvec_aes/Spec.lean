import LeanMode.bitvec_aes.BitvecAes
import Mathlib

open Aeneas Aeneas.Std Result
open bitvec_aes

set_option linter.unusedVariables false

/-!
# Specification: AES-128 (table-free, cast-free)

Encrypt/decrypt conformance against FIPS-197 / SP 800-38A is checked by
`#eval` in `Test_test.lean`.  The theorems below state the core structural
properties: the GF(2⁸) inverse, the S-box involution, and the roundtrip.
-/

/-- `gf_inv` computes the multiplicative inverse: for `a ≠ 0`, `a · a⁻¹ = 1`. -/
theorem gf_inv_inverse
    (a inv : BitVec 8)
    (h : gf_inv a = ok inv)
    (ha : a ≠ 0) :
    gf_mul a inv = ok (1 : BitVec 8) := by
  sorry

/-- The inverse S-box undoes the S-box. -/
theorem inv_sbox_inverse
    (x y : BitVec 8)
    (h : sbox x = ok y) :
    inv_sbox y = ok x := by
  sorry

/-- Decryption inverts encryption: `decrypt(encrypt(x, k), k) = x`. -/
theorem decrypt_encrypt_roundtrip
    (block key : Slice (BitVec 8)) (ct pt : alloc.vec.Vec (BitVec 8))
    (h1 : aes_encrypt block key = ok ct)
    (h2 : aes_decrypt (alloc.vec.Vec.deref ct) key = ok pt) :
    alloc.vec.Vec.deref pt = block := by
  sorry