import LeanMode.graph_min_mean_cycle.GraphMinMeanCycle
import Mathlib

open Aeneas Aeneas.Std Result
open graph_min_mean_cycle

set_option linter.unusedVariables false

/-!
# Specification: Karp's Minimum Mean Cycle

`graph_min_mean_cycle n weight` returns `ok (core.result.Result.Ok (num, den))`
when the directed graph has a cycle, and `ok (core.result.Result.Err ())` when
it has none.

The graph is a flat, row-major `n × n` weight matrix: `weight[idx(u, n, v)]` is
the weight of the directed edge `u → v`, or the sentinel `INF` when there is no
such edge.  Weights may be negative.

The *mean* of a directed cycle `C = v₀ → v₁ → … → v_{L-1} → v₀` (distinct
vertices) is `(Σ_{i} w(vᵢ, v_{i+1})) / L`.  Because this number is in general
rational, the function does not divide: it returns an exact fraction
`(num, den)` with `den > 0` whose value is the minimum cycle mean
(`num / den = min over cycles C of mean C`).  The specification pins this down
by saying that `num / den` is a lower bound on *every* cycle mean, and that it
is *attained* by some cycle.
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

/-- The total weight of a closed walk through `L` in order: the sum of the
    consecutive edge weights, plus the closing edge from the last vertex back
    to the first. -/
def cycle_weight {n : Nat} (w : Fin n → Fin n → Int) (L : List (Fin n)) : Int :=
  ((cyclic_pairs L).map (λ p : Fin n × Fin n => w p.1 p.2)).sum +
    match L.head?, L.getLast? with
    | some a, some b => w b a
    | _, _ => 0

/-- `L` is a directed cycle: a non-empty list of *distinct* vertices whose
    consecutive edges (including the closing edge) all exist, i.e. have weight
    strictly below the `INF` "no edge" sentinel (a finite diagonal entry is a
    weight-`w` self-loop, matching the implementation's edge test
    `w < INF`). -/
def IsCycle {n : Nat} (w : Fin n → Fin n → Int) (L : List (Fin n)) : Prop :=
  1 ≤ L.length ∧ L.Nodup ∧
  (∀ p : Fin n × Fin n, p ∈ cyclic_pairs L → w p.1 p.2 < INF) ∧
  (match L.head?, L.getLast? with
   | some a, some b => w b a < INF
   | _, _ => False)

/-- `num / den` (with `den > 0`) is the minimum cycle mean: it is a lower bound
    on the mean of every cycle, and some cycle attains it. -/
def IsMinMean {n : Nat} (w : Fin n → Fin n → Int) (num den : Int) : Prop :=
  0 < den ∧
  (∀ L : List (Fin n),
    IsCycle w L → num * (L.length : Int) ≤ den * cycle_weight w L) ∧
  (∃ L : List (Fin n),
    IsCycle w L ∧ num * (L.length : Int) = den * cycle_weight w L)


/-! ## Specification theorems -/

/-- Correctness: on `Ok (num, den)`, `num / den` is the minimum mean over all
    directed cycles. -/
theorem min_mean_cycle_correct
    (n : Std.Usize) (weight : Slice Int) (num den : Int)
    (h_weight : weight.length = n.val * n.val)
    (h : min_mean_cycle n weight = ok (core.result.Result.Ok (num, den))) :
    IsMinMean (toMatrix weight n.val h_weight) num den := by
  sorry

/-- Completeness: on `Err`, the graph has no cycle. -/
theorem min_mean_cycle_no_cycle
    (n : Std.Usize) (weight : Slice Int)
    (h_weight : weight.length = n.val * n.val)
    (h : min_mean_cycle n weight = ok (core.result.Result.Err ())) :
    ¬ ∃ L : List (Fin n.val), IsCycle (toMatrix weight n.val h_weight) L := by
  sorry
