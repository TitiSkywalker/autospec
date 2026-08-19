// DES test vectors (FIPS PUB 46-3 / NIST SP 800-17).

// NIST SP 800-17: key=133457799BBCDFF1, pt=0123456789ABCDEF → ct=85E813540F0AB405
#[test]
fn test_des_encrypt_sp800_17() {
    let ct = bitvec_des::des_encrypt(0x0123456789ABCDEF, 0x133457799BBCDFF1);
    assert_eq!(ct, 0x85E813540F0AB405);
}

// FIPS PUB 46-3: key=0123456789ABCDEF, pt=0123456789ABCDEF → ct=56CC09E7CFDC4CEF
#[test]
fn test_des_encrypt_fips46_3() {
    let ct = bitvec_des::des_encrypt(0x0123456789ABCDEF, 0x0123456789ABCDEF);
    assert_eq!(ct, 0x56CC09E7CFDC4CEF);
}

// All-zero key, all-zero plaintext → 8CA64DE9C1B123A7
#[test]
fn test_des_zero_zero() {
    let ct = bitvec_des::des_encrypt(0x0000000000000000, 0x0000000000000000);
    assert_eq!(ct, 0x8CA64DE9C1B123A7);
}

// NIST SP 800-17: key=7CA110454A1A6E57, pt=01A1D6D039776742 → ct=690F5B0D9A26939B
#[test]
fn test_des_encrypt_sp800_17_2() {
    let ct = bitvec_des::des_encrypt(0x01A1D6D039776742, 0x7CA110454A1A6E57);
    assert_eq!(ct, 0x690F5B0D9A26939B);
}

// Decrypt of FIPS 46-3 ciphertext
#[test]
fn test_des_decrypt_fips46_3() {
    let pt = bitvec_des::des_decrypt(0x56CC09E7CFDC4CEF, 0x0123456789ABCDEF);
    assert_eq!(pt, 0x0123456789ABCDEF);
}

// Decrypt roundtrip
#[test]
fn test_des_roundtrip() {
    let key = 0x0E329232EA6D0D73u64;
    let pt = 0x8787878787878787u64;
    let ct = bitvec_des::des_encrypt(pt, key);
    let back = bitvec_des::des_decrypt(ct, key);
    assert_eq!(back, pt);
}