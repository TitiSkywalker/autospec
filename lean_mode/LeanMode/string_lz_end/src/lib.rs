//! Greedy LZ-End factorization (Kreft & Navarro, 2010).
//!
//! Characters are `usize` byte values (0..=255).
//!
//! # Definition
//!
//! `lz_end` walks the string left to right and emits factors that *tile* `s`
//! (their lengths sum to `s.len()`).  At each step the current position `pos`
//! is the total length emitted so far, and the **factor boundaries** are the
//! end positions of the factors produced so far: `0` and every cumulative sum
//! of the factor lengths emitted so far.  In particular the current position
//! `pos` (the end of the last emitted factor) **is itself a boundary**.
//!
//! At position `pos` we look for the longest prefix `lz_end[pos .. pos + l]`
//! of the remaining suffix that occurs as a contiguous substring
//! `s[B - l .. B]` of the already-parsed prefix `s[0 .. pos]` whose **end `B`
//! is a factor boundary**.  The source is fully inside the parsed prefix
//! (`0 <= B - l`), and because `B <= pos` its length `l` satisfies
//! `l <= B - (B - l) <= pos`, i.e. a copy factor never overlaps the position it
//! is written to (`length <= offset`), exactly as in non-overlapping LZ77.
//!
//! * If such a copy of length `l >= 1` exists, the longest one is emitted as
//!   the factor `(offset, length) = (pos - (B - l), l)`; the offset is the
//!   distance from `pos` back to the start of the source.
//! * Otherwise a literal factor `(0, 1)` is emitted (`s[pos]` itself).
//!
//! **Tie rule.** When several boundaries admit a source of the same maximal
//! length `l`, the one met first in the scan below (the *smallest* boundary
//! `B`, since boundaries are visited in increasing order) is used.  The tie
//! rule only fixes *which* admissible source is reported; it does not affect
//! the soundness or maximality properties, which depend on the length alone.
//!
//! A length-`1` factor whose source is the single preceding character is also
//! a valid copy: at `pos` the boundary `pos` admits the source `s[pos-1 .. pos]`
//! whenever `s[pos-1] == s[pos]`.  In that case the copy `(1, 1)` is preferred
//! over the literal `(0, 1)` (a copy exists, so no literal is emitted); a
//! literal is emitted only when *no* copy of length `>= 1` exists.

/// One LZ-End factor.  `offset == 0` means a literal: the single character
/// `s[pos]` at the current position.  Otherwise the factor copies `length`
/// characters from `s[pos - offset ..]`, i.e. its source is
/// `s[pos - offset .. pos - offset + length]` and ends at the factor boundary
/// `pos - offset + length`.
pub struct LzEndFactor {
    pub offset: usize,
    pub length: usize,
}

/// Longest `l` with `1 <= l <= b`, `pos + l <= s.len()` and
/// `s[b - l .. b] == s[pos .. pos + l]`; `0` when no such `l` exists.
///
/// The source `s[b - l .. b]` is anchored at the boundary `b` (its *end* is
/// fixed at `b`), so as `l` grows the source window grows to the *left*; the
/// candidate lengths therefore cannot be extended one character at a time and
/// each is compared in full.
fn match_at_boundary(s: &[usize], pos: usize, b: usize) -> usize {
    let n = s.len();
    let mut best = 0;
    let mut l = 1;
    while l <= b {
        let end = pos + l;
        if end <= n {
            let a = b - l;
            let mut k = 0;
            let mut same = true;
            while k < l {
                let x = s[a + k];
                let y = s[pos + k];
                if x != y {
                    same = false;
                }
                k += 1;
            }
            if same {
                best = l;
            }
        }
        l += 1;
    }
    best
}

/// The offset of a copy factor with source `s[b - l .. b]` starting at `pos`:
/// `pos - (b - l)`, i.e. the distance from `pos` back to the source start.
fn offset_of(pos: usize, b: usize, l: usize) -> usize {
    let a = b - l;
    let off = pos - a;
    off
}

/// Greedy LZ-End decomposition of `s` (see the module documentation for the
/// exact definition and tie rule).
pub fn lz_end(s: &[usize]) -> Vec<LzEndFactor> {
    let n = s.len();
    let mut factors: Vec<LzEndFactor> = Vec::new();
    // The factor boundaries produced so far: `0` and every cumulative sum of
    // the emitted factor lengths.  The last element is always the current
    // position `pos`.
    let mut boundaries: Vec<usize> = Vec::new();
    boundaries.push(0);
    let mut pos = 0;
    while pos < n {
        let mut best_len = 0;
        let mut best_off = 0;
        let mut bi = 0;
        while bi < boundaries.len() {
            let b = boundaries[bi];
            let l = match_at_boundary(s, pos, b);
            if l > best_len {
                let off = offset_of(pos, b, l);
                best_len = l;
                best_off = off;
            }
            bi += 1;
        }
        if best_len == 0 {
            factors.push(LzEndFactor { offset: 0, length: 1 });
            pos += 1;
        } else {
            factors.push(LzEndFactor { offset: best_off, length: best_len });
            pos += best_len;
        }
        boundaries.push(pos);
    }
    factors
}

#[cfg(test)]
mod tests {
    //! Untranslated brute-force checks (this module is not picked up by the
    //! test translation).  They (a) re-implement the definition directly,
    //! (b) check the emitted factors satisfy the LZ-End validity predicate,
    //! (c) enumerate *all* valid factorizations of short strings to test the
    //! optimality conjecture `greedy <= every valid factorization`.
    use super::{lz_end, LzEndFactor};

    /// Reference greedy implementation, written straight from the definition.
    fn ref_greedy(s: &[usize]) -> Vec<(usize, usize)> {
        let n = s.len();
        let mut factors: Vec<(usize, usize)> = Vec::new();
        let mut pos = 0;
        let mut bs: Vec<usize> = vec![0];
        while pos < n {
            let mut best_len = 0;
            let mut best_off = 0;
            let mut bi = 0;
            while bi < bs.len() {
                let b = bs[bi];
                let mut l = 1;
                while l <= b && l <= n - pos {
                    let a = b - l;
                    if s[a..a + l] == s[pos..pos + l] {
                        if l > best_len {
                            best_len = l;
                            best_off = pos - a;
                        }
                    }
                    l += 1;
                }
                bi += 1;
            }
            if best_len == 0 {
                factors.push((0, 1));
                pos += 1;
            } else {
                factors.push((best_off, best_len));
                pos += best_len;
            }
            bs.push(pos);
        }
        factors
    }

    fn as_pairs(fs: &Vec<LzEndFactor>) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = Vec::new();
        let mut i = 0;
        while i < fs.len() {
            out.push((fs[i].offset, fs[i].length));
            i += 1;
        }
        out
    }

    /// The best copy length at `pos` wrt. the boundary set `bs` (0 if none).
    fn best_copy(s: &[usize], pos: usize, bs: &[usize]) -> usize {
        let n = s.len();
        let mut best = 0;
        let mut bi = 0;
        while bi < bs.len() {
            let b = bs[bi];
            let mut l = 1;
            while l <= b && l <= n - pos {
                let a = b - l;
                if s[a..a + l] == s[pos..pos + l] {
                    if l > best {
                        best = l;
                    }
                }
                l += 1;
            }
            bi += 1;
        }
        best
    }

    /// Replay the factors; `None` on a malformed factor list.
    fn replay(s: &[usize], fs: &[(usize, usize)]) -> Option<Vec<usize>> {
        let mut out: Vec<usize> = Vec::new();
        let mut pos = 0;
        let mut i = 0;
        while i < fs.len() {
            let (off, l) = fs[i];
            if off == 0 {
                if l != 1 {
                    return None;
                }
                out.push(s[pos]);
                pos += 1;
            } else {
                let mut j = 0;
                while j < l {
                    out.push(s[pos - off + j]);
                    j += 1;
                }
                pos += l;
            }
            i += 1;
        }
        if pos == s.len() {
            Some(out)
        } else {
            None
        }
    }

    /// The LZ-End validity predicate, written from the implementation's guard:
    /// every factor nonempty, literals of length one, copies start inside the
    /// parsed prefix and **end at a factor boundary**, and the factors tile `s`.
    fn is_valid(s: &[usize], fs: &[(usize, usize)]) -> bool {
        match replay(s, fs) {
            None => return false,
            Some(_) => {}
        }
        let mut pos = 0;
        let mut bs: Vec<usize> = vec![0];
        let mut i = 0;
        while i < fs.len() {
            let (off, l) = fs[i];
            if l < 1 {
                return false;
            }
            if off == 0 {
                if l != 1 {
                    return false;
                }
            } else {
                if off > pos {
                    return false;
                }
                let bend = pos - off + l;
                // ends at a factor boundary
                let mut found = false;
                let mut bi = 0;
                while bi < bs.len() {
                    if bs[bi] == bend {
                        found = true;
                    }
                    bi += 1;
                }
                if !found {
                    return false;
                }
                if s[pos - off..pos - off + l] != s[pos..pos + l] {
                    return false;
                }
            }
            pos += l;
            bs.push(pos);
            i += 1;
        }
        pos == s.len()
    }

    /// The maximality predicate: literals only where no copy exists, copies of
    /// maximal length.
    fn is_maximal(s: &[usize], fs: &[(usize, usize)]) -> bool {
        let mut pos = 0;
        let mut bs: Vec<usize> = vec![0];
        let mut i = 0;
        while i < fs.len() {
            let (off, l) = fs[i];
            let bc = best_copy(s, pos, &bs);
            if off == 0 {
                if bc != 0 {
                    return false;
                }
            } else {
                if l != bc {
                    return false;
                }
            }
            pos += l;
            bs.push(pos);
            i += 1;
        }
        true
    }

    /// Minimum factor count over all valid LZ-End factorizations (branch and
    /// bound over literals and every admissible copy length).
    fn min_factors(s: &[usize]) -> usize {
        let n = s.len();
        let mut best = usize::MAX;
        let mut stack: Vec<(usize, Vec<usize>, usize)> = vec![(0, vec![0], 0)];
        while let Some((pos, bs, cnt)) = stack.pop() {
            if cnt >= best {
                continue;
            }
            if pos == n {
                if cnt < best {
                    best = cnt;
                }
                continue;
            }
            // literal
            let mut bs2 = bs.clone();
            bs2.push(pos + 1);
            stack.push((pos + 1, bs2, cnt + 1));
            // copies
            let mut bi = 0;
            while bi < bs.len() {
                let b = bs[bi];
                let mut l = 1;
                while l <= b && l <= n - pos {
                    let a = b - l;
                    if s[a..a + l] == s[pos..pos + l] {
                        let mut nb = bs.clone();
                        nb.push(pos + l);
                        stack.push((pos + l, nb, cnt + 1));
                    }
                    l += 1;
                }
                bi += 1;
            }
        }
        best
    }

    fn all_binary(max_len: usize) -> Vec<Vec<usize>> {
        let mut out: Vec<Vec<usize>> = vec![Vec::new()];
        let mut l = 1;
        while l <= max_len {
            let prev: Vec<Vec<usize>> = out
                .iter()
                .filter(|w| w.len() == l - 1)
                .cloned()
                .collect();
            for w in prev {
                let mut a = w.clone();
                a.push(0);
                out.push(a);
                let mut b = w.clone();
                b.push(1);
                out.push(b);
            }
            l += 1;
        }
        out
    }

    #[test]
    fn impl_matches_reference() {
        // The AENEAS-style implementation agrees with the direct definition.
        for s in all_binary(11) {
            let got = as_pairs(&lz_end(&s));
            let want = ref_greedy(&s);
            assert_eq!(got, want, "mismatch on {:?}", s);
        }
    }

    #[test]
    fn emitted_factors_are_valid_and_maximal() {
        // (a) the implementation's output satisfies the predicate;
        // plus the maximality predicate.
        let mut count: usize = 0;
        for s in all_binary(12) {
            let fs = as_pairs(&lz_end(&s));
            assert!(is_valid(&s, &fs), "invalid on {:?}", s);
            assert!(is_maximal(&s, &fs), "non-maximal on {:?}", s);
            count += 1;
        }
        assert!(count > 4000);
    }

    #[test]
    fn greedy_is_not_optimal() {
        // (b) brute force DISPROVES `greedy <= every valid factorization`:
        // on s = [0,0,1,0,0,0,1,0] greedy emits 6 factors while
        // [(0,1),(0,1),(0,1),(0,1),(4,4)] is a valid 5-factor parsing.
        let s: Vec<usize> = vec![0, 0, 1, 0, 0, 0, 1, 0];
        let g = as_pairs(&lz_end(&s));
        assert_eq!(g.len(), 6);
        let m = vec![(0, 1), (0, 1), (0, 1), (0, 1), (4, 4)];
        assert!(is_valid(&s, &m));
        assert!(m.len() < g.len());

        // and greedy can never beat the minimum, of course.
        let mut checked: usize = 0;
        let mut optimals: usize = 0;
        for w in all_binary(10) {
            let g = as_pairs(&lz_end(&w));
            let mn = min_factors(&w);
            assert!(g.len() >= mn, "greedy below min on {:?}", w);
            if g.len() == mn {
                optimals += 1;
            }
            checked += 1;
        }
        assert!(checked > 2000);
        assert!(optimals < checked); // optimality genuinely fails somewhere
    }
}
