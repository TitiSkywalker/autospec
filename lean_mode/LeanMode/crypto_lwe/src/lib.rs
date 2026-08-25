//! LWE (Learning With Errors) encryption — simplified from tfhe-rs.
//!
//! Implements the core LWE encryption/decryption algorithm with dimension 2,
//! using explicit mask/noise parameters (no random generation).
//! Under the `bitvec` model, `u64` → `BitVec 64` (wrapping arithmetic).
//!
//! Source: tfhe-rs `core_crypto/algorithms/`
//! - `slice_algorithms.rs::slice_wrapping_dot_product`
//! - `lwe_encryption.rs::fill_lwe_mask_and_body_for_encryption_native_mod_compatible`
//! - `lwe_encryption.rs::decrypt_lwe_ciphertext_native_mod_compatible`
//! - `lwe_linear_algebra.rs::lwe_ciphertext_add_assign_native_mod_compatible`

/// Compute a wrapping dot product of two length-2 vectors.
/// From tfhe-rs: `slice_algorithms.rs::slice_wrapping_dot_product`
pub fn wrapping_dot_product(
    a0: u64, a1: u64,
    b0: u64, b1: u64,
) -> u64 {
    a0.wrapping_mul(b0).wrapping_add(a1.wrapping_mul(b1))
}

/// LWE encryption: compute the ciphertext body.
///
/// body = mask·sk + encoded_message + noise
///
/// From tfhe-rs: `lwe_encryption.rs::fill_lwe_mask_and_body_for_encryption_native_mod_compatible`
///   (line 107-113: mask_key_dot_product + encoded + noise)
pub fn lwe_encrypt_body(
    mask0: u64, mask1: u64,
    sk0: u64, sk1: u64,
    encoded_message: u64,
    noise: u64,
) -> u64 {
    let dot = wrapping_dot_product(mask0, mask1, sk0, sk1);
    dot.wrapping_add(encoded_message).wrapping_add(noise)
}

/// LWE decryption: recover the plaintext from the ciphertext body.
///
/// plaintext = body - mask·sk
///
/// From tfhe-rs: `lwe_encryption.rs::decrypt_lwe_ciphertext_native_mod_compatible`
///   (line 560-561: body - mask_key_dot_product)
pub fn lwe_decrypt_body(
    mask0: u64, mask1: u64,
    sk0: u64, sk1: u64,
    body: u64,
) -> u64 {
    let dot = wrapping_dot_product(mask0, mask1, sk0, sk1);
    body.wrapping_sub(dot)
}

/// LWE ciphertext addition: element-wise wrapping add of two ciphertexts.
///
/// ct_sum = ct_a + ct_b = (mask_a + mask_b, body_a + body_b)
///
/// From tfhe-rs: `lwe_linear_algebra.rs::lwe_ciphertext_add_assign_native_mod_compatible`
///   (line 101: slice_wrapping_add_assign(lhs, rhs))
pub fn lwe_ciphertext_add(
    mask0_a: u64, mask1_a: u64, body_a: u64,
    mask0_b: u64, mask1_b: u64, body_b: u64,
) -> (u64, u64, u64) {
    let sum0 = mask0_a.wrapping_add(mask0_b);
    let sum1 = mask1_a.wrapping_add(mask1_b);
    let sum_body = body_a.wrapping_add(body_b);
    (sum0, sum1, sum_body)
}