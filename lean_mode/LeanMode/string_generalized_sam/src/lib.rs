//! Generalized suffix automaton (GSAM) of a *set* of `usize`-encoded strings.
//!
//! Characters are `usize` byte values (`0..=255`).  The automaton is stored as
//! flat arrays exactly like the single-string suffix automaton:
//!
//! * `next[v * ALPHA + c]` is the transition from state `v` on character `c`
//!   (`0` means "no transition").  State `0` is the initial state (the empty
//!   string) and is never the target of a transition, so `0` doubles as the
//!   "absent" marker.
//! * `len[v]` is the length of the longest string of the state's `endpos`
//!   equivalence class; `link[v]` is the suffix link.
//! * `sz` is the number of states.
//!
//! `gsam_build s lens` reads the `k`-th input string from the concatenation `s`
//! as `s[Σ_{j<k} lens[j] .. Σ_{j≤k} lens[j])`.  Each string is inserted with the
//! ordinary SAM *extend* step, but `last` is **reset to the root (`0`) before
//! every string**.  That reset is what keeps the automaton from reporting
//! matches that span two strings: the accepted language is exactly the union of
//! the substring sets of the individual strings, never a substring of the naive
//! concatenation.
//!
//! The reset-`last` construction creates at most one `cur` and one `clone` per
//! inserted character, so `sz ≤ 2 * |s| + 1` (the size bound checked by the
//! specification), and `max_states = 2 * |s| + 1` is exactly enough room.
//!
//! `gsam_contains g p` walks the transitions from the root reading `p`.

const ALPHA: usize = 256;

/// A generalized suffix automaton of a set of strings.
pub struct Gsam {
    pub next: Vec<usize>,
    pub len: Vec<usize>,
    pub link: Vec<usize>,
    pub sz: usize,
}

/// Flat index for the transition table `next[v * alpha + c]`.
fn go_idx(v: usize, alpha: usize, c: usize) -> usize {
    let t = v * alpha;
    let r = t + c;
    r
}

/// `a + b`, routed through a helper so both operands are call arguments (Aeneas
/// only tracks `usize`-ness of parameters and literals).
fn add_usize(a: usize, b: usize) -> usize {
    let r = a + b;
    r
}

/// The distinct characters of `s`, in order of first occurrence.  Every
/// transition written by `gsam_build` is labelled with a character of `s`, so
/// these are the only columns of the table that are ever non-zero.
fn chars_used(s: &[usize]) -> Vec<usize> {
    let n = s.len();
    let mut a: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        let c = s[i];
        let mut seen = false;
        let mut k = 0;
        while k < a.len() {
            let d = a[k];
            if d == c {
                seen = true;
            }
            k += 1;
        }
        if !seen {
            a.push(c);
        }
        i += 1;
    }
    a
}

/// Build the generalized suffix automaton of the strings encoded in `s`/`lens`.
pub fn gsam_build(s: &[usize], lens: &[usize]) -> Gsam {
    let n = s.len();
    let alpha = ALPHA;
    let t2 = 2 * n;
    let max_states = t2 + 1;
    let table_len = max_states * alpha;
    let mut next: Vec<usize> = vec![0usize; table_len];
    let mut len: Vec<usize> = vec![0usize; max_states];
    let mut link: Vec<usize> = vec![0usize; max_states];

    // The characters of `s`; only these columns of `next` can ever be non-zero,
    // so a clone only needs to copy those columns.  (This is what keeps the
    // generated Lean evaluable on the `256`-wide table.)
    let chars = chars_used(s);
    let clen = chars.len();

    // State 0 is the root (the empty string); `link[0]` is a self-loop used only
    // as a terminator for the suffix-link walks.
    let mut sz: usize = 1;

    let nc = lens.len();
    let mut k: usize = 0;
    let mut base: usize = 0;
    while k < nc {
        let wl = lens[k];
        // Reset `last` to the root: this string gets its own automaton "run".
        let mut last: usize = 0;
        let mut i: usize = 0;
        while i < wl {
            let pos = add_usize(base, i);
            let c = s[pos];
            let cur = sz;
            sz += 1;
            let llast = len[last];
            let llast1 = llast + 1;
            len[cur] = llast1;

            // Add missing transitions along the suffix-link chain; stop at the
            // first state that already has a transition on `c` (or at the root).
            let mut p = last;
            let mut q = 0;
            let mut done = false;
            while !done {
                let idx = go_idx(p, alpha, c);
                let nv = next[idx];
                if nv == 0 {
                    next[idx] = cur;
                    if p == 0 {
                        link[cur] = 0;
                        done = true;
                    } else {
                        let np = link[p];
                        p = np;
                    }
                } else {
                    q = nv;
                    done = true;
                }
            }

            if q != 0 {
                let lp = len[p];
                let lp1 = lp + 1;
                let lq = len[q];
                if lp1 == lq {
                    // Continuous transition.
                    link[cur] = q;
                } else {
                    // Split `q` into a clone state.
                    let clone = sz;
                    sz += 1;
                    len[clone] = lp1;

                    // Copy every non-zero transition of `q` into `clone`.
                    let mut j: usize = 0;
                    while j < clen {
                        let cc = chars[j];
                        let qi = go_idx(q, alpha, cc);
                        let tv = next[qi];
                        let ci = go_idx(clone, alpha, cc);
                        next[ci] = tv;
                        j += 1;
                    }
                    let lq_link = link[q];
                    link[clone] = lq_link;

                    // Redirect transitions pointing to `q` up the suffix links.
                    let mut r = p;
                    let mut redir_done = false;
                    while !redir_done {
                        let idx = go_idx(r, alpha, c);
                        let nv = next[idx];
                        if nv == q {
                            next[idx] = clone;
                            if r == 0 {
                                redir_done = true;
                            } else {
                                let nr = link[r];
                                r = nr;
                            }
                        } else {
                            redir_done = true;
                        }
                    }
                    link[q] = clone;
                    link[cur] = clone;
                }
            }

            last = cur;
            i += 1;
        }
        let nb = add_usize(base, wl);
        base = nb;
        k += 1;
    }

    Gsam { next, len, link, sz }
}

/// Return whether `p` is a substring of *some* of the input strings, by walking
/// the transitions from the initial state.
pub fn gsam_contains(g: &Gsam, p: &[usize]) -> bool {
    let alpha = ALPHA;
    let n = p.len();
    let mut v = 0;
    let mut i = 0;
    let mut ok = true;
    while i < n && ok {
        let c = p[i];
        let idx = go_idx(v, alpha, c);
        let nv = g.next[idx];
        if nv == 0 {
            ok = false;
        } else {
            v = nv;
        }
        i += 1;
    }
    ok
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A tiny deterministic LCG, so the fuzzing below is reproducible without a
    /// dependency.
    fn lcg(state: &mut usize) -> usize {
        let a = 6364136223846793005usize;
        let c = 1442695040888963407usize;
        let t = state.wrapping_mul(a);
        let r = t.wrapping_add(c);
        *state = r;
        r
    }

    /// The `k`-th string encoded in `s`/`lens`.
    fn str_of(s: &[usize], lens: &[usize], k: usize) -> Vec<usize> {
        let c = lens.len();
        if k >= c {
            return Vec::new();
        }
        let mut base: usize = 0;
        let mut i: usize = 0;
        while i < k {
            let l = lens[i];
            let nb = base + l;
            base = nb;
            i += 1;
        }
        let l = lens[k];
        let mut out: Vec<usize> = Vec::new();
        let mut j: usize = 0;
        while j < l {
            let p = base + j;
            out.push(s[p]);
            j += 1;
        }
        out
    }

    /// Brute force: does `p` occur as a window of the `k`-th string?
    fn occurs_in(t: &[usize], p: &[usize]) -> bool {
        let n = t.len();
        let m = p.len();
        if m > n {
            return false;
        }
        let mut i = 0;
        while i + m <= n {
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
                return true;
            }
            i += 1;
        }
        false
    }

    /// Brute force: is `p` a substring of some input string?
    fn in_some(s: &[usize], lens: &[usize], p: &[usize]) -> bool {
        let c = lens.len();
        let mut k = 0;
        while k < c {
            let t = str_of(s, lens, k);
            if occurs_in(&t, p) {
                return true;
            }
            k += 1;
        }
        false
    }

    /// The end position set of `p`: the pairs `(k, j)` such that `p` ends at
    /// index `j` of the `k`-th string.
    fn endpos(s: &[usize], lens: &[usize], p: &[usize]) -> Vec<(usize, usize)> {
        let mut out: Vec<(usize, usize)> = Vec::new();
        if p.is_empty() {
            return out;
        }
        let c = lens.len();
        let mut k = 0;
        while k < c {
            let t = str_of(s, lens, k);
            let n = t.len();
            let m = p.len();
            if m <= n {
                let mut j = m - 1;
                while j < n {
                    let mut d = 0;
                    let mut same = true;
                    while d < m {
                        let x = t[j + 1 - m + d];
                        let y = p[d];
                        if x != y {
                            same = false;
                        }
                        d += 1;
                    }
                    if same {
                        out.push((k, j));
                    }
                    j += 1;
                }
            }
            k += 1;
        }
        out
    }

    /// The state reached from the root by the word `w` (`usize::MAX` = absent).
    fn state_of(g: &Gsam, w: &[usize]) -> usize {
        let mut v: usize = 0;
        let mut i: usize = 0;
        while i < w.len() {
            let c = w[i];
            let idx = v * 256 + c;
            let nv = g.next[idx];
            if nv == 0 {
                return usize::MAX;
            }
            v = nv;
            i += 1;
        }
        v
    }

    /// Build from a list of strings (concatenating them) and return `(s, lens)`.
    fn flatten(strings: &[Vec<usize>]) -> (Vec<usize>, Vec<usize>) {
        let mut s: Vec<usize> = Vec::new();
        let mut lens: Vec<usize> = Vec::new();
        let mut k = 0;
        while k < strings.len() {
            let t = &strings[k];
            let mut j = 0;
            while j < t.len() {
                s.push(t[j]);
                j += 1;
            }
            lens.push(t.len());
            k += 1;
        }
        (s, lens)
    }

    /// All nonempty substrings (as start/length pairs) of the input strings.
    fn all_substrings(s: &[usize], lens: &[usize]) -> Vec<Vec<usize>> {
        let mut out: Vec<Vec<usize>> = Vec::new();
        let c = lens.len();
        let mut k = 0;
        while k < c {
            let t = str_of(s, lens, k);
            let n = t.len();
            let mut i = 0;
            while i < n {
                let mut l = 1;
                while i + l <= n {
                    let mut w: Vec<usize> = Vec::new();
                    let mut d = 0;
                    while d < l {
                        w.push(t[i + d]);
                        d += 1;
                    }
                    out.push(w);
                    l += 1;
                }
                i += 1;
            }
            k += 1;
        }
        out
    }

    /// The endpos sets of two words coincide; used to compare with state
    /// equality (the endpos-class theorem).
    fn same_endpos(a: &Vec<(usize, usize)>, b: &Vec<(usize, usize)>) -> bool {
        if a.len() != b.len() {
            return false;
        }
        let mut i = 0;
        while i < a.len() {
            let x = a[i];
            let mut found = false;
            let mut j = 0;
            while j < b.len() {
                let y = b[j];
                if x.0 == y.0 && x.1 == y.1 {
                    found = true;
                }
                j += 1;
            }
            if !found {
                return false;
            }
            i += 1;
        }
        true
    }

    /// One full differential check of `gsam_build`/`gsam_contains` against brute
    /// force, plus the two research-level claims (endpos classes, size bound).
    /// Returns a list of failure descriptions.
    fn full_check(strings: &[Vec<usize>], alphabet: usize) -> Vec<&'static str> {
        let mut bad: Vec<&'static str> = Vec::new();
        let (s, lens) = flatten(strings);
        let g = gsam_build(&s, &lens);

        // (1) acceptance agrees with brute force on every word over the
        // alphabet of length 0..=3 (and on every substring, which is longer).
        let subs0 = all_substrings(&s, &lens);
        let mut subs = subs0;
        // add short words over the alphabet
        let mut l = 0;
        while l <= 3 {
            let total = pow_small(alphabet, l);
            let mut code = 0;
            while code < total {
                let mut word: Vec<usize> = Vec::new();
                let mut x = code;
                let mut d = 0;
                while d < l {
                    let digit = x - (x / alphabet) * alphabet;
                    word.push(digit);
                    x = x / alphabet;
                    d += 1;
                }
                subs.push(word);
                code += 1;
            }
            l += 1;
        }
        let mut i = 0;
        while i < subs.len() {
            let p = &subs[i];
            let got = gsam_contains(&g, p);
            let want = p.is_empty() || in_some(&s, &lens, p);
            if got != want {
                bad.push("acceptance");
            }
            i += 1;
        }

        // (2) endpos classes: two nonempty substrings reach the same state iff
        // they have the same end position set.
        let ss = all_substrings(&s, &lens);
        let mut i = 0;
        while i < ss.len() {
            let mut j = i;
            while j < ss.len() {
                let w1 = &ss[i];
                let w2 = &ss[j];
                let same_state = state_of(&g, w1) == state_of(&g, w2);
                let ep = same_endpos(&endpos(&s, &lens, w1), &endpos(&s, &lens, w2));
                if same_state != ep {
                    bad.push("endpos");
                }
                j += 1;
            }
            i += 1;
        }

        // (3) size bound
        let bound = 2 * s.len() + 1;
        if g.sz > bound {
            bad.push("size");
        }
        bad
    }

    fn pow_small(a: usize, e: usize) -> usize {
        let mut r = 1usize;
        let mut i = 0;
        while i < e {
            r = r * a;
            i += 1;
        }
        r
    }

    #[test]
    fn unit_basic_examples() {
        // "ab" and "bc": "b" is shared, "abc" spans the two strings.
        let strings = vec![vec![97, 98], vec![98, 99]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        let ab = vec![97, 98];
        let bc = vec![98, 99];
        let abc = vec![97, 98, 99];
        let ac = vec![97, 99];
        let b = vec![98];
        assert!(gsam_contains(&g, &ab));
        assert!(gsam_contains(&g, &bc));
        assert!(gsam_contains(&g, &b));
        assert!(!gsam_contains(&g, &abc));
        assert!(!gsam_contains(&g, &ac));
        assert!(gsam_contains(&g, &[]));
        assert!(full_check(&strings, 3).is_empty());
    }

    #[test]
    fn unit_shared_prefix() {
        // Both strings start with "ab": the reset-last construction shares the
        // "ab" path, so a duplicate `cur` state (unreachable) is created.
        let strings = vec![vec![97, 98], vec![97, 98, 97]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        assert!(gsam_contains(&g, &[97, 98, 97]));
        assert!(gsam_contains(&g, &[98, 97]));
        assert!(!gsam_contains(&g, &[97, 97]));
        assert!(!gsam_contains(&g, &[98, 98]));
        assert!(full_check(&strings, 2).is_empty());
    }

    #[test]
    fn unit_boundary_must_not_match() {
        // "a" | "b": "ab" crosses the boundary and must be rejected.
        let strings = vec![vec![97], vec![98]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        assert!(gsam_contains(&g, &[97]));
        assert!(gsam_contains(&g, &[98]));
        assert!(!gsam_contains(&g, &[97, 98]));
        assert!(!gsam_contains(&g, &[98, 97]));
        assert_eq!(g.sz, 3);
        assert!(full_check(&strings, 2).is_empty());
    }

    #[test]
    fn unit_no_crossing_longer() {
        // "abc" | "bc" | "cd": "abcd" / "bcd" / "cabc" all span boundaries.
        let strings = vec![vec![97, 98, 99], vec![98, 99], vec![99, 100]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        assert!(gsam_contains(&g, &[97, 98, 99]));
        assert!(gsam_contains(&g, &[99, 100]));
        assert!(gsam_contains(&g, &[98, 99]));
        assert!(!gsam_contains(&g, &[97, 98, 99, 100]));
        assert!(!gsam_contains(&g, &[99, 97]));
        assert!(!gsam_contains(&g, &[100, 97]));
        assert!(full_check(&strings, 4).is_empty());
    }

    #[test]
    fn unit_size_bound_attached_to_concatenation() {
        // The automaton must be built from the *individual* strings: only the
        // span `s` is read, through `lens`.
        let strings = vec![vec![97, 97, 97], vec![98, 98, 98]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        assert!(gsam_contains(&g, &[97, 97, 97]));
        assert!(gsam_contains(&g, &[98, 98, 98]));
        assert!(!gsam_contains(&g, &[97, 97, 97, 98]));
        assert!(!gsam_contains(&g, &[97, 97, 98]));
        assert!(g.sz <= 2 * s.len() + 1);
        assert!(full_check(&strings, 2).is_empty());
    }

    #[test]
    fn fuzz_small_random() {
        let mut state = 20241022usize;
        let mut trial = 0;
        while trial < 400 {
            let alphabet = 2 + lcg(&mut state) % 3; // 2..=4
            let kc = 1 + lcg(&mut state) % 3; // 1..=3 strings
            let mut strings: Vec<Vec<usize>> = Vec::new();
            let mut k = 0;
            while k < kc {
                let l = 1 + lcg(&mut state) % 4; // 1..=4
                let mut t: Vec<usize> = Vec::new();
                let mut j = 0;
                while j < l {
                    let c = lcg(&mut state) % alphabet;
                    t.push(c);
                    j += 1;
                }
                strings.push(t);
                k += 1;
            }
            let bad = full_check(&strings, alphabet);
            assert!(bad.is_empty());
            trial += 1;
        }
    }

    #[test]
    fn fuzz_binary_strings() {
        // Alphabet {0,1}: the densest sharing, hence the most clones.
        let mut state = 777usize;
        let mut trial = 0;
        while trial < 400 {
            let kc = 1 + lcg(&mut state) % 2;
            let mut strings: Vec<Vec<usize>> = Vec::new();
            let mut k = 0;
            while k < kc {
                let l = 1 + lcg(&mut state) % 6;
                let mut t: Vec<usize> = Vec::new();
                let mut j = 0;
                while j < l {
                    let c = lcg(&mut state) % 2;
                    t.push(c);
                    j += 1;
                }
                strings.push(t);
                k += 1;
            }
            assert!(full_check(&strings, 2).is_empty());
            trial += 1;
        }
    }

    #[test]
    fn fuzz_empty_string_in_set() {
        // A zero-length member contributes nothing but must not break anything.
        let strings = vec![vec![97, 98], Vec::new(), vec![98, 97]];
        let (s, lens) = flatten(&strings);
        let g = gsam_build(&s, &lens);
        assert!(gsam_contains(&g, &[97, 98]));
        assert!(gsam_contains(&g, &[98, 97]));
        assert!(!gsam_contains(&g, &[97, 98, 97]));
        assert!(full_check(&strings, 3).is_empty());
        // also with the empty string alone
        let strings2 = vec![Vec::new(), vec![97]];
        let (s2, lens2) = flatten(&strings2);
        let g2 = gsam_build(&s2, &lens2);
        assert!(gsam_contains(&g2, &[97]));
        assert!(!gsam_contains(&g2, &[98]));
        assert!(full_check(&strings2, 2).is_empty());
    }

    #[test]
    fn max_states_never_exceeded() {
        // The input to `full_check` is randomised, and `gsam_build` allocates
        // exactly `2 * s.len() + 1` states: make sure the bound is never hit
        // (an out-of-bounds write would panic).
        let mut state = 99991usize;
        let mut worst = 0usize;
        let mut trial = 0;
        while trial < 2000 {
            let kc = 1 + lcg(&mut state) % 4;
            let mut strings: Vec<Vec<usize>> = Vec::new();
            let mut k = 0;
            while k < kc {
                let l = lcg(&mut state) % 5;
                let mut t: Vec<usize> = Vec::new();
                let mut j = 0;
                while j < l {
                    let c = lcg(&mut state) % 2;
                    t.push(c);
                    j += 1;
                }
                strings.push(t);
                k += 1;
            }
            let (s, lens) = flatten(&strings);
            let g = gsam_build(&s, &lens);
            // `gsam_build` allocates exactly `2 * s.len() + 1` states, so this
            // is also the "no out-of-bounds state index" check.
            assert!(g.sz <= 2 * s.len() + 1);
            if g.sz > worst {
                worst = g.sz;
            }
            trial += 1;
        }
        // Recorded for the report: the largest state count seen.
        assert!(worst <= 2 * 5 * 4 + 1);
    }
}
