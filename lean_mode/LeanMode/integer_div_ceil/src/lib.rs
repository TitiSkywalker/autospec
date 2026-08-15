//! Ceiling division for unsigned integers: `ceil(lhs / rhs)`.

/// Computes `ceil(lhs / rhs)` for `rhs > 0`.
pub fn div_ceil_u64(lhs: u64, rhs: u64) -> u64 {
    assert!(rhs > 0);
    let d = lhs / rhs;
    let r = lhs % rhs;
    if r > 0 {
        d + 1
    } else {
        d
    }
}
