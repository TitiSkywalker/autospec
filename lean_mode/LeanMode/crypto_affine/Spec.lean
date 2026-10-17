import LeanMode.crypto_affine.CryptoAffine
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_affine

set_option linter.unusedVariables false

/-!
# Specification: Affine Cipher

The affine cipher is a monoalphabetic substitution cipher over the 26-letter
alphabet:

    encryption  E(x) = (a·x + b) mod 26
    decryption  D(y) = a⁻¹ · (y − b) mod 26

The key `a` must be coprime to 26 (`gcd(a, 26) = 1`) so that the modular
inverse `a⁻¹ mod 26` exists.  This testcase is the cleanest bridge between
number theory and crypto: the roundtrip `decrypt(encrypt(x)) = x` holds *iff*
`a` is invertible mod 26.

Conformance is checked by `#eval` in `Test_test.lean`.
-/

/-- Correctness of `gcd`: the result is the greatest common divisor. -/
theorem gcd_correct (a b result : Nat)
    (h : crypto_affine.gcd a b = ok result) :
    result = Nat.gcd a b := by
  sorry

/-- Correctness of the modular inverse: `mod_inverse a m = ok d` implies
    `(a · d) mod m = 1 mod m`. -/
theorem mod_inverse_correct (a m d : Nat)
    (h : mod_inverse a m = ok d) :
    (a * d) % m = 1 % m := by
  sorry

/-- Affine roundtrip: decryption inverts encryption when the multiplicative
    key `a` is coprime to 26 (so `a⁻¹ mod 26` exists).

    The `x < 26` precondition is carried by `h_enc` (via the `assert!` in
    `affine_encrypt`). -/
theorem affine_decrypt_encrypt_roundtrip (x a b result : Nat)
    (h_coprime : Nat.gcd a 26 = 1)
    (h_b : b < 26)
    (h_enc : affine_encrypt x a b = ok result) :
    affine_decrypt result a b = ok x := by
  sorry
