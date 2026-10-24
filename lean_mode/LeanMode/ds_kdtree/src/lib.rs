//! 2-D KD-Tree with axis-aligned bounding boxes.
//!
//! A binary tree over integer points `(x, y)` with an integer value `val`.
//! Insertion alternates the split dimension on depth (`x`, then `y`, …), and
//! each node stores the bounding box (min/max of `x` and `y` over its whole
//! subtree) plus the subtree value sum.  `kd_query` answers a rectangle sum
//! query by pruning subtrees whose bounding box is disjoint from the query
//! rectangle, and returning `sum` when the box is fully contained.

pub struct Node {
    pub x: i32,
    pub y: i32,
    pub val: i32,
    pub left: Option<Box<Node>>,
    pub right: Option<Box<Node>>,
    pub minx: i32,
    pub maxx: i32,
    pub miny: i32,
    pub maxy: i32,
    pub sum: i32,
}

// Integer comparison helpers (hand-expanded to return `bool`).
fn ilt(a: i32, b: i32) -> bool {
    if a < b { true } else { false }
}
fn ile(a: i32, b: i32) -> bool {
    if a <= b { true } else { false }
}
fn igt(a: i32, b: i32) -> bool {
    if a > b { true } else { false }
}
fn imin(a: i32, b: i32) -> i32 {
    if a < b { a } else { b }
}
fn imax(a: i32, b: i32) -> i32 {
    if a > b { a } else { b }
}
fn iadd(a: i32, b: i32) -> i32 {
    a + b
}
fn both(a: bool, b: bool) -> bool {
    if a { b } else { false }
}
fn either(a: bool, b: bool) -> bool {
    if a { true } else { b }
}

fn kd_insert_rec(t: Option<Box<Node>>, x: i32, y: i32, val: i32, use_x: bool) -> Option<Box<Node>> {
    match t {
        None => Some(Box::new(Node {
            x,
            y,
            val,
            left: None,
            right: None,
            minx: x,
            maxx: x,
            miny: y,
            maxy: y,
            sum: val,
        })),
        Some(n) => {
            let go_left = if use_x { ilt(x, n.x) } else { ilt(y, n.y) };
            let minx = imin(n.minx, x);
            let maxx = imax(n.maxx, x);
            let miny = imin(n.miny, y);
            let maxy = imax(n.maxy, y);
            let sum = iadd(n.sum, val);
            if go_left {
                let new_left = kd_insert_rec(n.left, x, y, val, !use_x);
                Some(Box::new(Node {
                    x: n.x,
                    y: n.y,
                    val: n.val,
                    left: new_left,
                    right: n.right,
                    minx,
                    maxx,
                    miny,
                    maxy,
                    sum,
                }))
            } else {
                let new_right = kd_insert_rec(n.right, x, y, val, !use_x);
                Some(Box::new(Node {
                    x: n.x,
                    y: n.y,
                    val: n.val,
                    left: n.left,
                    right: new_right,
                    minx,
                    maxx,
                    miny,
                    maxy,
                    sum,
                }))
            }
        }
    }
}

/// Insert a point `(x, y)` with value `val` into the tree.
pub fn kd_insert(t: Option<Box<Node>>, x: i32, y: i32, val: i32) -> Option<Box<Node>> {
    kd_insert_rec(t, x, y, val, true)
}

fn kd_query_rec(t: &Option<Box<Node>>, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    match t {
        None => 0,
        Some(n) => {
            let disjoint = either(either(igt(n.minx, x2), ilt(n.maxx, x1)), either(igt(n.miny, y2), ilt(n.maxy, y1)));
            if disjoint {
                0
            } else {
                let contained = both(both(ile(x1, n.minx), ile(n.maxx, x2)), both(ile(y1, n.miny), ile(n.maxy, y2)));
                if contained {
                    n.sum
                } else {
                    let left_contrib = kd_query_rec(&n.left, x1, y1, x2, y2);
                    let right_contrib = kd_query_rec(&n.right, x1, y1, x2, y2);
                    let child_sum = iadd(left_contrib, right_contrib);
                    let in_node = both(both(ile(x1, n.x), ile(n.x, x2)), both(ile(y1, n.y), ile(n.y, y2)));
                    if in_node {
                        iadd(n.val, child_sum)
                    } else {
                        child_sum
                    }
                }
            }
        }
    }
}

/// Sum of `val` over all points with `x1 ≤ x ≤ x2` and `y1 ≤ y ≤ y2`.
pub fn kd_query(t: &Option<Box<Node>>, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
    kd_query_rec(t, x1, y1, x2, y2)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Reference: a flat list of points.
    fn brute_query(pts: &Vec<(i32, i32, i32)>, x1: i32, y1: i32, x2: i32, y2: i32) -> i32 {
        let mut s = 0;
        let mut i: usize = 0;
        while i < pts.len() {
            let (x, y, v) = pts[i];
            if x1 <= x && x <= x2 && y1 <= y && y <= y2 {
                s = s + v;
            }
            i += 1;
        }
        s
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state
    }

    #[test]
    fn basic_query() {
        let mut t: Option<Box<Node>> = None;
        t = kd_insert(t, 2, 3, 5);
        t = kd_insert(t, 5, 4, 7);
        t = kd_insert(t, 9, 6, 11);
        t = kd_insert(t, 4, 7, 13);
        t = kd_insert(t, 8, 1, 17);
        // rectangle [0,10] x [0,10] contains all → 5+7+11+13+17 = 53
        assert_eq!(kd_query(&t, 0, 0, 10, 10), 53);
        // rectangle [3,7] x [3,7] contains (5,4)=7 and (4,7)=13 → 20
        assert_eq!(kd_query(&t, 3, 3, 7, 7), 20);
        // rectangle [0,0] x [0,0] empty → 0
        assert_eq!(kd_query(&t, 0, 0, 0, 0), 0);
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 777;
        let mut t: Option<Box<Node>> = None;
        let mut pts: Vec<(i32, i32, i32)> = Vec::new();
        let mut i: usize = 0;
        while i < 200 {
            let r = lcg(&mut state);
            let x = ((r >> 4) % 40) as i32;
            let y = ((r >> 12) % 40) as i32;
            let v = ((r >> 20) % 100) as i32;
            t = kd_insert(t, x, y, v);
            pts.push((x, y, v));
            i += 1;
        }
        let mut ops: usize = 0;
        while ops < 2000 {
            let r = lcg(&mut state);
            let x1 = ((r >> 4) % 50) as i32;
            let y1 = ((r >> 12) % 50) as i32;
            let x2 = x1 + ((r >> 20) % 15) as i32;
            let y2 = y1 + ((r >> 24) % 15) as i32;
            assert_eq!(kd_query(&t, x1, y1, x2, y2), brute_query(&pts, x1, y1, x2, y2));
            ops += 1;
        }
    }
}
