//! Counting Hamiltonian cycles in a `w × h` grid graph via plug DP
//! (broken-profile / connectivity DP).
//!
//! A Hamiltonian cycle is a simple cycle of grid edges visiting every cell
//! exactly once.  Each *undirected* cycle is counted once (direction and
//! starting point are not distinguished).
//!
//! ## Contour / state
//!
//! Cells are processed in row-major order.  Just before cell `(r, c)` the
//! staircase boundary between processed and unprocessed cells crosses exactly
//! `np = w + 1` grid edges ("plugs").  They are stored left to right in
//! `state[0 .. np]`:
//!   * `state[k]` for `k < c`  is the *down* plug of `(r, k)`,
//!   * `state[c]`              is the *left* plug of `(r, c)`,
//!   * `state[c + 1]`          is the *up* plug of `(r, c)`,
//!   * `state[k]` for `k > c+1` is the *up* plug of `(r, k-1)`.
//!
//! A plug's value is `0` when the edge is unused, otherwise a *component
//! label*: two plugs share a label iff the processed half of the cycle
//! already connects them.  Processing `(r, c)` reads the left/up plugs and
//! chooses the right/down plugs so that the cell has exactly degree two and
//! the chosen edges form one non-crossing structure:
//!   * both absent      -> create a *new* pair (right & down share a fresh label),
//!   * one present      -> extend that component through the cell,
//!   * both present,
//!       different      -> merge the two components (relabel),
//!       equal          -> close the cycle, allowed only at the very last cell.
//!
//! ## Canonical labelling
//!
//! Labels are bookkeeping only, so two states with the same *partition* of the
//! plug positions are the same state.  `canon` relabels by order of first
//! appearance (left to right) so equal partitions always encode identically;
//! the state map is then a plain association of label-array -> count.
//!
//! ## Counting convention
//!
//! Every Hamiltonian cycle induces exactly one sequence of per-cell edge
//! choices, and `dp[all-zero]` counts those sequences that close into the
//! empty frontier exactly at the last cell.  Hence the result is the number of
//! undirected Hamiltonian cycles.

struct Dp {
    /// Flat state table: state `t` occupies `keys[t*np .. t*np + np]`.
    keys: Vec<usize>,
    /// Multiplicity of state `t`.
    vals: Vec<usize>,
}

/// Flat index of block `a`, stride `n`, offset `b`.
fn idx3(a: usize, n: usize, b: usize) -> usize {
    let t = a * n;
    let r = t + b;
    r
}

/// Canonicalise a plug labelling: relabel by order of first appearance
/// (left to right).  `0` stays `0`.
fn canon(labels0: Vec<usize>, np: usize) -> Vec<usize> {
    let mut out = vec![0usize; np];
    let mut remap = vec![0usize; np + 2];
    let mut nxt: usize = 0;
    let mut i: usize = 0;
    while i < np {
        let x = labels0[i];
        if x != 0 {
            let mut m = remap[x];
            if m == 0 {
                nxt = nxt + 1;
                m = nxt;
                remap[x] = m;
            }
            out[i] = m;
        }
        i += 1;
    }
    out
}

/// Build a successor labelling from `labels0` (consumed): replace every
/// occurrence of label `from` by `to`, then overwrite positions `c` and
/// `c + 1` with `vc` and `vc1`.
fn modify(
    labels0: Vec<usize>,
    np: usize,
    c: usize,
    vc: usize,
    vc1: usize,
    from: usize,
    to: usize,
) -> Vec<usize> {
    let mut out = vec![0usize; np];
    let mut i: usize = 0;
    while i < np {
        let x = labels0[i];
        if x == from {
            out[i] = to;
        } else {
            out[i] = x;
        }
        i += 1;
    }
    out[c] = vc;
    let c1 = c + 1;
    out[c1] = vc1;
    out
}

/// Insert the canonical labelling `labels` (length `np`) with multiplicity
/// `cnt` into the state map `dp0`, merging with an equal state if present.
fn add_to(dp0: Dp, np: usize, labels: Vec<usize>, cnt: usize) -> Dp {
    let nstates = dp0.vals.len();
    let mut idxf = nstates;
    let mut t: usize = 0;
    while t < nstates {
        let mut matchc: usize = 0;
        let mut p: usize = 0;
        while p < np {
            let a = dp0.keys[idx3(t, np, p)];
            let b = labels[p];
            if a == b {
                matchc = matchc + 1;
            }
            p += 1;
        }
        if matchc == np {
            if t < idxf {
                idxf = t;
            }
        }
        t += 1;
    }
    if idxf < nstates {
        let old = dp0.vals[idxf];
        let s = old + cnt;
        let keys = dp0.keys;
        let mut vals = dp0.vals;
        vals[idxf] = s;
        Dp { keys: keys, vals: vals }
    } else {
        let mut keys = dp0.keys;
        let mut vals = dp0.vals;
        let mut p: usize = 0;
        while p < np {
            let b = labels[p];
            keys.push(b);
            p += 1;
        }
        vals.push(cnt);
        Dp { keys: keys, vals: vals }
    }
}

/// Process state `t` of `dpin`, accumulating its successor states into
/// `dpout`, then recurse on `t + 1`.  Returns the finished output map.
fn process_states(
    dpin: Dp,
    dpout: Dp,
    np: usize,
    w: usize,
    h: usize,
    r: usize,
    c: usize,
    t: usize,
) -> Dp {
    let nstates = dpin.vals.len();
    if t >= nstates {
        dpout
    } else {
        let cnt = dpin.vals[t];
        // Two identical copies: each successor is built from one of them.
        let mut la = vec![0usize; np];
        let mut lb = vec![0usize; np];
        let mut i: usize = 0;
        while i < np {
            let v = dpin.keys[idx3(t, np, i)];
            la[i] = v;
            lb[i] = v;
            i += 1;
        }
        let l = la[c];
        let c1 = c + 1;
        let u = la[c1];
        // Is `(r, c)` the last cell of the grid?
        let mut last = false;
        let r1 = r + 1;
        if r1 == h {
            if c1 == w {
                last = true;
            }
        }
        let mut out1 = dpout;
        if l == 0 {
            if u == 0 {
                // Both absent: create a new pair (right & down), possible only
                // when both cells exist.
                let mut both = false;
                let cp1 = c + 1;
                if cp1 < w {
                    let rp1 = r + 1;
                    if rp1 < h {
                        both = true;
                    }
                }
                if both {
                    let m = np + 1;
                    let s = modify(la, np, c, m, m, 0, 0);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
            } else {
                // Only the up plug: extend it right or down.
                let cp1 = c + 1;
                if cp1 < w {
                    let s = modify(la, np, c, 0, u, 0, 0);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
                let rp1 = r + 1;
                if rp1 < h {
                    let s = modify(lb, np, c, u, 0, 0, 0);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
            }
        } else {
            if u == 0 {
                // Only the left plug: extend it right or down.
                let cp1 = c + 1;
                if cp1 < w {
                    let s = modify(la, np, c, 0, l, 0, 0);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
                let rp1 = r + 1;
                if rp1 < h {
                    let s = modify(lb, np, c, l, 0, 0, 0);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
            } else {
                // Both plugs present: the cell is degree-two already.
                if l == u {
                    // Same component: closing a cycle, only at the last cell.
                    if last {
                        let s = modify(la, np, c, 0, 0, 0, 0);
                        let sc = canon(s, np);
                        out1 = add_to(out1, np, sc, cnt);
                    }
                } else {
                    // Different components: merge them (relabel `u` to `l`).
                    let s = modify(la, np, c, 0, 0, u, l);
                    let sc = canon(s, np);
                    out1 = add_to(out1, np, sc, cnt);
                }
            }
        }
        process_states(dpin, out1, np, w, h, r, c, t + 1)
    }
}

/// Advance every state of `dpin` past cell `(r, c)`.
fn step(dpin: Dp, np: usize, w: usize, h: usize, r: usize, c: usize) -> Dp {
    let keys: Vec<usize> = Vec::new();
    let vals: Vec<usize> = Vec::new();
    let dpout = Dp { keys: keys, vals: vals };
    process_states(dpin, dpout, np, w, h, r, c, 0)
}

/// Shift every state one position right (new `[i] = old[i-1]`, `new[0] = 0`).
/// Used between rows: the down plugs of row `r` become the up plugs of row
/// `r + 1`.  The shift preserves canonical labelling and is injective.
fn shift_all(dpin: Dp, np: usize) -> Dp {
    let nlen = dpin.keys.len();
    let nstates = dpin.vals.len();
    let mut out = vec![0usize; nlen];
    let mut t: usize = 0;
    while t < nstates {
        let mut i: usize = 1;
        while i < np {
            let im1 = i - 1;
            let v = dpin.keys[idx3(t, np, im1)];
            out[idx3(t, np, i)] = v;
            i += 1;
        }
        t += 1;
    }
    Dp { keys: out, vals: dpin.vals }
}

/// Multiplicity of the all-zero state (= number of completed cycles).
fn lookup_zero(dp: Dp, np: usize) -> usize {
    let nstates = dp.vals.len();
    let mut res: usize = 0;
    let mut t: usize = 0;
    while t < nstates {
        let mut nz: usize = 0;
        let mut i: usize = 0;
        while i < np {
            let v = dp.keys[idx3(t, np, i)];
            if v != 0 {
                nz = nz + 1;
            }
            i += 1;
        }
        if nz == 0 {
            res = dp.vals[t];
        }
        t += 1;
    }
    res
}

/// Number of Hamiltonian cycles of the `w × h` grid graph (`w` columns,
/// `h` rows).  Each undirected cycle is counted once.  Degenerate grids
/// (`w = 0` or `h = 0`, and `1 × n` grids) have no cycle and yield `0`.
pub fn ham_cycles_grid(w: usize, h: usize) -> usize {
    if w == 0 {
        return 0;
    }
    if h == 0 {
        return 0;
    }
    let np = w + 1;
    let keys = vec![0usize; np];
    let mut vals: Vec<usize> = Vec::new();
    vals.push(1usize);
    let mut dp = Dp { keys: keys, vals: vals };
    let mut r: usize = 0;
    while r < h {
        if r > 0 {
            dp = shift_all(dp, np);
        }
        let mut c: usize = 0;
        while c < w {
            dp = step(dp, np, w, h, r, c);
            c += 1;
        }
        r += 1;
    }
    lookup_zero(dp, np)
}
