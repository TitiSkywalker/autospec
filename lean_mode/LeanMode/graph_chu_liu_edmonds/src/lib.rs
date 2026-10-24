//! Minimum-weight spanning arborescence (branching) rooted at `r`.
//!
//! This is the Chu–Liu / Edmonds algorithm (1965 / 1967): repeatedly pick the
//! minimum-weight incoming edge of every vertex, and whenever those choices
//! close a directed cycle, *contract* the cycle into a single supernode
//! (re-weighting the edges entering the cycle by subtracting the minimum
//! in-edge weight of their head), then iterate.
//!
//! The graph is a flat edge list `from` / `to` / `w` on `nv` vertices
//! (`0 .. nv`), labelled with `usize`.  Parallel edges are allowed, self-loops
//! are ignored.

/// Minimum-weight arborescence (branching) rooted at `r` in the directed graph
/// on `nv` vertices with edge lists `from`/`to`/`w` (parallel edges allowed,
/// self-loops ignored).  Returns `(exists, weight)`: `exists == false` iff no
/// spanning arborescence rooted at `r` exists (in which case `weight` is 0).
///
/// It is assumed that every edge endpoint and `r` is `< nv`.
pub fn optimum_branching(
    nv: usize,
    from: &[usize],
    to: &[usize],
    w: &[usize],
    r: usize,
) -> (bool, usize) {
    // ---- the current (possibly contracted) graph -------------------------
    // vertices are relabelled `0 .. nn`; `root_comp` is the current label of
    // the component containing the root `r`.
    let mut nn: usize = nv;
    let mut root_comp: usize = r;
    let mut ef: Vec<usize> = from.to_vec();
    let mut et: Vec<usize> = to.to_vec();
    let mut ew: Vec<usize> = w.to_vec();

    // total weight accumulated over all contraction rounds
    let mut total: usize = 0;
    let mut exists: bool = true;
    let mut done: bool = false;

    while !done {
        // ===== 1. minimum in-edge of every non-root component =============
        let mut minc: Vec<usize> = vec![0usize; nn];
        let mut mine: Vec<usize> = vec![0usize; nn];
        let mut have: Vec<usize> = vec![0usize; nn];

        // one pass over the edges: keep the cheapest incoming edge of every
        // non-root component (self-loops and edges into the root are skipped)
        let m = ef.len();
        let mut e: usize = 0;
        while e < m {
            let h = et[e];
            let f = ef[e];
            if h != root_comp {
                if f != h {
                    let we = ew[e];
                    let hh = have[h];
                    if hh == 0 {
                        have[h] = 1;
                        mine[h] = e;
                        minc[h] = we;
                    } else {
                        let cur = minc[h];
                        if we < cur {
                            mine[h] = e;
                            minc[h] = we;
                        }
                    }
                }
            }
            e += 1;
        }

        // is some component left without any incoming edge?
        let mut bad: bool = false;
        let mut v: usize = 0;
        while v < nn {
            if v != root_comp {
                let hv = have[v];
                if hv == 0 {
                    bad = true;
                }
            }
            v += 1;
        }

        if bad {
            // some component has no incoming edge at all: no arborescence
            exists = false;
            done = true;
        } else {
            // ===== 2. cycle detection on the chosen in-edges ==============
            // state: 0 = unvisited, 1 = on the current walk, 2 = finished
            let mut state: Vec<usize> = vec![0usize; nn];
            let mut cyc: usize = nn; // sentinel: no cycle found
            let mut v2: usize = 0;
            while v2 < nn {
                if v2 != root_comp {
                    if state[v2] == 0 {
                        // walk the parent chain from v2 until the root, a
                        // finished vertex, or a vertex of the current walk
                        let mut x = v2;
                        let mut walk: bool = true;
                        while walk {
                            if x == root_comp {
                                walk = false;
                            } else {
                                let sx = state[x];
                                if sx != 0 {
                                    walk = false;
                                } else {
                                    state[x] = 1;
                                    let ei = mine[x];
                                    let nx = ef[ei];
                                    x = nx;
                                }
                            }
                        }
                        if x != root_comp {
                            let sx2 = state[x];
                            if sx2 == 1 {
                                // x is on the current walk: a cycle closing at x
                                cyc = x;
                            }
                        }
                        // paint the whole walk as finished
                        let mut y = v2;
                        let mut clr: bool = true;
                        while clr {
                            let sy = state[y];
                            if sy == 1 {
                                state[y] = 2;
                                let ei = mine[y];
                                let ny = ef[ei];
                                y = ny;
                            } else {
                                clr = false;
                            }
                        }
                    }
                }
                v2 += 1;
            }

            if cyc == nn {
                // the chosen in-edges already form an arborescence: its weight
                // is the sum of the chosen in-edges of this round
                let mut v3: usize = 0;
                while v3 < nn {
                    if v3 != root_comp {
                        let c = minc[v3];
                        total = total + c;
                    }
                    v3 += 1;
                }
                done = true;
            } else {
                // ===== 3. contract the cycle into one supernode ===========
                // newmap[old label] = new label; sentinel `nn` = unassigned,
                // the cycle is relabelled to 0.
                let mut newmap: Vec<usize> = vec![nn; nn];
                let mut y1 = cyc;
                let mut t: usize = 0;
                while t < nn {
                    newmap[y1] = 0;
                    let ei = mine[y1];
                    let ny = ef[ei];
                    y1 = ny;
                    t += 1;
                }
                // Chu–Liu accounting: the optimum of the current graph is the
                // optimum of the contracted (re-weighted) graph plus the sum of
                // the chosen in-edge weights of the *cycle* nodes.
                let mut v4: usize = 0;
                while v4 < nn {
                    if newmap[v4] == 0 {
                        let c = minc[v4];
                        total = total + c;
                    }
                    v4 += 1;
                }
                let mut new_n: usize = 1;
                let mut i: usize = 0;
                while i < nn {
                    if newmap[i] == nn {
                        newmap[i] = new_n;
                        new_n += 1;
                    }
                    i += 1;
                }

                // rebuild the edge list: drop the edges internal to the cycle,
                // re-weight the edges entering it.
                let mut nef: Vec<usize> = Vec::new();
                let mut net: Vec<usize> = Vec::new();
                let mut new_ew: Vec<usize> = Vec::new();
                let m2 = ef.len();
                let mut e2: usize = 0;
                while e2 < m2 {
                    let f = ef[e2];
                    let h = et[e2];
                    let nf = newmap[f];
                    let nh = newmap[h];
                    if nf != nh {
                        let we = ew[e2];
                        let mut nw = we;
                        if nh == 0 {
                            // entering the contracted cycle: subtract the
                            // minimum in-edge weight of the head
                            let mi = minc[h];
                            nw = we - mi;
                        }
                        nef.push(nf);
                        net.push(nh);
                        new_ew.push(nw);
                    }
                    e2 += 1;
                }
                ef = nef;
                et = net;
                ew = new_ew;
                let nr = newmap[root_comp];
                root_comp = nr;
                nn = new_n;
            }
        }
    }

    let out_w = if exists { total } else { 0usize };
    (exists, out_w)
}
