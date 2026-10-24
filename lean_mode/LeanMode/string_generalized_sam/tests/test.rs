//! Tests for the generalized suffix automaton (GSAM) of a *set* of strings.
//!
//! Everything here is small and concrete: a handful of multi-string inputs with
//! hand-checkable answers, plus an O(n²) brute-force cross-check used only on
//! those short inputs.  The heavy/randomised differential fuzzing and the
//! endpos-class invariant live in the (untranslated) `#[cfg(test)] mod tests` of
//! `src/lib.rs`.

use string_generalized_sam::{gsam_build, gsam_contains};

/// `a + b` (both operands become call arguments, which keeps the `usize` type
/// tracking happy).
fn t_add(a: usize, b: usize) -> usize {
    let r = a + b;
    r
}

/// The `k`-th string encoded in `s`/`lens` (empty when `k` is out of range).
fn t_str_at(s: &[usize], lens: &[usize], k: usize) -> Vec<usize> {
    let c = lens.len();
    if k >= c {
        return Vec::new();
    }
    let mut base: usize = 0;
    let mut i: usize = 0;
    while i < k {
        let l = lens[i];
        let nb = t_add(base, l);
        base = nb;
        i += 1;
    }
    let l = lens[k];
    let mut out: Vec<usize> = Vec::new();
    let mut j: usize = 0;
    while j < l {
        let p = t_add(base, j);
        out.push(s[p]);
        j += 1;
    }
    out
}

/// Whether `p` occurs as a window of `t`.
fn t_occurs(t: &[usize], p: &[usize]) -> bool {
    let n = t.len();
    let m = p.len();
    let mut found = false;
    if m <= n {
        let mut i = 0;
        while i + m <= n && !found {
            let mut k = 0;
            let mut same = true;
            while k < m {
                let x = t[i + k];
                let y = p[k];
                if x != y {
                    same = false;
                }
                k += 1;
            }
            if same {
                found = true;
            }
            i += 1;
        }
    }
    found
}

/// Brute force: does `p` occur as a window of *some* of the input strings?
fn t_in_some(s: &[usize], lens: &[usize], p: &[usize]) -> bool {
    let c = lens.len();
    let mut k = 0;
    let mut found = false;
    while k < c && !found {
        let t = t_str_at(s, lens, k);
        if t_occurs(&t, p) {
            found = true;
        }
        k += 1;
    }
    found
}

/// `0` iff `gsam_contains g` agrees with the brute-force membership test on the
/// flat pattern list `(pats, patlens)`, then `+ 1` for each pattern that occurs
/// in some input string yet is not reachable in the automaton (the
/// completeness direction, checked over the substrings of the inputs).
fn t_mismatches(s: &[usize], lens: &[usize], pats: &[usize], patlens: &[usize]) -> usize {
    let g = gsam_build(s, lens);
    let np = patlens.len();
    let mut bad: usize = 0;
    let mut base: usize = 0;
    let mut i = 0;
    while i < np {
        let m = patlens[i];
        let mut p: Vec<usize> = Vec::new();
        let mut j = 0;
        while j < m {
            let q = t_add(base, j);
            p.push(pats[q]);
            j += 1;
        }
        let got = gsam_contains(&g, &p);
        let want = t_in_some(s, lens, &p);
        if got != want {
            bad += 1;
        }
        let nb = t_add(base, m);
        base = nb;
        i += 1;
    }
    // Every nonempty substring of every input string must be reachable.
    let c = lens.len();
    let mut k = 0;
    while k < c {
        let t = t_str_at(s, lens, k);
        let n = t.len();
        let mut i2 = 0;
        while i2 < n {
            let mut l = 1;
            while i2 + l <= n {
                let mut w: Vec<usize> = Vec::new();
                let mut d = 0;
                while d < l {
                    let q = t_add(i2, d);
                    w.push(t[q]);
                    d += 1;
                }
                if !gsam_contains(&g, &w) {
                    bad += 1;
                }
                l += 1;
            }
            i2 += 1;
        }
        k += 1;
    }
    bad
}

#[test]
fn test_two_strings_ab_bc() {
    // "ab" ++ "bc", so lens = [2, 2].
    let s: [usize; 4] = [97, 98, 98, 99];
    let lens: [usize; 2] = [2, 2];
    let g = gsam_build(&s, &lens);

    let ab: [usize; 2] = [97, 98];
    let bc: [usize; 2] = [98, 99];
    let b: [usize; 1] = [98];
    let abc: [usize; 3] = [97, 98, 99]; // spans the boundary
    let ac: [usize; 2] = [97, 99]; // substrings of neither
    let empty: [usize; 0] = [];

    assert_eq!(gsam_contains(&g, &ab), true);
    assert_eq!(gsam_contains(&g, &bc), true);
    assert_eq!(gsam_contains(&g, &b), true);
    assert_eq!(gsam_contains(&g, &abc), false);
    assert_eq!(gsam_contains(&g, &ac), false);
    assert_eq!(gsam_contains(&g, &empty), true);

    // "ab" | "bc" | "b" | "abc" | "ac"
    let pats: [usize; 11] = [97, 98, 98, 99, 98, 97, 98, 99, 97, 99, 98];
    let patlens: [usize; 5] = [2, 2, 1, 3, 2];
    assert_eq!(t_mismatches(&s, &lens, &pats, &patlens), 0);
}

#[test]
fn test_single_char_boundary() {
    // "a" ++ "b": every 2-letter word spans the boundary.
    let s: [usize; 2] = [97, 98];
    let lens: [usize; 2] = [1, 1];
    let g = gsam_build(&s, &lens);

    let a: [usize; 1] = [97];
    let b: [usize; 1] = [98];
    let ab: [usize; 2] = [97, 98];
    let ba: [usize; 2] = [98, 97];

    assert_eq!(gsam_contains(&g, &a), true);
    assert_eq!(gsam_contains(&g, &b), true);
    assert_eq!(gsam_contains(&g, &ab), false);
    assert_eq!(gsam_contains(&g, &ba), false);
    assert_eq!(g.sz, 3);

    let pats: [usize; 6] = [97, 98, 97, 98, 98, 97]; // a, b, ab, ba
    let patlens: [usize; 4] = [1, 1, 2, 2];
    assert_eq!(t_mismatches(&s, &lens, &pats, &patlens), 0);
}

#[test]
fn test_shared_prefix() {
    // "ab" ++ "aba": the second insertion reuses the "ab" path from the root,
    // so the reset-last insert produces a duplicate (unreachable) state.
    let s: [usize; 5] = [97, 98, 97, 98, 97];
    let lens: [usize; 2] = [2, 3];
    let g = gsam_build(&s, &lens);

    let ab: [usize; 2] = [97, 98];
    let aba: [usize; 3] = [97, 98, 97];
    let ba: [usize; 2] = [98, 97];
    let aa: [usize; 2] = [97, 97];
    let bb: [usize; 2] = [98, 98];
    let abab: [usize; 4] = [97, 98, 97, 98]; // crosses the boundary
    let bab: [usize; 3] = [98, 97, 98]; // crosses the boundary

    assert_eq!(gsam_contains(&g, &ab), true);
    assert_eq!(gsam_contains(&g, &aba), true);
    assert_eq!(gsam_contains(&g, &ba), true);
    assert_eq!(gsam_contains(&g, &aa), false);
    assert_eq!(gsam_contains(&g, &bb), false);
    assert_eq!(gsam_contains(&g, &abab), false);
    assert_eq!(gsam_contains(&g, &bab), false);
    assert!(g.sz <= 2 * 5 + 1);

    let pats: [usize; 23] = [
        97, 98, // ab
        97, 98, 97, // aba
        98, 97, // ba
        97, 97, // aa
        98, 98, // bb
        97, 98, 97, 98, // abab
        98, 97, 98, // bab
        97, // a
        98, // b
        98, 97, 97, // baa
    ];
    let patlens: [usize; 10] = [2, 3, 2, 2, 2, 4, 3, 1, 1, 3];
    assert_eq!(t_mismatches(&s, &lens, &pats, &patlens), 0);
}

#[test]
fn test_three_strings() {
    // "abc" ++ "bc" ++ "cd", so lens = [3, 2, 2].
    let s: [usize; 7] = [97, 98, 99, 98, 99, 99, 100];
    let lens: [usize; 3] = [3, 2, 2];
    let g = gsam_build(&s, &lens);

    let abc: [usize; 3] = [97, 98, 99];
    let bc: [usize; 2] = [98, 99];
    let cd: [usize; 2] = [99, 100];
    let bcd: [usize; 3] = [98, 99, 100]; // crosses "bc" / "cd"
    let ca: [usize; 2] = [99, 97]; // crosses "abc" / "bc"
    let da: [usize; 2] = [100, 97]; // crosses "cd" / ... nothing

    assert_eq!(gsam_contains(&g, &abc), true);
    assert_eq!(gsam_contains(&g, &bc), true);
    assert_eq!(gsam_contains(&g, &cd), true);
    assert_eq!(gsam_contains(&g, &bcd), false);
    assert_eq!(gsam_contains(&g, &ca), false);
    assert_eq!(gsam_contains(&g, &da), false);

    let pats: [usize; 14] = [
        97, 98, 99, // abc
        98, 99, // bc
        99, 100, // cd
        98, 99, 100, // bcd
        99, 97, // ca
        100, 97, // da
    ];
    let patlens: [usize; 6] = [3, 2, 2, 3, 2, 2];
    assert_eq!(t_mismatches(&s, &lens, &pats, &patlens), 0);
}

#[test]
fn test_repeated_chars_and_size_bound() {
    // "aaa" ++ "aa": both are runs of 'a'.
    let s: [usize; 5] = [97, 97, 97, 97, 97];
    let lens: [usize; 2] = [3, 2];
    let g = gsam_build(&s, &lens);

    let a: [usize; 1] = [97];
    let aa: [usize; 2] = [97, 97];
    let aaa: [usize; 3] = [97, 97, 97];
    let aaaa: [usize; 4] = [97, 97, 97, 97]; // spans the boundary
    let b: [usize; 1] = [98];

    assert_eq!(gsam_contains(&g, &a), true);
    assert_eq!(gsam_contains(&g, &aa), true);
    assert_eq!(gsam_contains(&g, &aaa), true);
    assert_eq!(gsam_contains(&g, &aaaa), false);
    assert_eq!(gsam_contains(&g, &b), false);
    assert!(g.sz <= 2 * 5 + 1);

    let pats: [usize; 11] = [97, 97, 97, 97, 97, 97, 97, 97, 97, 97, 98];
    let patlens: [usize; 5] = [1, 2, 3, 4, 1];
    assert_eq!(t_mismatches(&s, &lens, &pats, &patlens), 0);
}
