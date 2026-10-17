//! Convex hull via Andrew's monotone chain.

/// Cross product `(b-a) × (c-a)` for three point indices into a flat `[x0,y0,…]`.
fn turn(points: &[f64], a: usize, b: usize, c: usize) -> f64 {
    let ax = points[2 * a];
    let ay = points[2 * a + 1];
    let bx = points[2 * b];
    let by = points[2 * b + 1];
    let cx = points[2 * c];
    let cy = points[2 * c + 1];
    let r = (bx - ax) * (cy - ay) - (by - ay) * (cx - ax);
    r
}

/// Convex hull of `n` points, returning the hull vertices in counter-clockwise
/// order as a flat `[x0, y0, x1, y1, ...]`.
///
/// Collinear points on the hull boundary are dropped (only the extreme corners
/// are kept).  With fewer than three points the input is returned unchanged.
pub fn convex_hull(points: &[f64], n: usize) -> Vec<f64> {
    if n <= 1 {
        return points.to_vec();
    }

    // Index array, sorted by (x, then y) with a bubble sort.
    let mut idx: Vec<usize> = vec![0usize; n];
    let mut k = 0;
    while k < n {
        idx[k] = k;
        k += 1;
    }
    let mut a = 0;
    while a < n {
        let mut b = 0;
        while b + 1 < n {
            let ia = idx[b];
            let ib = idx[b + 1];
            let ax = points[2 * ia];
            let ay = points[2 * ia + 1];
            let bx = points[2 * ib];
            let by = points[2 * ib + 1];
            let swap = if ax > bx {
                true
            } else if ax < bx {
                false
            } else if ay > by {
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

    // Lower hull: a fixed-capacity stack with an explicit top counter, so a
    // "pop" is just decrementing the counter (no `Vec::pop`).
    let mut lower: Vec<usize> = vec![0usize; n];
    let mut ltop = 0;
    let mut i = 0;
    while i < n {
        let si = idx[i];
        loop {
            if ltop < 2 {
                break;
            }
            let p1 = lower[ltop - 2];
            let p2 = lower[ltop - 1];
            let cr = turn(points, p1, p2, si);
            if cr > 0.0 {
                break;
            }
            ltop -= 1;
        }
        lower[ltop] = si;
        ltop += 1;
        i += 1;
    }

    // Upper hull, scanning in reverse order.
    let mut upper: Vec<usize> = vec![0usize; n];
    let mut utop = 0;
    let mut j = n;
    while j > 0 {
        j -= 1;
        let si = idx[j];
        loop {
            if utop < 2 {
                break;
            }
            let p1 = upper[utop - 2];
            let p2 = upper[utop - 1];
            let cr = turn(points, p1, p2, si);
            if cr > 0.0 {
                break;
            }
            utop -= 1;
        }
        upper[utop] = si;
        utop += 1;
    }

    // Concatenate lower[0..ltop-1] then upper[0..utop-1] into coordinate pairs.
    let mut result: Vec<f64> = Vec::new();
    let mut t = 0;
    while t + 1 < ltop {
        let si = lower[t];
        result.push(points[2 * si]);
        result.push(points[2 * si + 1]);
        t += 1;
    }
    let mut u = 0;
    while u + 1 < utop {
        let si = upper[u];
        result.push(points[2 * si]);
        result.push(points[2 * si + 1]);
        u += 1;
    }
    result
}
