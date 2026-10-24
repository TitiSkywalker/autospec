import LeanMode.graph_negative_cycle.GraphNegativeCycle
import Mathlib

open Aeneas Aeneas.Std Result
open graph_negative_cycle

set_option linter.unusedVariables false

/-!
# Specification: Negative-Cycle Detection (potential certificate)

`graph_negative_cycle n cap cost` returns `ok (has_neg_cycle, h)` where `cap` / `cost`
are flat, row-major `n × n` matrices (edge `u → v` exists iff `cap u v > 0`,
with per-unit cost `cost u v`, which may be negative), `has_neg_cycle` tells
whether a directed cycle of negative total cost exists, and — when it does not —
`h` is a *feasible potential*: every edge has non-negative reduced cost
`cost u v + h u - h v ≥ 0`.

The subtle part is the equivalence "no negative cycle iff a feasible potential
exists" — the LP-duality / potential argument (Johnson's technique in reverse).
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

/-- View a `Vec Int` of length `len` as `Fin len → Int`. -/
def toVecInt (v : alloc.vec.Vec Int) (len : Nat) (h : v.length = len) : Fin len → Int :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound

/-- The successor of `i` cyclically: `(i + 1) mod m`. -/
def next (m : Nat) (hm : 0 < m) (i : Fin m) : Fin m :=
  ⟨(i.val + 1) % m, Nat.mod_lt (i.val + 1) hm⟩

/-- The total cost of a cycle `v : Fin m → Fin n` (sum of the costs of its
    cyclically consecutive edges). -/
def cycleCost {n : Nat} (cost : Fin n → Fin n → Int) (m : Nat) (hm : 0 < m)
    (v : Fin m → Fin n) : Int :=
  ∑ i : Fin m, cost (v i) (v (next m hm i))

/-- The graph has a negative-cost simple directed cycle. -/
def has_graph_negative_cycle {n : Nat} (cap cost : Fin n → Fin n → Int) : Prop :=
  ∃ (m : Nat) (hm : 0 < m) (v : Fin m → Fin n),
    Function.Injective v ∧
    (∀ i : Fin m, 0 < cap (v i) (v (next m hm i))) ∧
    cycleCost cost m hm v < 0

/-- `h` is a feasible potential: every edge has non-negative reduced cost. -/
def feasible_potential {n : Nat} (cap cost : Fin n → Fin n → Int) (h : Fin n → Int) : Prop :=
  ∀ u v : Fin n, 0 < cap u v → 0 ≤ cost u v + h u - h v


/-! ## Specification theorems -/

/-- The dual/potential characterization: no negative cycle iff a feasible
    potential exists. -/
theorem graph_negative_cycle_potential_duality {n : Nat} (cap cost : Fin n → Fin n → Int) :
    (¬ has_graph_negative_cycle cap cost) ↔ ∃ h : Fin n → Int, feasible_potential cap cost h := by
  sorry

/-- Soundness: when `has_neg = true`, there is a negative cycle. -/
theorem graph_negative_cycle_sound
    (n : Std.Usize) (cap cost : Slice Int) (has_neg : Bool) (h : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_cost : cost.length = n.val * n.val)
    (h_h : h.length = n.val)
    (hcall : graph_negative_cycle n cap cost = ok (has_neg, h)) :
    has_neg = true →
      has_graph_negative_cycle (toMatrix cap n.val h_cap) (toMatrix cost n.val h_cost) := by
  sorry

/-- Completeness: when `has_neg = false`, there is no negative cycle and `h` is
    a feasible potential. -/
theorem graph_negative_cycle_complete
    (n : Std.Usize) (cap cost : Slice Int) (has_neg : Bool) (h : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_cost : cost.length = n.val * n.val)
    (h_h : h.length = n.val)
    (hcall : graph_negative_cycle n cap cost = ok (has_neg, h)) :
    has_neg = false →
      (¬ has_graph_negative_cycle (toMatrix cap n.val h_cap) (toMatrix cost n.val h_cost)) ∧
      feasible_potential (toMatrix cap n.val h_cap) (toMatrix cost n.val h_cost)
        (toVecInt h n.val h_h) := by
  sorry
