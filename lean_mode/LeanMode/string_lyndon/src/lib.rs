//! Lyndon factorization via Duval's algorithm.

/// Return the lengths of the Lyndon factors of `s`, in order.
///
/// A Lyndon word is a nonempty string strictly smaller (lexicographically)
/// than all of its non-trivial proper suffixes.  Duval's algorithm partitions
/// `s` into Lyndon words `w_1 · w_2 · … · w_k` that are lexicographically
/// non-increasing (`w_1 ≥ w_2 ≥ … ≥ w_k`), in O(n).
pub fn lyndon_factorization(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut lens: Vec<usize> = Vec::new();
    if n == 0 {
        return lens;
    }

    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        let mut k = i;
        // Extend the pre-simple factor while `s[k] <= s[j]`.
        let mut more = true;
        while more {
            if j < n {
                if s[k] <= s[j] {
                    if s[k] < s[j] {
                        k = i;
                    } else {
                        k += 1;
                    }
                    j += 1;
                } else {
                    more = false;
                }
            } else {
                more = false;
            }
        }
        // Emit each Lyndon factor of length `j - k`.
        while i <= k {
            let flen = j - k;
            lens.push(flen);
            i += flen;
        }
    }
    lens
}
