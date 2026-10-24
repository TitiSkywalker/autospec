import LeanMode.graph_general_match.GraphGeneralMatch
import Mathlib

open Aeneas Aeneas.Std Result
open graph_general_match

set_option linter.unusedVariables false

/-!
# Specification: Maximum Matching in General Graphs (Edmonds' Blossom)

`blossom n adj` returns `ok (size, matched)` where `adj` is a flat, row-major,
symmetric `n × n` 0/1 adjacency matrix of a simple undirected graph (no
self-loops) and `matched` is a flat `n × n` 0/1 matrix marking a matching
(symmetric, at most one partner per vertex).  `size` is the matching
cardinality.

The theorems assert that the returned matrix is a matching, that `size` equals
its cardinality, and that it is a *maximum* matching (Berge's augmenting-path
theorem for general graphs, with the blossom lemma, is the subtle part).
-/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Int`. -/
def toMatrix (s : Slice Int) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Int :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    s[i.val * n + j.val]'h_bound

/-- `matched` is a matching of the graph `adj`: 0/1 symmetric entries, ones
    only on real edges, at most one partner per vertex. -/
def is_matching {n : Nat} (adj matched : Fin n → Fin n → Int) : Prop :=
  (∀ u v : Fin n, matched u v = 0 ∨ matched u v = 1) ∧
  (∀ u v : Fin n, matched u v = matched v u) ∧
  (∀ u v : Fin n, matched u v = 1 → adj u v = 1) ∧
  (∀ u : Fin n, (∑ v : Fin n, matched u v) ≤ 1)

/-- The cardinality of a matching (number of matched edges). -/
def matching_size {n : Nat} (matched : Fin n → Fin n → Int) : Int :=
  (∑ u : Fin n, ∑ v : Fin n, matched u v) / 2

/-- `matched` is a maximum matching: it is a matching, and no matching is
    larger. -/
def is_max_matching {n : Nat} (adj matched : Fin n → Fin n → Int) : Prop :=
  is_matching adj matched ∧
  ∀ other : Fin n → Fin n → Int,
    is_matching adj other → matching_size other ≤ matching_size matched


/-! ## Specification theorems -/

/-- `blossom` returns a matching matrix of length `n * n`. -/
theorem blossom_length
    (n : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h : blossom n adj = ok (size, matched)) :
    (alloc.vec.Vec.deref matched).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a valid matching. -/
theorem blossom_is_matching
    (n : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix adj n.val h_adj i j =
                                 toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
       toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : blossom n adj = ok (size, matched)) :
    is_matching (toMatrix adj n.val h_adj)
                (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- The returned `size` equals the cardinality of the returned matching. -/
theorem blossom_size
    (n : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : blossom n adj = ok (size, matched)) :
    size = matching_size (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- Optimality: the returned matching is maximum. -/
theorem blossom_maximal
    (n : Std.Usize) (adj : Slice Int) (size : Int) (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix adj n.val h_adj i j =
                                 toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
       toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : blossom n adj = ok (size, matched)) :
    is_max_matching (toMatrix adj n.val h_adj)
                    (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry
