//! Threefish-256 block cipher — 72-round encrypt/decrypt (Skein hash function core).
//!
//! Under the `bitvec` int-model `u64` → `BitVec 64`, `u8` → `BitVec 8`.
//! Uses `rotate_left`/`rotate_right` (rewritten by `_bitvec_ops_to_bv`),
//! `wrapping_add`/`wrapping_sub` (scalar-lowered by AENEAS).

// ── constants ────────────────────────────────────────────────────────────────

const C240: u64 = 0x1BD11BDAA9FC1A22;

const R256: [[u8; 2]; 8] = [
    [14, 16], [52, 57], [23, 40], [5, 37],
    [25, 33], [46, 12], [58, 22], [32, 32],
];

const P256: [u8; 4] = [0, 3, 2, 1];

/// Precomputed `s as u64` for s = 0..19.
const S_U64: [u64; 19] = [
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9,
    10, 11, 12, 13, 14, 15, 16, 17, 18,
];

// ── helpers (while-based modulo/division) ────────────────────────────────────

fn mod5(x: usize) -> usize {
    let mut r = x;
    while r >= 5 { r = r - 5; }
    r
}

fn mod3(x: usize) -> usize {
    let mut r = x;
    while r >= 3 { r = r - 3; }
    r
}

fn mod8(x: usize) -> usize {
    let mut r = x;
    while r >= 8 { r = r - 8; }
    r
}

fn mod4(x: usize) -> usize {
    let mut r = x;
    while r >= 4 { r = r - 4; }
    r
}

fn div4(x: usize) -> usize {
    let mut q = 0usize;
    let mut r = x;
    while r >= 4 { r = r - 4; q = q + 1; }
    q
}

// ── key schedule ─────────────────────────────────────────────────────────────

/// Expand 256-bit key + 128-bit tweak into 19 round subkeys (flat Vec<u64>, 76 elements).
pub fn key_schedule(key: [u64; 4], tweak: [u64; 2]) -> Vec<u64> {
    let k0 = key[0];
    let k1 = key[1];
    let k2 = key[2];
    let k3 = key[3];
    let k4 = C240 ^ k0 ^ k1 ^ k2 ^ k3;
    let t0 = tweak[0];
    let t1 = tweak[1];
    let t2 = t0 ^ t1;

    let mut k = vec![0u64; 5];
    k[0] = k0; k[1] = k1; k[2] = k2; k[3] = k3; k[4] = k4;
    let mut t = vec![0u64; 3];
    t[0] = t0; t[1] = t1; t[2] = t2;

    let mut sk = vec![0u64; 76];
    let mut s = 0usize;
    while s < 19 {
        let mut i = 0usize;
        while i < 4 {
            let idx = mod5(s + i);
            let base = 4 * s + i;
            let kval = k[idx];
            sk[base] = kval;
            if i == 1 {
                let m = mod3(s);
                let add = t[m];
                sk[base] = sk[base].wrapping_add(add);
            } else if i == 2 {
                let m = mod3(s + 1);
                let add = t[m];
                sk[base] = sk[base].wrapping_add(add);
            } else if i == 3 {
                let add = S_U64[s];
                sk[base] = sk[base].wrapping_add(add);
            }
            i = i + 1;
        }
        s = s + 1;
    }
    sk
}

// ── encrypt / decrypt ────────────────────────────────────────────────────────

/// Threefish-256 single-block encryption (72 rounds).
pub fn threefish256_encrypt(block: [u64; 4], key: [u64; 4], tweak: [u64; 2]) -> [u64; 4] {
    let sk = key_schedule(key, tweak);
    let mut v0 = block[0];
    let mut v1 = block[1];
    let mut v2 = block[2];
    let mut v3 = block[3];
    let mut d = 0usize;
    while d < 72 {
        let p0 = v0; let p1 = v1; let p2 = v2; let p3 = v3;
        let mut j = 0usize;
        while j < 2 {
            let x0; let x1;
            if j == 0 { x0 = p0; x1 = p1; } else { x0 = p2; x1 = p3; }
            let (e0, e1) = if mod4(d) == 0 {
                let q = div4(d);
                let s0 = sk[4 * q + 2 * j];
                let s1 = sk[4 * q + 2 * j + 1];
                (x0.wrapping_add(s0), x1.wrapping_add(s1))
            } else {
                (x0, x1)
            };
            let r = R256[mod8(d)][j];
            let y0 = e0.wrapping_add(e1);
            let y1 = e1.rotate_left(r as u32) ^ y0;
            let pi0 = P256[2 * j] as usize;
            let pi1 = P256[2 * j + 1] as usize;
            if pi0 == 0 { v0 = y0; } else if pi0 == 1 { v1 = y0; } else if pi0 == 2 { v2 = y0; } else { v3 = y0; }
            if pi1 == 0 { v0 = y1; } else if pi1 == 1 { v1 = y1; } else if pi1 == 2 { v2 = y1; } else { v3 = y1; }
            j = j + 1;
        }
        d = d + 1;
    }
    let s0 = sk[72]; let s1 = sk[73]; let s2 = sk[74]; let s3 = sk[75];
    v0 = v0.wrapping_add(s0);
    v1 = v1.wrapping_add(s1);
    v2 = v2.wrapping_add(s2);
    v3 = v3.wrapping_add(s3);
    [v0, v1, v2, v3]
}

/// Threefish-256 single-block decryption (72 rounds).
pub fn threefish256_decrypt(block: [u64; 4], key: [u64; 4], tweak: [u64; 2]) -> [u64; 4] {
    let sk = key_schedule(key, tweak);
    let mut v0 = block[0];
    let mut v1 = block[1];
    let mut v2 = block[2];
    let mut v3 = block[3];
    v0 = v0.wrapping_sub(sk[72]);
    v1 = v1.wrapping_sub(sk[73]);
    v2 = v2.wrapping_sub(sk[74]);
    v3 = v3.wrapping_sub(sk[75]);
    let mut d = 72usize;
    while d > 0 {
        d = d - 1;
        let p0 = v0; let p1 = v1; let p2 = v2; let p3 = v3;
        let mut j = 0usize;
        while j < 2 {
            let pi0 = P256[2 * j] as usize;
            let pi1 = P256[2 * j + 1] as usize;
            let f0; let f1;
            if pi0 == 0 { f0 = p0; } else if pi0 == 1 { f0 = p1; } else if pi0 == 2 { f0 = p2; } else { f0 = p3; }
            if pi1 == 0 { f1 = p0; } else if pi1 == 1 { f1 = p1; } else if pi1 == 2 { f1 = p2; } else { f1 = p3; }
            let r = R256[mod8(d)][j];
            let x1 = (f0 ^ f1).rotate_right(r as u32);
            let x0 = f0.wrapping_sub(x1);
            let (t0, t1) = if mod4(d) == 0 {
                let q = div4(d);
                (x0.wrapping_sub(sk[4 * q + 2 * j]),
                 x1.wrapping_sub(sk[4 * q + 2 * j + 1]))
            } else {
                (x0, x1)
            };
            if j == 0 { v0 = t0; v1 = t1; } else { v2 = t0; v3 = t1; }
            j = j + 1;
        }
    }
    [v0, v1, v2, v3]
}