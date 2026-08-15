//! Perfect cube test via binary search.
//! Source: TheAlgorithms/Rust `src/math/perfect_cube.rs`.

/// Checks whether `n` is a perfect cube (∃k, k³ = n) via binary search.
pub fn is_perfect_cube(n: u64) -> bool {
    if n < 2 {
        return true; // 0 and 1 are perfect cubes
    }
    let mut left = 1u64;
    let mut right = n;
    while left <= right {
        let mid = left + (right - left) / 2;
        // mid³ ≤ n  ⇔  mid ≤ n/mid  ∧  mid² ≤ n/mid (each avoids overflow).
        if mid <= n / mid && mid * mid <= n / mid {
            let cube = mid * mid * mid;
            if cube == n {
                return true;
            }
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    false
}
