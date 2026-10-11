//! Rabin–Karp string search with a rolling polynomial hash.
//!
//! Characters are integer-encoded as `u64` (byte values widened to the hash
//! type) so the rolling hash needs no widening casts — integer `as` casts only
//! translate under the `bitvec` model, not under `nat`.

const BASE: u64 = 256;
const MOD: u64 = 1000000007;

/// Return the index of the first occurrence of `pattern` in `text`, or `None`
/// if `pattern` does not occur. An empty `pattern` matches at index `0`.
///
/// The rolling hash is only a filter: on a hash hit the window is compared
/// byte-by-byte, so the result is exact (no false positives).
pub fn rabin_karp(text: &[u64], pattern: &[u64]) -> Option<usize> {
    let n = text.len();
    let m = pattern.len();
    if m == 0 {
        return Some(0);
    }
    if m > n {
        return None;
    }

    // Hash of the pattern.
    let mut pat_hash = 0u64;
    let mut i = 0;
    while i < m {
        pat_hash = (pat_hash * BASE + pattern[i]) % MOD;
        i += 1;
    }

    // Hash of the first window.
    let mut win_hash = 0u64;
    let mut i = 0;
    while i < m {
        win_hash = (win_hash * BASE + text[i]) % MOD;
        i += 1;
    }

    // BASE^(m-1) mod MOD, for the sliding step.
    let mut pow = 1u64;
    let mut k = 0;
    while k < m - 1 {
        pow = (pow * BASE) % MOD;
        k += 1;
    }

    // `found_idx == n` means "not found yet" (a match index is always < n).
    let mut found_idx = n;
    let mut i = 0;
    while i + m <= n {
        // Compute the next window's hash up front, so each `% MOD` is bound to a
        // variable rather than left as a bare tail (bare `%` tails don't get
        // `ok`-wrapped by AENEAS).
        let mut next_hash = win_hash;
        if i + m < n {
            let out = (text[i] * pow) % MOD;
            let sub = win_hash + MOD - out;
            next_hash = (sub * BASE + text[i + m]) % MOD;
        }

        if win_hash == pat_hash {
            let mut j = 0;
            let mut ok = true;
            while j < m {
                if text[i + j] != pattern[j] {
                    ok = false;
                }
                j += 1;
            }
            if ok {
                if found_idx == n {
                    found_idx = i;
                }
            }
        }

        win_hash = next_hash;
        i += 1;
    }

    if found_idx == n {
        None
    } else {
        Some(found_idx)
    }
}
