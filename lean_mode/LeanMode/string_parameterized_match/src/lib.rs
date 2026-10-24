//! Parameterized pattern matching (p-matching, Baker) via the prev-occurrence
//! encoding.
//!
//! A pattern `p` *p-matches* the length-`m` text window `t[i .. i+m]`
//! (`m = p.len()`) when some bijection `φ` between the distinct characters of
//! the pattern and those of the window carries `p` onto the window:
//! `φ(p[k]) = t[i+k]` for every `k < m`.  (The occurrence list is the exact
//! set of matching positions — no misses, no extras.)
//!
//! The prev-occurrence encoding makes the check a plain equality:
//!
//! * `prev(s)[k] = 0` when `s[k]` does not occur in the prefix `s[..k]`;
//! * `prev(s)[k] = k - j` where `j` is the **last** index `< k` with
//!   `s[j] = s[k]`.
//!
//! Then `p` p-matches the window iff `prev(p) = prev(window)` (Baker's
//! theorem).  The window's encoding must be computed **window-locally**: a
//! character that occurs only *before* the window is "new" inside the window
//! and must not count as a previous occurrence.
//!
//! Characters are integer-encoded as `usize` (integer `as` casts do not
//! translate under the `nat` model; encoding the characters directly avoids
//! them).

/// `a - b`, routed through a function so both operands become *call
/// arguments*.  A `usize` subtraction of two computed locals is otherwise
/// mistranslated to a pure `let` and then fails to elaborate.
fn usize_sub(a: usize, b: usize) -> usize {
    a - b
}

/// The prev-occurrence encoding of the length-`m` window `s[base .. base+m]`,
/// with positions measured *relative to the window start*: `enc[k] = 0` when
/// `s[base+k]` does not occur in `s[base .. base+k]`, and `enc[k] = k - j`
/// where `j` is the last index in `0..k` with `s[base+j] = s[base+k]`.
fn prev_range(s: &[usize], base: usize, m: usize) -> Vec<usize> {
    let mut enc: Vec<usize> = Vec::new();
    let mut k: usize = 0;
    while k < m {
        let c = s[base + k];
        // `m` is an out-of-range sentinel: a real match index is `< k < m`.
        let mut last: usize = m;
        let mut j: usize = 0;
        while j < k {
            if s[base + j] == c {
                last = j;
            }
            j += 1;
        }
        let mut dist: usize = 0;
        if last != m {
            dist = usize_sub(k, last);
        }
        enc.push(dist);
        k += 1;
    }
    enc
}

/// The prev-occurrence encoding of `s`: `prev[k] = 0` when `s[k]` does not
/// occur in `s[..k]`, else `prev[k] = k - j` for the last index `j < k` with
/// `s[j] = s[k]`.
pub fn prev_encoding(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    prev_range(s, 0usize, n)
}

/// All start positions `i` with `i + p.len() <= t.len()` at which the pattern
/// `p` p-matches the window `t[i .. i+p.len()]`, in increasing order.
///
/// The encoding of `p` is computed once; each window is encoded locally and
/// the two encodings are compared element by element, so the occurrence list
/// is exactly the set of matching positions.
///
/// An empty pattern (`p.len() == 0`) p-matches nowhere under this definition;
/// the result is the empty vector.  (An empty pattern would p-match *every*
/// position vacuously, which is not what a search routine is expected to
/// report, so it is excluded; the specification theorem correspondingly
/// assumes `0 < p.len()`.)
pub fn pmatch(t: &[usize], p: &[usize]) -> Vec<usize> {
    let m = p.len();
    let n = t.len();
    let mut occ: Vec<usize> = Vec::new();
    if m == 0 {
        return occ;
    }
    if m > n {
        return occ;
    }
    let ep = prev_encoding(p);
    let mut i: usize = 0;
    while i + m <= n {
        let ew = prev_range(t, i, m);
        let mut same = true;
        let mut k: usize = 0;
        while k < m {
            let a = ew[k];
            let b = ep[k];
            if a != b {
                same = false;
            }
            k += 1;
        }
        if same {
            occ.push(i);
        }
        i += 1;
    }
    occ
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Reference p-match of equal-length `p` and window `w`, written directly
    /// from the definition: `φ(p[k]) = w[k]` is a well-defined injection iff
    /// equal pattern characters correspond exactly to equal window characters.
    fn pairwise_bijective(w: &[usize], p: &[usize]) -> bool {
        let m = p.len();
        if w.len() != m {
            return false;
        }
        let mut k1 = 0;
        while k1 < m {
            let mut k2 = 0;
            while k2 < m {
                if (p[k1] == p[k2]) != (w[k1] == w[k2]) {
                    return false;
                }
                k2 += 1;
            }
            k1 += 1;
        }
        true
    }

    /// Brute-force occurrence list straight from the definitional predicate.
    fn brute_pmatch(t: &[usize], p: &[usize]) -> Vec<usize> {
        let m = p.len();
        let n = t.len();
        let mut occ = Vec::new();
        if m == 0 || m > n {
            return occ;
        }
        let mut i = 0;
        while i + m <= n {
            if pairwise_bijective(&t[i..i + m], p) {
                occ.push(i);
            }
            i += 1;
        }
        occ
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        *state
    }

    fn random_vec(state: &mut u64, len: usize, alpha: usize) -> Vec<usize> {
        let mut v = Vec::new();
        let mut i = 0;
        while i < len {
            let x = lcg(state);
            let c = (x % (alpha as u64)) as usize;
            v.push(c);
            i += 1;
        }
        v
    }

    /// Exhaustive check over the alphabet `{0,1,2}`, `|t| <= 5`, `|p| <= 4`,
    /// against the definitional brute force.
    #[test]
    fn exhaustive_small_alphabet() {
        let mut n = 0;
        while n <= 5 {
            let mut patn = 0;
            while patn <= 4 {
                let total_t = 3usize.pow(n as u32);
                let mut ti = 0u64;
                while (ti as usize) < total_t {
                    let mut t = Vec::new();
                    let mut x = ti;
                    let mut d = 0;
                    while d < n {
                        t.push((x % 3) as usize);
                        x /= 3;
                        d += 1;
                    }
                    let total_p = 3usize.pow(patn as u32);
                    let mut pi = 0u64;
                    while (pi as usize) < total_p {
                        let mut p = Vec::new();
                        let mut y = pi;
                        let mut e = 0;
                        while e < patn {
                            p.push((y % 3) as usize);
                            y /= 3;
                            e += 1;
                        }
                        let got = pmatch(&t, &p);
                        let want = brute_pmatch(&t, &p);
                        assert_eq!(got, want, "t={:?} p={:?}", t, p);
                        if p.is_empty() {
                            // documented: empty pattern matches nowhere
                            assert_eq!(want.len(), 0);
                        }
                        pi += 1;
                    }
                    ti += 1;
                }
                patn += 1;
            }
            n += 1;
        }
    }

    /// Exhaustive version of Baker's theorem on equal-length lists (the
    /// conclusion of `prev_encoding_iff_pmatch`): alphabet `{0,1,2}`,
    /// length `<= 4`.
    #[test]
    fn exhaustive_prev_encoding_iff_pairwise() {
        let mut m = 0;
        while m <= 4 {
            let total = 3usize.pow(m as u32);
            let mut wi = 0u64;
            while (wi as usize) < total {
                let mut w = Vec::new();
                let mut x = wi;
                let mut d = 0;
                while d < m {
                    w.push((x % 3) as usize);
                    x /= 3;
                    d += 1;
                }
                let mut pi = 0u64;
                while (pi as usize) < total {
                    let mut p = Vec::new();
                    let mut y = pi;
                    let mut e = 0;
                    while e < m {
                        p.push((y % 3) as usize);
                        y /= 3;
                        e += 1;
                    }
                    assert_eq!(
                        prev_encoding(&w) == prev_encoding(&p),
                        pairwise_bijective(&w, &p),
                        "w={:?} p={:?}",
                        w,
                        p
                    );
                    pi += 1;
                }
                wi += 1;
            }
            m += 1;
        }
    }

    /// Random differential test against the definitional brute force.
    #[test]
    fn random_vs_brute() {
        let mut state: u64 = 0x0123_4567_89ab_cdef;
        let mut iter = 0;
        while iter < 200_000 {
            let alpha = ((lcg(&mut state) % 4) as usize) + 1;
            let n = (lcg(&mut state) % 13) as usize;
            let m = (lcg(&mut state) % 8) as usize;
            let t = random_vec(&mut state, n, alpha);
            let p = random_vec(&mut state, m, alpha);
            assert_eq!(pmatch(&t, &p), brute_pmatch(&t, &p), "t={:?} p={:?}", t, p);
            iter += 1;
        }
    }

    /// Baker's theorem, checked in isolation: for equal-length lists,
    /// `prev_encoding p = prev_encoding w` iff `p` p-matches `w`
    /// (this is the conclusion of the `prev_encoding_iff_pmatch` spec theorem).
    #[test]
    fn prev_encoding_iff_pairwise_bijection() {
        let mut state: u64 = 0xdead_beef;
        let mut iter = 0;
        while iter < 100_000 {
            let alpha = ((lcg(&mut state) % 4) as usize) + 1;
            let m = (lcg(&mut state) % 9) as usize;
            let w = random_vec(&mut state, m, alpha);
            let p = random_vec(&mut state, m, alpha);
            let enc_eq = prev_encoding(&w) == prev_encoding(&p);
            let p_match = pairwise_bijective(&w, &p);
            assert_eq!(enc_eq, p_match, "w={:?} p={:?}", w, p);
            iter += 1;
        }
    }

    /// The window encoding is window-local: `prev_range(t, i, m)` equals the
    /// prev-encoding of the window slice — in particular a character that
    /// occurs only before the window is "new" inside it.
    #[test]
    fn window_encoding_is_local() {
        let mut state: u64 = 0x2222_3333;
        let mut iter = 0;
        while iter < 50_000 {
            let alpha = ((lcg(&mut state) % 3) as usize) + 1;
            let n = (lcg(&mut state) % 12) as usize;
            let t = random_vec(&mut state, n, alpha);
            let mut i = 0;
            while i <= n {
                let m = n - i;
                let w: Vec<usize> = t[i..].to_vec();
                assert_eq!(prev_range(&t, i, m), prev_encoding(&w), "t={:?} i={}", t, i);
                i += 1;
            }
            iter += 1;
        }
    }
}
