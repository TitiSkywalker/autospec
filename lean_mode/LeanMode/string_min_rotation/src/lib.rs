//! Index of the lexicographically smallest cyclic rotation (Booth's algorithm).
//!
//! Characters are `usize` byte values (`0..=255`); the empty string has
//! minimum rotation `0`.

/// Build the doubled string `t = s ++ s` (two `while` loops + `push`, so no
/// modulo is needed anywhere in the algorithm).
fn doubled(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut t: Vec<usize> = Vec::new();
    let mut p = 0;
    while p < n {
        let c = s[p];
        t.push(c);
        p += 1;
    }
    let mut q = 0;
    while q < n {
        let c = s[q];
        t.push(c);
        q += 1;
    }
    t
}

/// Index of the lexicographically smallest cyclic rotation of `s`
/// (0 for the empty string).
///
/// Booth's algorithm over the doubled string `t = s ++ s`: two candidate
/// starting positions `i` and `j` are advanced while their rotations share a
/// prefix of length `k`; the loser's candidate is pushed past the mismatch.
/// The winner is `min i j`, and both candidates stay below `n`.
pub fn min_rotation(s: &[usize]) -> usize {
    let n = s.len();
    if n == 0 {
        return 0;
    }
    let t = doubled(s);
    let mut i = 0;
    let mut j = 1;
    let mut k = 0;
    let mut run = true;
    while run {
        if i < n {
            if j < n {
                if k < n {
                    let a = t[i + k];
                    let b = t[j + k];
                    if a == b {
                        k += 1;
                    } else {
                        if a > b {
                            let mut ni = i + k + 1;
                            if ni <= j {
                                ni = j + 1;
                            }
                            i = ni;
                        } else {
                            let mut nj = j + k + 1;
                            if nj <= i {
                                nj = i + 1;
                            }
                            j = nj;
                        }
                        k = 0;
                    }
                } else {
                    run = false;
                }
            } else {
                run = false;
            }
        } else {
            run = false;
        }
    }
    if i < j { i } else { j }
}
