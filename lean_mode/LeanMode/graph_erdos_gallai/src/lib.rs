//! Graphical degree-sequence test (Erdős–Gallai / Havel–Hakimi).
//!
//! A finite sequence `d` of non-negative integers is *graphical* when some
//! simple undirected graph (no loops, no parallel edges) has `d` as its degree
//! sequence.  Erdős–Gallai (1960) characterise the graphical sequences by a
//! family of linear inequalities; the classical constructive proof is the
//! Havel–Hakimi reduction, which is what this program runs.
//!
//! Havel–Hakimi step: sort the degrees in non-increasing order, take the
//! largest entry `d1` (the other `n-1` entries being `d2 ≥ ... ≥ dn`), and
//! replace the sequence by
//!     `(d2 - 1, d3 - 1, ..., d_{d1+1} - 1, d_{d1+2}, ..., dn)`.
//! The original sequence is graphical iff `d1 ≤ #positives - 1` (so that the
//! reduction is defined at all) and the reduced sequence is graphical.
//! Iterating until all entries are zero decides graphicality.

/// Decide whether the degree sequence `d` is graphical (realisable by a simple
/// undirected graph), via the Havel–Hakimi reduction.
///
/// The algorithm works on a destructive copy of `d` and does not modify the
/// input.  It returns `true` for the empty sequence and for the all-zero
/// sequence.
pub fn is_graphical(d: &[usize]) -> bool {
    let n = d.len();

    // Working copy of the degrees (Havel–Hakimi reduces them destructively).
    let mut w: Vec<usize> = vec![0usize; n];
    let mut i: usize = 0;
    while i < n {
        w[i] = d[i];
        i += 1;
    }

    let mut graphical: bool = true;
    let mut done: bool = false;

    while !done {
        // --- keep `w` sorted in non-increasing order (selection sort) ---
        let mut a: usize = 0;
        while a < n {
            let mut b: usize = a + 1;
            while b < n {
                let wa = w[a];
                let wb = w[b];
                if wb > wa {
                    w[a] = wb;
                    w[b] = wa;
                }
                b += 1;
            }
            a += 1;
        }

        if n == 0 {
            // The empty sequence is graphical.
            done = true;
        } else {
            let d1 = w[0];
            if d1 == 0 {
                // All degrees are zero: graphical (the empty graph).
                done = true;
            } else {
                // How many *other* positive degrees remain?
                let mut cnt: usize = 0;
                let mut t: usize = 1;
                while t < n {
                    let y = w[t];
                    if y > 0 {
                        cnt += 1;
                    }
                    t += 1;
                }

                if d1 > cnt {
                    // A vertex of degree `d1` needs `d1` distinct neighbours,
                    // but only `cnt` other positive degrees remain.
                    graphical = false;
                    done = true;
                } else {
                    // Subtract 1 from the `cnt`-sorted next `d1` degrees.
                    let mut k: usize = 1;
                    let mut failed: bool = false;
                    while k <= d1 {
                        let v = w[k];
                        if v == 0 {
                            // A neighbour slot is missing: not graphical.
                            failed = true;
                        } else {
                            let nv = v - 1;
                            w[k] = nv;
                        }
                        k += 1;
                    }
                    if failed {
                        graphical = false;
                        done = true;
                    } else {
                        // Remove the largest degree and iterate.
                        w[0] = 0;
                    }
                }
            }
        }
    }

    graphical
}
