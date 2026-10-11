//! Knuth–Morris–Pratt (KMP) string search over byte slices.

/// Compute the KMP prefix function for `pattern`: `pi[i]` is the length of the
/// longest proper prefix of `pattern[..=i]` that is also a suffix of it.
pub fn prefix_function(pattern: &[u8]) -> Vec<usize> {
    let n = pattern.len();
    let mut pi: Vec<usize> = Vec::new();
    let mut i = 0;
    while i < n {
        pi.push(0);
        i += 1;
    }

    let mut i = 1;
    while i < n {
        let mut j = pi[i - 1];
        // Fall back while we can and the next byte does not extend the match.
        let mut need_fall_back = j > 0 && pattern[i] != pattern[j];
        while need_fall_back {
            j = pi[j - 1];
            need_fall_back = j > 0 && pattern[i] != pattern[j];
        }
        if pattern[i] == pattern[j] {
            j += 1;
        }
        pi[i] = j;
        i += 1;
    }
    pi
}

/// Return the index of the first occurrence of `pattern` in `text`, or `None`
/// if `pattern` does not occur. An empty `pattern` matches at index `0`.
pub fn kmp_search(text: &[u8], pattern: &[u8]) -> Option<usize> {
    let m = pattern.len();
    if m == 0 {
        return Some(0);
    }

    let pi = prefix_function(pattern);
    let n = text.len();
    let mut i = 0;
    let mut j = 0;
    while i < n {
        let mut need_fall_back = j > 0 && text[i] != pattern[j];
        while need_fall_back {
            j = pi[j - 1];
            need_fall_back = j > 0 && text[i] != pattern[j];
        }
        if text[i] == pattern[j] {
            j += 1;
            if j == m {
                return Some(i + 1 - m);
            }
        }
        i += 1;
    }
    None
}
