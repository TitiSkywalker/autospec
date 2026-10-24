import LeanMode.graph_minimum_cycle.GraphMinimumCycle
import Mathlib

open Aeneas Aeneas.Std Result
open graph_minimum_cycle

set_option linter.unusedVariables false

/-!
# Specification: Minimum Cycle (Floyd–Warshall girth)

`graph_minimum_cycle n weight` returns `ok (Ok min)` where `min` is the minimum total
weight of a simple cycle with at least three distinct vertices in the undirected
weighted graph given by the flat, row-major, symmetric `n × n` weight matrix
`weight`, or `ok (Err ())` when the graph has no cycle.

The subtle part of the proof is the Floyd "split" invariant: every simple cycle
is decomposed at its largest-indexed vertex `k` into two edges `(i, k)`, `(k, j)`
and a shortest `j ⇝ i` path using only vertices `< k`.
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

/-- Consecutive pairs of a vertex list (each vertex paired with its successor). -/
def cyclic_pairs {n : Nat} (L : List (Fin n)) : List (Fin n × Fin n) :=
  L.zip (L.drop 1)

/-- The total weight of a cycle: the sum of the weights of consecutive edges,
    plus the closing edge from the last vertex back to the first. -/
def cycle_weight {n : Nat} (w : Fin n → Fin n → Int) (L : List (Fin n)) : Int :=
  ((cyclic_pairs L).map (λ p : Fin n × Fin n => w p.1 p.2)).sum +
    match L.head?, L.getLast? with
    | some a, some b => w b a
    | _, _ => 0

/-- `L` is a simple cycle (at least three distinct vertices) whose consecutive
    edges (cyclically) all exist, i.e. have weight strictly below the `INF`
    "no edge" sentinel (zero-weight edges are allowed, matching the
    implementation). -/
def is_cycle {n : Nat} (w : Fin n → Fin n → Int) (L : List (Fin n)) : Prop :=
  3 ≤ L.length ∧ L.Nodup ∧
  (∀ p : Fin n × Fin n, p ∈ cyclic_pairs L → w p.1 p.2 < INF) ∧
  (match L.head?, L.getLast? with
   | some a, some b => w b a < INF
   | _, _ => False)

/-- `value` is the minimum cycle weight: attained by some cycle, and no cycle
    is smaller. -/
def is_min_cycle {n : Nat} (w : Fin n → Fin n → Int) (value : Int) : Prop :=
  (∃ L : List (Fin n), is_cycle w L ∧ cycle_weight w L = value) ∧
  (∀ L : List (Fin n), is_cycle w L → value ≤ cycle_weight w L)


/-! ## Specification theorems -/

/-- Soundness: on `Ok value`, `value` is the minimum cycle weight. -/
theorem graph_minimum_cycle_sound
    (n : Std.Usize) (weight : Slice Int) (value : Int)
    (h_weight : weight.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix weight n.val h_weight i j =
                                 toMatrix weight n.val h_weight j i)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix weight n.val h_weight i j)
    (h_diag : ∀ i : Fin n.val, toMatrix weight n.val h_weight i i = 0)
    (h : graph_minimum_cycle n weight = ok (core.result.Result.Ok value)) :
    is_min_cycle (toMatrix weight n.val h_weight) value := by
  sorry

/-- Completeness: on `Err`, the graph has no cycle. -/
theorem graph_minimum_cycle_complete
    (n : Std.Usize) (weight : Slice Int)
    (h_weight : weight.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix weight n.val h_weight i j =
                                 toMatrix weight n.val h_weight j i)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix weight n.val h_weight i j)
    (h_diag : ∀ i : Fin n.val, toMatrix weight n.val h_weight i i = 0)
    (h : graph_minimum_cycle n weight = ok (core.result.Result.Err ())) :
    ¬ ∃ L : List (Fin n.val), is_cycle (toMatrix weight n.val h_weight) L := by
  sorry
