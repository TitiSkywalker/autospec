import LeanMode.graph_weighted_matching.GraphWeightedMatching
import Mathlib

open Aeneas Aeneas.Std Result
open graph_weighted_matching

set_option linter.unusedVariables false

/-!
# Specification: Maximum-Weight Matching in General Graphs

`max_weight_matching n adj w` returns `ok (total, matched)` where

* `adj` is a flat, row-major, symmetric `n × n` **0/1 adjacency matrix** of a
  simple undirected graph (no self-loops), and
* `w` is a flat, row-major, symmetric `n × n` weight matrix,

and `matched` is a flat, row-major `n × n` 0/1 matrix marking a matching
(symmetric, at most one partner per vertex, ones only on real edges), whose
total weight is `total`.

The specification below is stated **declaratively** — it never mentions the
subset DP that the implementation runs.  The theorems say that the returned
matrix is a valid `is_matching` of the graph, that `total` is its `weight`, that
it is **globally optimal** (no other matching of the same graph has larger
weight), and (Theorem `max_weight_matching_dual_certificate`) that it admits an
**odd-set strong-duality certificate**.

## Duality

The planning doc sketched a strong-duality certificate with **vertex-only**
potentials `y` (`∃ y, y_u + y_v ≥ w_uv ∧ weight w m = ∑ y`).  That clause is
*false* for non-bipartite graphs: on the unit triangle the maximum matching has
weight `1`, whereas the vertex-only dual LP (`y_u + y_v ≥ w_uv`) has optimum
`3/2`.  The *correct* LP dual of Edmonds' matching polytope has, in addition to
the vertex potentials `y ≥ 0`, **odd-set (blossom) dual variables** `z_B ≥ 0`
for odd `B`:

  `min  Σ_v y_v + Σ_{B odd} ((|B|−1)/2)·z_B`
  ` s.t. y_u + y_v + Σ_{B odd, B ∋ u,v} z_B ≥ w_uv`  for every edge `uv`.

By strong duality (the matching polytope is integral, so its LP optimum equals
the maximum matching weight; extreme duals are half-integral — hence `ℚ`), for
an optimal matching `M` there are feasible `y`, `z` whose objective equals
`weight w M` and for which every matched edge is tight.  This is
`max_weight_matching_dual_certificate` below, with `IsDualFeasible` / `dualValue`
formalising the constraint system and the objective.
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

/-- `matched` is a matching of the graph `adj`: 0/1 entries, symmetric, ones
    only on real edges, at most one partner per vertex. -/
def is_matching {n : Nat} (adj matched : Fin n → Fin n → Int) : Prop :=
  (∀ u v : Fin n, matched u v = 0 ∨ matched u v = 1) ∧
  (∀ u v : Fin n, matched u v = matched v u) ∧
  (∀ u v : Fin n, matched u v = 1 → adj u v = 1) ∧
  (∀ u : Fin n, (∑ v : Fin n, matched u v) ≤ 1)

/-- The total weight of a matching: each matched edge is counted once (the
    half-sum is exact for a symmetric `matched` and symmetric `w`). -/
def weight {n : Nat} (w matched : Fin n → Fin n → Int) : Int :=
  (∑ u : Fin n, ∑ v : Fin n, matched u v * w u v) / 2

/-- `matched` is a *maximum-weight* matching of the graph `adj` with weights
    `w`: it is a matching, and no other matching has larger weight. -/
def is_max_weight_matching {n : Nat}
    (adj w matched : Fin n → Fin n → Int) : Prop :=
  is_matching adj matched ∧
  ∀ other : Fin n → Fin n → Int,
    is_matching adj other → weight w other ≤ weight w matched

/-- Feasibility of the **odd-set (Edmonds / Cunningham–Marsh) LP dual** of
    maximum-weight matching: vertex duals `y ≥ 0`, odd-set duals `z ≥ 0`
    vanishing on even sets, and for every edge `uv` the reduced-cost inequality
    `y u + y v + Σ_{B odd, B ∋ u,v} z B ≥ w u v`.

    The LP pairs each edge with the odd sets containing *both* its endpoints.
    Quantifying `z` over *all* `B : Finset (Fin n)` while requiring `z B = 0` for
    even `B` is harmless: a set of size `< 2` can never contain a pair, and the
    coefficient `((|B|−1)/2)` is `0` when `|B| = 1`. -/
def IsDualFeasible {n : Nat} (adj w : Fin n → Fin n → Int)
    (y : Fin n → ℚ) (z : Finset (Fin n) → ℚ) : Prop :=
  (∀ v, 0 ≤ y v) ∧
  (∀ B, 0 ≤ z B) ∧
  (∀ B, B.card % 2 = 0 → z B = 0) ∧
  (∀ u v, adj u v = 1 →
    (w u v : ℚ) ≤ y u + y v +
      (Finset.univ.filter (fun B => u ∈ B ∧ v ∈ B ∧ B.card % 2 = 1)).sum
        (fun B => z B))

/-- The dual objective `Σ_v y_v + Σ_{B odd} ((|B|−1)/2)·z_B`. -/
def dualValue {n : Nat} (y : Fin n → ℚ) (z : Finset (Fin n) → ℚ) : ℚ :=
  (∑ v : Fin n, y v) +
    (Finset.univ.filter (fun B => B.card % 2 = 1)).sum
      (fun B => (((B.card - 1 : Nat) : ℚ) / 2) * z B)

/-! ## Specification theorems -/

/-- `max_weight_matching` returns a matching matrix of length `n * n`. -/
theorem max_weight_matching_length
    (n : Std.Usize) (adj w : Slice Int) (total : Int)
    (matched : alloc.vec.Vec Int)
    (h_n : n.val ≤ 20)
    (h : max_weight_matching n adj w = ok (total, matched)) :
    (alloc.vec.Vec.deref matched).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a valid matching of the graph.  The
    hypotheses say `adj` is a symmetric 0/1 `n × n` matrix with zero diagonal
    (a simple undirected graph). -/
theorem max_weight_matching_is_matching
    (n : Std.Usize) (adj w : Slice Int) (total : Int)
    (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_n : n.val ≤ 20)
    (h_symm : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : max_weight_matching n adj w = ok (total, matched)) :
    is_matching (toMatrix adj n.val h_adj)
                (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- The returned `total` equals the total weight of the returned matching.
    `h_wsymm` says the weight matrix is symmetric (an undirected weighted
    graph). -/
theorem max_weight_matching_weight
    (n : Std.Usize) (adj w : Slice Int) (total : Int)
    (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_w : w.length = n.val * n.val)
    (h_n : n.val ≤ 20)
    (h_wsymm : ∀ i j : Fin n.val,
      toMatrix w n.val h_w i j = toMatrix w n.val h_w j i)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : max_weight_matching n adj w = ok (total, matched)) :
    total = weight (toMatrix w n.val h_w)
                   (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- Optimality: the returned matching has maximum total weight among all
    matchings of the graph.  This is the whole "validity + global optimality"
    primary specification, stated directly as `is_max_weight_matching`. -/
theorem max_weight_matching_optimal
    (n : Std.Usize) (adj w : Slice Int) (total : Int)
    (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_w : w.length = n.val * n.val)
    (h_n : n.val ≤ 20)
    (h_symm : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h_wsymm : ∀ i j : Fin n.val,
      toMatrix w n.val h_w i j = toMatrix w n.val h_w j i)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : max_weight_matching n adj w = ok (total, matched)) :
    is_max_weight_matching (toMatrix adj n.val h_adj)
                           (toMatrix w n.val h_w)
                           (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- **Odd-set strong-duality certificate (Edmonds / Cunningham–Marsh).**

    The returned matching is optimal, so — by strong duality of Edmonds'
    matching polytope (whose extreme duals are half-integral; hence the `ℚ`
    duals) — there exist feasible duals `y` (per vertex) and `z` (per odd set)
    whose objective `dualValue y z` equals the returned matching's `weight`, and
    for which every matched edge is *tight* (`y u + y v + Σ z_B = w u v`).

    This is the vertex-potentials-plus-blossoms dual the planning doc asked for;
    the doc's *vertex-only* clause is false for non-bipartite graphs (the unit
    triangle has max matching weight `1` while the vertex-only dual optimum is
    `3/2`), which is why the odd-set variables `z` are needed. -/
theorem max_weight_matching_dual_certificate
    (n : Std.Usize) (adj w : Slice Int) (total : Int)
    (matched : alloc.vec.Vec Int)
    (h_adj : adj.length = n.val * n.val)
    (h_w : w.length = n.val * n.val)
    (h_n : n.val ≤ 20)
    (h_symm : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h_wsymm : ∀ i j : Fin n.val,
      toMatrix w n.val h_w i j = toMatrix w n.val h_w j i)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : max_weight_matching n adj w = ok (total, matched)) :
    ∃ (y : Fin n.val → ℚ) (z : Finset (Fin n.val) → ℚ),
      IsDualFeasible (toMatrix adj n.val h_adj) (toMatrix w n.val h_w) y z ∧
      (∀ u v, toMatrix (alloc.vec.Vec.deref matched) n.val h_matched u v = 1 →
        (toMatrix w n.val h_w u v : ℚ) =
          y u + y v +
            (Finset.univ.filter (fun B => u ∈ B ∧ v ∈ B ∧ B.card % 2 = 1)).sum
              (fun B => z B)) ∧
      dualValue y z =
        (weight (toMatrix w n.val h_w)
                (toMatrix (alloc.vec.Vec.deref matched) n.val h_matched) : ℚ) := by
  sorry
