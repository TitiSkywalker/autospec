import LeanMode.graph_floyd_warshall.GraphFloydWarshall
import Mathlib

open Aeneas Aeneas.Std Result
open graph_floyd_warshall

set_option linter.unusedVariables false

/-!
# Specification: Floyd–Warshall All-Pairs Shortest Paths

`floyd_warshall n weight` returns `ok (some dist)` where `dist` is the
flattened `n × n` matrix of all-pairs shortest distances (row-major), or
`ok none` when a negative-weight cycle is present.

Distances are `Option Int`: `none` means "no path", `some d` means a distance
`d`.  This reserves no sentinel value in the integer domain.  The abstract
weight matrix `w : Fin n → Fin n → Option Int` is the mathematical object the
theorems quantify over (`some x` = an edge of weight `x`, `none` = no edge).
-/

/-- A path from `i` to `j` using only existing edges (weight `some`), with
    total weight `d`. -/
inductive has_path_w (n : Nat) (w : Fin n → Fin n → Option Int) : Fin n → Fin n → Int → Prop
  | empty (i : Fin n) : has_path_w n w i i 0
  | step (i k j : Fin n) (x w_rest : Int) :
      w i k = some x →
      has_path_w n w k j w_rest →
      has_path_w n w i j (x + w_rest)

/-- `d` is the shortest distance from `i` to `j`. -/
def is_shortest_dist (n : Nat) (w : Fin n → Fin n → Option Int) (i j : Fin n) (d : Int) : Prop :=
  has_path_w n w i j d ∧ ∀ e : Int, has_path_w n w i j e → d ≤ e

/-- The graph has a negative-weight cycle: a path from some `i` back to `i`
    with negative total weight. -/
def has_negative_cycle (n : Nat) (w : Fin n → Fin n → Option Int) : Prop :=
  ∃ (i : Fin n) (d : Int), has_path_w n w i i d ∧ d < 0


/-! ## View helpers -/

/-- View a flattened `Vec (Option Int)` as an `n × n` row-major matrix. -/
def toDistMatrix (v : alloc.vec.Vec (Option Int)) (n : Nat) (h : v.length = n * n)
    : Fin n → Fin n → Option Int :=
  λ i j =>
    have h_bound : i.val * n + j.val < v.length := by
      rw [h]
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left j.is_lt (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt i.is_lt)
    v[i.val * n + j.val]'h_bound

/-- View a flattened `Slice (Option Int)` as an `n × n` row-major matrix. -/
def toWeightMatrix (s : Slice (Option Int)) (n : Nat) (h : s.length = n * n)
    : Fin n → Fin n → Option Int :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left j.is_lt (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt i.is_lt)
    s[i.val * n + j.val]'h_bound


/-! ## Specification theorems -/

/-- Soundness: a finite distance returned by Floyd–Warshall is the shortest
    distance (no shorter path exists).  Requires no negative-weight cycle. -/
theorem floyd_warshall_sound
    (n : Std.Usize)
    (weight : Slice (Option Int))
    (h_weight_len : weight.length = n.val * n.val)
    (w : Fin n.val → Fin n.val → Option Int)
    (h_w : toWeightMatrix weight n.val h_weight_len = w)
    (h_diag : ∀ i : Fin n.val, w i i = some 0)
    (h_no_neg_cycle : ¬ has_negative_cycle n.val w)
    (dist : alloc.vec.Vec (Option Int))
    (h_len : dist.length = n.val * n.val)
    (h : floyd_warshall n weight = ok (some dist)) :
    ∀ (i j : Fin n.val) (d : Int),
      toDistMatrix dist n.val h_len i j = some d →
      is_shortest_dist n.val w i j d := by
  sorry

/-- Completeness: if `i` can reach `j` by a finite path, Floyd–Warshall
    reports a finite distance.  Requires no negative-weight cycle. -/
theorem floyd_warshall_complete
    (n : Std.Usize)
    (weight : Slice (Option Int))
    (h_weight_len : weight.length = n.val * n.val)
    (w : Fin n.val → Fin n.val → Option Int)
    (h_w : toWeightMatrix weight n.val h_weight_len = w)
    (h_diag : ∀ i : Fin n.val, w i i = some 0)
    (h_no_neg_cycle : ¬ has_negative_cycle n.val w)
    (dist : alloc.vec.Vec (Option Int))
    (h_len : dist.length = n.val * n.val)
    (h : floyd_warshall n weight = ok (some dist)) :
    ∀ (i j : Fin n.val),
      (∃ d : Int, has_path_w n.val w i j d) →
      ∃ d : Int, toDistMatrix dist n.val h_len i j = some d := by
  sorry

/-- If Floyd–Warshall reports `none`, the graph has a negative-weight cycle. -/
theorem floyd_warshall_negative_cycle
    (n : Std.Usize)
    (weight : Slice (Option Int))
    (h_weight_len : weight.length = n.val * n.val)
    (w : Fin n.val → Fin n.val → Option Int)
    (h_w : toWeightMatrix weight n.val h_weight_len = w)
    (h_diag : ∀ i : Fin n.val, w i i = some 0)
    (h : floyd_warshall n weight = ok none) :
    has_negative_cycle n.val w := by
  sorry
