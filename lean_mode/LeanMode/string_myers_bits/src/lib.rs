//! Bit-parallel Shift-And (Myers) string matching over integer-encoded
//! characters (`usize`), all occurrences.
//!
//! The automaton keeps a 64-bit state word: after reading the text prefix
//! `t[..=i]`, bit `j` of `state` is set iff that prefix ends with the length
//! `j+1` prefix of the pattern.  A match of the whole pattern ends at `i` iff
//! bit `m-1` is set, where `m = p.len()`.
//!
//! Under the `bitvec` int-model the active state word is a `BitVec 64`, so the
//! `<<` / `|` / `&` / `!=` on it lower through Aeneas's symbolic bit-vector
//! path.  `usize` stays `Std.Usize` and is used only for indexing and the
//! occurrence positions.

/// All start indices at which `p` occurs in `t`, in increasing order, via the
/// bit-parallel Shift-And algorithm:
/// `state = ((state << 1) | 1) & mask[c]`, where `mask[c]` has bit `j` set iff
/// `p[j] = c`; a match ends at position `i` iff bit `m-1` of `state` is set
/// after reading `t[i]`.
///
/// Preconditions (the algorithm relies on them; violating any of the first
/// three makes the function return an empty vector rather than misbehave):
/// * `m = p.len()` with `1 <= m <= 64` — the whole pattern must fit in one
///   64-bit state word;
/// * `m <= t.len()`;
/// * every character of `t` and `p` is `< 256` — the mask table has `256`
///   entries (a byte alphabet).
///
/// An empty pattern (`m = 0`) returns the empty vector: this interface reports
/// only the genuine matches of a non-empty pattern.  A pattern longer than the
/// text, or longer than 64, likewise returns the empty vector.
pub fn shift_and(t: &[usize], p: &[usize]) -> Vec<usize> {
    let m = p.len();
    let mut occ: Vec<usize> = Vec::new();
    if m == 0 || m > 64 || m > t.len() {
        return occ;
    }

    // mask[c] has bit j set iff p[j] == c (256-entry byte alphabet).
    let mut mask: Vec<u64> = vec![0u64; 256];
    let mut bit: u64 = 1u64;
    let mut j: usize = 0;
    while j < m {
        let c = p[j];
        let old = mask[c];
        let newv = old | bit;
        mask[c] = newv;
        let nb = bit << 1u32;
        bit = nb;
        j += 1;
    }

    // accept = bit m-1 = 2^(m-1).
    let mut accept: u64 = 1u64;
    let mut k: usize = 1;
    while k < m {
        let na = accept << 1u32;
        accept = na;
        k += 1;
    }

    // Scan the text, maintaining the automaton state.
    let mut state: u64 = 0u64;
    let mut i: usize = 0;
    while i < t.len() {
        let c = t[i];
        let sh = state << 1u32;
        let orv = sh | 1u64;
        let mc = mask[c];
        let st = orv & mc;
        state = st;
        let hit = st & accept;
        if hit != 0u64 {
            let pos = i + 1usize - m;
            occ.push(pos);
        }
        i += 1;
    }
    occ
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Brute-force O(n·m) reference: all start indices where `p` occurs in `t`.
    fn brute(t: &[usize], p: &[usize]) -> Vec<usize> {
        let n = t.len();
        let m = p.len();
        let mut out: Vec<usize> = Vec::new();
        if m == 0 || m > n {
            return out;
        }
        let mut i = 0;
        while i + m <= n {
            let mut ok = true;
            let mut j = 0;
            while j < m {
                if t[i + j] != p[j] {
                    ok = false;
                }
                j += 1;
            }
            if ok {
                out.push(i);
            }
            i += 1;
        }
        out
    }

    /// Deterministic pseudo-random generator (xorshift) so tests need no crates.
    fn xs(state: &mut u64) -> u64 {
        let mut x = *state;
        x ^= x << 13u32;
        x ^= x >> 7u32;
        x ^= x << 17u32;
        *state = x;
        x
    }

    #[test]
    fn test_matches_brute_force_small() {
        // Exhaustive over a tiny alphabet and short texts/patterns.
        let alpha = 3usize;
        let mut s: u64 = 0x1234_5678_9abc_def1u64;
        let mut n = 0usize;
        while n <= 8 {
            let mut t: Vec<usize> = Vec::new();
            let mut i = 0;
            while i < n {
                let r = xs(&mut s) % alpha as u64;
                t.push(r as usize);
                i += 1;
            }
            let mut m = 1usize;
            while m <= 6 {
                let mut p: Vec<usize> = Vec::new();
                let mut k = 0;
                while k < m {
                    let r = xs(&mut s) % alpha as u64;
                    p.push(r as usize);
                    k += 1;
                }
                assert_eq!(shift_and(&t, &p), brute(&t, &p));
                m += 1;
            }
            n += 1;
        }
    }

    #[test]
    fn test_matches_brute_force_random() {
        let mut s: u64 = 0xdead_beef_cafe_babeu64;
        let mut iter = 0usize;
        while iter < 3000 {
            let n = (xs(&mut s) % 40) as usize;
            let m = 1 + (xs(&mut s) % 12) as usize;
            let mut t: Vec<usize> = Vec::new();
            let mut i = 0;
            while i < n {
                let r = xs(&mut s) % 4;
                t.push(r as usize);
                i += 1;
            }
            let mut p: Vec<usize> = Vec::new();
            let mut k = 0;
            while k < m {
                let r = xs(&mut s) % 4;
                p.push(r as usize);
                k += 1;
            }
            assert_eq!(shift_and(&t, &p), brute(&t, &p));
            iter += 1;
        }
    }

    #[test]
    fn test_max_pattern_len_64() {
        // m = 64 fits; the accept bit 2^63 is exercised.
        let t: Vec<usize> = (0..200usize).map(|x| x % 5).collect();
        let mut p: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < 64 {
            p.push(k % 5);
            k += 1;
        }
        assert_eq!(shift_and(&t, &p), brute(&t, &p));
    }

    #[test]
    fn test_preconditions() {
        let t: Vec<usize> = vec![1, 2, 3];
        let empty: Vec<usize> = Vec::new();
        // empty pattern -> empty
        assert_eq!(shift_and(&t, &empty), empty);
        // pattern longer than text -> empty
        let long: Vec<usize> = vec![1, 2, 3, 4];
        assert_eq!(shift_and(&t, &long), empty);
        // pattern longer than 64 -> empty
        let huge: Vec<usize> = (0..65usize).map(|x| x % 3).collect();
        assert_eq!(shift_and(&t, &huge), empty);
    }
}
