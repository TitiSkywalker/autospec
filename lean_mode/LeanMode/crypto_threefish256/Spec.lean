import LeanMode.crypto_threefish256.CryptoThreefish256
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_threefish256

set_option linter.unusedVariables false

/-!
# Specification: Threefish-256 (Skein block cipher)

Conformance is checked by `#eval` in `Test_test.lean`.  The theorem below
is the structural roundtrip.
-/

/-- Decryption inverts encryption:
`decrypt(encrypt(x, k, t), k, t) = x`. -/
theorem threefish256_decrypt_encrypt_roundtrip
    (block key : Array (BitVec 64) 4#usize) (tweak : Array (BitVec 64) 2#usize)
    (result ct : Array (BitVec 64) 4#usize)
    (h1 : threefish256_encrypt block key tweak = ok ct)
    (h2 : threefish256_decrypt ct key tweak = ok result) :
    result = block := by
  sorry
