//! Manacher's algorithm for odd-length palindromes.

/// Return `d[i]` = the radius (in the "e-maxx `d1`" convention) of the longest
/// odd-length palindrome centered at `i`: the palindrome is `s[i-(d[i]-1) ..= i+(d[i]-1)]`
/// and has length `2*d[i] - 1`.  A single character has `d[i] = 1`.
pub fn manacher(s: &[u8]) -> Vec<usize> {
    let n = s.len();
    let mut d: Vec<usize> = vec![0usize; n];
    if n == 0 {
        return d;
    }
    d[0] = 1;

    // `l`, `r` are the inclusive bounds of the rightmost palindrome found so far.
    let mut l = 0;
    let mut r = 0;
    let mut i = 1;
    while i < n {
        // Seed the radius from the mirror, if `i` lies inside the current
        // rightmost palindrome.
        let mut k = 1;
        if i <= r {
            let mirror = l + r - i;
            let mirror_d = d[mirror];
            let limit = r - i + 1;
            let k_seed = if mirror_d < limit { mirror_d } else { limit };
            k = k_seed;
        }

        // Extend the palindrome as far as possible.
        let mut j = k;
        let mut more = true;
        while more {
            let hi = i + j;
            if hi < n {
                if i >= j {
                    let lo = i - j;
                    if s[hi] != s[lo] {
                        more = false;
                    } else {
                        j += 1;
                    }
                } else {
                    more = false;
                }
            } else {
                more = false;
            }
        }
        d[i] = j;

        let new_r = i + j - 1;
        if new_r > r {
            let new_l = i + 1 - j;
            l = new_l;
            r = new_r;
        }
        i += 1;
    }
    d
}
