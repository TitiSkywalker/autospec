//! Hash table with open addressing and linear probing.
//!
//! Keys are non-negative `usize`; an empty slot is tracked by a parallel
//! `occupied` boolean array (so `0` is a valid key).  The probe sequence is
//! `h(key) = key mod capacity`, then linear probing `h + 1, h + 2, …`.

/// An open-addressing hash table with linear probing.
pub struct OpenTable {
    pub keys: Vec<usize>,
    pub occupied: Vec<bool>,
}

/// Create an empty table with `capacity` slots (`capacity > 0`).
pub fn ot_new(capacity: usize) -> OpenTable {
    let mut keys: Vec<usize> = vec![0usize; capacity];
    let mut occupied: Vec<bool> = vec![false; capacity];
    OpenTable { keys, occupied }
}

/// `key mod cap` via repeated subtraction (exact for `cap > 0`).
fn mod_cap(key: usize, cap: usize) -> usize {
    let mut r = key;
    while r >= cap {
        r -= cap;
    }
    r
}

/// Insert `key`, returning the updated table and `true` iff the key was newly
/// inserted.  A `false` result means either the key was already present or the
/// table is full (in both cases the stored keys are unchanged).
pub fn ot_insert(t: OpenTable, key: usize) -> (OpenTable, bool) {
    let mut r = t;
    let cap = r.keys.len();
    let mut idx = mod_cap(key, cap);
    let mut inserted = false;
    let mut steps = 0;
    let mut done = false;
    while !done {
        if !r.occupied[idx] {
            r.keys[idx] = key;
            r.occupied[idx] = true;
            inserted = true;
            done = true;
        } else if r.keys[idx] == key {
            inserted = false;
            done = true;
        } else {
            let t2 = idx + 1;
            idx = if t2 >= cap { t2 - cap } else { t2 };
            steps += 1;
            if steps >= cap {
                inserted = false;
                done = true;
            }
        }
    }
    (r, inserted)
}

/// Return `true` iff `key` is present in the table.
pub fn ot_contains(t: &OpenTable, key: usize) -> bool {
    let cap = t.keys.len();
    let mut idx = mod_cap(key, cap);
    let mut found = false;
    let mut steps = 0;
    let mut done = false;
    while !done {
        if t.occupied[idx] {
            if t.keys[idx] == key {
                found = true;
                done = true;
            } else {
                let t2 = idx + 1;
                idx = if t2 >= cap { t2 - cap } else { t2 };
                steps += 1;
                if steps >= cap {
                    found = false;
                    done = true;
                }
            }
        } else {
            found = false;
            done = true;
        }
    }
    found
}
