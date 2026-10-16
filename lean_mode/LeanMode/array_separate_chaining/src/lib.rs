//! Hash table with separate chaining.
//!
//! Each bucket is a linked list stored in flat parallel arrays (an adjacency
//! list) rather than a nested `Vec<Vec<…>>`: `head[b]` is the first node of
//! bucket `b` (`None` if empty), and `next[node]` links to the next node in the
//! same bucket.  Keys are non-negative `usize`.

/// A separate-chaining hash table.
pub struct ChainTable {
    pub head: Vec<Option<usize>>,
    pub keys: Vec<usize>,
    pub next: Vec<Option<usize>>,
}

/// Create a table with `num_buckets` empty buckets (`num_buckets > 0`).
pub fn ct_new(num_buckets: usize) -> ChainTable {
    // Build the head array element-by-element (a `vec![None; n]` would pull in a
    // noncomputable `Clone` instance for `Option` under translation).
    let mut head: Vec<Option<usize>> = Vec::new();
    let mut i = 0;
    while i < num_buckets {
        head.push(None);
        i += 1;
    }
    let keys: Vec<usize> = Vec::new();
    let next: Vec<Option<usize>> = Vec::new();
    ChainTable { head, keys, next }
}

/// `key mod nb` via repeated subtraction (exact for `nb > 0`).
fn mod_buckets(key: usize, nb: usize) -> usize {
    let mut r = key;
    while r >= nb {
        r -= nb;
    }
    r
}

/// Insert `key`, prepending it to its bucket; returns the updated table and
/// `true` iff the key was newly inserted.
pub fn ct_insert(t: ChainTable, key: usize) -> (ChainTable, bool) {
    let mut r = t;
    let nb = r.head.len();
    let b = mod_buckets(key, nb);

    // Walk bucket `b`'s chain looking for `key`.
    let mut cur = r.head[b];
    let mut found = false;
    let mut done = false;
    while !done {
        match cur {
            None => {
                done = true;
            }
            Some(node) => {
                if r.keys[node] == key {
                    found = true;
                    done = true;
                } else {
                    cur = r.next[node];
                }
            }
        }
    }

    if found {
        (r, false)
    } else {
        let new_node = r.keys.len();
        r.keys.push(key);
        r.next.push(r.head[b]);
        r.head[b] = Some(new_node);
        (r, true)
    }
}

/// Return `true` iff `key` is present in the table.
pub fn ct_contains(t: &ChainTable, key: usize) -> bool {
    let nb = t.head.len();
    let b = mod_buckets(key, nb);

    let mut cur = t.head[b];
    let mut found = false;
    let mut done = false;
    while !done {
        match cur {
            None => {
                done = true;
            }
            Some(node) => {
                if t.keys[node] == key {
                    found = true;
                    done = true;
                } else {
                    cur = t.next[node];
                }
            }
        }
    }
    found
}
