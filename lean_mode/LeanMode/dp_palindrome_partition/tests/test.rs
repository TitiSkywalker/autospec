// ── Independent brute force ─────────────────────────────────────────────────
//
// A completely different formulation from the DP: recursively pick the last
// palindromic piece of the prefix `s[0..m]` and recurse on the remaining
// prefix, taking the minimum over all choices.  Exponential, but only used on
// tiny inputs.

/// Minimum number of palindromic pieces covering the prefix `s[0..m]`.
fn brute_from(s: &[usize], m: usize) -> usize {
    if m == 0 {
        return 0;
    }
    // `m` singleton pieces always work, so `m + 1` is a safe sentinel.
    let mut best: usize = m + 1;
    let mut j = 0;
    while j < m {
        let mut a = j;
        let mut b = m - 1;
        let mut mismatches: usize = 0;
        while a < b {
            if s[a] != s[b] {
                mismatches += 1;
            }
            a += 1;
            b -= 1;
        }
        if mismatches == 0 {
            let sub = brute_from(s, j);
            let cand: usize = sub + 1;
            if cand < best {
                best = cand;
            }
        }
        j += 1;
    }
    best
}

/// Brute-force minimum palindromic partition of `s`.
fn brute_min_partition(s: &[usize]) -> usize {
    brute_from(s, s.len())
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_single_char() {
    let s = [7usize];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

#[test]
fn test_two_equal() {
    let s = [3usize, 3];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

#[test]
fn test_two_distinct() {
    let s = [3usize, 4];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 2);
}

#[test]
fn test_aab() {
    // "aab" = "aa" | "b"
    let s = [1usize, 1, 2];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 2);
}

#[test]
fn test_abba() {
    let s = [1usize, 2, 2, 1];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

#[test]
fn test_abcba() {
    let s = [1usize, 2, 3, 2, 1];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

#[test]
fn test_abac() {
    // "abac" = "aba" | "c"
    let s = [1usize, 2, 1, 3];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 2);
}

#[test]
fn test_abacaba() {
    // "abacaba" is itself a palindrome
    let s = [1usize, 2, 1, 3, 1, 2, 1];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

#[test]
fn test_all_distinct() {
    let s = [1usize, 2, 3, 4, 5, 6];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 6);
}

#[test]
fn test_all_equal() {
    let s = [9usize, 9, 9, 9, 9];
    assert_eq!(dp_palindrome_partition::min_palindrome_partition(&s), 1);
}

// ── Exhaustive cross-check against the brute force ──────────────────────────

/// Compare the DP against the brute force on every binary string of length
/// exactly `len` (`len = 0` covers the empty string).
fn check_binary(len: usize) {
    let total = 1usize << len;
    let mut code = 0;
    while code < total {
        let mut s = vec![0usize; len];
        let mut t = code;
        let mut k = 0;
        while k < len {
            let half = t / 2;
            let low = t - half * 2;
            s[k] = low;
            t = half;
            k += 1;
        }
        let got = dp_palindrome_partition::min_palindrome_partition(&s);
        let want = brute_min_partition(&s);
        assert_eq!(got, want);
        code += 1;
    }
}

/// Compare the DP against the brute force on every ternary string of length
/// exactly `len`.
fn check_ternary(len: usize) {
    let mut total = 1usize;
    let mut p = 0;
    while p < len {
        total = total * 3;
        p += 1;
    }
    let mut code = 0;
    while code < total {
        let mut s = vec![0usize; len];
        let mut t = code;
        let mut k = 0;
        while k < len {
            let q = t / 3;
            let digit = t - q * 3;
            s[k] = digit;
            t = q;
            k += 1;
        }
        let got = dp_palindrome_partition::min_palindrome_partition(&s);
        let want = brute_min_partition(&s);
        assert_eq!(got, want);
        code += 1;
    }
}

#[test]
fn test_exhaustive_binary() {
    let mut len = 0;
    while len <= 8 {
        check_binary(len);
        len += 1;
    }
}

#[test]
fn test_exhaustive_ternary() {
    let mut len = 0;
    while len <= 5 {
        check_ternary(len);
        len += 1;
    }
}
