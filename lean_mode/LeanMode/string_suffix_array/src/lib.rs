//! Suffix array construction by prefix doubling (with counting sorts),
//! followed by the Kasai linear-time LCP array.
//!
//! Strings are `usize`-encoded byte values in `0..=255`.  No `as` casts are
//! used, so the only numeric model involved is `usize` (→ `Std.Usize`).
//!
//! Every buffer is flat and all index arithmetic is routed through small
//! helpers whose parameters are `usize`, so that `row * n + col` never appears
//! inline in a slice read or write.

/// The suffix array of a string together with its LCP array.
pub struct SaResult {
    /// `sa[i]` is the start index of the `i`-th lexicographically smallest
    /// suffix.
    pub sa: Vec<usize>,
    /// `lcp[i]` is the length of the longest common prefix of the suffixes
    /// starting at `sa[i]` and `sa[i + 1]`; `lcp[n - 1] = 0`.
    pub lcp: Vec<usize>,
}

/// Boolean conjunction folded through a call (avoids long `&&` chains).
fn and2(a: bool, b: bool) -> bool {
    if a {
        b
    } else {
        false
    }
}

/// Boolean disjunction folded through a call (avoids long `||` chains).
fn or2(a: bool, b: bool) -> bool {
    if a {
        true
    } else {
        b
    }
}

/// `min` of two `usize` values.
fn minu(a: usize, b: usize) -> usize {
    if a < b {
        a
    } else {
        b
    }
}

/// Stable counting sort of the indices `0..n` by `key[i]`, where every key
/// lies in `0..=maxkey`.  Returns the sorted index list.
fn cs_indices(n: usize, key: &[usize], maxkey: usize) -> Vec<usize> {
    let m = maxkey + 1;
    let mut cnt: Vec<usize> = vec![0usize; m];
    let mut i = 0;
    while i < n {
        let k = key[i];
        let c = cnt[k];
        let c1 = c + 1;
        cnt[k] = c1;
        i += 1;
    }
    // Turn the counts into start offsets: `cnt[k]` = #elements with a key `< k`.
    let mut acc = 0;
    let mut k2 = 0;
    while k2 < m {
        let c = cnt[k2];
        cnt[k2] = acc;
        let a2 = acc + c;
        acc = a2;
        k2 += 1;
    }
    let mut out: Vec<usize> = vec![0usize; n];
    let mut i2 = 0;
    while i2 < n {
        let k = key[i2];
        let pos = cnt[k];
        out[pos] = i2;
        let p1 = pos + 1;
        cnt[k] = p1;
        i2 += 1;
    }
    out
}

/// Stable counting sort of the elements of `items` by `key[items[i]]`, where
/// every key lies in `0..=maxkey`.  Returns the sorted element list.
fn cs_items(items: &[usize], key: &[usize], maxkey: usize) -> Vec<usize> {
    let n = items.len();
    let m = maxkey + 1;
    let mut cnt: Vec<usize> = vec![0usize; m];
    let mut i = 0;
    while i < n {
        let e = items[i];
        let k = key[e];
        let c = cnt[k];
        let c1 = c + 1;
        cnt[k] = c1;
        i += 1;
    }
    let mut acc = 0;
    let mut k2 = 0;
    while k2 < m {
        let c = cnt[k2];
        cnt[k2] = acc;
        let a2 = acc + c;
        acc = a2;
        k2 += 1;
    }
    let mut out: Vec<usize> = vec![0usize; n];
    let mut i2 = 0;
    while i2 < n {
        let e = items[i2];
        let k = key[e];
        let pos = cnt[k];
        out[pos] = e;
        let p1 = pos + 1;
        cnt[k] = p1;
        i2 += 1;
    }
    out
}

/// Build the suffix array and the LCP array of `s` by prefix doubling.
///
/// `rank[i]` is the class of the length-`k` prefix starting at `i` (classes are
/// numbered in suffix order); doubling the prefix length refines the classes
/// until every suffix has its own, at which point `sa` is the suffix order.
/// The LCP array is then produced by Kasai's algorithm.
pub fn build_sa(s: &[usize]) -> SaResult {
    let n = s.len();
    if n == 0 {
        return SaResult {
            sa: Vec::new(),
            lcp: Vec::new(),
        };
    }

    // Initial order: stable counting sort by the first character (0..=255).
    let mut sa: Vec<usize> = cs_indices(n, s, 255);

    // `rank[i]` = class of the length-1 prefix starting at `i`.
    let mut rank: Vec<usize> = vec![0usize; n];
    let first = sa[0];
    rank[first] = 0;
    let mut classes = 1;
    let mut j = 1;
    while j < n {
        let x = sa[j];
        let y = sa[j - 1];
        let sx = s[x];
        let sy = s[y];
        let inc = if sx != sy { 1 } else { 0 };
        let ry = rank[y];
        let r = ry + inc;
        rank[x] = r;
        let c2 = classes + inc;
        classes = c2;
        j += 1;
    }

    let mut k = 1;
    let mut cont = and2(k < n, classes < n);
    while cont {
        // Stable counting sort by the *second* half of the 2k-prefix.
        let mut key2: Vec<usize> = vec![0usize; n];
        let mut i = 0;
        while i < n {
            let ik = i + k;
            if ik < n {
                let r = rank[ik];
                let r1 = r + 1;
                key2[i] = r1;
            } else {
                key2[i] = 0;
            }
            i += 1;
        }
        let tmp: Vec<usize> = cs_indices(n, &key2, classes);
        // Stable counting sort by the *first* half: a radix sort of the pairs.
        sa = cs_items(&tmp, &rank, classes);

        // Re-derive the classes from the new order.
        let mut newrank: Vec<usize> = vec![0usize; n];
        let first2 = sa[0];
        newrank[first2] = 0;
        let mut ncls = 1;
        let mut j2 = 1;
        while j2 < n {
            let p = sa[j2];
            let q = sa[j2 - 1];
            let rp = rank[p];
            let rq = rank[q];
            let pk = p + k;
            let mut kp = 0;
            if pk < n {
                let rr = rank[pk];
                kp = rr + 1;
            }
            let qk = q + k;
            let mut kq = 0;
            if qk < n {
                let rr = rank[qk];
                kq = rr + 1;
            }
            let differ = or2(rp != rq, kp != kq);
            let inc = if differ { 1 } else { 0 };
            let nq = newrank[q];
            let v = nq + inc;
            newrank[p] = v;
            let c2 = ncls + inc;
            ncls = c2;
            j2 += 1;
        }
        let mut j3 = 0;
        while j3 < n {
            let v = newrank[j3];
            rank[j3] = v;
            j3 += 1;
        }
        classes = ncls;
        let k2 = k * 2;
        k = k2;
        cont = and2(k < n, classes < n);
    }

    let lcp = kasai_lcp(s, &sa);
    SaResult { sa, lcp }
}

/// Kasai's algorithm: `lcp[i]` is the length of the longest common prefix of
/// the suffixes starting at `sa[i]` and `sa[i + 1]`, and `lcp[n - 1] = 0`.
pub fn kasai_lcp(s: &[usize], sa: &[usize]) -> Vec<usize> {
    let n = sa.len();
    let mut lcp: Vec<usize> = vec![0usize; n];
    if n == 0 {
        return lcp;
    }

    // `rankpos[i]` is the position of the suffix starting at `i` inside `sa`.
    let mut rankpos: Vec<usize> = vec![0usize; n];
    let mut j = 0;
    while j < n {
        let p = sa[j];
        rankpos[p] = j;
        j += 1;
    }

    // `h` is the LCP of the suffixes `i` and `sa[rankpos[i] + 1]`; Kasai's
    // amortization relies on `h := max(h - 1, 0)` after each position.
    let mut h = 0;
    let mut i = 0;
    while i < n {
        let r = rankpos[i];
        let r1 = r + 1;
        if r1 < n {
            let jj = sa[r1];
            let rest_i = n - i;
            let rest_j = n - jj;
            let lim = minu(rest_i, rest_j);
            let mut go = true;
            let mut run = and2(h < lim, go);
            while run {
                let ih = i + h;
                let jh = jj + h;
                let a = s[ih];
                let b = s[jh];
                if a != b {
                    go = false;
                } else {
                    let h1 = h + 1;
                    h = h1;
                }
                run = and2(h < lim, go);
            }
            let hl = h;
            lcp[r] = hl;
            if h > 0 {
                let hm = h - 1;
                h = hm;
            }
        }
        i += 1;
    }
    lcp
}
