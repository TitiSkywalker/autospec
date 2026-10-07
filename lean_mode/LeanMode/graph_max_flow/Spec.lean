import LeanMode.graph_max_flow.GraphMaxFlow
import Mathlib

open Aeneas Aeneas.Std Result
open graph_max_flow

set_option linter.unusedVariables false

/-!
# Specification: Maximum Flow (Edmonds-Karp)

`max_flow n capacity source sink` returns `ok (value, flow)` where
`value` is the value of a maximum `source`–`sink` flow and `flow` is a
skew-symmetric `n × n` flow matrix.

`capacity` is a flat, row-major `n × n` capacity matrix: the directed edge
`u → v` has capacity `capacity[idx(u, n, v)]`.  A *feasible flow* is a
skew-symmetric matrix `f` with `f u v ≤ capacity u v` and flow conservation
at every vertex except `source` and `sink`.  The theorems assert that the
returned matrix is feasible, that the returned `value` equals its flow value,
and that no feasible flow has a larger value (max-flow ≤ min-cut is the
optimality direction).
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

/-- `f` is a feasible `source`–`sink` flow on capacity matrix `cap`:
    capacity bound, skew symmetry, and conservation at non-terminal vertices. -/
def is_flow {n : Nat} (cap f : Fin n → Fin n → Int) (source sink : Fin n) : Prop :=
  (∀ u v : Fin n, f u v ≤ cap u v) ∧
  (∀ u v : Fin n, f u v = - f v u) ∧
  (∀ u : Fin n, u ≠ source → u ≠ sink → (∑ v : Fin n, f u v) = 0)

/-- The value of a flow: net outflow from `source`. -/
def flow_value {n : Nat} (f : Fin n → Fin n → Int) (source : Fin n) : Int :=
  ∑ v : Fin n, f source v


/-! ## Specification theorems -/

/-- `max_flow` returns a flow matrix of length `n * n`. -/
theorem max_flow_length
    (n : Std.Usize) (capacity : Slice Int) (source sink : Std.Usize)
    (value : Int) (flow : alloc.vec.Vec Int)
    (h : max_flow n capacity source sink = ok (value, flow)) :
    (alloc.vec.Vec.deref flow).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a feasible `source`–`sink` flow. -/
theorem max_flow_is_flow
    (n : Std.Usize) (capacity : Slice Int) (source sink : Std.Usize)
    (value : Int) (flow : alloc.vec.Vec Int)
    (h_cap : capacity.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : max_flow n capacity source sink = ok (value, flow)) :
    is_flow (toMatrix capacity n.val h_cap)
            (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
            ⟨source.val, h_source⟩ ⟨sink.val, h_sink⟩ := by
  sorry

/-- The returned `value` equals the flow value of the returned matrix. -/
theorem max_flow_value
    (n : Std.Usize) (capacity : Slice Int) (source sink : Std.Usize)
    (value : Int) (flow : alloc.vec.Vec Int)
    (h_cap : capacity.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h : max_flow n capacity source sink = ok (value, flow)) :
    value = flow_value (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
                       ⟨source.val, h_source⟩ := by
  sorry

/-- Optimality: no feasible `source`–`sink` flow has value larger than the
    returned flow (the max-flow ≤ min-cut direction). -/
theorem max_flow_optimal
    (n : Std.Usize) (capacity : Slice Int) (source sink : Std.Usize)
    (value : Int) (flow : alloc.vec.Vec Int)
    (h_cap : capacity.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : max_flow n capacity source sink = ok (value, flow)) :
    ∀ (g : Fin n.val → Fin n.val → Int),
      is_flow (toMatrix capacity n.val h_cap) g
              ⟨source.val, h_source⟩ ⟨sink.val, h_sink⟩ →
      flow_value g ⟨source.val, h_source⟩ ≤
        flow_value (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
                   ⟨source.val, h_source⟩ := by
  sorry
