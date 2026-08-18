//! Fixed-width bitvector operations on `u32`.
//!
//! Exercises the `bitvec` int-model: unsigned integers map to Lean `BitVec 32`
//! with fixed-width wrapping/wrapping-rotation semantics (the model crypto
//! hash/AES kernels need).  Covers wrapping arithmetic, rotation, bitwise
//! logic, shifts, and popcount.

// ── rotation ────────────────────────────────────────────────────────────────

/// Rotate the bits of `x` left by `n` positions (cyclic shift).
pub fn rotate_left(x: u32, n: u32) -> u32 {
    x.rotate_left(n)
}

/// Rotate the bits of `x` right by `n` positions (cyclic shift).
pub fn rotate_right(x: u32, n: u32) -> u32 {
    x.rotate_right(n)
}

// ── wrapping arithmetic ─────────────────────────────────────────────────────

/// Wrapping addition — overflow wraps modulo 2^32 without panicking.
pub fn wrapping_add(a: u32, b: u32) -> u32 {
    a.wrapping_add(b)
}

/// Wrapping subtraction — underflow wraps modulo 2^32.
pub fn wrapping_sub(a: u32, b: u32) -> u32 {
    a.wrapping_sub(b)
}

/// Wrapping multiplication — overflow wraps modulo 2^32.
pub fn wrapping_mul(a: u32, b: u32) -> u32 {
    a.wrapping_mul(b)
}

/// Wrapping negation — two's-complement modulo 2^32.
///
/// Implemented as `0 - x` (mod 2^32) rather than the `wrapping_neg` intrinsic:
/// Aeneas does not scalar-lower `u32::wrapping_neg` (it would become an opaque
/// `rust_fun` axiom, defeating `#eval`). `0.wrapping_sub(x)` is the same value.
pub fn wrapping_neg(x: u32) -> u32 {
    0u32.wrapping_sub(x)
}

// ── bitwise logic ───────────────────────────────────────────────────────────

/// Bitwise AND of `a` and `b`.
pub fn bit_and(a: u32, b: u32) -> u32 {
    a & b
}

/// Bitwise OR of `a` and `b`.
pub fn bit_or(a: u32, b: u32) -> u32 {
    a | b
}

/// Bitwise XOR of `a` and `b`.
pub fn bit_xor(a: u32, b: u32) -> u32 {
    a ^ b
}

/// Bitwise NOT of `x` (one's complement).
pub fn bit_not(x: u32) -> u32 {
    !x
}

// ── shifts ──────────────────────────────────────────────────────────────────

/// Logical shift left by `n` positions (fills with zeroes, wraps modulo 2^32).
pub fn shift_left(x: u32, n: u32) -> u32 {
    x << n
}

/// Logical shift right by `n` positions (fills with zeroes).
pub fn shift_right(x: u32, n: u32) -> u32 {
    x >> n
}

// ── population count ────────────────────────────────────────────────────────

/// Number of set bits in `x` (Kernighan's method).
pub fn popcount(mut x: u32) -> u32 {
    let mut c = 0;
    while x != 0 {
        x = x & (x - 1);
        c = c + 1;
    }
    c
}