import LeanMode.graph_flow_bound.GraphFlowBound
import Mathlib

open Aeneas Aeneas.Std Result
open graph_flow_bound

set_option linter.unusedVariables false

/-!
# Specification: Feasible Circulation with Lower Bounds

`flow_bound n low cap` returns `ok (Result.Ok flow)` where `flow` is a flat
`n × n` matrix satisfying the lower/upper bounds `low[u,v] ≤ flow[u,v] ≤
cap[u,v]` and conservation at every vertex (inflow equals outflow), or
`ok (Result.Err ())` when no such circulation exists.

`low` and `cap` are flat, row-major `n × n` matrices; the directed edge
`u → v` must carry between `low u v` and `cap u v` units.  The correctness of
the transformation goes through the auxiliary super-source / super-sink
construction (each vertex's lower-bound imbalance is balanced by an auxiliary
edge), which is the subtle part of the proof.
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

/-- `f` is a feasible circulation: within the lower/upper bounds and conserved
    at every vertex. -/
def is_circulation {n : Nat} (low cap f : Fin n → Fin n → Int) : Prop :=
  (∀ u v : Fin n, low u v ≤ f u v ∧ f u v ≤ cap u v) ∧
  (∀ u : Fin n, (∑ v : Fin n, f u v) = (∑ v : Fin n, f v u))


/-! ## Specification theorems -/

/-- `flow_bound` returns a flow matrix of length `n * n`. -/
theorem flow_bound_length
    (n : Std.Usize) (low cap : Slice Int) (flow : alloc.vec.Vec Int)
    (h : flow_bound n low cap = ok (core.result.Result.Ok flow)) :
    (alloc.vec.Vec.deref flow).length = n.val * n.val := by
  sorry

/-- Soundness: the returned flow is a feasible circulation. -/
theorem flow_bound_is_feasible
    (n : Std.Usize) (low cap : Slice Int) (flow : alloc.vec.Vec Int)
    (h_low : low.length = n.val * n.val)
    (h_cap : cap.length = n.val * n.val)
    (h_bounds : ∀ i j : Fin n.val, toMatrix low n.val h_low i j ≤
                                   toMatrix cap n.val h_cap i j)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h : flow_bound n low cap = ok (core.result.Result.Ok flow)) :
    is_circulation (toMatrix low n.val h_low) (toMatrix cap n.val h_cap)
                   (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow) := by
  sorry

/-- Completeness: on `Err`, no feasible circulation exists. -/
theorem flow_bound_infeasible
    (n : Std.Usize) (low cap : Slice Int)
    (h_low : low.length = n.val * n.val)
    (h_cap : cap.length = n.val * n.val)
    (h : flow_bound n low cap = ok (core.result.Result.Err ())) :
    ∀ (f : Fin n.val → Fin n.val → Int),
      ¬ is_circulation (toMatrix low n.val h_low) (toMatrix cap n.val h_cap) f := by
  sorry

/-- The lower-bound imbalance of vertex `v`: inflow from lower bounds minus
    outflow to lower bounds. -/
def imbalance {n : Nat} (low : Fin n → Fin n → Int) (v : Fin n) : Int :=
  (∑ u : Fin n, low u v) - (∑ w : Fin n, low v w)

/-- Feasibility equivalence (the auxiliary source–sink transformation, stated
    as an existence claim): a feasible circulation exists iff there is a
    non-negative residual flow `r`, within `cap − low`, that exactly cancels
    each vertex's lower-bound imbalance. -/
theorem flow_bound_feasibility_equiv
    (n : Std.Usize) (low cap : Slice Int)
    (h_low : low.length = n.val * n.val)
    (h_cap : cap.length = n.val * n.val)
    (h_bounds : ∀ i j : Fin n.val, toMatrix low n.val h_low i j ≤ toMatrix cap n.val h_cap i j) :
    (∃ f : Fin n.val → Fin n.val → Int,
       is_circulation (toMatrix low n.val h_low) (toMatrix cap n.val h_cap) f) ↔
    (∃ r : Fin n.val → Fin n.val → Int,
       (∀ u v : Fin n.val, 0 ≤ r u v ∧ r u v ≤
         (toMatrix cap n.val h_cap) u v - (toMatrix low n.val h_low) u v) ∧
       (∀ v : Fin n.val,
         (∑ u : Fin n.val, r u v) - (∑ w : Fin n.val, r v w) =
           - imbalance (toMatrix low n.val h_low) ⟨v.val, v.is_lt⟩)) := by
  sorry
