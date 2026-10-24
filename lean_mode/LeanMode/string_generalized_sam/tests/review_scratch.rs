//! Temporary exhaustive search for the GSAM review. DELETE ME.

use string_generalized_sam::{gsam_build, gsam_contains, Gsam};

fn str_at(s: &[usize], lens: &[usize], k: usize) -> Vec<usize> {
    if k >= lens.len() {
        return Vec::new();
    }
    let mut base = 0;
    for i in 0..k {
        base += lens[i];
    }
    let l = lens[k];
    let mut out = Vec::new();
    for j in 0..l {
        out.push(s[base + j]);
    }
    out
}

fn occurs_in(t: &[usize], p: &[usize]) -> bool {
    let n = t.len();
    let m = p.len();
    if m > n {
        return false;
    }
    for i in 0..=(n - m) {
        if t[i..i + m] == *p {
            return true;
        }
    }
    false
}

fn in_some(s: &[usize], lens: &[usize], p: &[usize]) -> bool {
    for k in 0..lens.len() {
        let t = str_at(s, lens, k);
        if occurs_in(&t, p) {
            return true;
        }
    }
    false
}

fn state_of(g: &Gsam, w: &[usize]) -> usize {
    let mut v = 0usize;
    for &c in w {
        let idx = v * 256 + c;
        let nv = g.next[idx];
        if nv == 0 {
            return usize::MAX;
        }
        v = nv;
    }
    v
}

fn endpos(s: &[usize], lens: &[usize], p: &[usize]) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    if p.is_empty() {
        return out;
    }
    for k in 0..lens.len() {
        let t = str_at(s, lens, k);
        let n = t.len();
        let m = p.len();
        if m <= n {
            for j in (m - 1)..n {
                if t[j + 1 - m..j + 1] == *p {
                    out.push((k, j));
                }
            }
        }
    }
    out
}

fn same_endpos(a: &[(usize, usize)], b: &[(usize, usize)]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for x in a {
        if !b.contains(x) {
            return false;
        }
    }
    true
}

fn all_substrings(s: &[usize], lens: &[usize]) -> Vec<Vec<usize>> {
    let mut out = Vec::new();
    for k in 0..lens.len() {
        let t = str_at(s, lens, k);
        let n = t.len();
        for i in 0..n {
            for l in 1..=(n - i) {
                out.push(t[i..i + l].to_vec());
            }
        }
    }
    out
}

fn gen_words(alphabet: usize, maxlen: usize) -> Vec<Vec<usize>> {
    let mut out = vec![Vec::new()];
    let mut frontier = vec![Vec::new()];
    for _ in 0..maxlen {
        let mut next = Vec::new();
        for w in &frontier {
            for c in 0..alphabet {
                let mut w2 = w.clone();
                w2.push(c);
                next.push(w2);
            }
        }
        out.extend(next.iter().cloned());
        frontier = next;
    }
    out
}

fn full_check(strings: &[Vec<usize>], alphabet: usize, verbose: bool) -> usize {
    let mut s = Vec::new();
    let mut lens = Vec::new();
    for t in strings {
        s.extend_from_slice(t);
        lens.push(t.len());
    }
    let g = gsam_build(&s, &lens);
    let mut bad = 0;

    // acceptance
    for w in gen_words(alphabet, 7) {
        let got = gsam_contains(&g, &w);
        let want = w.is_empty() || in_some(&s, &lens, &w);
        if got != want {
            if verbose {
                eprintln!(
                    "ACCEPT MISMATCH strings={:?} word={:?} got={} want={}",
                    strings, w, got, want
                );
            }
            bad += 1;
        }
    }

    // endpos classes over all nonempty substrings
    let ss = all_substrings(&s, &lens);
    for i in 0..ss.len() {
        for j in 0..ss.len() {
            let w1 = &ss[i];
            let w2 = &ss[j];
            let same_state = state_of(&g, w1) == state_of(&g, w2);
            let ep = same_endpos(&endpos(&s, &lens, w1), &endpos(&s, &lens, w2));
            if same_state != ep {
                if verbose {
                    eprintln!(
                        "ENDPOS MISMATCH strings={:?} w1={:?} w2={:?} same_state={} same_endpos={}",
                        strings, w1, w2, same_state, ep
                    );
                }
                bad += 1;
            }
        }
    }

    // size bound
    if g.sz > 2 * s.len() + 1 {
        if verbose {
            eprintln!("SIZE strings={:?} sz={} bound={}", strings, g.sz, 2 * s.len() + 1);
        }
        bad += 1;
    }

    bad
}

fn enumerate_string_sets(alphabet: usize, max_strings: usize, max_len: usize, verbose: bool) {
    // generate all strings over alphabet of length 0..=max_len
    let mut all: Vec<Vec<usize>> = Vec::new();
    for l in 0..=max_len {
        all.extend(gen_words(alphabet, l).into_iter().filter(|w| w.len() == l));
    }
    // enumerate sets of up to max_strings strings (unordered, with repetition)
    let n = all.len();
    let mut checked = 0usize;
    let mut bad_total = 0usize;

    // single string
    for a in &all {
        checked += 1;
        bad_total += full_check(&[a.clone()], alphabet, verbose);
    }
    // two strings
    for i in 0..n {
        for j in i..n {
            checked += 1;
            bad_total += full_check(&[all[i].clone(), all[j].clone()], alphabet, verbose);
        }
    }
    // three strings
    if max_strings >= 3 {
        for i in 0..n {
            for j in i..n {
                for k in j..n {
                    checked += 1;
                    bad_total += full_check(&[all[i].clone(), all[j].clone(), all[k].clone()], alphabet, verbose);
                }
            }
        }
    }
    eprintln!("checked {} sets, {} bad", checked, bad_total);
}

#[test]
fn exhaustive_binary() {
    // alphabet {0,1}, up to 3 strings, length up to 5
    enumerate_string_sets(2, 3, 5, false);
}

#[test]
fn exhaustive_ternary_small() {
    // alphabet {0,1,2}, up to 2 strings, length up to 4
    enumerate_string_sets(3, 2, 4, false);
}

#[test]
fn targeted() {
    // print details on any failure for a few tricky families
    let mut verbose = true;
    // runs of same char split across strings
    full_check(&[vec![0,0], vec![0,0,0]], 2, verbose);
    full_check(&[vec![0,0,0], vec![0,0]], 2, verbose);
    full_check(&[vec![0,1], vec![0,1,0]], 2, verbose);
    full_check(&[vec![0,1], vec![1,0]], 2, verbose);
    full_check(&[vec![0,1,0], vec![1,0,1]], 2, verbose);
    full_check(&[vec![0,1,2], vec![1,2,0]], 3, verbose);
    full_check(&[vec![0,1,2], vec![2,1,0]], 3, verbose);
}

fn lcg(state: &mut u64) -> u64 {
    let a = 6364136223846793005u64;
    let c = 1442695040888963407u64;
    let t = state.wrapping_mul(a).wrapping_add(c);
    *state = t;
    t
}

#[test]
fn randomized_long() {
    let mut state = 20241022u64;
    let mut trial = 0u64;
    let mut bad_total = 0usize;
    while trial < 20000 {
        let alphabet = 2 + (lcg(&mut state) % 4) as usize; // 2..=5
        let kc = 1 + (lcg(&mut state) % 4) as usize; // 1..=4 strings
        let mut strings: Vec<Vec<usize>> = Vec::new();
        for _ in 0..kc {
            let l = (lcg(&mut state) % 16) as usize; // 0..=15
            let mut t = Vec::new();
            for _ in 0..l {
                t.push((lcg(&mut state) % alphabet as u64) as usize);
            }
            strings.push(t);
        }
        let mut s = Vec::new();
        let mut lens = Vec::new();
        for t in &strings {
            s.extend_from_slice(t);
            lens.push(t.len());
        }
        let g = gsam_build(&s, &lens);

        // acceptance over all words up to length 9 over the alphabet (bounded)
        let words = gen_words(alphabet, 9);
        for w in &words {
            let got = gsam_contains(&g, w);
            let want = w.is_empty() || in_some(&s, &lens, w);
            if got != want {
                eprintln!("ACCEPT MISMATCH strings={:?} word={:?} got={} want={}", strings, w, got, want);
                bad_total += 1;
                if bad_total > 20 { return; }
            }
        }

        // endpos classes over all nonempty substrings
        let ss = all_substrings(&s, &lens);
        for i in 0..ss.len() {
            for j in 0..ss.len() {
                let w1 = &ss[i];
                let w2 = &ss[j];
                let same_state = state_of(&g, w1) == state_of(&g, w2);
                let ep = same_endpos(&endpos(&s, &lens, w1), &endpos(&s, &lens, w2));
                if same_state != ep {
                    eprintln!("ENDPOS MISMATCH strings={:?} w1={:?} w2={:?} same_state={} same_endpos={}", strings, w1, w2, same_state, ep);
                    bad_total += 1;
                    if bad_total > 20 { return; }
                }
            }
        }

        if g.sz > 2 * s.len() + 1 {
            eprintln!("SIZE strings={:?} sz={} bound={}", strings, g.sz, 2 * s.len() + 1);
            bad_total += 1;
        }

        trial += 1;
    }
    eprintln!("randomized_long done, {} trials, {} bad", trial, bad_total);
}
