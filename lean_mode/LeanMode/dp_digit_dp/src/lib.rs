//! Digit DP ("数位 DP"): count the numbers `x ∈ [0, N]` whose decimal digits
//! contain no `4` and whose digit sum is divisible by `k`.
//!
//! `n` holds the decimal digits of `N`, most significant first (so `N` is
//! `decode(n)`), and every digit of `n` is `< 10`.
//!
//! The state is `(tight, rem)`:
//!
//! * `tight` is `1` while the prefix built so far equals the corresponding
//!   prefix of `N` — then the next digit may not exceed `n[pos]`; once the
//!   prefix drops below `N`'s prefix the remaining digits are free (`0 ..= 9`);
//! * `rem` is the digit sum of the prefix, modulo `k`.
//!
//! `dp[tight * k + rem]` counts the prefixes with that state, so one sweep over
//! the positions (each step multiplying by the `≤ 10` digit choices) counts all
//! `len(n)`-digit strings `≤ n` — i.e. all numbers `0 ..= N` written with
//! leading zeros.  Leading zeros are harmless: the digit `0` is allowed and
//! contributes nothing to the digit sum.

/// `x + y`, routed through a helper so that computed operands are always
/// function arguments at the call site.
fn add_usize(x: usize, y: usize) -> usize {
    x + y
}

/// `x * y`.
fn mul_usize(x: usize, y: usize) -> usize {
    x * y
}

/// `x / y`.
fn div_usize(x: usize, y: usize) -> usize {
    x / y
}

/// `x - y`.
fn sub_usize(x: usize, y: usize) -> usize {
    x - y
}

/// Number of `x ∈ [0, N]` with no decimal digit `4` and `k | digitsum(x)`.
///
/// Returns `0` when `k == 0` (nothing is "divisible by zero"); callers must pass
/// `k >= 1`.
pub fn digit_count(n: &[usize], k: usize) -> usize {
    let len = n.len();
    if k == 0 {
        return 0;
    }
    let width = mul_usize(2, k);
    // State (tight, rem): `cur[tight * k + rem]` is the number of prefixes.
    let mut cur = vec![0usize; width];
    // The empty prefix is tight and has digit sum 0.
    cur[k] = 1;
    let mut pos = 0;
    while pos < len {
        let last = n[pos];
        let mut next = vec![0usize; width];
        let mut tight = 0;
        while tight < 2 {
            let mut rem = 0;
            while rem < k {
                let idx = add_usize(mul_usize(tight, k), rem);
                let c = cur[idx];
                if c != 0 {
                    // While tight, the digit is bounded by the digit of N.
                    let limit = if tight == 1 { last } else { 9usize };
                    let mut d = 0;
                    while d <= limit {
                        if d != 4 {
                            let s = add_usize(rem, d);
                            let q = div_usize(s, k);
                            let nr = sub_usize(s, mul_usize(q, k));
                            let same = if d == last { 1usize } else { 0usize };
                            let nt = mul_usize(tight, same);
                            let j = add_usize(mul_usize(nt, k), nr);
                            let old = next[j];
                            let t = add_usize(old, c);
                            next[j] = t;
                        }
                        d += 1;
                    }
                }
                rem += 1;
            }
            tight += 1;
        }
        cur = next;
        pos += 1;
    }
    // Both tight states are admissible: every counted number is ≤ N.
    let total = add_usize(cur[0], cur[k]);
    total
}
