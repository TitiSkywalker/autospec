import LeanMode.crypto_rsa.CryptoRsa
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_rsa

set_option linter.unusedVariables false

/-!
# Specification: RSA Encryption and Decryption

RSA is an asymmetric cryptosystem: encryption uses a public key `(e, n)`,
decryption uses a private key `(d, n)`.  When `n = p * q` and
`e * d ≡ 1 (mod (p-1)*(q-1))`, the roundtrip `decrypt(encrypt(m)) = m` holds.

Conformance is checked by `#eval` in `Test_test.lean`.
-/

/-- Correctness of `gcd`: the result is the greatest common divisor. -/
theorem gcd_correct (a b result : Nat)
    (h : crypto_rsa.gcd a b = ok result) :
    result = Nat.gcd a b := by
  sorry

/-- Correctness of modular exponentiation: `mod_pow base exp modulus`
    computes `base^exp mod modulus`. -/
theorem mod_pow_correct (base exp modulus result : Nat)
    (h : mod_pow base exp modulus = ok result) :
    result = (base ^ exp) % modulus := by
  sorry

/-- Correctness of the modular inverse:
    `mod_inverse e phi = ok d` implies `(e * d) % phi = 1 % phi`. -/
theorem mod_inverse_correct (e phi d : Nat)
    (h : mod_inverse e phi = ok d) :
    (e * d) % phi = 1 % phi := by
  sorry

/-- RSA roundtrip: decryption inverts encryption when the keys are correct.
    Requires:
    - `n = p * q` (modulus is product of two primes)
    - `e` is coprime to `φ(n) = (p-1)*(q-1)`
    - `d` is the modular inverse of `e` modulo `φ(n)` -/
theorem rsa_decrypt_encrypt_roundtrip
    (message e d n p q : Nat)
    (h_n : n = p * q)
    (h_coprime : Nat.gcd e ((p - 1) * (q - 1)) = 1)
    (h_inv : mod_inverse e ((p - 1) * (q - 1)) = ok d)
    (h_enc : rsa_encrypt message e n = ok result) :
    rsa_decrypt result d n = ok message := by
  sorry