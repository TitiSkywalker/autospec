// ── Independent brute force: enumerate every submask of every mask ──────────

/// `x mod m` written without the `%` operator (`UScalar.rem` is not always
/// available in translated test files).
fn rem_ui(x: usize, m: usize) -> usize {
    let q = x / m;
    x - q * m
}

/// Product of two values, routed through a helper so that neither operand is a
/// computed local at the call site.
fn mul_ui(x: usize, y: usize) -> usize {
    x * y
}

/// `x + y`, for the same reason.
fn add_ui(x: usize, y: usize) -> usize {
    x + y
}

/// `O(3ⁿ)` subset convolution: `c[S] = Σ_{T ⊆ S} a[T] * b[S - T]`, summing over
/// every `T` that is a submask of `S`.
fn brute_conv(a: &[usize], b: &[usize]) -> Vec<usize> {
    let len = a.len();
    let mut c = vec![0usize; len];
    let mut s = 0;
    while s < len {
        let mut acc = 0usize;
        let mut t = 0;
        while t < len {
            let sub = t & s;
            if sub == t {
                let rest = s - t;
                let p = mul_ui(a[t], b[rest]);
                acc += p;
            }
            t += 1;
        }
        c[s] = acc;
        s += 1;
    }
    c
}

/// Compare two `Vec<usize>` element by element (AENEAS has no `PartialEq` on
/// `Vec`).
fn same_vec(x: &[usize], y: &[usize]) -> bool {
    let mut ok = true;
    if x.len() != y.len() {
        ok = false;
    } else {
        let mut i = 0;
        while i < x.len() {
            if x[i] != y[i] {
                ok = false;
            }
            i += 1;
        }
    }
    ok
}

// ── Known-answer tests ─────────────────────────────────────────────────────

#[test]
fn test_single_element() {
    // The empty-set convolution is the plain product of the two empty-set values.
    let a = vec![3];
    let b = vec![5];
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    assert_eq!(c[0], 15);
    assert!(same_vec(&c, &brute_conv(&a, &b)));
}

#[test]
fn test_two_elements() {
    // n = 1: c[{}] = a[{}]b[{}], c[{0}] = a[{0}]b[{}] + a[{}]b[{0}].
    let a = vec![2, 3];
    let b = vec![5, 7];
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    assert_eq!(c[0], 10);
    assert_eq!(c[1], 29);
    assert!(same_vec(&c, &brute_conv(&a, &b)));
}

#[test]
fn test_unit_vectors() {
    // a = e_{∅} and b = e_{∅} → c = e_{∅}.
    let n: usize = 3;
    let size = 1usize << n;
    let mut a = vec![0usize; size];
    let mut b = vec![0usize; size];
    a[0] = 1;
    b[0] = 1;
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    assert_eq!(c[0], 1);
    let mut i: usize = 1;
    while i < size {
        assert_eq!(c[i], 0);
        i += 1;
    }
}

#[test]
fn test_delta_shift() {
    // a = e_{S} with S = 5 = {0,2}: (e_S * b)[T] = b[T \ S] when S ⊆ T.
    let n: usize = 3;
    let size = 1usize << n;
    let mut a = vec![0usize; size];
    a[5] = 1;
    let mut b = vec![0usize; size];
    let mut i: usize = 0;
    while i < size {
        let v = i + 1;
        b[i] = v;
        i += 1;
    }
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    assert_eq!(c[5], b[0]);
    assert_eq!(c[7], b[2]);
    assert_eq!(c[1], 0);
    assert_eq!(c[2], 0);
    assert_eq!(c[4], 0);
    assert!(same_vec(&c, &brute_conv(&a, &b)));
}

#[test]
fn test_ones_multiply_by_two() {
    // a = b = all ones: c[S] = Σ_{T ⊆ S} 1 = 2^popcount(S).
    let n: usize = 4;
    let size = 1usize << n;
    let a = vec![1usize; size];
    let b = vec![1usize; size];
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    let mut s: usize = 0;
    while s < size {
        // 2^popcount(s), computed without a popcount helper.
        let mut want: usize = 1;
        let mut r: usize = 0;
        while r < n {
            let bit = 1usize << r;
            let has = s & bit;
            if has != 0 {
                want = mul_ui(want, 2);
            }
            r += 1;
        }
        assert_eq!(c[s], want);
        s += 1;
    }
    assert!(same_vec(&c, &brute_conv(&a, &b)));
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force_small() {
    let mut n: usize = 0;
    while n <= 4 {
        let size = 1usize << n;
        let mut a = vec![0usize; size];
        let mut b = vec![0usize; size];
        let mut i: usize = 0;
        while i < size {
            let ia = rem_ui(add_ui(mul_ui(i, i), 3), 11);
            a[i] = ia;
            let ib = rem_ui(add_ui(mul_ui(i, 5), 1), 7);
            b[i] = ib;
            i += 1;
        }
        let got = dp_subset_convolution::subset_convolution(&a, &b);
        let want = brute_conv(&a, &b);
        assert!(same_vec(&got, &want));
        n += 1;
    }
}

#[test]
fn test_agrees_with_brute_force_zero_heavy() {
    // Many zero entries: the rank layers must not leak across each other.
    let n: usize = 4;
    let size = 1usize << n;
    let mut a = vec![0usize; size];
    let mut b = vec![0usize; size];
    let mut i: usize = 0;
    while i < size {
        let m3 = rem_ui(i, 3);
        if m3 == 0 {
            let v = i + 1;
            a[i] = v;
        }
        let m2 = rem_ui(i, 2);
        if m2 == 1 {
            let w = i + 2;
            b[i] = w;
        }
        i += 1;
    }
    let got = dp_subset_convolution::subset_convolution(&a, &b);
    let want = brute_conv(&a, &b);
    assert!(same_vec(&got, &want));
}

#[test]
fn test_agrees_with_brute_force_full_values() {
    // A larger instance (n = 5, values up to 32).
    let n: usize = 5;
    let size = 1usize << n;
    let mut a = vec![0usize; size];
    let mut b = vec![0usize; size];
    let mut i: usize = 0;
    while i < size {
        let va = i + 1;
        a[i] = va;
        let vb = size - i;
        b[i] = vb;
        i += 1;
    }
    let got = dp_subset_convolution::subset_convolution(&a, &b);
    let want = brute_conv(&a, &b);
    assert!(same_vec(&got, &want));
}

#[test]
fn test_not_plain_pointwise_product() {
    // The subset convolution is *not* the pointwise product; make sure the rank
    // layers are actually doing work.
    let n: usize = 2;
    let size = 1usize << n;
    let a = vec![1usize; size];
    let b = vec![1usize; size];
    let c = dp_subset_convolution::subset_convolution(&a, &b);
    // c[S] = 2^|S|, whereas the pointwise product is all ones.
    assert_eq!(c[3], 4);
    assert_eq!(c[1], 2);
}
