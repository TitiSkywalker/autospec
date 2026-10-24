//! Temporary efficient randomized search for the GSAM review. DELETE ME.

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
        if occurs_in(&str_at(s, lens, k), p) {
            return true;
        }
    }
    false
}

fn state_of(g: &Gsam, w: &[usize]) -> usize {
    let mut v = 0usize;
    for &c in w {
        let nv = g.next[v * 256 + c];
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

fn all_substrings_of_concat(s: &[usize]) -> Vec<Vec<usize>> {
    let n = s.len();
    let mut out = Vec::new();
    for i in 0..n {
        for l in 1..=(n - i) {
            out.push(s[i..i + l].to_vec());
        }
    }
    out
}

fn all_substrings_of_strings(s: &[usize], lens: &[usize]) -> Vec<Vec<usize>> {
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

fn lcg(state: &mut u64) -> u64 {
    let a = 6364136223846793005u64;
    let c = 1442695040888963407u64;
    let t = state.wrapping_mul(a).wrapping_add(c);
    *state = t;
    t
}

#[test]
fn randomized_efficient() {
    let mut state = 20241022u64;
    let mut trial = 0u64;
    let mut bad_total = 0usize;
    while trial < 20000 {
        let alphabet = 2 + (lcg(&mut state) % 4) as usize; // 2..=5
        let kc = 1 + (lcg(&mut state) % 4) as usize; // 1..=4 strings
        let mut strings: Vec<Vec<usize>> = Vec::new();
        for _ in 0..kc {
            let l = (lcg(&mut state) % 12) as usize; // 0..=11
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

        // Acceptance: every substring of the naive concatenation s must be
        // accepted iff it is a substring of *some individual* string.  This is
        // complete for both directions (soundness + completeness).
        for w in all_substrings_of_concat(&s) {
            let got = gsam_contains(&g, &w);
            let want = in_some(&s, &lens, &w);
            if got != want {
                eprintln!(
                    "ACCEPT MISMATCH strings={:?} word={:?} got={} want={}",
                    strings, w, got, want
                );
                bad_total += 1;
                if bad_total > 20 {
                    return;
                }
            }
        }
        // empty pattern
        if !gsam_contains(&g, &[]) {
            eprintln!("EMPTY MISMATCH strings={:?}", strings);
            bad_total += 1;
        }

        // Endpos classes over all pairs of substrings of the individual strings.
        let ss = all_substrings_of_strings(&s, &lens);
        for i in 0..ss.len() {
            for j in 0..ss.len() {
                let w1 = &ss[i];
                let w2 = &ss[j];
                let same_state = state_of(&g, w1) == state_of(&g, w2);
                let ep = same_endpos(&endpos(&s, &lens, w1), &endpos(&s, &lens, w2));
                if same_state != ep {
                    eprintln!(
                        "ENDPOS MISMATCH strings={:?} w1={:?} w2={:?} same_state={} same_endpos={}",
                        strings, w1, w2, same_state, ep
                    );
                    bad_total += 1;
                    if bad_total > 20 {
                        return;
                    }
                }
            }
        }

        if g.sz > 2 * s.len() + 1 {
            eprintln!("SIZE strings={:?} sz={} bound={}", strings, g.sz, 2 * s.len() + 1);
            bad_total += 1;
        }

        trial += 1;
    }
    eprintln!("randomized_efficient done, {} trials, {} bad", trial, bad_total);
}
