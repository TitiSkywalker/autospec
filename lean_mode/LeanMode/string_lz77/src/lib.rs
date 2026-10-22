//! Greedy LZ77 factorization (non-overlapping variant).
//!
//! Characters are `usize` byte values (0..=255).

/// One LZ77 factor.  `offset == 0` means a literal: the single character
/// `s[pos]` at the current position.  Otherwise the factor copies `length`
/// characters from `s[pos - offset ..]`.
pub struct LzFactor {
    pub offset: usize,
    pub length: usize,
}

/// Greedy LZ77 decomposition of `s` into non-overlapping factors.
///
/// Walk the string left to right.  At each position `pos` pick the longest
/// match of `s[pos..]` against some earlier occurrence `s[q..]` that does not
/// overlap the current position (`q + l <= pos`) and does not run past the end
/// of the string (`pos + l <= n`).  If the longest match is nonempty, emit a
/// copy factor `(pos - q, l)` and advance by `l`; otherwise emit the literal
/// `(0, 1)` and advance by one.
pub fn lz77(s: &[usize]) -> Vec<LzFactor> {
    let n = s.len();
    let mut factors: Vec<LzFactor> = Vec::new();
    let mut pos = 0;
    while pos < n {
        let mut best_len = 0;
        let mut best_off = 0;
        let mut q = 0;
        while q < pos {
            let mut l = 0;
            let mut more = true;
            while more {
                let pl = pos + l;
                let ql = q + l;
                if pl < n && ql < pos && s[ql] == s[pl] {
                    l += 1;
                } else {
                    more = false;
                }
            }
            if l > best_len {
                let off = pos - q;
                best_len = l;
                best_off = off;
            }
            q += 1;
        }
        if best_len == 0 {
            factors.push(LzFactor { offset: 0, length: 1 });
            pos += 1;
        } else {
            factors.push(LzFactor { offset: best_off, length: best_len });
            pos += best_len;
        }
    }
    factors
}
