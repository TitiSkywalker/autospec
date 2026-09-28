import LeanMode.crypto_hill.CryptoHill
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_hill

set_option linter.unusedVariables false

/-!
# Specification: Hill Cipher (2×2 matrix, mod 26)

The Hill cipher is a polygraphic substitution cipher based on linear algebra:

    encryption  c = K·p mod 26
    decryption  p = K⁻¹·c mod 26,   K⁻¹ = det(K)⁻¹ · adj(K)

The key matrix `K` is invertible mod 26 iff its determinant is coprime to 26
(`gcd(det K, 26) = 1`).  This testcase bridges linear algebra, number theory,
and crypto: the roundtrip `decrypt(encrypt(p)) = p` is exactly the matrix
identity `K⁻¹·K = I mod 26`.

Conformance is checked by `#eval` in `Test_test.lean`.
-/

/-- Correctness of the modular inverse: `mod_inverse a m = ok d` implies
    `(a · d) mod m = 1`. -/
theorem mod_inverse_correct (a m d : Int)
    (h : mod_inverse a m = ok d) :
    (a * d) % m = 1 := by
  sorry

/-- Hill roundtrip: decryption inverts encryption when the determinant of the
    key matrix is coprime to 26 (so `K` is invertible mod 26).

    The plaintext range preconditions `0 ≤ pi < 26` are carried by `h_enc`
    (via the `assert!`s in `hill_encrypt`). -/
theorem hill_decrypt_encrypt_roundtrip
    (p1 p2 k11 k12 k21 k22 c1 c2 : Int)
    (h_coprime : Int.gcd (k11 * k22 - k12 * k21) 26 = 1)
    (h_enc : hill_encrypt p1 p2 k11 k12 k21 k22 = ok (c1, c2)) :
    hill_decrypt c1 c2 k11 k12 k21 k22 = ok (p1, p2) := by
  sorry
