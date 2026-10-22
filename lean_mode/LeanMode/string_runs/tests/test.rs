//! Tests for the runs algorithm.
//!
//! Every case is cross-checked against an exhaustive `O(n³)` reference that
//! enumerates every interval `[a, b)`, computes its minimal period `p` (among
//! the periods with `2p <= b - a`), checks both maximality directions
//! (`[a-1, b)` and `[a, b+1)` must not be `p`-periodic) and collects the
//! surviving intervals.  The two sets are then compared element by element
//! (through linear membership scans, never `assert_eq!` on `Vec`s of structs).

use string_runs::{runs, Run};

// ----------------------- exhaustive reference -----------------------

/// Does `[a, b)` have period `p`?  Requires `p >= 1`.
///
/// This is weak periodicity: `s[k] == s[k + p]` for every `k` with
/// `a <= k` and `k + p < b`.
fn ref_periodic(s: &[usize], a: usize, b: usize, p: usize) -> bool {
    let mut ok = true;
    let mut k = a;
    let mut go = true;
    while go {
        let kp = k + p;
        if kp < b {
            let x = s[k];
            let y = s[kp];
            if x != y {
                ok = false;
            }
            k += 1;
        } else {
            go = false;
        }
    }
    ok
}

/// The smallest `p` with `2 * p <= b - a` and `[a, b)` `p`-periodic, or `0`
/// when no such `p` exists.  (When the true minimal period `p0` satisfies
/// `2 * p0 <= b - a` this is exactly `p0`; otherwise no larger period can
/// satisfy the length bound either.)
fn ref_run_period(s: &[usize], a: usize, b: usize) -> usize {
    let len = b - a;
    let mut res: usize = 0;
    let mut p: usize = 1;
    let mut go = true;
    while go {
        let twop = 2 * p;
        if twop <= len {
            let per = ref_periodic(s, a, b, p);
            if per {
                if res == 0 {
                    res = p;
                }
            }
            p += 1;
        } else {
            go = false;
        }
    }
    res
}

/// The runs of `s`, by exhaustive enumeration of every interval.
fn ref_runs(s: &[usize]) -> Vec<Run> {
    let n = s.len();
    let mut out: Vec<Run> = Vec::new();
    let mut a: usize = 0;
    while a < n {
        let mut b = a + 2;
        while b <= n {
            let p = ref_run_period(s, a, b);
            if p != 0 {
                let mut left_ok = true;
                if a > 0 {
                    let lp = ref_periodic(s, a - 1, b, p);
                    if lp {
                        left_ok = false;
                    }
                }
                let mut right_ok = true;
                if b < n {
                    let rp = ref_periodic(s, a, b + 1, p);
                    if rp {
                        right_ok = false;
                    }
                }
                if left_ok {
                    if right_ok {
                        out.push(Run { start: a, end: b, period: p });
                    }
                }
            }
            b += 1;
        }
        a += 1;
    }
    out
}

// --------------------------- comparison ---------------------------

/// Does the run list `v` contain the run `(st, en, pe)`?
fn contains_run(v: &Vec<Run>, st: usize, en: usize, pe: usize) -> bool {
    let m = v.len();
    let mut res: usize = m;
    let mut e: usize = 0;
    while e < m {
        let es = v[e].start;
        let ee = v[e].end;
        let ep = v[e].period;
        let mut same = true;
        if es != st {
            same = false;
        }
        if ee != en {
            same = false;
        }
        if ep != pe {
            same = false;
        }
        if same {
            if e < res {
                res = e;
            }
        }
        e += 1;
    }
    let found = res != m;
    found
}

/// Set equality of two duplicate-free run lists.
fn same_runs(x: &Vec<Run>, y: &Vec<Run>) -> bool {
    let mut ok = true;
    let lx = x.len();
    let ly = y.len();
    if lx != ly {
        ok = false;
    }
    let mut i: usize = 0;
    while i < lx {
        let st = x[i].start;
        let en = x[i].end;
        let pe = x[i].period;
        let c = contains_run(y, st, en, pe);
        if !c {
            ok = false;
        }
        i += 1;
    }
    let mut j: usize = 0;
    while j < ly {
        let st = y[j].start;
        let en = y[j].end;
        let pe = y[j].period;
        let c = contains_run(x, st, en, pe);
        if !c {
            ok = false;
        }
        j += 1;
    }
    ok
}

/// Check the algorithm's output against the exhaustive reference.
fn check_case(s: &[usize]) {
    let got = runs(s);
    let want = ref_runs(s);
    assert!(same_runs(&got, &want));
}

// ------------------------------ tests ------------------------------

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let got = runs(&s);
    assert_eq!(got.len(), 0);
    check_case(&s);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97];
    let got = runs(&s);
    assert_eq!(got.len(), 0);
    check_case(&s);
}

#[test]
fn test_aa() {
    let s: [usize; 2] = [97, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 2, 1));
    check_case(&s);
}

#[test]
fn test_aaa() {
    let s: [usize; 3] = [97, 97, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 3, 1));
    check_case(&s);
}

#[test]
fn test_aaaa() {
    let s: [usize; 4] = [97, 97, 97, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 4, 1));
    check_case(&s);
}

#[test]
fn test_ab() {
    let s: [usize; 2] = [97, 98];
    let got = runs(&s);
    assert_eq!(got.len(), 0);
    check_case(&s);
}

#[test]
fn test_abab() {
    let s: [usize; 4] = [97, 98, 97, 98];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 4, 2));
    check_case(&s);
}

#[test]
fn test_ababa() {
    let s: [usize; 5] = [97, 98, 97, 98, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 5, 2));
    check_case(&s);
}

#[test]
fn test_abcabcabc() {
    let s: [usize; 9] = [97, 98, 99, 97, 98, 99, 97, 98, 99];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 0, 9, 3));
    check_case(&s);
}

#[test]
fn test_aabaa() {
    // "aabaa": two runs, [0, 2) ("aa") and [3, 5) ("aa").
    let s: [usize; 5] = [97, 97, 98, 97, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 2);
    assert!(contains_run(&got, 0, 2, 1));
    assert!(contains_run(&got, 3, 5, 1));
    check_case(&s);
}

#[test]
fn test_banana() {
    // "banana": the single run "anana" = [1, 6) with period 2.
    let s: [usize; 6] = [98, 97, 110, 97, 110, 97];
    let got = runs(&s);
    assert_eq!(got.len(), 1);
    assert!(contains_run(&got, 1, 6, 2));
    check_case(&s);
}

#[test]
fn test_fibonacci_word() {
    // The length-13 Fibonacci word, a classic worst case for run counts.
    let s: [usize; 13] = [
        97, 98, 97, 97, 98, 97, 98, 97, 97, 98, 97, 97, 98,
    ];
    check_case(&s);
}

#[test]
fn test_mixed_alphabet() {
    let s: [usize; 18] = [
        97, 97, 98, 97, 97, 98, 97, 97, 98, 99, 99, 97, 98, 97, 98, 97, 98, 100,
    ];
    check_case(&s);
}
