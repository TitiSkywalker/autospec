// ── Independent brute force: test every number in `[0, N]` directly ─────────

/// `x + y`.
fn t_add(x: usize, y: usize) -> usize {
    x + y
}

/// `x * y`.
fn t_mul(x: usize, y: usize) -> usize {
    x * y
}

/// `x mod m`, written without `%` (`UScalar.rem` is not always available in
/// translated test files).
fn t_rem(x: usize, m: usize) -> usize {
    let q = x / m;
    x - q * m
}

/// Sum of the decimal digits of `x`.
fn t_digsum(x: usize) -> usize {
    let mut y = x;
    let mut s = 0usize;
    while y > 0 {
        let d = y - (y / 10) * 10;
        s += d;
        y = y / 10;
    }
    s
}

/// `1` when some decimal digit of `x` is `4`, else `0`.
fn t_has4(x: usize) -> usize {
    let mut y = x;
    let mut bad = 0usize;
    while y > 0 {
        let d = y - (y / 10) * 10;
        if d == 4 {
            bad = 1;
        }
        y = y / 10;
    }
    bad
}

/// Value of the digit list `n` (most significant first).
fn t_decode(n: &[usize]) -> usize {
    let mut v = 0usize;
    let mut i = 0;
    while i < n.len() {
        let t = t_mul(v, 10);
        let d = n[i];
        v = t_add(t, d);
        i += 1;
    }
    v
}

/// Number of `x ∈ [0, N]` with no digit `4` and `k | digitsum(x)`, by testing
/// every `x` in the range.
fn t_count(n: &[usize], k: usize) -> usize {
    let limit = t_decode(n);
    let mut count = 0usize;
    let mut x = 0usize;
    while x <= limit {
        let s = t_digsum(x);
        let r = t_rem(s, k);
        let f = t_has4(x);
        let both = t_add(r, f);
        if both == 0 {
            count += 1;
        }
        x += 1;
    }
    count
}

// ── Known-answer tests ─────────────────────────────────────────────────────

#[test]
fn test_zero() {
    // Only x = 0 is ≤ 0; its digit sum is 0, which every k ≥ 1 divides.
    let n = vec![0];
    assert_eq!(dp_digit_dp::digit_count(&n, 1), 1);
    assert_eq!(dp_digit_dp::digit_count(&n, 7), 1);
    assert_eq!(t_count(&n, 7), 1);
}

#[test]
fn test_zero_k_is_empty() {
    // Documented convention: nothing is divisible by zero.
    let n = vec![1, 0];
    assert_eq!(dp_digit_dp::digit_count(&n, 0), 0);
}

#[test]
fn test_single_digit_no_four() {
    // Digits 0..4 with k = 1: the numbers 0,1,2,3 survive, 4 is excluded.
    let n = vec![4];
    assert_eq!(dp_digit_dp::digit_count(&n, 1), 4);
    assert_eq!(t_count(&n, 1), 4);
}

#[test]
fn test_single_digit_divisible() {
    // Digits 0..9, digit sum divisible by 3: 0, 3, 6, 9 (none contains a 4).
    let n = vec![9];
    assert_eq!(dp_digit_dp::digit_count(&n, 3), 4);
    assert_eq!(t_count(&n, 3), 4);
}

#[test]
fn test_excludes_four_within_range() {
    // Digits 0..9, k = 1: everything except 4.
    let n = vec![9];
    assert_eq!(dp_digit_dp::digit_count(&n, 1), 9);
    assert_eq!(t_count(&n, 1), 9);
}

#[test]
fn test_leading_zero_digits() {
    // n = [0, 5] is the number 5: the leading zero must not change anything.
    let n = vec![0, 5];
    assert_eq!(dp_digit_dp::digit_count(&n, 1), 5);
    assert_eq!(t_count(&n, 1), 5);
}

#[test]
fn test_two_digits() {
    let n = vec![1, 9];
    let mut k: usize = 1;
    while k <= 9 {
        let got = dp_digit_dp::digit_count(&n, k);
        let want = t_count(&n, k);
        assert_eq!(got, want);
        k += 1;
    }
}

#[test]
fn test_three_digits() {
    // 100: the first multi-digit bound that exercises a tight digit 0.
    let n = vec![1, 0, 0];
    let mut k: usize = 1;
    while k <= 9 {
        let got = dp_digit_dp::digit_count(&n, k);
        let want = t_count(&n, k);
        assert_eq!(got, want);
        k += 1;
    }
}

#[test]
fn test_four_digits() {
    let n = vec![2, 4, 6, 8];
    let mut k: usize = 1;
    while k <= 12 {
        let got = dp_digit_dp::digit_count(&n, k);
        let want = t_count(&n, k);
        assert_eq!(got, want);
        k += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_many() {
    // A spread of bounds and moduli.
    let bounds: Vec<Vec<usize>> = vec![
        vec![1],
        vec![9],
        vec![1, 0],
        vec![4, 2],
        vec![9, 9],
        vec![1, 0, 0],
        vec![5, 3, 1],
        vec![9, 9, 9],
        vec![1, 0, 0, 0],
    ];
    let mut b: usize = 0;
    while b < bounds.len() {
        let n = &bounds[b];
        let mut k: usize = 1;
        while k <= 11 {
            let got = dp_digit_dp::digit_count(n, k);
            let want = t_count(n, k);
            assert_eq!(got, want);
            k += 1;
        }
        b += 1;
    }
}
