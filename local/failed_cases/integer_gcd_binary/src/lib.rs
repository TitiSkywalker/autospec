//! Stein's binary GCD algorithm.
//! Source: num-integer `src/lib.rs` (Integer::gcd).

/// Computes the greatest common divisor of two non-negative integers using
/// Stein's binary GCD algorithm.
pub fn gcd_binary(a: u64, b: u64) -> u64 {
    if a == 0 {
        return b;
    }
    if b == 0 {
        return a;
    }

    // Strip factors of 2 from both operands, counting them for the common shift.
    let mut m = a;
    let mut tz_a = 0u64;
    while m % 2 == 0 {
        m /= 2;
        tz_a += 1;
    }
    let mut n = b;
    let mut tz_b = 0u64;
    while n % 2 == 0 {
        n /= 2;
        tz_b += 1;
    }
    let shift = if tz_a < tz_b { tz_a } else { tz_b };

    // Main Stein loop: both operands are now odd.
    while m != n {
        if m > n {
            m -= n;
            while m % 2 == 0 {
                m /= 2;
            }
        } else {
            n -= m;
            while n % 2 == 0 {
                n /= 2;
            }
        }
    }

    // Restore the common factor of 2 (m << shift, without using shifts).
    let mut result = m;
    let mut i = 0u64;
    while i < shift {
        result *= 2;
        i += 1;
    }
    result
}
