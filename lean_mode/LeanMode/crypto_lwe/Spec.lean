import LeanMode.crypto_lwe.CryptoLwe
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_lwe

set_option linter.unusedVariables false

/-!
# Specification: LWE (Learning With Errors) Encryption

LWE encryption with dimension 2, BitVec 64 model (wrapping arithmetic).

-/

/-- LWE decryption exactly cancels the mask·key dot product.
    If `body = mask·sk + encoded_message + noise`,
    then `decrypt(body) = encoded_message + noise`. -/
theorem lwe_encrypt_decrypt_exact
    (mask0 mask1 sk0 sk1 encoded_message noise body : BitVec 64)
    (h_enc : lwe_encrypt_body mask0 mask1 sk0 sk1 encoded_message noise = ok body) :
    lwe_decrypt_body mask0 mask1 sk0 sk1 body = ok (encoded_message + noise) := by
  sorry

/-- LWE additive homomorphism: decryption distributes over ciphertext addition.
    `decrypt(ct1 + ct2) = decrypt(ct1) + decrypt(ct2)`.
-/
theorem lwe_additive_homomorphism
    (mask0_a mask1_a body_a mask0_b mask1_b body_b sk0 sk1 : BitVec 64)
    (mask0_sum mask1_sum body_sum decrypt_a decrypt_b : BitVec 64)
    (h_add : lwe_ciphertext_add mask0_a mask1_a body_a mask0_b mask1_b body_b = ok (mask0_sum, mask1_sum, body_sum))
    (h_dec_a : lwe_decrypt_body mask0_a mask1_a sk0 sk1 body_a = ok decrypt_a)
    (h_dec_b : lwe_decrypt_body mask0_b mask1_b sk0 sk1 body_b = ok decrypt_b) :
    lwe_decrypt_body mask0_sum mask1_sum sk0 sk1 body_sum = ok (decrypt_a + decrypt_b) := by
  sorry
