import LeanMode.bitvec_tea.BitvecTea
import Mathlib

open Aeneas Aeneas.Std Result
open bitvec_tea

set_option linter.unusedVariables false

/-!
# Specification: TEA (Tiny Encryption Algorithm)

Conformance is checked by `#eval` in `Test_test.lean`.  The theorem below
is the structural roundtrip.
-/

/-- Decryption inverts encryption: `decrypt(encrypt(x, k0, k1), k0, k1) = x`. -/
theorem tea_decrypt_encrypt_roundtrip
    (block key0 key1 : BitVec 64) (result : BitVec 64)
    (h : tea_encrypt block key0 key1 = ok result) :
    tea_decrypt result key0 key1 = ok block := by
  sorry