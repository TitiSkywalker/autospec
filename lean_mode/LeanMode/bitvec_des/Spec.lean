import LeanMode.bitvec_des.BitvecDes
import Mathlib

open Aeneas Aeneas.Std Result
open bitvec_des

set_option linter.unusedVariables false

/-!
# Specification: DES (table-driven, cast-fixed)

Conformance is checked by `#eval` in `Test_test.lean` (encrypt + decrypt
against NIST vectors).  The theorem below is the structural roundtrip.
-/

/-- Decryption inverts encryption: `decrypt(encrypt(x, k), k) = x`. -/
theorem des_decrypt_encrypt_roundtrip
    (block key : BitVec 64) (result : BitVec 64)
    (h : des_encrypt block key = ok result) :
    des_decrypt result key = ok block := by
  sorry