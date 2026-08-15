//! Perfect square test via binary search.
//! Source: TheAlgorithms/Rust `src/math/perfect_square.rs`.

/// Checks whether `n` is a perfect square (∃k, k² = n) via binary search.
pub fn is_perfect_square(n: u64) -> bool {
    if n < 2 {
        return true; // 0 and 1 are perfect squares
    }
    let mut left = 1u64;
    let mut right = n;
    while left <= right {
        let mid = left + (right - left) / 2;
        if mid <= n / mid {
            // mid*mid <= n, so the multiplication below cannot overflow.
            let sq = mid * mid;
            if sq == n {
                return true;
            }
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    false
}
