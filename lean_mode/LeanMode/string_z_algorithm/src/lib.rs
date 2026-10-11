//! Z-algorithm: compute the Z-array of a byte string.

/// Return the Z-array of `s`: `z[i]` is the length of the longest common prefix
/// of `s` and `s[i..]` (so `z[0] = s.len()`).
pub fn z_algorithm(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut z: Vec<usize> = vec![0usize; n];
    if n == 0 {
        return z;
    }
    z[0] = n;

    let mut l = 0;
    let mut r = 0;
    let mut i = 1;
    while i < n {
        let z_i = if i < r {
            let a = r - i;
            let b = z[i - l];
            if a < b {
                a
            } else {
                b
            }
        } else {
            0
        };
        z[i] = z_i;

        // Extend the match as far as possible.
        let mut j = z_i;
        let mut more = true;
        while more {
            let next = i + j;
            if next < n {
                if s[next] != s[j] {
                    more = false;
                } else {
                    j += 1;
                }
            } else {
                more = false;
            }
        }
        z[i] = j;

        if i + j > r {
            l = i;
            r = i + j;
        }
        i += 1;
    }
    z
}
