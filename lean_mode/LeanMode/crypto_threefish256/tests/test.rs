// Threefish-256 test vectors from Crypto++ / Skein specification.
// u64 values are little-endian (matching the reference implementation's
// `u64::from_le_bytes` conversion).

use crypto_threefish256::*;

// All-zero key, tweak, and plaintext.
#[test]
fn test_threefish256_zero() {
    let ct = threefish256_encrypt(
        [0u64; 4],
        [0u64; 4],
        [0u64; 2],
    );
    assert_eq!(ct[0], 0x94EEEA8B1F2ADA84u64);
    assert_eq!(ct[1], 0xADF103313EAE6670u64);
    assert_eq!(ct[2], 0x952419A1F4B16D53u64);
    assert_eq!(ct[3], 0xD83F13E63C9F6B11u64);
}

// Known vector from Crypto++ test suite.
#[test]
fn test_threefish256_known() {
    let key = [
        0x1716151413121110u64,
        0x1F1E1D1C1B1A1918u64,
        0x2726252423222120u64,
        0x2F2E2D2C2B2A2928u64,
    ];
    let tweak = [
        0x0706050403020100u64,
        0x0F0E0D0C0B0A0908u64,
    ];
    let pt = [
        0xF8F9FAFBFCFDFEFFu64,
        0xF0F1F2F3F4F5F6F7u64,
        0xE8E9EAEBECEDEEEFu64,
        0xE0E1E2E3E4E5E6E7u64,
    ];
    let ct = threefish256_encrypt(pt, key, tweak);
    assert_eq!(ct[0], 0xDF8FEA0EFF91D0E0u64);
    assert_eq!(ct[1], 0xD50AD82EE69281C9u64);
    assert_eq!(ct[2], 0x76F48D58085D869D_u64);
    assert_eq!(ct[3], 0xDF975E95B5567065u64);
}

// Roundtrip: encrypt then decrypt returns original.
#[test]
fn test_threefish256_roundtrip() {
    let key = [
        0x1716151413121110u64,
        0x1F1E1D1C1B1A1918u64,
        0x2726252423222120u64,
        0x2F2E2D2C2B2A2928u64,
    ];
    let tweak = [0u64; 2];
    let pt = [
        0xF8F9FAFBFCFDFEFFu64,
        0xF0F1F2F3F4F5F6F7u64,
        0xE8E9EAEBECEDEEEFu64,
        0xE0E1E2E3E4E5E6E7u64,
    ];
    let ct = threefish256_encrypt(pt, key, tweak);
    let back = threefish256_decrypt(ct, key, tweak);
    assert_eq!(back[0], pt[0]);
    assert_eq!(back[1], pt[1]);
    assert_eq!(back[2], pt[2]);
    assert_eq!(back[3], pt[3]);
}

// Decrypt of the known vector.
#[test]
fn test_threefish256_decrypt_known() {
    let key = [
        0x1716151413121110u64,
        0x1F1E1D1C1B1A1918u64,
        0x2726252423222120u64,
        0x2F2E2D2C2B2A2928u64,
    ];
    let tweak = [
        0x0706050403020100u64,
        0x0F0E0D0C0B0A0908u64,
    ];
    let pt = [
        0xF8F9FAFBFCFDFEFFu64,
        0xF0F1F2F3F4F5F6F7u64,
        0xE8E9EAEBECEDEEEFu64,
        0xE0E1E2E3E4E5E6E7u64,
    ];
    let ct = [
        0xDF8FEA0EFF91D0E0u64,
        0xD50AD82EE69281C9u64,
        0x76F48D58085D869Du64,
        0xDF975E95B5567065u64,
    ];
    let back = threefish256_decrypt(ct, key, tweak);
    assert_eq!(back[0], pt[0]);
    assert_eq!(back[1], pt[1]);
    assert_eq!(back[2], pt[2]);
    assert_eq!(back[3], pt[3]);
}