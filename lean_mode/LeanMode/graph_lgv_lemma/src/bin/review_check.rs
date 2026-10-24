// Scratch brute-force checker for lgv_lemma spec review. DELETE AFTER USE.
use std::collections::HashSet;

fn det(m: &[i64], n: usize) -> i64 {
    if n == 0 { return 1; }
    if n == 1 { return m[0]; }
    let mut d = 0i64;
    for c in 0..n {
        let mut minor = Vec::with_capacity((n - 1) * (n - 1));
        for i in 1..n {
            for j in 0..n {
                if j == c { continue; }
                minor.push(m[i * n + j]);
            }
        }
        let cof = det(&minor, n - 1);
        let sign = if c % 2 == 0 { 1i64 } else { -1i64 };
        d = d.wrapping_add(sign.wrapping_mul(m[c].wrapping_mul(cof)));
    }
    d
}

fn paths(adj: &[i64], n: usize, u: usize, v: usize) -> Vec<Vec<usize>> {
    if u > v { return vec![]; }
    if u == v { return vec![]; }
    let mut result = vec![];
    let mut cur = vec![u];
    dfs(adj, n, v, &mut cur, &mut result);
    result
}

fn dfs(adj: &[i64], n: usize, v: usize, cur: &mut Vec<usize>, result: &mut Vec<Vec<usize>>) {
    let last = *cur.last().unwrap();
    if last == v {
        if cur.len() >= 2 { result.push(cur.clone()); }
        return;
    }
    for w in (last + 1)..=v {
        if adj[last * n + w] == 1 {
            cur.push(w);
            dfs(adj, n, v, cur, result);
            cur.pop();
        }
    }
}

fn sign(perm: &[usize]) -> i64 {
    let mut inv = 0usize;
    for i in 0..perm.len() { for j in (i + 1)..perm.len() { if perm[i] > perm[j] { inv += 1; } } }
    if inv % 2 == 0 { 1 } else { -1 }
}

fn permutations(k: usize) -> Vec<Vec<usize>> {
    let mut res = vec![];
    let mut cur = vec![];
    let mut used = vec![false; k];
    fn rec(k: usize, cur: &mut Vec<usize>, used: &mut Vec<bool>, res: &mut Vec<Vec<usize>>) {
        if cur.len() == k { res.push(cur.clone()); return; }
        for i in 0..k {
            if !used[i] {
                used[i] = true; cur.push(i); rec(k, cur, used, res); cur.pop(); used[i] = false;
            }
        }
    }
    rec(k, &mut cur, &mut used, &mut res);
    res
}

fn count_tuples(all_paths: &[Vec<Vec<usize>>], i: usize, chosen: &mut Vec<Vec<usize>>) -> i64 {
    if i == all_paths.len() { return 1; }
    let mut total = 0i64;
    for p in &all_paths[i] {
        let ps: HashSet<usize> = p.iter().cloned().collect();
        let disjoint = chosen.iter().all(|c| c.iter().all(|x| !ps.contains(x)));
        if disjoint {
            chosen.push(p.clone());
            total += count_tuples(all_paths, i + 1, chosen);
            chosen.pop();
        }
    }
    total
}

fn disjoint_family_count(adj: &[i64], n: usize, src: &[usize], snk: &[usize], sigma: &[usize]) -> i64 {
    let k = src.len();
    let all_paths: Vec<Vec<Vec<usize>>> = (0..k).map(|i| paths(adj, n, src[i], snk[sigma[i]])).collect();
    count_tuples(&all_paths, 0, &mut Vec::new())
}

fn e_matrix(adj: &[i64], n: usize, src: &[usize], snk: &[usize]) -> Vec<i64> {
    let k = src.len();
    let mut e = vec![0i64; k * k];
    for i in 0..k { for j in 0..k { e[i * k + j] = paths(adj, n, src[i], snk[j]).len() as i64; } }
    e
}

fn lgv_rhs(adj: &[i64], n: usize, src: &[usize], snk: &[usize]) -> i64 {
    let k = src.len();
    let mut total = 0i64;
    for sigma in permutations(k) {
        total += sign(&sigma) * disjoint_family_count(adj, n, src, snk, &sigma);
    }
    total
}

fn all_tuples(n: usize, k: usize) -> Vec<Vec<usize>> {
    let mut res = vec![];
    let mut cur = vec![];
    fn rec(n: usize, k: usize, cur: &mut Vec<usize>, res: &mut Vec<Vec<usize>>) {
        if cur.len() == k { res.push(cur.clone()); return; }
        for x in 0..n { cur.push(x); rec(n, k, cur, res); cur.pop(); }
    }
    rec(n, k, &mut cur, &mut res);
    res
}

fn is_forward(adj: &[i64], n: usize) -> bool {
    for u in 0..n { for v in 0..n { if adj[u * n + v] == 1 && !(u < v) { return false; } } }
    true
}

fn rng(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
    *seed >> 33
}

fn random_forward_mats(n: usize, count: usize, seed: &mut u64) -> Vec<Vec<i64>> {
    let mut out = vec![];
    for _ in 0..count {
        let mut adj = vec![0i64; n * n];
        for u in 0..n { for v in (u + 1)..n { adj[u * n + v] = (rng(seed) & 1) as i64; } }
        out.push(adj);
    }
    out
}

fn random_01_mats(n: usize, count: usize, seed: &mut u64) -> Vec<Vec<i64>> {
    let mut out = vec![];
    for _ in 0..count {
        let mut adj = vec![0i64; n * n];
        for b in adj.iter_mut() { *b = (rng(seed) & 1) as i64; }
        out.push(adj);
    }
    out
}

fn random_src_snk(n: usize, k: usize, seed: &mut u64) -> (Vec<usize>, Vec<usize>) {
    let src: Vec<usize> = (0..k).map(|_| (rng(seed) % n as u64) as usize).collect();
    let snk: Vec<usize> = (0..k).map(|_| (rng(seed) % n as u64) as usize).collect();
    (src, snk)
}

fn main() {
    let mut seed = 777u64;

    // 1) lgv_identity exhaustive n=2..4 (all 0/1 adj, all src/snk under h_disjoint)
    for n in 2..=4usize {
        let total_adj = 1usize << (n * n);
        for adjmask in 0..total_adj {
            let adj: Vec<i64> = (0..(n * n)).map(|b| ((adjmask >> b) & 1) as i64).collect();
            for k in 1..=3usize {
                let tuples = all_tuples(n, k);
                for src in &tuples {
                    for snk in &tuples {
                        if !(0..k).all(|i| (0..k).all(|j| src[i] != snk[j])) { continue; }
                        let lhs = det(&e_matrix(&adj, n, src, snk), k);
                        let rhs = lgv_rhs(&adj, n, src, snk);
                        if lhs != rhs {
                            println!("LGV_IDENTITY FAIL n={} k={} adj={:?} src={:?} snk={:?} det={} rhs={}", n, k, adj, src, snk, lhs, rhs);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        println!("lgv_identity exhaustive OK n={}", n);
    }

    // 2) lgv_identity random n=5..7 (arbitrary 0/1 adj), k=1..4, random src/snk under h_disjoint
    for n in 5..=7usize {
        let matss = random_01_mats(n, 3000, &mut seed);
        for k in 1..=4usize {
            for _ in 0..300 {
                let (src, snk) = random_src_snk(n, k, &mut seed);
                if !(0..k).all(|i| (0..k).all(|j| src[i] != snk[j])) { continue; }
                for adj in &matss {
                    let lhs = det(&e_matrix(adj, n, &src, &snk), k);
                    let rhs = lgv_rhs(adj, n, &src, &snk);
                    if lhs != rhs {
                        println!("LGV_IDENTITY RANDOM FAIL n={} k={} adj={:?} src={:?} snk={:?} det={} rhs={}", n, k, adj, src, snk, lhs, rhs);
                        std::process::exit(1);
                    }
                }
            }
        }
        println!("lgv_identity random OK n={}", n);
    }

    // 3) lgv_lemma_correct: exhaustive n=2,3; random n=4..7 forward, k=0..4, random src/snk under h_src_ne_snk
    for n in 2..=7usize {
        let samples: Vec<Vec<i64>> = if n <= 3 {
            let total = 1usize << (n * n);
            (0..total).map(|m| (0..(n * n)).map(|b| ((m >> b) & 1) as i64).collect()).collect()
        } else {
            random_forward_mats(n, 4000, &mut seed)
        };
        let k_range = if n <= 3 { 0..=3 } else { 0..=4 };
        for adj in &samples {
            for k in k_range.clone() {
                let (trials, use_all): (usize, bool) = if n <= 3 { (1, true) } else { (100, false) };
                if use_all {
                    let tuples = all_tuples(n, k);
                    for src in &tuples {
                        for snk in &tuples {
                            if !(0..k).all(|i| (0..k).all(|j| src[i] != snk[j])) { continue; }
                            let expected = det(&e_matrix(adj, n, src, snk), k);
                            let got = lgv_lemma::lgv_lemma(n, adj, k, src, snk);
                            if got != expected {
                                println!("LGV_LEMMA_CORRECT FAIL n={} k={} adj={:?} src={:?} snk={:?} got={} expected={}", n, k, adj, src, snk, got, expected);
                                std::process::exit(1);
                            }
                        }
                    }
                } else {
                    for _ in 0..trials {
                        let (src, snk) = random_src_snk(n, k, &mut seed);
                        if !(0..k).all(|i| (0..k).all(|j| src[i] != snk[j])) { continue; }
                        let expected = det(&e_matrix(adj, n, &src, &snk), k);
                        let got = lgv_lemma::lgv_lemma(n, adj, k, &src, &snk);
                        if got != expected {
                            println!("LGV_LEMMA_CORRECT FAIL n={} k={} adj={:?} src={:?} snk={:?} got={} expected={}", n, k, adj, src, snk, got, expected);
                            std::process::exit(1);
                        }
                    }
                }
            }
        }
        println!("lgv_lemma_correct OK n={} ({} samples)", n, samples.len());
    }

    // 4) det_int stress: reimplement and compare to recursive det on random int matrices.
    for n in 1..=6usize {
        for _ in 0..20000 {
            let mut m = vec![0i64; n * n];
            for x in m.iter_mut() { *x = (rng(&mut seed) % 9) as i64 - 4; }
            let a = det_int_reimpl(n, &m);
            let b = det(&m, n);
            if a != b {
                println!("DET_INT FAIL n={} m={:?} reimpl={} recursive={}", n, m, a, b);
                std::process::exit(1);
            }
        }
    }
    println!("det_int reimpl OK");

    println!("ALL CHECKS PASSED");
}

fn det_int_reimpl(n: usize, a: &[i64]) -> i64 {
    if n == 0 { return 1; }
    let mut m = a.to_vec();
    let mut swaps = 0usize;
    let mut prev = 1i64;
    let mut k = 0usize;
    while k + 1 < n {
        let mut r = n;
        let mut i = k;
        while i < n {
            if m[i * n + k] != 0 && i < r { r = i; }
            i += 1;
        }
        if r == n { return 0; }
        if r != k {
            for j in 0..n {
                let t = m[r * n + j]; m[r * n + j] = m[k * n + j]; m[k * n + j] = t;
            }
            swaps += 1;
        }
        let pivot = m[k * n + k];
        let mut i2 = k + 1;
        while i2 < n {
            let a_ik = m[i2 * n + k];
            let mut j = k + 1;
            while j < n {
                let a_ij = m[i2 * n + j];
                let a_kj = m[k * n + j];
                let num = a_ij * pivot - a_ik * a_kj;
                m[i2 * n + j] = num / prev;
                j += 1;
            }
            i2 += 1;
        }
        prev = pivot;
        k += 1;
    }
    let last = m[(n - 1) * n + (n - 1)];
    if swaps % 2 == 0 { last } else { 0 - last }
}
