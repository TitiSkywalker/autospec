//! Red-Black Tree (Okasaki-style, purely functional).
//!
//! A persistent red-black tree using the "double-black" formulation of
//! Chris Okasaki's algorithm.  Insert reuses the classic 4-case `balance`;
//! delete threads a transient *double-black* state through `del` + `fix_db`
//! and re-blackens the root with `make_black`.  `Box` is erased by Aeneas,
//! yielding a pure inductive `Tree` in Lean.

/// The color of a red-black node.  `DoubleBlack` only occurs transiently
/// while deleting; a valid tree never contains it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Black,
    DoubleBlack,
}

/// A red-black tree.  `Empty` is the NIL leaf; `DoubleBlackEmpty` is a
/// transient doubly-black leaf produced during deletion.
pub enum Tree {
    Empty,
    Node(Color, Box<Tree>, i32, Box<Tree>),
    DoubleBlackEmpty,
}

/// Insert `v` into the tree (a set: inserting an existing key leaves it
/// unchanged).  The result is a valid red-black tree with a black root.
pub fn rbt_insert(t: Tree, v: i32) -> Tree {
    make_black(ins(v, t))
}

/// Delete `v` from the tree.  Deleting an absent key leaves the set of keys
/// unchanged (the coloring may still change).  The result is a valid
/// red-black tree with a black root.
pub fn rbt_delete(t: Tree, v: i32) -> Tree {
    make_black(del(v, t))
}

/// Search for `v` in the tree.
pub fn rbt_find(t: &Tree, v: i32) -> bool {
    match t {
        Tree::Empty => false,
        Tree::DoubleBlackEmpty => false,
        Tree::Node(_, l, k, r) => {
            if v < *k {
                rbt_find(&**l, v)
            } else if v > *k {
                rbt_find(&**r, v)
            } else {
                true
            }
        }
    }
}

/// Recolor the root black (identity on leaves).
fn make_black(t: Tree) -> Tree {
    match t {
        Tree::Node(_, l, k, r) => Tree::Node(Color::Black, l, k, r),
        _ => Tree::Empty,
    }
}

/// Which Okasaki rebalance case applies to `(l, r)`?  0 = none (identity).
fn right_case(r: &Tree) -> i32 {
    match r {
        Tree::Node(Color::Red, b, _, r1) => match &**r1 {
            Tree::Node(Color::Red, _, _, _) => 3,
            _ => match &**b {
                Tree::Node(Color::Red, _, _, _) => 4,
                _ => 0,
            },
        },
        _ => 0,
    }
}

fn balance_case(color: Color, l: &Tree, r: &Tree) -> i32 {
    match color {
        Color::Black => match l {
            Tree::Node(Color::Red, l1, _, c) => match &**l1 {
                Tree::Node(Color::Red, _, _, _) => 1,
                _ => match &**c {
                    Tree::Node(Color::Red, _, _, _) => 2,
                    _ => right_case(r),
                },
            },
            _ => right_case(r),
        },
        _ => 0,
    }
}

/// The classic Okasaki 4-case rebalance, applied to a node with color
/// `color`, left subtree `l`, key `k`, and right subtree `r`.  Only a black
/// root with a red-red violation among its grandchildren is rearranged.
fn balance(color: Color, l: Tree, k: i32, r: Tree) -> Tree {
    let case = balance_case(color, &l, &r);
    if case == 1 {
        // case 1: l = Node Red (Node Red a x b) y c
        match (l, r) {
            (Tree::Node(Color::Red, l1, y, c), d) => match *l1 {
                Tree::Node(Color::Red, a, x, b) =>
                    Tree::Node(Color::Red,
                        Box::new(Tree::Node(Color::Black, a, x, b)),
                        y,
                        Box::new(Tree::Node(Color::Black, c, k, Box::new(d)))),
                _ => Tree::Empty,
            },
            _ => Tree::Empty,
        }
    } else if case == 2 {
        // case 2: l = Node Red l1 y (Node Red b y2 c2)
        match (l, r) {
            (Tree::Node(Color::Red, l1, y, c), d) => match *c {
                Tree::Node(Color::Red, b, y2, c2) =>
                    Tree::Node(Color::Red,
                        Box::new(Tree::Node(Color::Black, l1, y, b)),
                        y2,
                        Box::new(Tree::Node(Color::Black, c2, k, Box::new(d)))),
                _ => Tree::Empty,
            },
            _ => Tree::Empty,
        }
    } else if case == 3 {
        // case 3: r = Node Red b y (Node Red c z d)
        match (l, r) {
            (a, Tree::Node(Color::Red, b, y, r1)) => match *r1 {
                Tree::Node(Color::Red, c, z, d) =>
                    Tree::Node(Color::Red,
                        Box::new(Tree::Node(Color::Black, Box::new(a), k, b)),
                        y,
                        Box::new(Tree::Node(Color::Black, c, z, d))),
                _ => Tree::Empty,
            },
            _ => Tree::Empty,
        }
    } else if case == 4 {
        // case 4: r = Node Red (Node Red b1 y2 c1) y r1
        match (l, r) {
            (a, Tree::Node(Color::Red, b, y, r1)) => match *b {
                Tree::Node(Color::Red, b1, y2, c1) =>
                    Tree::Node(Color::Red,
                        Box::new(Tree::Node(Color::Black, Box::new(a), k, b1)),
                        y2,
                        Box::new(Tree::Node(Color::Black, c1, y, r1))),
                _ => Tree::Empty,
            },
            _ => Tree::Empty,
        }
    } else {
        Tree::Node(color, Box::new(l), k, Box::new(r))
    }
}

/// Insert helper; may leave a red root (fixed up by `make_black`).
fn ins(x: i32, t: Tree) -> Tree {
    match t {
        Tree::Empty =>
            Tree::Node(Color::Red, Box::new(Tree::Empty), x, Box::new(Tree::Empty)),
        Tree::Node(color, l, k, r) => {
            if x < k {
                balance(color, ins(x, *l), k, *r)
            } else if x > k {
                balance(color, *l, k, ins(x, *r))
            } else {
                Tree::Node(color, l, k, r)
            }
        }
        Tree::DoubleBlackEmpty => Tree::DoubleBlackEmpty,
    }
}

/// Delete helper; may introduce a transient double-black node, removed by
/// `fix_db` / `make_black`.
fn del(x: i32, t: Tree) -> Tree {
    match t {
        Tree::Empty => Tree::Empty,
        Tree::Node(color, l, k, r) => {
            if x < k {
                fix_db(color, del(x, *l), k, *r)
            } else if x > k {
                fix_db(color, *l, k, del(x, *r))
            } else {
                if is_empty(&*l) {
                    if color == Color::Black { shift_black(*r) } else { *r }
                } else if is_empty(&*r) {
                    if color == Color::Black { shift_black(*l) } else { *l }
                } else {
                    let m = min_key(&*r);
                    fix_db(color, *l, m, del(m, *r))
                }
            }
        }
        Tree::DoubleBlackEmpty => Tree::DoubleBlackEmpty,
    }
}

/// Leftmost key of a non-empty tree.
fn min_key(t: &Tree) -> i32 {
    match t {
        Tree::Node(_, l, k, _) => {
            match &**l {
                Tree::Empty => *k,
                _ => min_key(&**l),
            }
        }
        _ => 0,
    }
}

/// `Empty` is the only "empty" tree (`DoubleBlackEmpty` is transient, not
/// a real empty leaf here).
fn is_empty(t: &Tree) -> bool {
    match t {
        Tree::Empty => true,
        _ => false,
    }
}

/// Add one unit of blackness: black → double-black, red → black, and the
/// empty leaf toggles between `Empty` and `DoubleBlackEmpty`.
fn shift_black(t: Tree) -> Tree {
    match t {
        Tree::Node(Color::Black, l, k, r) => Tree::Node(Color::DoubleBlack, l, k, r),
        Tree::Node(_, l, k, r) => Tree::Node(Color::Black, l, k, r),
        Tree::Empty => Tree::DoubleBlackEmpty,
        Tree::DoubleBlackEmpty => Tree::Empty,
    }
}

/// Fix a black-height violation where `a` (left) is doubly black and `sib`
/// (right) is a normal sibling.
fn fix_db_right(color: Color, a: Tree, k: i32, sib: Tree) -> Tree {
    match sib {
        Tree::Node(Color::Black, lb, y, rb) => match *lb {
            Tree::Node(Color::Red, b, y2, c) =>
                // sibling black, its left child red
                Tree::Node(color,
                    Box::new(Tree::Node(Color::Black, Box::new(shift_black(a)), k, b)),
                    y2,
                    Box::new(Tree::Node(Color::Black, c, y, rb))),
            _ => match *rb {
                Tree::Node(Color::Red, c, z, d) =>
                    // sibling black, its right child red
                    Tree::Node(color,
                        Box::new(Tree::Node(Color::Black, Box::new(shift_black(a)), k, lb)),
                        y,
                        Box::new(Tree::Node(Color::Black, c, z, d))),
                _ =>
                    // sibling black, both children black: propagate blackness up
                    shift_black(Tree::Node(color,
                        Box::new(shift_black(a)), k,
                        Box::new(Tree::Node(Color::Red, lb, y, rb)))),
            },
        },
        Tree::Node(Color::Red, b, y, c) => {
            if color == Color::Black {
                // sibling red: rotate so the sibling becomes black, then retry
                fix_db(Color::Black, fix_db(Color::Red, a, k, *b), y, *c)
            } else {
                Tree::Node(color, Box::new(a), k,
                    Box::new(Tree::Node(Color::Red, b, y, c)))
            }
        }
        _ => Tree::Node(color, Box::new(a), k, Box::new(sib)),
    }
}

/// Fix a black-height violation where `a` (right) is doubly black and `sib`
/// (left) is a normal sibling.
fn fix_db_left(color: Color, sib: Tree, k: i32, a: Tree) -> Tree {
    match sib {
        Tree::Node(Color::Black, la, x, rb) => match *rb {
            Tree::Node(Color::Red, b, y, c) =>
                // sibling black, its right child red
                Tree::Node(color,
                    Box::new(Tree::Node(Color::Black, la, x, b)),
                    y,
                    Box::new(Tree::Node(Color::Black, c, k, Box::new(shift_black(a))))),
            _ => match *la {
                Tree::Node(Color::Red, a1, x2, b1) =>
                    // sibling black, its left child red
                    Tree::Node(color,
                        Box::new(Tree::Node(Color::Black, a1, x2, b1)),
                        x,
                        Box::new(Tree::Node(Color::Black, rb, k, Box::new(shift_black(a))))),
                _ =>
                    // sibling black, both children black: propagate blackness up
                    shift_black(Tree::Node(color,
                        Box::new(Tree::Node(Color::Red, la, x, rb)), k,
                        Box::new(shift_black(a)))),
            },
        },
        Tree::Node(Color::Red, a1, x, b1) => {
            if color == Color::Black {
                fix_db(Color::Black, *a1, x, fix_db(Color::Red, *b1, k, a))
            } else {
                Tree::Node(color, Box::new(Tree::Node(Color::Red, a1, x, b1)), k,
                    Box::new(a))
            }
        }
        _ => Tree::Node(color, Box::new(sib), k, Box::new(a)),
    }
}

/// Restore the black-height invariant after `del` produced a doubly-black
/// subtree on one side.
fn fix_db(color: Color, l: Tree, k: i32, r: Tree) -> Tree {
    match l {
        Tree::Node(Color::DoubleBlack, la, lk, lr) =>
            fix_db_right(color, Tree::Node(Color::DoubleBlack, la, lk, lr), k, r),
        Tree::DoubleBlackEmpty =>
            fix_db_right(color, Tree::DoubleBlackEmpty, k, r),
        _ => match r {
            Tree::Node(Color::DoubleBlack, ra, rk, rr) =>
                fix_db_left(color, l, k, Tree::Node(Color::DoubleBlack, ra, rk, rr)),
            Tree::DoubleBlackEmpty =>
                fix_db_left(color, l, k, Tree::DoubleBlackEmpty),
            _ => Tree::Node(color, Box::new(l), k, Box::new(r)),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- invariant checker (native-only; not translated) --------------------

    fn black_height(t: &Tree) -> i32 {
        match t {
            Tree::Empty => 0,
            Tree::DoubleBlackEmpty => -1,
            Tree::Node(c, l, _, r) => {
                let hl = black_height(&**l);
                let hr = black_height(&**r);
                if hl < 0 || hr < 0 || hl != hr {
                    -1
                } else {
                    hl + if *c == Color::Black { 1 } else { 0 }
                }
            }
        }
    }

    fn no_red_red(t: &Tree) -> bool {
        match t {
            Tree::Empty => true,
            Tree::DoubleBlackEmpty => false,
            Tree::Node(c, l, _, r) => {
                let self_red = *c == Color::Red;
                let l_red = match &**l {
                    Tree::Node(cc, _, _, _) => *cc == Color::Red,
                    _ => false,
                };
                let r_red = match &**r {
                    Tree::Node(cc, _, _, _) => *cc == Color::Red,
                    _ => false,
                };
                let ok = if self_red {
                    !(l_red || r_red)
                } else {
                    true
                };
                ok && no_red_red(&**l) && no_red_red(&**r)
            }
        }
    }

    fn no_db(t: &Tree) -> bool {
        match t {
            Tree::Empty => true,
            Tree::DoubleBlackEmpty => false,
            Tree::Node(c, l, _, r) => {
                match c {
                    Color::DoubleBlack => false,
                    _ => no_db(&**l) && no_db(&**r),
                }
            }
        }
    }

    fn inorder(t: &Tree, acc: &mut Vec<i32>) {
        match t {
            Tree::Empty | Tree::DoubleBlackEmpty => {}
            Tree::Node(_, l, k, r) => {
                inorder(&**l, acc);
                acc.push(*k);
                inorder(&**r, acc);
            }
        }
    }

    fn is_sorted_unique(keys: &Vec<i32>) -> bool {
        let mut i: usize = 1;
        while i < keys.len() {
            if keys[i - 1] >= keys[i] {
                return false;
            }
            i += 1;
        }
        true
    }

    fn is_valid_rb(t: &Tree) -> bool {
        let root_black = match t {
            Tree::Empty => true,
            Tree::Node(c, _, _, _) => *c == Color::Black,
            Tree::DoubleBlackEmpty => false,
        };
        let mut ks: Vec<i32> = Vec::new();
        inorder(t, &mut ks);
        root_black && no_red_red(t) && no_db(t) && black_height(t) >= 0
            && is_sorted_unique(&ks)
    }

    // -- reference sorted set (Vec<i32>, unique) ----------------------------

    fn contains(v: &Vec<i32>, x: i32) -> bool {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i] == x {
                return true;
            }
            i += 1;
        }
        false
    }

    fn insert_sorted(v: &mut Vec<i32>, x: i32) {
        if contains(v, x) {
            return;
        }
        let mut i: usize = 0;
        while i < v.len() && v[i] < x {
            i += 1;
        }
        v.insert(i, x);
    }

    fn remove_sorted(v: &mut Vec<i32>, x: i32) {
        let mut i: usize = 0;
        while i < v.len() {
            if v[i] == x {
                v.remove(i);
                return;
            }
            i += 1;
        }
    }

    fn lcg(state: &mut u64) -> u64 {
        *state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *state
    }

    // -- tests -------------------------------------------------------------

    #[test]
    fn insert_find_basic() {
        let mut t = Tree::Empty;
        t = rbt_insert(t, 5);
        t = rbt_insert(t, 3);
        t = rbt_insert(t, 7);
        t = rbt_insert(t, 2);
        t = rbt_insert(t, 4);
        t = rbt_insert(t, 6);
        t = rbt_insert(t, 8);
        assert!(rbt_find(&t, 2));
        assert!(rbt_find(&t, 5));
        assert!(rbt_find(&t, 8));
        assert!(!rbt_find(&t, 1));
        assert!(!rbt_find(&t, 9));
        assert!(is_valid_rb(&t));
    }

    #[test]
    fn insert_duplicate_unchanged() {
        let mut t = Tree::Empty;
        t = rbt_insert(t, 10);
        t = rbt_insert(t, 10);
        t = rbt_insert(t, 10);
        assert!(rbt_find(&t, 10));
        assert!(is_valid_rb(&t));
    }

    #[test]
    fn delete_basic() {
        let mut t = Tree::Empty;
        let mut i = 0;
        while i < 20 {
            t = rbt_insert(t, i * 2);
            i += 1;
        }
        // delete every other key
        let mut i = 0;
        while i < 20 {
            if i % 2 == 0 {
                t = rbt_delete(t, i * 2);
            }
            i += 1;
        }
        assert!(is_valid_rb(&t));
        let mut i = 0;
        while i < 20 {
            let expect = i % 2 == 1;
            assert_eq!(rbt_find(&t, i * 2), expect);
            i += 1;
        }
    }

    #[test]
    fn delete_absent() {
        let mut t = Tree::Empty;
        t = rbt_insert(t, 5);
        t = rbt_insert(t, 3);
        t = rbt_delete(t, 99);
        assert!(rbt_find(&t, 5));
        assert!(rbt_find(&t, 3));
        assert!(!rbt_find(&t, 99));
        assert!(is_valid_rb(&t));
    }

    #[test]
    fn delete_to_empty() {
        let mut t = Tree::Empty;
        t = rbt_insert(t, 7);
        t = rbt_delete(t, 7);
        assert!(!rbt_find(&t, 7));
        assert!(is_valid_rb(&t));
    }

    #[test]
    fn randomized_differential() {
        let mut state: u64 = 12345;
        let mut t = Tree::Empty;
        let mut reference: Vec<i32> = Vec::new();
        let mut ops: usize = 0;
        while ops < 20000 {
            let r = lcg(&mut state);
            let val = ((r >> 8) % 201) as i32 - 100;
            if (r & 1) == 0 {
                t = rbt_insert(t, val);
                insert_sorted(&mut reference, val);
            } else {
                t = rbt_delete(t, val);
                remove_sorted(&mut reference, val);
            }
            assert!(is_valid_rb(&t));
            let mut probe = -100;
            while probe <= 100 {
                assert_eq!(rbt_find(&t, probe), contains(&reference, probe));
                probe += 1;
            }
            ops += 1;
        }
    }
}
