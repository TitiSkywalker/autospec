//! Concrete, translated tests for the de Bruijn construction.
//!
//! Everything here is deliberately tiny: this file is translated and `#eval`'d
//! by CI.  The heavy exhaustive checking lives in the (untranslated)
//! `#[cfg(test)] mod tests` of `src/lib.rs`.
//!
//! `t_cyclic` is only used on inputs with `k ≥ 2`, where `k^n ≥ n - 1`, so
//! `b ++ b[0 .. n-1]` is a well-defined cyclic linearisation.

/// The cyclic word `b ++ b[0 .. n-1]` of `b`.
fn t_cyclic(b: &[usize], n: usize) -> Vec<usize> {
    let len = b.len();
    let mut cw: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < len {
        let x = b[i];
        cw.push(x);
        i += 1;
    }
    let mut j = 0;
    while j + 1 < n {
        let x = b[j];
        cw.push(x);
        j += 1;
    }
    cw
}

/// The number of length-`n` windows of a word of length `len` (`len - n + 1`),
/// routed through a helper so both operands are tracked `usize` parameters.
fn t_win_count(len: usize, n: usize) -> usize {
    let a = len - n;
    let b = a + 1;
    b
}

/// The number of pairs of equal length-`n` windows of `cw`.  Zero means the
/// windows are all distinct; since there are exactly `k^n` of them and `k^n`
/// possible words, each then occurs exactly once.
fn t_dup_windows(cw: &[usize], n: usize) -> usize {
    let len = cw.len();
    let nwords = t_win_count(len, n);
    let mut dups: usize = 0;
    let mut i = 0;
    while i < nwords {
        let mut j = i + 1;
        while j < nwords {
            let mut same = true;
            let mut d = 0;
            while d < n {
                let a = cw[i + d];
                let b = cw[j + d];
                if a != b {
                    same = false;
                }
                d += 1;
            }
            if same {
                dups += 1;
            }
            j += 1;
        }
        i += 1;
    }
    dups
}

#[test]
fn test_two_one() {
    let b = string_debruijn::de_bruijn(2usize, 1usize);
    assert_eq!(b.len(), 2);
    assert_eq!(b[0], 0);
    assert_eq!(b[1], 1);
}

#[test]
fn test_two_two() {
    let b = string_debruijn::de_bruijn(2usize, 2usize);
    assert_eq!(b.len(), 4);
    let cw = t_cyclic(&b, 2usize);
    assert_eq!(t_dup_windows(&cw, 2usize), 0);
}

#[test]
fn test_two_three() {
    let b = string_debruijn::de_bruijn(2usize, 3usize);
    assert_eq!(b.len(), 8);
    let cw = t_cyclic(&b, 3usize);
    assert_eq!(t_dup_windows(&cw, 3usize), 0);
}

#[test]
fn test_three_two() {
    let b = string_debruijn::de_bruijn(3usize, 2usize);
    assert_eq!(b.len(), 9);
    let cw = t_cyclic(&b, 2usize);
    assert_eq!(t_dup_windows(&cw, 2usize), 0);
}

#[test]
fn test_one_four() {
    let b = string_debruijn::de_bruijn(1usize, 4usize);
    assert_eq!(b.len(), 1);
    assert_eq!(b[0], 0);
}
