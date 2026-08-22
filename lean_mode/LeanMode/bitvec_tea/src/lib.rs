//! TEA (Tiny Encryption Algorithm) single-block encryption and decryption.
//!
//! TEA is a Feistel cipher with 32 rounds, operating on 64-bit blocks
//! with a 128-bit key.  Under the `bitvec` int-model `u32` → `BitVec 32`,
//! `u64` → `BitVec 64`.

/// TEA single-block encryption (32 rounds).
/// `key0`, `key1`: two 64-bit halves of the 128-bit key.
pub fn tea_encrypt(block: u64, key0: u64, key1: u64) -> u64 {
    let k0 = key0 as u32;
    let k1 = (key0 >> 32u32) as u32;
    let k2 = key1 as u32;
    let k3 = (key1 >> 32u32) as u32;
    let mut v0 = block as u32;
    let mut v1 = (block >> 32u32) as u32;
    let mut sum = 0u32;
    let delta = 0x9E3779B9u32;
    let mut i = 0u32;
    while i < 32u32 {
        sum = sum.wrapping_add(delta);
        let t0 = (v1 << 4u32).wrapping_add(k0);
        let t1 = v1.wrapping_add(sum);
        let t2 = (v1 >> 5u32).wrapping_add(k1);
        v0 = v0.wrapping_add(t0 ^ t1 ^ t2);
        let t3 = (v0 << 4u32).wrapping_add(k2);
        let t4 = v0.wrapping_add(sum);
        let t5 = (v0 >> 5u32).wrapping_add(k3);
        v1 = v1.wrapping_add(t3 ^ t4 ^ t5);
        i = i + 1u32;
    }
    (v0 as u64) | ((v1 as u64) << 32u32)
}

/// TEA single-block decryption (32 rounds).
pub fn tea_decrypt(block: u64, key0: u64, key1: u64) -> u64 {
    let k0 = key0 as u32;
    let k1 = (key0 >> 32u32) as u32;
    let k2 = key1 as u32;
    let k3 = (key1 >> 32u32) as u32;
    let mut v0 = block as u32;
    let mut v1 = (block >> 32u32) as u32;
    let mut sum = 0xC6EF3720u32;
    let delta = 0x9E3779B9u32;
    let mut i = 0u32;
    while i < 32u32 {
        let t0 = (v0 << 4u32).wrapping_add(k2);
        let t1 = v0.wrapping_add(sum);
        let t2 = (v0 >> 5u32).wrapping_add(k3);
        v1 = v1.wrapping_sub(t0 ^ t1 ^ t2);
        let t3 = (v1 << 4u32).wrapping_add(k0);
        let t4 = v1.wrapping_add(sum);
        let t5 = (v1 >> 5u32).wrapping_add(k1);
        v0 = v0.wrapping_sub(t3 ^ t4 ^ t5);
        sum = sum.wrapping_sub(delta);
        i = i + 1u32;
    }
    (v0 as u64) | ((v1 as u64) << 32u32)
}