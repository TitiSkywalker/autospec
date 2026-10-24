//! de Bruijn sequences via the FKM (Fredricksen–Kessler–Maiorana) construction.
//!
//! A de Bruijn sequence of order `n` on the alphabet `{0, …, k-1}` is a cyclic
//! word in which every length-`n` word occurs exactly once.  The FKM
//! construction lists the Lyndon words whose length divides `n` in
//! lexicographic order and concatenates them; the result has length `k^n` and
//! is a de Bruijn sequence.
//!
//! The enumeration is the classic recursive "prefer-zero" backtracking over
//! pre-necklaces: `fkm(t, p)` fixes the character at position `t` to the one at
//! position `t - p`, recurses, and then tries every larger character (each
//! restarting the period at `t`).  A pre-necklace of length `> n` and period
//! `p` is emitted iff `p` divides `n`.
//!
//! All buffers are flat `Vec`s, characters are `usize` (→ `Nat`) and the only
//! loop is a counting `while`; the recursion threads its whole mutable state
//! (the necklace buffer `a` and the accumulator `seq`) by value, which matches
//! AENEAS's consume-and-return model of `&mut`.

/// Mutable state threaded through the recursive enumeration.
///
/// `a` is the working necklace buffer: only `a[1 ..= n]` is meaningful and
/// `a[0]` stays `0`.  `seq` accumulates the concatenation of the emitted Lyndon
/// words.
struct DbState {
    a: Vec<usize>,
    seq: Vec<usize>,
}

/// Recursive FKM step on the pre-necklace prefix of length `t - 1` with period
/// `p`, returning the updated state.
///
/// Callers guarantee `1 ≤ p`, `1 ≤ t` and `1 ≤ k`, so `n % p` never divides by
/// zero and `a[t - p]` is in bounds.
fn fkm(t: usize, p: usize, k: usize, n: usize, mut st: DbState) -> DbState {
    if t > n {
        // A complete pre-necklace of period `p`: emit its prefix of length `p`
        // iff `p` divides `n` (so it is a Lyndon word of length `p`).
        if n % p == 0 {
            let mut j = 1;
            while j <= p {
                let x = st.a[j];
                st.seq.push(x);
                j += 1;
            }
        }
    } else {
        // First branch: repeat the character `a[t - p]`.
        let tp = t - p;
        let atp = st.a[tp];
        st.a[t] = atp;
        st = fkm(t + 1, p, k, n, st);
        // Second branch: every larger character restarts the period at `t`.
        let mut j = atp + 1;
        while j < k {
            st.a[t] = j;
            st = fkm(t + 1, t, k, n, st);
            j += 1;
        }
    }
    st
}

/// A de Bruijn sequence of order `n` on the alphabet `{0, …, k-1}`: a cyclic
/// word in which every length-`n` word occurs exactly once.
///
/// The result has `k^n` characters.  Degenerate cases: `k = 0` yields the empty
/// word (an empty alphabet has no length-`n` word for `n ≥ 1`); `n = 0` yields
/// the single-word sequence `[0]` (length `k^0 = 1`).  The documented
/// correctness statement assumes `1 ≤ k` and `1 ≤ n`.
pub fn de_bruijn(k: usize, n: usize) -> Vec<usize> {
    let mut result: Vec<usize> = Vec::new();
    if k == 0 {
        // Empty alphabet: nothing to enumerate.
    } else if n == 0 {
        result.push(0usize);
    } else {
        // `a[0]` is read by `fkm(1, 1, …)` (as `a[1 - 1]`) and `a[1 ..= n]` is
        // written, so the buffer needs `n + 1` slots; `k * n + 1 ≥ n + 1`.
        let a: Vec<usize> = vec![0usize; k * n + 1];
        let seq: Vec<usize> = Vec::new();
        let st = DbState { a: a, seq: seq };
        let out = fkm(1, 1, k, n, st);
        result = out.seq;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The linearised cyclic word of `b`: `b` repeated, truncated to
    /// `b.len() + n - 1` characters.
    fn bf_cyclic(b: &[usize], n: usize) -> Vec<usize> {
        let mut out: Vec<usize> = Vec::new();
        while out.len() < b.len() + n - 1 {
            out.extend_from_slice(b);
        }
        out.truncate(b.len() + n - 1);
        out
    }

    /// Whether `b` is a de Bruijn sequence of order `n` on `{0, …, k-1}`: it has
    /// `k^n` characters and every length-`n` word occurs exactly once in its
    /// cyclic word.  Words are compared by explicit enumeration, not hashing.
    fn bf_is_de_bruijn(k: usize, n: usize, b: &[usize]) -> bool {
        let mut pw = 1usize;
        let mut e = 0;
        while e < n {
            pw *= k;
            e += 1;
        }
        if b.len() != pw {
            return false;
        }
        let cw = bf_cyclic(b, n);
        let positions = cw.len() - n + 1;
        if positions != pw {
            return false;
        }
        // Every window must be distinct (positions = pw = number of words).
        let mut i = 0;
        while i < positions {
            let mut j = i + 1;
            while j < positions {
                let w1 = &cw[i..i + n];
                let w2 = &cw[j..j + n];
                if w1 == w2 {
                    return false;
                }
                j += 1;
            }
            i += 1;
        }
        // Every character is a letter.
        for &x in b.iter() {
            if x >= k {
                return false;
            }
        }
        true
    }

    #[test]
    fn brute_small() {
        // Exhaustive over all small (k, n) with k^n bounded (Lean's Vec is
        // list-backed, but this module is not translated so speed is free).
        let mut k = 1usize;
        while k <= 5 {
            let mut n = 1usize;
            while n <= 6 {
                let mut pw = 1usize;
                let mut e = 0;
                while e < n {
                    pw *= k;
                    e += 1;
                }
                if pw <= 2000 {
                    let b = de_bruijn(k, n);
                    assert!(
                        bf_is_de_bruijn(k, n, &b),
                        "de_bruijn({k}, {n}) is not a de Bruijn sequence: {b:?}"
                    );
                }
                n += 1;
            }
            k += 1;
        }
    }

    #[test]
    fn known_small_values() {
        assert_eq!(de_bruijn(2, 1), vec![0, 1]);
        assert_eq!(de_bruijn(2, 2), vec![0, 0, 1, 1]);
        assert_eq!(de_bruijn(2, 3), vec![0, 0, 0, 1, 0, 1, 1, 1]);
        assert_eq!(de_bruijn(3, 1), vec![0, 1, 2]);
        assert_eq!(de_bruijn(1, 4), vec![0]);
    }

    #[test]
    fn degenerate() {
        assert_eq!(de_bruijn(0, 3), Vec::<usize>::new());
        assert_eq!(de_bruijn(2, 0), vec![0]);
    }
}
