import LeanMode.crypto_dh.CryptoDh
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_dh

set_option linter.unusedVariables false

/-!
# Specification: Diffie-Hellman Key Exchange

DH allows two parties to agree on a shared secret over an insecure channel.
The core correctness property is commutativity:

  `(g^a)^b ≡ (g^b)^a (mod p)`

## Requirements on `g` and `p`

- **Correctness** (what this theorem proves): commutativity holds for any
  `g` and any `p > 1` — the identity is purely algebraic.  The only
  hypothesis is `h_p : 1 < p` (to avoid division by zero in `mod_pow`).
- **Security** (outside the scope of this pipeline): `p` must be a large
  prime and `g` must be a primitive root modulo `p` (generator of the
  multiplicative group).  These are required for the discrete-logarithm
  problem to be hard, but they do not affect functional correctness and
  are not stated as hypotheses.

Conformance is checked by `#eval` in `Test_test.lean`.
-/

/-- Correctness of modular exponentiation: `mod_pow base exp modulus`
    computes `base^exp mod modulus`. -/
theorem mod_pow_correct (base exp modulus result : Nat)
    (h : mod_pow base exp modulus = ok result) :
    result = (base ^ exp) % modulus := by
  sorry

/-- Diffie-Hellman key agreement:

    Alice computes  `pub_a = g^a mod p`,  then  `shared = pub_b^a mod p`.
    Bob computes    `pub_b = g^b mod p`,  then  `shared = pub_a^b mod p`.

    The theorem states the two shared-secret computations produce equal results. -/
theorem dh_shared_secret_agreement (g a b p pub_a pub_b : Nat)
    (h_p : 1 < p)
    (h_pub_a : dh_public_key g a p = ok pub_a)
    (h_pub_b : dh_public_key g b p = ok pub_b) :
    dh_shared_secret pub_b a p = dh_shared_secret pub_a b p := by
  sorry
