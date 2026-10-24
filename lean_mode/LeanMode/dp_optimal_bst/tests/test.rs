// ── Independent brute force: enumerate every binary search tree ─────────────
//
// One self-recursive function handles both jobs, keyed by the `scanning` flag:
//
// * `scanning = false` — min cost of a BST on the *half-open* key range
//   `[lo, hi)` whose root sits at absolute depth `depth`.
// * `scanning = true` — the same minimum, but only over the candidate roots
//   `r, r+1, …, hi-1` (called with `r < hi`).
//
// The cost being minimised is the weighted internal path length
// `Σ_k freq[k] * (depth(k) + 1)` (a key at depth `d` costs `d + 1` visits).
// Choosing root `r` splits `[lo, hi)` into `[lo, r)` and `[r+1, hi)`, both one
// level deeper — a completely different enumeration from the interval DP.

fn bf_min(freq: &[usize], lo: usize, hi: usize, depth: usize, r: usize, scanning: bool) -> usize {
    if scanning {
        // Candidate root `r` (`r < hi` is guaranteed by the call sites).
        let d1 = depth + 1;
        let w = freq[r] * d1;
        let lc = bf_min(freq, lo, r, d1, 0, false);
        let rp1 = r + 1;
        let rc = bf_min(freq, rp1, hi, d1, 0, false);
        let cand = w + lc + rc;
        if rp1 < hi {
            let rest = bf_min(freq, lo, hi, depth, rp1, true);
            if cand < rest {
                cand
            } else {
                rest
            }
        } else {
            cand
        }
    } else if lo >= hi {
        0
    } else {
        bf_min(freq, lo, hi, depth, lo, true)
    }
}

/// Brute-force OBST cost for the frequency vector `freq`.
fn brute_optimal_bst(freq: &[usize]) -> usize {
    bf_min(freq, 0, freq.len(), 0, 0, false)
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_single_key() {
    let f = vec![7];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 7);
}

#[test]
fn test_two_keys() {
    // Either key can be the root; both cost 1*1 + 1*2 = 3.
    let f = vec![1, 1];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 3);
}

#[test]
fn test_three_equal() {
    // freq = [1,1,1]: the balanced tree (root 1, children 0 and 2) costs
    // 1 + 2 + 2 = 5, better than any chain (1 + 2 + 3 = 6).
    let f = vec![1, 1, 1];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 5);
}

#[test]
fn test_three_classic() {
    // freq = [10,20,30]: root 1 gives 20 + 2*10 + 2*30 = 100.
    let f = vec![10, 20, 30];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 100);
}

#[test]
fn test_three_increasing() {
    // freq = [1,2,3]: optimum is root 1 (cost 2 + 2*1 + 2*3 = 10).
    let f = vec![1, 2, 3];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 10);
}

#[test]
fn test_four_skewed() {
    // freq = [1,2,4,8]: moving the heavy keys up wins — the optimal tree is the
    // left chain 3 → 2 → 1 → 0, at depths 0,1,2,3:
    // 8*1 + 4*2 + 2*3 + 1*4 = 8 + 8 + 6 + 4 = 26.
    let f = vec![1, 2, 4, 8];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 26);
}

// ── Degenerate inputs ───────────────────────────────────────────────────────

#[test]
fn test_empty() {
    let f: Vec<usize> = vec![];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 0);
}

#[test]
fn test_single_zero() {
    let f = vec![0];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 0);
}

#[test]
fn test_zero_frequencies() {
    let f = vec![0, 0, 0];
    assert_eq!(dp_optimal_bst::optimal_bst(&f), 0);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force() {
    let mut n = 1;
    while n <= 6 {
        // Pattern A: all ones.
        let mut fa = vec![0usize; n];
        let mut ka = 0;
        while ka < n {
            fa[ka] = 1;
            ka += 1;
        }
        assert_eq!(dp_optimal_bst::optimal_bst(&fa), brute_optimal_bst(&fa));

        // Pattern B: strictly increasing.
        let mut fb = vec![0usize; n];
        let mut kb = 0;
        while kb < n {
            let tb = kb + 1;
            fb[kb] = tb;
            kb += 1;
        }
        assert_eq!(dp_optimal_bst::optimal_bst(&fb), brute_optimal_bst(&fb));

        // Pattern C: strictly decreasing.
        let mut fc = vec![0usize; n];
        let mut kc = 0;
        while kc < n {
            let tc = n - kc;
            fc[kc] = tc;
            kc += 1;
        }
        assert_eq!(dp_optimal_bst::optimal_bst(&fc), brute_optimal_bst(&fc));

        // Pattern D: includes a zero frequency.
        let mut fd = vec![0usize; n];
        let mut kd = 0;
        while kd < n {
            fd[kd] = kd;
            kd += 1;
        }
        assert_eq!(dp_optimal_bst::optimal_bst(&fd), brute_optimal_bst(&fd));

        n += 1;
    }
}
