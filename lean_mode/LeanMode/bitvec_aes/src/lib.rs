//! AES-128 single-block encryption (FIPS-197), **table-free and cast-free**.
//!
//! Everything is `u8` → `BitVec 8` under the `bitvec` int-model.  The S-box is
//! computed arithmetically (GF(2^8) multiplicative inverse + affine transform)
//! and MixColumns uses `xtime` — no lookup tables, no `as` casts.

/// `xtime(a)`: multiply by 2 in GF(2^8) (reduction polynomial x^8+x^4+x^3+x+1 = 0x11b).
pub fn xtime(a: u8) -> u8 {
    let hi = a & 0x80;
    let t = a << 1;
    if hi != 0 {
        t ^ 0x1b
    } else {
        t
    }
}

/// Multiply two bytes in GF(2^8) (shift-and-add).
pub fn gf_mul(a: u8, b: u8) -> u8 {
    let mut r = 0u8;
    let mut x = a;
    let mut y = b;
    while y != 0 {
        if y & 1 == 1 {
            r = r ^ x;
        }
        x = xtime(x);
        y = y >> 1;
    }
    r
}

/// Multiplicative inverse in GF(2^8): `a^254` (the multiplicative group has order 255).
pub fn gf_inv(a: u8) -> u8 {
    let mut r = 1u8;
    let mut base = a;
    let mut e = 254u8;
    while e > 0 {
        if e & 1 == 1 {
            r = gf_mul(r, base);
        }
        base = gf_mul(base, base);
        e = e >> 1;
    }
    r
}

/// AES S-box: affine transform of the GF(2^8) multiplicative inverse.
pub fn sbox(x: u8) -> u8 {
    let p = gf_inv(x);
    p ^ p.rotate_left(1) ^ p.rotate_left(2) ^ p.rotate_left(3) ^ p.rotate_left(4) ^ 0x63
}

/// `rcon(j)` = 2^(j-1) in GF(2^8) (the round constant).
pub fn rcon(j: usize) -> u8 {
    let mut r = 1u8;
    let mut k = 1usize;
    while k < j {
        r = xtime(r);
        k = k + 1;
    }
    r
}

/// AES-128 key expansion: 16-byte key → 176-byte round-key schedule (11 round keys).
pub fn key_expansion(key: &[u8]) -> Vec<u8> {
    let mut rk = vec![0u8; 176];
    let mut i = 0usize;
    while i < 16 {
        rk[i] = key[i];
        i = i + 1;
    }

    let mut i = 4usize;
    while i < 44 {
        let t0 = rk[4 * (i - 1) + 0];
        let t1 = rk[4 * (i - 1) + 1];
        let t2 = rk[4 * (i - 1) + 2];
        let t3 = rk[4 * (i - 1) + 3];
        let mut u0 = t0;
        let mut u1 = t1;
        let mut u2 = t2;
        let mut u3 = t3;
        if i % 4 == 0 {
            // RotWord [t0,t1,t2,t3] -> [t1,t2,t3,t0], then SubWord, then xor Rcon.
            let s0 = sbox(t1);
            let s1 = sbox(t2);
            let s2 = sbox(t3);
            let s3 = sbox(t0);
            u0 = s0 ^ rcon(i / 4);
            u1 = s1;
            u2 = s2;
            u3 = s3;
        }
        let w0 = rk[4 * (i - 4) + 0] ^ u0;
        let w1 = rk[4 * (i - 4) + 1] ^ u1;
        let w2 = rk[4 * (i - 4) + 2] ^ u2;
        let w3 = rk[4 * (i - 4) + 3] ^ u3;
        rk[4 * i + 0] = w0;
        rk[4 * i + 1] = w1;
        rk[4 * i + 2] = w2;
        rk[4 * i + 3] = w3;
        i = i + 1;
    }
    rk
}

/// XOR the 16-byte state with round key `round` (bytes `16*round .. 16*round+16` of `rk`).
pub fn add_round_key(state: Vec<u8>, rk: &[u8], round: usize) -> Vec<u8> {
    let mut out = state;
    let mut i = 0usize;
    while i < 16 {
        let t = out[i] ^ rk[16 * round + i];
        out[i] = t;
        i = i + 1;
    }
    out
}

/// SubBytes: apply the S-box to every byte.
pub fn sub_bytes(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut i = 0usize;
    while i < 16 {
        out[i] = sbox(state[i]);
        i = i + 1;
    }
    out
}

/// ShiftRows: rotate row `r` left by `r` positions (column-major 4x4 state).
pub fn shift_rows(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut r = 0usize;
    while r < 4 {
        let mut c = 0usize;
        while c < 4 {
            out[r + 4 * c] = state[r + 4 * ((c + r) % 4)];
            c = c + 1;
        }
        r = r + 1;
    }
    out
}

/// MixColumns: each column multiplied in GF(2^8) by the fixed MDS matrix.
pub fn mix_columns(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut c = 0usize;
    while c < 4 {
        let a0 = state[0 + 4 * c];
        let a1 = state[1 + 4 * c];
        let a2 = state[2 + 4 * c];
        let a3 = state[3 + 4 * c];
        let b0 = xtime(a0) ^ (xtime(a1) ^ a1) ^ a2 ^ a3;
        let b1 = a0 ^ xtime(a1) ^ (xtime(a2) ^ a2) ^ a3;
        let b2 = a0 ^ a1 ^ xtime(a2) ^ (xtime(a3) ^ a3);
        let b3 = (xtime(a0) ^ a0) ^ a1 ^ a2 ^ xtime(a3);
        out[0 + 4 * c] = b0;
        out[1 + 4 * c] = b1;
        out[2 + 4 * c] = b2;
        out[3 + 4 * c] = b3;
        c = c + 1;
    }
    out
}

/// AES-128 encrypt a single 16-byte block (FIPS-197 §5.1, 10 rounds).
pub fn aes_encrypt(block: &[u8], key: &[u8]) -> Vec<u8> {
    let rk = key_expansion(key);
    let mut state = block.to_vec();
    state = add_round_key(state, &rk, 0);
    let mut round = 1usize;
    while round < 10 {
        state = sub_bytes(state);
        state = shift_rows(state);
        state = mix_columns(state);
        state = add_round_key(state, &rk, round);
        round = round + 1;
    }
    state = sub_bytes(state);
    state = shift_rows(state);
    state = add_round_key(state, &rk, 10);
    state
}

// ── decryption ─────────────────────────────────────────────────────────────

/// Inverse affine transform: `L^-1(y) = y <<< 1 ^ y <<< 3 ^ y <<< 6 ^ 0x05`.
pub fn inv_affine(x: u8) -> u8 {
    x.rotate_left(1) ^ x.rotate_left(3) ^ x.rotate_left(6) ^ 0x05
}

/// Inverse S-box: apply the inverse affine, then take the GF(2^8) inverse.
pub fn inv_sbox(x: u8) -> u8 {
    gf_inv(inv_affine(x))
}

/// InvSubBytes: apply the inverse S-box to every byte.
pub fn inv_sub_bytes(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut i = 0usize;
    while i < 16 {
        out[i] = inv_sbox(state[i]);
        i = i + 1;
    }
    out
}

/// InvShiftRows: rotate row `r` right by `r` positions.
pub fn inv_shift_rows(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut r = 0usize;
    while r < 4 {
        let mut c = 0usize;
        while c < 4 {
            out[r + 4 * c] = state[r + 4 * ((c + 4 - r) % 4)];
            c = c + 1;
        }
        r = r + 1;
    }
    out
}

/// InvMixColumns: multiply each column by the inverse MDS matrix
/// (coefficients 0e, 0b, 0d, 09 in GF(2^8)).
pub fn inv_mix_columns(state: Vec<u8>) -> Vec<u8> {
    let mut out = vec![0u8; 16];
    let mut c = 0usize;
    while c < 4 {
        let a0 = state[0 + 4 * c];
        let a1 = state[1 + 4 * c];
        let a2 = state[2 + 4 * c];
        let a3 = state[3 + 4 * c];
        let b0 = gf_mul(a0, 14) ^ gf_mul(a1, 11) ^ gf_mul(a2, 13) ^ gf_mul(a3, 9);
        let b1 = gf_mul(a0, 9) ^ gf_mul(a1, 14) ^ gf_mul(a2, 11) ^ gf_mul(a3, 13);
        let b2 = gf_mul(a0, 13) ^ gf_mul(a1, 9) ^ gf_mul(a2, 14) ^ gf_mul(a3, 11);
        let b3 = gf_mul(a0, 11) ^ gf_mul(a1, 13) ^ gf_mul(a2, 9) ^ gf_mul(a3, 14);
        out[0 + 4 * c] = b0;
        out[1 + 4 * c] = b1;
        out[2 + 4 * c] = b2;
        out[3 + 4 * c] = b3;
        c = c + 1;
    }
    out
}

/// AES-128 decrypt a single 16-byte block (FIPS-197 §5.3, 10 rounds).
pub fn aes_decrypt(block: &[u8], key: &[u8]) -> Vec<u8> {
    let rk = key_expansion(key);
    let mut state = block.to_vec();
    state = add_round_key(state, &rk, 10);
    let mut round = 9usize;
    while round > 0 {
        state = inv_shift_rows(state);
        state = inv_sub_bytes(state);
        state = add_round_key(state, &rk, round);
        state = inv_mix_columns(state);
        round = round - 1;
    }
    state = inv_shift_rows(state);
    state = inv_sub_bytes(state);
    state = add_round_key(state, &rk, 0);
    state
}