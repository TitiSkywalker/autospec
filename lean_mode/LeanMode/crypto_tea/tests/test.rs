// TEA test vectors (verified against the reference implementation).

// All-zeros plaintext and key: known ciphertext.
#[test]
fn test_tea_encrypt_zero() {
    let ct = crypto_tea::tea_encrypt(0x0000000000000000u64, 0x0000000000000000u64, 0x0000000000000000u64);
    assert_eq!(ct, 0x94BAA94041EA3A0Au64);
}

// Roundtrip: decrypt(encrypt(x, k), k) = x.
#[test]
fn test_tea_roundtrip() {
    let key0 = 0x17E223B010084520u64;
    let key1 = 0x8AA400EEF2D681C3u64;
    let pt = 0x558AF6A031D4CC1Bu64;
    let ct = crypto_tea::tea_encrypt(pt, key0, key1);
    let back = crypto_tea::tea_decrypt(ct, key0, key1);
    assert_eq!(back, pt);
}

// Known test vector: pt=0x0123456789ABCDEF, key=(0x0123456789ABCDEF, 0xFEDCBA9876543210)
#[test]
fn test_tea_encrypt_known() {
    let ct = crypto_tea::tea_encrypt(0x0123456789ABCDEFu64, 0x0123456789ABCDEFu64, 0xFEDCBA9876543210u64);
    assert_eq!(ct, 0x9cffb8e3b7931a67u64);
}

// Decrypt of the known vector.
#[test]
fn test_tea_decrypt_known() {
    let pt = crypto_tea::tea_decrypt(0x9cffb8e3b7931a67u64, 0x0123456789ABCDEFu64, 0xFEDCBA9876543210u64);
    assert_eq!(pt, 0x0123456789ABCDEFu64);
}