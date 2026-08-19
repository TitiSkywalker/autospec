import LeanMode.bitvec_aes.BitvecAes
import Mathlib

open Aeneas Aeneas.Std Result
open bitvec_aes

set_option linter.unusedVariables false

/-!
# Specification: AES-128 (table-driven)

Encrypt/decrypt conformance against FIPS-197 / SP 800-38A is checked by
`#eval` in `Test_test.lean`.  The theorem below is the structural roundtrip.
-/

/-- Decryption inverts encryption: `decrypt(encrypt(x, k), k) = x`. -/
theorem decrypt_encrypt_roundtrip
    (block key : Slice (BitVec 8)) (ct pt : alloc.vec.Vec (BitVec 8))
    (h1 : aes_encrypt block key = ok ct)
    (h2 : aes_decrypt (alloc.vec.Vec.deref ct) key = ok pt) :
    alloc.vec.Vec.deref pt = block := by
  sorry