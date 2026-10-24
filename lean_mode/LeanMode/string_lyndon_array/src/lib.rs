//! The Lyndon array.
//!
//! `L[i]` is the length of the *longest* Lyndon word that is a prefix of
//! `s[i..]`.  A word is Lyndon iff it is strictly lexicographically smaller
//! than all of its non-trivial proper suffixes (equivalently, than all its
//! proper suffixes, the empty suffix excluded).  Every single character is
//! Lyndon, so `L[i] >= 1` for every `i < n`, and the output has one entry per
//! start position (`n` entries; the empty string gives an empty vector).
//!
//! This is the direct scan: for each start `i` and each length `l` it tests
//! whether `s[i..i + l]` is Lyndon and keeps the largest such `l`.  The Lyndon
//! test itself compares `w` against each of its proper suffixes, so the whole
//! routine is `O(n^4)` in the worst case — plenty fast for the small test
//! strings, and it needs no suffix-array machinery.
//!
//! Strings are `usize`-encoded characters; every index computation is routed
//! through a `usize`-parameter helper so that no arithmetic on two computed
//! locals reaches the translator inline.

/// `x + y` on `usize`.
fn usize_add(x: usize, y: usize) -> usize {
    x + y
}

/// `x - y` on `usize` (the caller guarantees `y <= x`).
fn usize_sub(x: usize, y: usize) -> usize {
    x - y
}

/// Whether the window `s[a .. a + la]` is strictly lexicographically smaller
/// than the window `s[b .. b + lb]`.  Both windows lie inside `s`.
fn win_lt(s: &[usize], a: usize, la: usize, b: usize, lb: usize) -> bool {
    let m = if la < lb { la } else { lb };
    // Walk while the two windows agree; stop at the first difference.
    let mut k = 0;
    let mut eq = true;
    while eq {
        if k < m {
            let ia = usize_add(a, k);
            let ib = usize_add(b, k);
            let x = s[ia];
            let y = s[ib];
            if x != y {
                eq = false;
            } else {
                k += 1;
            }
        } else {
            eq = false;
        }
    }
    // `k == m` means the windows agree up to the shorter one, so the shorter
    // window is the smaller; otherwise the first difference decides.
    let mut res = la < lb;
    if k < m {
        let ia = usize_add(a, k);
        let ib = usize_add(b, k);
        let x = s[ia];
        let y = s[ib];
        res = x < y;
    }
    res
}

/// Whether the window `s[start .. start + len]` (`len >= 1`) is a Lyndon word,
/// i.e. strictly smaller than each of its non-trivial proper suffixes.
fn win_is_lyndon(s: &[usize], start: usize, len: usize) -> bool {
    let mut ok = true;
    let mut k = 1;
    while k < len {
        let b = usize_add(start, k);
        let rem = usize_sub(len, k);
        let lt = win_lt(s, start, len, b, rem);
        if !lt {
            ok = false;
        }
        k += 1;
    }
    ok
}

/// The largest `l <= maxlen` such that `s[start .. start + l]` is Lyndon.  The
/// maximum is at least `1` because a single character is always Lyndon.  The
/// running maximum is kept in a one-element `Vec` (writing to a Vec element
/// keeps the loop's scalar state to `l` alone, which the translator prefers).
fn longest_lyndon_prefix(s: &[usize], start: usize, maxlen: usize) -> usize {
    let mut best: Vec<usize> = vec![1usize; 1];
    let mut l: usize = 1;
    while l <= maxlen {
        if win_is_lyndon(s, start, l) {
            best[0] = l;
        }
        l += 1;
    }
    best[0]
}

/// `L[i]` = length of the longest Lyndon word that is a prefix of `s[i..]`.
pub fn lyndon_array(s: &[usize]) -> Vec<usize> {
    let n: usize = s.len();
    let mut out: Vec<usize> = vec![0usize; n];
    let mut i: usize = 0;
    while i < n {
        let rem = usize_sub(n, i);
        let b = longest_lyndon_prefix(s, i, rem);
        out[i] = b;
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference Lyndon test, written independently of the implementation using
    /// Rust's slice order (not translated to Lean).
    fn is_lyndon_ref(w: &[usize]) -> bool {
        if w.is_empty() {
            return false;
        }
        (1..w.len()).all(|k| w < &w[k..])
    }

    /// Reference Lyndon array: for each start, the maximum `l` with
    /// `is_lyndon_ref (s[i..i+l])`.
    fn lyndon_array_ref(s: &[usize]) -> Vec<usize> {
        let n = s.len();
        (0..n)
            .map(|i| {
                let mut best = 0;
                for l in 1..=(n - i) {
                    if is_lyndon_ref(&s[i..i + l]) {
                        best = l;
                    }
                }
                best
            })
            .collect()
    }

    /// The greedy segmentation induced by the Lyndon array: repeatedly jump
    /// `i -> i + L[i]`.
    fn greedy_factors(s: &[usize]) -> Vec<Vec<usize>> {
        let l = lyndon_array(s);
        let n = s.len();
        let mut res = Vec::new();
        let mut i = 0;
        while i < n {
            let len = l[i];
            res.push(s[i..i + len].to_vec());
            i += len;
        }
        res
    }

    /// Duval's factorization, an independent reference for the greedy factors.
    fn duval(s: &[usize]) -> Vec<Vec<usize>> {
        let n = s.len();
        let mut res = Vec::new();
        let mut i = 0;
        while i < n {
            let mut j = i + 1;
            let mut k = i;
            while j < n && s[k] <= s[j] {
                if s[k] < s[j] {
                    k = i;
                } else {
                    k += 1;
                }
                j += 1;
            }
            while i <= k {
                res.push(s[i..i + (j - k)].to_vec());
                i += j - k;
            }
        }
        res
    }

    /// `0` iff the greedy segment lengths form a valid Lyndon factorization of
    /// `s`: they cover `s`, every segment is Lyndon, and they are
    /// lexicographically non-increasing.  This is exactly the conclusion of the
    /// spec's `lyndon_array_greedy_factorization`.
    fn factorization_props_ok(s: &[usize]) -> usize {
        let fs = greedy_factors(s);
        let mut bad = 0usize;
        let mut total = 0usize;
        for f in &fs {
            total += f.len();
            if !is_lyndon_ref(f) {
                bad += 1;
            }
        }
        if total != s.len() {
            bad += 1;
        }
        let mut i = 0;
        while i + 1 < fs.len() {
            if fs[i] < fs[i + 1] {
                bad += 1;
            }
            i += 1;
        }
        bad
    }

    fn all_strings(n: usize, alpha: usize) -> Vec<Vec<usize>> {
        let mut out = vec![Vec::new()];
        for _ in 0..n {
            let mut next = Vec::new();
            for w in &out {
                for c in 1..=alpha {
                    let mut v = w.clone();
                    v.push(c);
                    next.push(v);
                }
            }
            out = next;
        }
        out
    }

    #[test]
    fn matches_reference_exhaustive() {
        // Every string of length 0..=7 over a 3-letter alphabet.
        let mut checked = 0usize;
        for n in 0..=7 {
            for s in all_strings(n, 3) {
                let got = lyndon_array(&s);
                let want = lyndon_array_ref(&s);
                assert_eq!(got, want, "lyndon_array mismatch on {:?}", s);
                checked += 1;
            }
        }
        assert!(checked > 3000);
    }

    #[test]
    fn greedy_factors_equal_duval_exhaustive() {
        // The classic consequence used in the spec: the greedy Lyndon-array
        // segmentation reproduces Duval's factorization.
        let mut checked = 0usize;
        for n in 0..=8 {
            for s in all_strings(n, 3) {
                assert_eq!(
                    greedy_factors(&s),
                    duval(&s),
                    "greedy/duval mismatch on {:?}",
                    s
                );
                checked += 1;
            }
        }
        assert!(checked > 9000);
    }

    #[test]
    fn greedy_factors_are_a_lyndon_factorization() {
        // Directly check the *conclusion* of the spec theorem on the greedy
        // factors, exhaustively.
        let mut checked = 0usize;
        for n in 0..=8 {
            for s in all_strings(n, 3) {
                assert_eq!(
                    factorization_props_ok(&s),
                    0,
                    "factorization props failed on {:?}",
                    s
                );
                checked += 1;
            }
        }
        assert!(checked > 9000);
    }

    #[test]
    fn matches_reference_random() {
        // A small deterministic LCG over a 4-letter alphabet, lengths up to 40.
        let mut state: u64 = 0x2545F4914F6CDD1D;
        let mut next = || {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            (state >> 33) as usize
        };
        for _ in 0..400 {
            let n = next() % 41;
            let s: Vec<usize> = (0..n).map(|_| 1 + next() % 4).collect();
            assert_eq!(lyndon_array(&s), lyndon_array_ref(&s), "on {:?}", s);
            assert_eq!(factorization_props_ok(&s), 0, "props on {:?}", s);
        }
    }

    #[test]
    fn every_entry_is_lyndon_and_maximal() {
        // Re-check the definition directly for a few strings.
        for s in [
            vec![],
            vec![1],
            vec![1, 1, 1],
            vec![1, 2, 3],
            vec![3, 2, 1],
            vec![1, 2, 1, 2],
            vec![2, 1, 3, 1, 3, 1],
        ] {
            let l = lyndon_array(&s);
            assert_eq!(l.len(), s.len());
            for i in 0..s.len() {
                let len = l[i];
                assert!(len >= 1);
                assert!(is_lyndon_ref(&s[i..i + len]), "prefix not lyndon");
                for lp in (len + 1)..=(s.len() - i) {
                    assert!(!is_lyndon_ref(&s[i..i + lp]), "longer prefix lyndon");
                }
            }
        }
    }
}
