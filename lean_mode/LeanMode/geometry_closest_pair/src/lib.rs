//! Closest pair of points via divide and conquer.
//!
//! Distances are compared in squared form (no `sqrt`), so the algorithm stays
//! in exact rational arithmetic and the result is the minimum *squared*
//! Euclidean distance.

/// Squared Euclidean distance between points `a` and `b`.
fn sq_dist(xs: &[f64], ys: &[f64], a: usize, b: usize) -> f64 {
    let dx = xs[a] - xs[b];
    let dy = ys[a] - ys[b];
    let r = dx * dx + dy * dy;
    r
}

/// Brute force over the small range `[lo, hi)` (at least two points).
fn brute_closest(xs: &[f64], ys: &[f64], idx: &[usize], lo: usize, hi: usize) -> f64 {
    let mut best = sq_dist(xs, ys, idx[lo], idx[lo + 1]);
    let mut i = lo;
    while i < hi {
        let mut j = i + 1;
        while j < hi {
            let d = sq_dist(xs, ys, idx[i], idx[j]);
            if d < best {
                best = d;
            }
            j += 1;
        }
        i += 1;
    }
    best
}

/// Closest pair over `idx[lo..hi]`, which is sorted by x-coordinate.
fn closest_rec(xs: &[f64], ys: &[f64], idx: &[usize], lo: usize, hi: usize) -> f64 {
    let count = hi - lo;
    let result = if count <= 3 {
        brute_closest(xs, ys, idx, lo, hi)
    } else {
        let mid = lo + count / 2;
        let dl = closest_rec(xs, ys, idx, lo, mid);
        let dr = closest_rec(xs, ys, idx, mid, hi);
        let mut best = if dl < dr { dl } else { dr };
        let mid_x = xs[idx[mid]];

        // Collect the strip of points within `best` (squared) of the divider.
        let mut strip: Vec<usize> = Vec::new();
        let mut i = lo;
        while i < hi {
            let dx = xs[idx[i]] - mid_x;
            let dx2 = dx * dx;
            if dx2 < best {
                strip.push(idx[i]);
            }
            i += 1;
        }

        // Sort the strip by y-coordinate.
        let m = strip.len();
        let mut a = 0;
        while a < m {
            let mut b = 0;
            while b + 1 < m {
                let sa = strip[b];
                let sb = strip[b + 1];
                if ys[sa] > ys[sb] {
                    strip[b] = sb;
                    strip[b + 1] = sa;
                }
                b += 1;
            }
            a += 1;
        }

        // Compare each strip point with the following ones, stopping when the
        // vertical gap alone already exceeds `best`.
        let mut i2 = 0;
        while i2 < m {
            let mut j = i2 + 1;
            while j < m {
                let dy = ys[strip[j]] - ys[strip[i2]];
                let dy2 = dy * dy;
                if dy2 >= best {
                    break;
                }
                let d = sq_dist(xs, ys, strip[i2], strip[j]);
                if d < best {
                    best = d;
                }
                j += 1;
            }
            i2 += 1;
        }
        best
    };
    result
}

/// Minimum squared distance between any pair of the `n` points (flat
/// `[x0, y0, x1, y1, ...]`).  Returns `0.0` when there are fewer than two
/// points.
pub fn closest_pair(points: &[f64], n: usize) -> f64 {
    let result = if n < 2 {
        0.0
    } else {
        let mut xs: Vec<f64> = vec![0.0; n];
        let mut ys: Vec<f64> = vec![0.0; n];
        let mut idx: Vec<usize> = vec![0usize; n];
        let mut k = 0;
        while k < n {
            xs[k] = points[2 * k];
            ys[k] = points[2 * k + 1];
            idx[k] = k;
            k += 1;
        }

        // Sort indices by (x, then y).
        let mut a = 0;
        while a < n {
            let mut b = 0;
            while b + 1 < n {
                let ia = idx[b];
                let ib = idx[b + 1];
                let swap = if xs[ia] > xs[ib] {
                    true
                } else if xs[ia] < xs[ib] {
                    false
                } else if ys[ia] > ys[ib] {
                    true
                } else {
                    false
                };
                if swap {
                    idx[b] = ib;
                    idx[b + 1] = ia;
                }
                b += 1;
            }
            a += 1;
        }

        closest_rec(&xs, &ys, &idx, 0, n)
    };
    result
}
