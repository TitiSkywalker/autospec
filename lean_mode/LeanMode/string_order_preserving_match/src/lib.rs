//! Order-preserving pattern matching (Kubica, Kulczyński, Radoszewski, Rytter,
//! Waleń, "A linear time algorithm for consecutive permutation pattern
//! matching", 2013).
//!
//! A pattern `p` *order-matches* the text window `t[i..i+m]` when the two words
//! are **order-isomorphic**: for all `a, b < m`,
//! `p[a] < p[b]` iff `t[i+a] < t[i+b]`.  Equivalently the two words have the
//! same "relative order with ties" (the strict order determines the equality
//! relations as well, since both are total orders).
//!
//! The characterisation used here is the *rank encoding*
//!
//! ```text
//! rank(s)[k] = #{ j : s[j] < s[k] }
//! ```
//!
//! (ties receive equal ranks).  For two words `w`, `p` of the same length,
//! `rank(w) = rank(p)` holds **iff** `w` and `p` are order-isomorphic — this is
//! exact even in the presence of repeated elements, because within either word
//! the map `k ↦ rank(k)` is strictly order-preserving, so
//! `w[a] < w[b] ↔ rank(w)[a] < rank(w)[b]` and likewise for `p`.  (The static
//! per-window encoding is the definitional route; keeping ties makes the
//! *sliding* variant subtle, which is why `op_match` recomputes each window's
//! ranks from scratch.)  See `Spec.lean` for the precise statement and the
//! brute-force evidence in the unit tests below.
//!
//! Characters are integer-encoded as `usize`; no `as` casts and no closures are
//! used, so the only numeric model involved is `usize` (→ `Std.Usize`).

/// Rank encoding of `s`: `r[k] = #{ j : s[j] < s[k] }` (elements equal to
/// `s[k]` are not counted, so ties get equal ranks).
pub fn rank_encoding(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut r: Vec<usize> = vec![0usize; n];
    let mut k = 0;
    while k < n {
        let mut cnt: usize = 0;
        let mut j = 0;
        while j < n {
            let x = s[j];
            let y = s[k];
            if x < y {
                cnt += 1;
            }
            j += 1;
        }
        r[k] = cnt;
        k += 1;
    }
    r
}

/// All start positions `i` at which `p` order-matches `t`, in increasing order.
///
/// The window ranks are computed locally (`t[i..i+m]`), so the search is exact
/// for every text/pattern, ties included.
///
/// Conventions: the empty pattern matches *nowhere* (the result is empty), and
/// a pattern longer than the text also yields the empty result.
pub fn op_match(t: &[usize], p: &[usize]) -> Vec<usize> {
    let n = t.len();
    let m = p.len();
    let mut occ: Vec<usize> = Vec::new();
    if m == 0 {
        return occ;
    }
    if m > n {
        return occ;
    }

    let rp = rank_encoding(p);

    let mut i = 0;
    while i + m <= n {
        // Compare the rank encoding of `t[i..i+m]` with `rp` position by
        // position.
        let mut ok = true;
        let mut a = 0;
        while a < m {
            let mut cnt: usize = 0;
            let mut b = 0;
            while b < m {
                let ti = i + b;
                let ta = i + a;
                let x = t[ti];
                let y = t[ta];
                if x < y {
                    cnt += 1;
                }
                b += 1;
            }
            let r = rp[a];
            if cnt != r {
                ok = false;
            }
            a += 1;
        }
        if ok {
            occ.push(i);
        }
        i += 1;
    }
    occ
}

// ---------------------------------------------------------------------------
// Randomized differential tests (NOT translated — the crate's own unit-test
// module is not picked up by the test translation; only `tests/test.rs` is).
//
// These brute-force check (i) the `op_match` result against a `∀ a b` order
// isomorphism checker, (ii) the theorem `op_match_correct`'s conclusion for
// every position, and (iii) the rank-encoding characterisation
// `rank(w) = rank(p) ↔ order-isomorphic` on random words *with ties*.
// ---------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    /// Deterministic LCG so the fuzzers have no external dependencies.
    struct Rng {
        state: u64,
    }

    impl Rng {
        fn new(seed: u64) -> Rng {
            Rng { state: seed }
        }
        fn next(&mut self) -> u64 {
            // Numerical Recipes LCG constants.
            self.state = self
                .state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.state >> 33
        }
    }

    /// Brute-force order isomorphism: equal length and `w[a] < w[b] ↔ p[a] < p[b]`.
    fn brute_iso(w: &[usize], p: &[usize]) -> bool {
        if w.len() != p.len() {
            return false;
        }
        let n = w.len();
        let mut a = 0;
        while a < n {
            let mut b = 0;
            while b < n {
                if (w[a] < w[b]) != (p[a] < p[b]) {
                    return false;
                }
                b += 1;
            }
            a += 1;
        }
        true
    }

    /// Brute-force occurrence list from the definition (empty pattern → none).
    fn brute_op(t: &[usize], p: &[usize]) -> Vec<usize> {
        let n = t.len();
        let m = p.len();
        let mut occ: Vec<usize> = Vec::new();
        if m == 0 {
            return occ;
        }
        let mut i = 0;
        while i + m <= n {
            if brute_iso(&t[i..i + m], p) {
                occ.push(i);
            }
            i += 1;
        }
        occ
    }

    fn rnd_vec(rng: &mut Rng, len: usize, alpha: u64) -> Vec<usize> {
        let mut v: Vec<usize> = Vec::new();
        let mut i = 0;
        while i < len {
            v.push((rng.next() % alpha) as usize);
            i += 1;
        }
        v
    }

    #[test]
    fn fuzz_op_match_membership() {
        let mut rng = Rng::new(0x1234_5678);
        let mut trial = 0;
        while trial < 4000 {
            let n = (rng.next() % 9) as usize;
            let m = (rng.next() % 9) as usize;
            let alpha = 1 + (rng.next() % 4); // alphabet size 1..=4
            let t = rnd_vec(&mut rng, n, alpha);
            let p = rnd_vec(&mut rng, m, alpha);

            let got = op_match(&t, &p);
            let want = brute_op(&t, &p);
            assert_eq!(got, want, "op_match mismatch: t={:?} p={:?}", t, p);

            // The conclusion of `op_match_correct`, for every position.
            let mut k = 0;
            while k < n + 1 {
                let in_occ = got.contains(&k);
                let mut want_mem = false;
                if m > 0 && k + m <= n {
                    want_mem = brute_iso(&t[k..k + m], &p);
                }
                assert_eq!(in_occ, want_mem, "membership at {}: t={:?} p={:?}", k, t, p);
                k += 1;
            }

            // Positions are strictly increasing and leave room for the pattern.
            let mut j = 0;
            while j < got.len() {
                assert!(got[j] + m <= n);
                if j > 0 {
                    assert!(got[j - 1] < got[j]);
                }
                j += 1;
            }
            trial += 1;
        }
    }

    #[test]
    fn fuzz_rank_encoding_iff() {
        let mut rng = Rng::new(0xDEAD_BEEF);
        let mut trial = 0;
        while trial < 4000 {
            let m = (rng.next() % 8) as usize;
            let alpha = 1 + (rng.next() % 4);
            let w = rnd_vec(&mut rng, m, alpha);
            let p = rnd_vec(&mut rng, m, alpha);
            let rw = rank_encoding(&w);
            let rp = rank_encoding(&p);
            assert_eq!(
                rw == rp,
                brute_iso(&w, &p),
                "rank iff iso failed: w={:?} p={:?}",
                w,
                p
            );
            trial += 1;
        }
    }

    #[test]
    fn fuzz_rank_encoding_closed_form() {
        let mut rng = Rng::new(0x0BAD_F00D);
        let mut trial = 0;
        while trial < 2000 {
            let n = (rng.next() % 12) as usize;
            let alpha = 1 + (rng.next() % 5);
            let s = rnd_vec(&mut rng, n, alpha);
            let r = rank_encoding(&s);
            let mut k = 0;
            while k < n {
                let mut cnt = 0;
                let mut j = 0;
                while j < n {
                    if s[j] < s[k] {
                        cnt += 1;
                    }
                    j += 1;
                }
                assert_eq!(r[k], cnt);
                k += 1;
            }
            trial += 1;
        }
    }
}
