//! Singly-linked list as a recursive ADT.
//!
//! A cons-list built from a recursive enum.  `Box` is erased by Aeneas during
//! translation, yielding a pure inductive `LinkedList` type in Lean.  All operations
//! are structural recursions over the list spine.

/// A singly-linked list node: either empty, or a head value plus a tail.
pub enum LinkedList {
    Nil,
    Cons(i32, Box<LinkedList>),
}

/// Prepend `v` to `list`.
pub fn push_front(list: LinkedList, v: i32) -> LinkedList {
    LinkedList::Cons(v, Box::new(list))
}

/// Remove and return the head, together with the remaining list.
pub fn pop_front(list: LinkedList) -> (LinkedList, Option<i32>) {
    match list {
        LinkedList::Nil => (LinkedList::Nil, None),
        LinkedList::Cons(head, tail) => (*tail, Some(head)),
    }
}

/// Append `b` after `a` (structural recursion over `a`).
pub fn append(a: LinkedList, b: LinkedList) -> LinkedList {
    match a {
        LinkedList::Nil => b,
        LinkedList::Cons(head, tail) => LinkedList::Cons(head, Box::new(append(*tail, b))),
    }
}

/// Reverse `list` (tail-recursive accumulator form).
pub fn reverse(list: LinkedList) -> LinkedList {
    reverse_acc(list, LinkedList::Nil)
}

fn reverse_acc(list: LinkedList, acc: LinkedList) -> LinkedList {
    match list {
        LinkedList::Nil => acc,
        LinkedList::Cons(head, tail) => reverse_acc(*tail, LinkedList::Cons(head, Box::new(acc))),
    }
}

/// Length accumulator: count the spine while appending to `n`.
fn length_acc(list: LinkedList, n: usize) -> usize {
    match list {
        LinkedList::Nil => n,
        LinkedList::Cons(_, tail) => length_acc(*tail, n + 1),
    }
}

/// Length of `list`.
pub fn length(list: LinkedList) -> usize {
    length_acc(list, 0)
}
