//! Test vectors for the LWE (Learning With Errors) homomorphic encryption scheme.
//!
//! This test suite verifies three properties:
//!
//! 1. **wrapping_dot_product**: correctly computes a0*b0 + a1*b1 with wrapping arithmetic.
//!
//! 2. **Encrypt/decrypt correctness**:
//!    ```
//!    encrypt: body = mask·sk + encoded_msg + noise
//!    decrypt: body - mask·sk = encoded_msg + noise
//!    ```
//!    The mask·sk dot product cancels perfectly, leaving encoded_msg + noise.
//!
//! 3. **Additive homomorphism**:
//!    ```
//!    decrypt(ct_a + ct_b) = decrypt(ct_a) + decrypt(ct_b)
//!    ```
//!    This holds because the dot product is linear: (mask_a + mask_b)·sk = mask_a·sk + mask_b·sk.
//!    Note: noise is also additive — the summed ciphertext has noise_a + noise_b.

use crypto_lwe;

// ---------------------------------------------------------------------------
// Helper: encrypt a plaintext value and return the ciphertext body.
// ---------------------------------------------------------------------------
fn encrypt(
    mask: (u64, u64),
    sk: (u64, u64),
    encoded_msg: u64,
    noise: u64,
) -> u64 {
    crypto_lwe::lwe_encrypt_body(mask.0, mask.1, sk.0, sk.1, encoded_msg, noise)
}

// ---------------------------------------------------------------------------
// Helper: decrypt a ciphertext body.
// ---------------------------------------------------------------------------
fn decrypt(mask: (u64, u64), sk: (u64, u64), body: u64) -> u64 {
    crypto_lwe::lwe_decrypt_body(mask.0, mask.1, sk.0, sk.1, body)
}

// ---------------------------------------------------------------------------
// Helper: encrypt a message with a given delta and noise, returning
// (encoded_msg, body, decrypted). The "encoded_msg" is msg * delta.
// ---------------------------------------------------------------------------
fn encrypt_msg(msg: u64, delta: u64, noise: u64, mask: (u64, u64), sk: (u64, u64)) -> (u64, u64, u64) {
    let encoded = msg.wrapping_mul(delta);
    let body = encrypt(mask, sk, encoded, noise);
    let decrypted = decrypt(mask, sk, body);
    (encoded, body, decrypted)
}

// ---------------------------------------------------------------------------
// Helper: add two ciphertexts.
// ---------------------------------------------------------------------------
fn ciphertext_add(
    ct_a: (u64, u64, u64),
    ct_b: (u64, u64, u64),
) -> (u64, u64, u64) {
    crypto_lwe::lwe_ciphertext_add(ct_a.0, ct_a.1, ct_a.2, ct_b.0, ct_b.1, ct_b.2)
}

// ===========================================================================
// PART 1: wrapping_dot_product
// ===========================================================================

#[test]
fn dot_product_both_zero() {
    // 0*0 + 0*0 = 0
    assert_eq!(crypto_lwe::wrapping_dot_product(0, 0, 0, 0), 0);
}

#[test]
fn dot_product_simple() {
    // 1*2 + 3*4 = 2 + 12 = 14
    assert_eq!(crypto_lwe::wrapping_dot_product(1, 3, 2, 4), 14);
}

#[test]
fn dot_product_wraps_on_overflow() {
    // u64::MAX * 1 + u64::MAX * 1 = 0xFFFFFFFFFFFFFFFE
    assert_eq!(
        crypto_lwe::wrapping_dot_product(u64::MAX, u64::MAX, 1, 1),
        u64::MAX.wrapping_mul(1).wrapping_add(u64::MAX.wrapping_mul(1))
    );
}

#[test]
fn dot_product_with_zero_secret_key() {
    // a·0 = 0, regardless of mask values
    assert_eq!(crypto_lwe::wrapping_dot_product(42, 999, 0, 0), 0);
}

#[test]
fn dot_product_one_sided() {
    // Only the sk0 component is non-zero: mask0*sk0 + 0*0 = 5*10 = 50
    assert_eq!(crypto_lwe::wrapping_dot_product(5, 0, 10, 0), 50);
}

// ===========================================================================
// PART 2: Encrypt / Decrypt correctness
// ===========================================================================

#[test]
fn encrypt_decrypt_small_values() {
    // Simple case: decrypt recovers exactly encoded_msg + noise.
    let mask = (42u64, 123u64);
    let sk = (7u64, 13u64);
    let msg = 3u64;
    let delta = 256u64;
    let noise = 17u64;

    let (encoded, _body, decrypted) = encrypt_msg(msg, delta, noise, mask, sk);

    assert_eq!(
        decrypted,
        encoded.wrapping_add(noise),
        "decrypted ({decrypted}) should equal encoded ({encoded}) + noise ({noise})"
    );
}

#[test]
fn encrypt_decrypt_binary_secret_key() {
    // Secret key is binary (0 or 1), which is the typical LWE setting.
    let mask = (0xABCDu64, 0x1234u64);
    let sk = (1u64, 0u64);
    let msg = 10u64;
    let delta = 100u64;
    let noise = 3u64;

    let (encoded, _body, decrypted) = encrypt_msg(msg, delta, noise, mask, sk);

    assert_eq!(decrypted, encoded.wrapping_add(noise));
}

#[test]
fn encrypt_decrypt_zero_noise() {
    // With zero noise, decryption recovers the exact encoded message.
    let mask = (100u64, 200u64);
    let sk = (5u64, 6u64);
    let msg = 5u64;
    let delta = 100u64;
    let noise = 0u64;

    let (encoded, _body, decrypted) = encrypt_msg(msg, delta, noise, mask, sk);

    assert_eq!(decrypted, encoded, "with zero noise, decrypted should equal encoded exactly");
}

#[test]
fn encrypt_decrypt_large_mask() {
    // Large mask values, binary secret key — still works.
    let mask = (0xDEADBEEFCAFEu64, 0xBABEFACE1234u64);
    let sk = (0u64, 1u64);
    let msg = 1u64;
    let delta = 0x0100000000000000u64; // delta = 2^56
    let noise = 0xFFu64;

    let (encoded, _body, decrypted) = encrypt_msg(msg, delta, noise, mask, sk);

    assert_eq!(decrypted, encoded.wrapping_add(noise));
}

#[test]
fn encrypt_decrypt_wrapping_arithmetic() {
    // Values near u64::MAX — verifies wrapping arithmetic works correctly.
    let mask = (u64::MAX, u64::MAX);
    let sk = (u64::MAX, u64::MAX);
    let encoded = u64::MAX;
    let noise = 1u64;

    let body = encrypt(mask, sk, encoded, noise);
    let decrypted = decrypt(mask, sk, body);

    assert_eq!(decrypted, encoded.wrapping_add(noise));
}

#[test]
fn encrypt_decrypt_message_is_additive() {
    // encrypt(msg_a) + encrypt(msg_b) decrypts to (msg_a + msg_b) * delta + noise_sum
    // but we test encoding directly here: encoded(msg_a) + encoded(msg_b) = encoded(msg_a + msg_b)
    let mask = (10u64, 20u64);
    let sk = (3u64, 5u64);
    let delta = 256u64;
    let msg_a = 2u64;
    let msg_b = 3u64;
    let noise_a = 5u64;
    let noise_b = 7u64;

    let encoded_a = msg_a.wrapping_mul(delta);
    let encoded_b = msg_b.wrapping_mul(delta);

    let body_a = encrypt(mask, sk, encoded_a, noise_a);
    let body_b = encrypt(mask, sk, encoded_b, noise_b);

    let dec_a = decrypt(mask, sk, body_a);
    let dec_b = decrypt(mask, sk, body_b);

    // Each decrypts to encoded_msg + noise
    assert_eq!(dec_a, encoded_a.wrapping_add(noise_a));
    assert_eq!(dec_b, encoded_b.wrapping_add(noise_b));

    // And the encoded messages are additive: encoded_a + encoded_b = (msg_a + msg_b) * delta
    assert_eq!(
        dec_a.wrapping_add(dec_b),
        encoded_a.wrapping_add(encoded_b).wrapping_add(noise_a).wrapping_add(noise_b)
    );
}

// ===========================================================================
// PART 3: Ciphertext addition (additive homomorphism)
// ===========================================================================

#[test]
fn ciphertext_add_basic() {
    // Element-wise addition: (1,2,3) + (4,5,6) = (5,7,9)
    let (s0, s1, s_body) = crypto_lwe::lwe_ciphertext_add(1, 2, 3, 4, 5, 6);
    assert_eq!((s0, s1, s_body), (5, 7, 9));
}

#[test]
fn ciphertext_add_wraps_on_overflow() {
    // u64::MAX + 1 = 0 (wrapping)
    let (s0, s1, s_body) = crypto_lwe::lwe_ciphertext_add(u64::MAX, u64::MAX, 0, 1, 2, 3);
    assert_eq!(s0, 0); // u64::MAX + 1 wraps to 0
    assert_eq!(s1, 1); // u64::MAX + 2 wraps to 1
    assert_eq!(s_body, 3); // 0 + 3 = 3
}

#[test]
fn homomorphism_encrypt_then_add_then_decrypt() {
    // Core property: decrypt(ct_a + ct_b) = decrypt(ct_a) + decrypt(ct_b)
    //
    // This works because:
    //   ct_a + ct_b = (mask_a + mask_b, body_a + body_b)
    //   decrypt(sum) = (body_a + body_b) - (mask_a + mask_b)·sk
    //                = (body_a - mask_a·sk) + (body_b - mask_b·sk)
    //                = decrypt(ct_a) + decrypt(ct_b)
    let sk = (7u64, 13u64);
    let delta = 256u64;
    let msg_a = 3u64;
    let msg_b = 5u64;
    let noise_a = 17u64;
    let noise_b = 23u64;

    // Encrypt both messages with different masks
    let mask_a = (42u64, 123u64);
    let mask_b = (10u64, 20u64);

    let encoded_a = msg_a.wrapping_mul(delta);
    let encoded_b = msg_b.wrapping_mul(delta);

    let body_a = encrypt(mask_a, sk, encoded_a, noise_a);
    let body_b = encrypt(mask_b, sk, encoded_b, noise_b);

    // Decrypt individually
    let plain_a = decrypt(mask_a, sk, body_a);
    let plain_b = decrypt(mask_b, sk, body_b);

    // Add ciphertexts
    let ct_a = (mask_a.0, mask_a.1, body_a);
    let ct_b = (mask_b.0, mask_b.1, body_b);
    let (mask_sum0, mask_sum1, body_sum) = ciphertext_add(ct_a, ct_b);

    // Decrypt the sum
    let plain_sum = decrypt((mask_sum0, mask_sum1), sk, body_sum);

    // Homomorphism: decrypt(sum) == decrypt(a) + decrypt(b)
    assert_eq!(
        plain_sum,
        plain_a.wrapping_add(plain_b),
        "additive homomorphism: decrypt(ct_a + ct_b) should equal decrypt(ct_a) + decrypt(ct_b)"
    );

    // Also verify what the plaintext sum actually represents:
    // plain_a + plain_b = (encoded_a + noise_a) + (encoded_b + noise_b)
    //                   = (msg_a + msg_b) * delta + (noise_a + noise_b)
    let expected_encoded = (msg_a + msg_b).wrapping_mul(delta);
    let expected_noise = noise_a.wrapping_add(noise_b);
    assert_eq!(
        plain_sum,
        expected_encoded.wrapping_add(expected_noise),
        "decrypted sum = encoded(msg_a + msg_b) + noise_sum"
    );
}

#[test]
fn homomorphism_binary_secret_key() {
    // Same property with a binary secret key (sk0=0, sk1=1).
    let sk = (0u64, 1u64);
    // delta = 1 (no scaling) — we pass msg directly as encoded
    let msg_a = 500u64;
    let msg_b = 700u64;
    let noise_a = 1u64;
    let noise_b = 2u64;

    let mask_a = (100u64, 200u64);
    let mask_b = (300u64, 400u64);

    let body_a = encrypt(mask_a, sk, msg_a, noise_a);
    let body_b = encrypt(mask_b, sk, msg_b, noise_b);

    let plain_a = decrypt(mask_a, sk, body_a);
    let plain_b = decrypt(mask_b, sk, body_b);

    let ct_a = (mask_a.0, mask_a.1, body_a);
    let ct_b = (mask_b.0, mask_b.1, body_b);
    let (mask_sum0, mask_sum1, body_sum) = ciphertext_add(ct_a, ct_b);

    let plain_sum = decrypt((mask_sum0, mask_sum1), sk, body_sum);

    assert_eq!(plain_sum, plain_a.wrapping_add(plain_b));
}

#[test]
fn homomorphism_noise_grows_with_addition() {
    // Demonstrate that noise accumulates: after adding ciphertexts,
    // the total noise is noise_a + noise_b.
    // This is the key limitation of LWE-based homomorphic encryption.
    let sk = (7u64, 13u64);
    let mask = (42u64, 123u64);

    let noise_a = 10u64;
    let noise_b = 20u64;
    let _msg_a = 0u64; // encoded = 0
    let _msg_b = 0u64; // encoded = 0

    // When encoded messages are 0, decrypted = noise only
    let body_a = encrypt(mask, sk, 0, noise_a);
    let body_b = encrypt(mask, sk, 0, noise_b);

    let plain_a = decrypt(mask, sk, body_a);
    let plain_b = decrypt(mask, sk, body_b);

    assert_eq!(plain_a, noise_a, "plain_a should be just noise_a");
    assert_eq!(plain_b, noise_b, "plain_b should be just noise_b");

    // Add the two zero-message ciphertexts
    let ct_a = (mask.0, mask.1, body_a);
    let ct_b = (mask.0, mask.1, body_b);
    let (mask_sum0, mask_sum1, body_sum) = ciphertext_add(ct_a, ct_b);

    let plain_sum = decrypt((mask_sum0, mask_sum1), sk, body_sum);

    // The result is noise_a + noise_b (since messages are zero)
    assert_eq!(
        plain_sum,
        noise_a.wrapping_add(noise_b),
        "noise is additive: noise_sum = noise_a + noise_b"
    );
}