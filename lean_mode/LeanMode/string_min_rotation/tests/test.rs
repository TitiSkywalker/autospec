//! Tests for `string_min_rotation`.
//!
//! Every case is cross-checked against an O(n²) brute-force reference that
//! compares all rotations element by element.

/// `true` iff rotation `k` is strictly smaller than rotation `j`, where `t` is
/// the doubled string (`t = s ++ s`) of a string of length `n`.
///
/// The two rotations are compared element by element; `d` is the length of
/// their longest common prefix, and the decision is taken at position `d`.
fn rotation_less(t: &[usize], n: usize, k: usize, j: usize) -> bool {
    let mut d = 0;
    let mut same = true;
    while same {
        if d < n {
            let x = t[k + d];
            let y = t[j + d];
            if x == y {
                d += 1;
            } else {
                same = false;
            }
        } else {
            same = false;
        }
    }
    let mut less = false;
    if d < n {
        let x = t[k + d];
        let y = t[j + d];
        if x < y {
            less = true;
        }
    }
    less
}

/// O(n²) reference: try every rotation and keep the smallest.
fn brute_min_rotation(s: &[usize]) -> usize {
    let n = s.len();
    if n == 0 {
        return 0;
    }
    // Explicitly doubled buffer, so no modulo is needed.
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
    let mut best = 0;
    let mut k = 1;
    while k < n {
        if rotation_less(&t, n, k, best) {
            best = k;
        }
        k += 1;
    }
    best
}

#[test]
fn test_empty() {
    let s: [usize; 0] = [];
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 0);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_single() {
    let s: [usize; 1] = [97]; // "a"
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 0);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_two_chars() {
    let s: [usize; 2] = [97, 98]; // "ab" -> "ab"
    assert_eq!(string_min_rotation::min_rotation(&s), 0);
    assert_eq!(brute_min_rotation(&s), 0);

    let t: [usize; 2] = [98, 97]; // "ba" -> "ab" at index 1
    assert_eq!(string_min_rotation::min_rotation(&t), 1);
    assert_eq!(brute_min_rotation(&t), 1);
}

#[test]
fn test_aba() {
    let s: [usize; 3] = [97, 98, 97]; // "aba" -> "aab" at index 2
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 2);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_abab() {
    let s: [usize; 4] = [97, 98, 97, 98]; // "abab" -> "abab" (index 0)
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 0);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_all_same() {
    let s: [usize; 4] = [97, 97, 97, 97]; // "aaaa" -> every rotation equal
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 0);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_cba() {
    let s: [usize; 3] = [99, 98, 97]; // "cba" -> "acb" at index 2
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 2);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_baab() {
    let s: [usize; 4] = [98, 97, 97, 98]; // "baab" -> "aabb" at index 1
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 1);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_aabaa() {
    let s: [usize; 5] = [97, 97, 98, 97, 97]; // "aabaa" -> "aaaab" at index 3
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 3);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_bbaaccaa() {
    let s: [usize; 8] = [98, 98, 97, 97, 99, 99, 97, 97]; // "bbaaccaa" -> index 6
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 6);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_bbaaccaadd() {
    // "bbaaccaadd" -> "aaccaaddbb" at index 2
    let s: [usize; 10] = [98, 98, 97, 97, 99, 99, 97, 97, 100, 100];
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(k, 2);
    assert_eq!(brute_min_rotation(&s), k);
}

#[test]
fn test_periodic_and_prefix_heavy() {
    // "abaababaab" (period 3 facet): every rotation cross-checked.
    let s: [usize; 10] = [97, 98, 97, 97, 98, 97, 98, 97, 97, 98];
    let k = string_min_rotation::min_rotation(&s);
    assert_eq!(brute_min_rotation(&s), k);

    // "zyxwvu": the reversal of a sorted string, minimum is the full reversal.
    let t: [usize; 6] = [122, 121, 120, 119, 118, 117];
    let l = string_min_rotation::min_rotation(&t);
    assert_eq!(l, 5);
    assert_eq!(brute_min_rotation(&t), l);
}
