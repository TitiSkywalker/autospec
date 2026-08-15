//! Euclidean (floored) division and modulo, and ceiling division.
//!
//! These use the Daan Leijen algorithm (Division and Modulus for Computer
//! Scientists, 2001) to derive floored/ceiled results from truncating
//! Rust `%`/`/`.

/// Floored division: `q` such that `a = q * b + r` with `0 <= r < |b|`.
pub fn div_floor(a: i64, b: i64) -> i64 {
    assert!(b != 0);
    let d = a / b;
    let r = a % b;
    if (r > 0 && b < 0) || (r < 0 && b > 0) {
        d - 1
    } else {
        d
    }
}

/// Floored modulo: `r` such that `a = q * b + r` with `0 <= r < |b|`.
pub fn mod_floor(a: i64, b: i64) -> i64 {
    assert!(b != 0);
    let r = a % b;
    if (r > 0 && b < 0) || (r < 0 && b > 0) {
        r + b
    } else {
        r
    }
}

/// Ceiling division: the smallest integer `q` with `q * b >= a` (for `b > 0`).
pub fn div_ceil(a: i64, b: i64) -> i64 {
    assert!(b != 0);
    let d = a / b;
    let r = a % b;
    if (r > 0 && b > 0) || (r < 0 && b < 0) {
        d + 1
    } else {
        d
    }
}
