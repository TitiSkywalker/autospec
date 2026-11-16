//! Karp's minimum mean cycle (1978).
//!
//! For a directed graph with `n` vertices given as a flat, row-major `n × n`
//! weight matrix (`INF` = absent edge), the *minimum mean cycle* is
//!
//!   μ* = min over directed cycles C of (Σ_{e∈C} w_e) / |C|.
//!
//! Karp's theorem computes μ* through a min–max of the DP table
//!
//!   dp[k][v] = minimum weight of a walk of *exactly* `k` edges ending at `v`
//!              (walks may start at any vertex: `dp[0][v] = 0`),
//!
//! namely
//!
//!   μ* = min_{v} max_{0 ≤ k < n} (dp[n][v] − dp[k][v]) / (n − k).
//!
//! (The `dp[0][v] = 0` convention is equivalent to joining a super-source to
//! every vertex by a weight-0 edge; it makes every cycle reachable, so the
//! minimum is attained.)
//!
//! The minimum mean is in general a rational, so instead of dividing we return
//! the exact pair `(num, den)` with `num / den = μ*` and `den > 0`.  `Err(())`
//! is returned exactly when the graph has no cycle at all.

/// "Infinity" sentinel for absent edges and unreachable DP states.  Large
/// enough to be unattainable by any real walk, small enough that finite sums
/// cannot overflow `i64` for the modest weights used in the test suite.
const INF: i64 = 1_000_000_000_000_000_000;

/// Flat index for a row-major `n × n` matrix: element `(row, col)` lives at
/// `row * n + col`.
fn idx(row: usize, n: usize, col: usize) -> usize {
    let t = row * n;
    let r = t + col;
    r
}

/// Convert a `usize` to `i64`.  Used only for the small counters (`n`, `n - k`)
/// so that the fraction arithmetic stays in a single signed type.
fn usize_to_i64(x: usize) -> i64 {
    let mut r: i64 = 0;
    let mut i: usize = 0;
    while i < x {
        r = r + 1;
        i = i + 1;
    }
    r
}

/// Minimum mean cycle of a directed weighted graph (Karp's algorithm).
///
/// `weight` is a flat, row-major `n × n` matrix; `weight[idx(u, n, v)]` is the
/// weight of the directed edge `u → v`, or `INF` when there is no such edge.
/// Weights may be negative (a negative cycle is allowed); the matrix need not
/// be symmetric.
///
/// Returns `Ok((num, den))` with `num / den` the minimum cycle mean and
/// `den > 0`, or `Err(())` when the graph has no cycle.
pub fn min_mean_cycle(n: usize, weight: &[i64]) -> Result<(i64, i64), ()> {
    // dp is an (n + 1) × n row-major matrix.  Row 0 is all zeros
    // (`dp[0][v] = 0`); rows 1..=n are filled in by the DP below.
    let size = n * n + n;
    let mut dp: Vec<i64> = vec![0i64; size];

    // dp[k + 1][v] = min over u of ( dp[k][u] + weight[u][v] )
    let mut k: usize = 0;
    while k < n {
        let mut v: usize = 0;
        while v < n {
            let mut best: i64 = INF;
            let mut u: usize = 0;
            while u < n {
                let a = dp[idx(k, n, u)];
                let wuv = weight[idx(u, n, v)];
                if a < INF && wuv < INF {
                    let c = a + wuv;
                    if c < best {
                        best = c;
                    }
                }
                u += 1;
            }
            dp[idx(k + 1, n, v)] = best;
            v += 1;
        }
        k += 1;
    }

    // mu* = min_v max_{0 <= k < n} (dp[n][v] - dp[k][v]) / (n - k).
    // Fractions are compared by cross-multiplication; every denominator
    // `n - k` is positive, so the comparison is exact.
    let mut found: bool = false;
    let mut bnum: i64 = 0;
    let mut bden: i64 = 1;
    let mut v: usize = 0;
    while v < n {
        let dnv = dp[idx(n, n, v)];
        if dnv < INF {
            let mut have: bool = false;
            let mut mnum: i64 = 0;
            let mut mden: i64 = 1;
            let mut kk: usize = 0;
            let mut den: i64 = usize_to_i64(n);
            while kk < n {
                let dkv = dp[idx(kk, n, v)];
                if dkv < INF {
                    let num = dnv - dkv;
                    if !have {
                        mnum = num;
                        mden = den;
                        have = true;
                    } else {
                        // num / den  >  mnum / mden  <=>  num * mden > mnum * den
                        let l = num * mden;
                        let r = mnum * den;
                        if l > r {
                            mnum = num;
                            mden = den;
                        }
                    }
                }
                kk += 1;
                den -= 1;
            }
            if have {
                if !found {
                    bnum = mnum;
                    bden = mden;
                    found = true;
                } else {
                    // mnum / mden  <  bnum / bden  <=>  mnum * bden < bnum * mden
                    let l = mnum * bden;
                    let r = bnum * mden;
                    if l < r {
                        bnum = mnum;
                        bden = mden;
                    }
                }
            }
        }
        v += 1;
    }

    if found {
        Ok((bnum, bden))
    } else {
        Err(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Enumerate every simple directed cycle (distinct vertices, length ≥ 1)
    /// of the `n`-vertex graph given by the flat matrix `w`.
    /// Returns `(vertices, weight_sum, length)`.
    fn brute_cycles(n: usize, w: &[i64]) -> Vec<(Vec<usize>, i64, usize)> {
        let mut res: Vec<(Vec<usize>, i64, usize)> = Vec::new();
        let mut mask: usize = 1;
        while mask < (1usize << n) {
            let verts: Vec<usize> = (0..n).filter(|&i| (mask >> i) & 1 == 1).collect();
            let l = verts.len();
            // canonical cyclic order: smallest element first
            let mut rest: Vec<usize> = verts[1..].to_vec();
            rest.sort_unstable();
            permutations(&mut rest, 0, &mut |perm: &mut Vec<usize>| {
                let mut order: Vec<usize> = Vec::with_capacity(l);
                order.push(verts[0]);
                for &x in perm.iter() {
                    order.push(x);
                }
                let mut s: i64 = 0;
                let mut ok = true;
                for i in 0..l {
                    let u = order[i];
                    let v = order[(i + 1) % l];
                    let e = w[u * n + v];
                    if e >= INF {
                        ok = false;
                        break;
                    }
                    s += e;
                }
                if ok {
                    res.push((order, s, l));
                }
            });
            mask += 1;
        }
        res
    }

    /// In-place heap-free permutation walker (Heap's algorithm).
    fn permutations<F: FnMut(&mut Vec<usize>)>(a: &mut Vec<usize>, k: usize, f: &mut F) {
        if k == a.len() {
            f(a);
            return;
        }
        for i in k..a.len() {
            a.swap(k, i);
            permutations(a, k + 1, f);
            a.swap(k, i);
        }
    }

    /// Does `num / den` equal the minimum cycle mean, encoded exactly?
    fn spec_holds(num: i64, den: i64, cyc: &[(Vec<usize>, i64, usize)]) -> bool {
        if den <= 0 {
            return false;
        }
        let all_le = cyc
            .iter()
            .all(|(_, s, l)| num * (*l as i64) <= den * *s);
        let some_eq = cyc
            .iter()
            .any(|(_, s, l)| num * (*l as i64) == den * *s);
        all_le && some_eq
    }

    fn lcg(seed: &mut u64) -> u64 {
        *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *seed >> 33
    }

    #[test]
    fn differential_random_graphs() {
        let mut seed: u64 = 0x1234_5678_9abc_def0;
        for _ in 0..3000 {
            let n = 1 + (lcg(&mut seed) % 6) as usize; // 1..=6
            let thr = (lcg(&mut seed) % 100) as u64; // density threshold
            let mut w = vec![INF; n * n];
            for u in 0..n {
                for v in 0..n {
                    if (lcg(&mut seed) % 100) <= thr {
                        let mag = (lcg(&mut seed) % 13) as i64; // 0..=12
                        let sgn = (lcg(&mut seed) % 2) as i64;
                        w[u * n + v] = if sgn == 0 { mag - 5 } else { mag };
                    }
                }
            }
            let cyc = brute_cycles(n, &w);
            let alg = min_mean_cycle(n, &w);
            if cyc.is_empty() {
                assert!(alg.is_err(), "expected no cycle for w = {:?}", w);
            } else {
                match alg {
                    Ok((num, den)) => assert!(
                        spec_holds(num, den, &cyc),
                        "spec violated: w = {:?} num = {} den = {} cycles = {:?}",
                        w,
                        num,
                        den,
                        cyc
                    ),
                    Err(()) => panic!("expected a cycle for w = {:?}", w),
                }
            }
        }
    }

    #[test]
    fn known_small_cases() {
        // unit triangle 0->1->2->0 (diagonal = INF, i.e. no self-loop): mean 1
        let w = vec![INF, 1, INF, INF, INF, 1, 1, INF, INF];
        let (num, den) = min_mean_cycle(3, &w).unwrap();
        assert_eq!(num, den);

        // 3-cycle with weights 2, 3, 4 -> mean 3
        let w = vec![INF, 2, INF, INF, INF, 3, 4, INF, INF];
        let (num, den) = min_mean_cycle(3, &w).unwrap();
        assert_eq!(num, den * 3);

        // a zero-weight self-loop beats the unit triangle below: mean 0
        let w = vec![0i64, 1, 1, 1, 0, 1, 1, 1, 0];
        let (num, den) = min_mean_cycle(3, &w).unwrap();
        assert!(den > 0);
        assert_eq!(num, 0);

        // The only cycle is 1 -> 2 -> 1 (weights 3, 5): mean 4.  Karp's
        // correct min_v-max_k formula gives 4 here, whereas the (false)
        // min_k-max_v variant would give 11/3.
        let w = vec![INF, 1, 3, INF, INF, 3, INF, 5, INF];
        let (num, den) = min_mean_cycle(3, &w).unwrap();
        assert_eq!(num, den * 4);

        // directed 2-cycle 0->1 (5), 1->0 (1) -> mean 3
        let w = vec![INF, 5, 1, INF];
        let (num, den) = min_mean_cycle(2, &w).unwrap();
        assert_eq!(num, den * 3);

        // negative 3-cycle -> mean -1
        let w = vec![INF, -1, INF, INF, INF, -1, -1, INF, INF];
        let (num, den) = min_mean_cycle(3, &w).unwrap();
        assert_eq!(num, -den);

        // DAG 0->1->2 : no cycle
        let w = vec![INF, 1, INF, INF, INF, 1, INF, INF, INF];
        assert_eq!(min_mean_cycle(3, &w), Err(()));

        // single vertex with a positive self-loop -> mean 7
        let w = vec![7i64];
        assert_eq!(min_mean_cycle(1, &w), Ok((7, 1)));
    }
}
