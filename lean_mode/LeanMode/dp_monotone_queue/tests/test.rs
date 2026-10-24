// ── Independent brute force: min over all increasing chains ─────────────────

/// `|a - b|` (test-local copy; distinct name from the library helper).
fn ud2(a: usize, b: usize) -> usize {
    let d = if a >= b { a - b } else { b - a };
    d
}

/// Multiplication routed through a helper, so the operands are call arguments
/// (AENEAS mis-classifies `d * d` on a locally-derived `usize` as a pure `Int`
/// expression otherwise).
fn mq_mul(a: usize, b: usize) -> usize {
    a * b
}

/// Addition routed through a helper (same reason as `mq_mul`).
fn mq_add(a: usize, b: usize) -> usize {
    a + b
}

/// Third, structurally independent oracle.  Unrolling the recurrence
/// `dp[i] = min_{j<i} (dp[j] + (p[i]-p[j])^2)` gives
///
///   `dp[n-1] = min over chains 0 = i0 < i1 < ... < ik = n-1 of Σ (p[i_{t+1}]-p[i_t])^2`,
///
/// so we enumerate every increasing chain from `cur` to `n-1` recursively and
/// take the minimum.  Exponential in `n`; only call with `n <= 8`.
fn chain_min(p: &[usize], n: usize, cur: usize) -> usize {
    if cur + 1 >= n {
        return 0;
    }
    let pc = p[cur];
    let p1 = p[cur + 1];
    let d1 = ud2(p1, pc);
    let sq1 = mq_mul(d1, d1);
    let sub1 = chain_min(p, n, cur + 1);
    let mut best = mq_add(sq1, sub1);
    let mut nxt = cur + 2;
    while nxt < n {
        let pn = p[nxt];
        let d = ud2(pn, pc);
        let sq = mq_mul(d, d);
        let sub = chain_min(p, n, nxt);
        let c = mq_add(sq, sub);
        if c < best {
            best = c;
        }
        nxt += 1;
    }
    best
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_empty_and_single() {
    let e: Vec<usize> = vec![];
    assert_eq!(dp_monotone_queue::dp_naive(&e), 0);
    assert_eq!(dp_monotone_queue::dp_optimized(&e), 0);
    let one = vec![7usize];
    assert_eq!(dp_monotone_queue::dp_naive(&one), 0);
    assert_eq!(dp_monotone_queue::dp_optimized(&one), 0);
}

#[test]
fn test_consecutive_positions() {
    // For p = 0..n-1 every unit step is optimal, so dp[n-1] = n-1.
    let p = vec![0usize, 1, 2, 3, 4, 5];
    assert_eq!(dp_monotone_queue::dp_naive(&p), 5);
    assert_eq!(dp_monotone_queue::dp_optimized(&p), 5);
}

#[test]
fn test_repeated_positions() {
    // Equal positions cost nothing to cross.
    let p = vec![0usize, 0, 0, 0];
    assert_eq!(dp_monotone_queue::dp_naive(&p), 0);
    assert_eq!(dp_monotone_queue::dp_optimized(&p), 0);
}

#[test]
fn test_two_points() {
    let p = vec![0usize, 5];
    assert_eq!(dp_monotone_queue::dp_naive(&p), 25);
    assert_eq!(dp_monotone_queue::dp_optimized(&p), 25);
}

#[test]
fn test_small_hand_computed() {
    let a = vec![0usize, 3, 4]; // dp = [0, 9, 10]
    assert_eq!(dp_monotone_queue::dp_naive(&a), 10);
    assert_eq!(dp_monotone_queue::dp_optimized(&a), 10);

    let b = vec![0usize, 10, 11]; // dp = [0, 100, 101]
    assert_eq!(dp_monotone_queue::dp_naive(&b), 101);
    assert_eq!(dp_monotone_queue::dp_optimized(&b), 101);

    let c = vec![0usize, 1, 100]; // best is to jump 0 -> 1 -> 100: 1 + 9801
    assert_eq!(dp_monotone_queue::dp_naive(&c), 9802);
    assert_eq!(dp_monotone_queue::dp_optimized(&c), 9802);

    let d = vec![0usize, 1, 2, 100]; // 1 + 1 + 98^2
    assert_eq!(dp_monotone_queue::dp_naive(&d), 9606);
    assert_eq!(dp_monotone_queue::dp_optimized(&d), 9606);
}

#[test]
fn test_triangular_gaps() {
    // p = 0,1,3,6,10 (triangular); dp = [0, 1, 5, 14, 30].
    let p = vec![0usize, 1, 3, 6, 10];
    assert_eq!(dp_monotone_queue::dp_naive(&p), 30);
    assert_eq!(dp_monotone_queue::dp_optimized(&p), 30);
}

// ── Cross-checks ────────────────────────────────────────────────────────────

#[test]
fn test_opt_agrees_with_naive_fixed_size() {
    let mut seed: usize = 7;
    let mut trial = 0;
    while trial < 30 {
        let mut p: Vec<usize> = vec![0usize; 6];
        let mut v: usize = 0;
        let mut k = 0;
        while k < 6 {
            let s1 = seed * 5 + 3;
            let q = s1 / 64;
            seed = s1 - q * 64;
            v = v + seed;
            p[k] = v;
            k += 1;
        }
        let a = dp_monotone_queue::dp_naive(&p);
        let b = dp_monotone_queue::dp_optimized(&p);
        assert_eq!(a, b);
        let c = chain_min(&p, 6, 0);
        assert_eq!(a, c);
        trial += 1;
    }
}

#[test]
fn test_three_way_all_sizes() {
    let mut seed: usize = 12345;
    let mut n = 1;
    while n <= 7 {
        let mut p: Vec<usize> = vec![0usize; n];
        let mut v: usize = 0;
        let mut k = 0;
        while k < n {
            let s1 = seed * 3 + 1;
            let q = s1 / 50;
            seed = s1 - q * 50;
            v = v + seed;
            p[k] = v;
            k += 1;
        }
        let naive = dp_monotone_queue::dp_naive(&p);
        let opt = dp_monotone_queue::dp_optimized(&p);
        let brute = chain_min(&p, n, 0);
        assert_eq!(naive, opt);
        assert_eq!(naive, brute);
        n += 1;
    }
}
