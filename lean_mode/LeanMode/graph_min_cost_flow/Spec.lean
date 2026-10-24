import LeanMode.graph_min_cost_flow.GraphMinCostFlow
import Mathlib

open Aeneas Aeneas.Std Result
open graph_min_cost_flow

set_option linter.unusedVariables false

/-!
# Specification: Minimum-Cost Maximum Flow

`min_cost_flow n cap cost source sink` returns `ok (Result.Ok (value, total_cost,
flow))` where `flow` is a non-negative flat `n × n` flow matrix, `value` is the
net outflow from `source`, and `total_cost` is `Σ flow u v * cost u v`.  It
returns `ok (Result.Err ())` when the graph contains a negative-cost cycle.

`cap` and `cost` are flat, row-major `n × n` matrices: the directed edge `u → v`
has capacity `cap u v` (0 = absent) and per-unit cost `cost u v`.

The theorems assert feasibility, value, cost, maximality, and cost-optimality
(minimum cost among maximum flows) — the last is the hard direction, whose proof
goes through the dual-potential / no-negative-cycle optimality condition.
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
    non-negativity, capacity bound, and conservation at non-terminal vertices. -/
def is_flow {n : Nat} (cap f : Fin n → Fin n → Int) (source sink : Fin n) : Prop :=
  (∀ u v : Fin n, 0 ≤ f u v) ∧
  (∀ u v : Fin n, f u v ≤ cap u v) ∧
  (∀ u : Fin n, u ≠ source → u ≠ sink →
      (∑ v : Fin n, f u v) = (∑ v : Fin n, f v u))

/-- The value of a flow: net outflow from `source`. -/
def flow_value {n : Nat} (f : Fin n → Fin n → Int) (source : Fin n) : Int :=
  (∑ v : Fin n, f source v) - (∑ v : Fin n, f v source)

/-- The total cost of a flow: `Σ flow u v * cost u v`. -/
def flow_cost {n : Nat} (cost f : Fin n → Fin n → Int) : Int :=
  ∑ u : Fin n, ∑ v : Fin n, f u v * cost u v

/-- A directed edge `u → v` exists when `cap u v > 0`. -/
def has_edge {n : Nat} (cap : Fin n → Fin n → Int) (u v : Fin n) : Prop :=
  0 < cap u v

/-- A path from `u` to `v` with total cost (following existing edges). -/
inductive has_path (n : Nat) (cap cost : Fin n → Fin n → Int) : Fin n → Fin n → Int → Prop
  | empty (i : Fin n) : has_path n cap cost i i 0
  | step (i k j : Fin n) (c_k rest : Int) :
      has_edge cap i k →
      has_path n cap cost k j rest →
      has_path n cap cost i j (cost i k + rest)

/-- A negative-cost cycle: a non-trivial path from some `v` back to `v` with
    negative total cost. -/
def has_negative_cycle (n : Nat) (cap cost : Fin n → Fin n → Int) : Prop :=
  ∃ (v : Fin n) (d : Int), has_path n cap cost v v d ∧ d < 0


/-! ## Specification theorems -/

/-- `min_cost_flow` returns a flow matrix of length `n * n`. -/
theorem min_cost_flow_length
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    (alloc.vec.Vec.deref flow).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a feasible `source`–`sink` flow. -/
theorem min_cost_flow_is_flow
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix cap n.val h_cap i j)
    (h_cost : cost.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    is_flow (toMatrix cap n.val h_cap)
            (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
            ⟨source.val, h_source⟩ ⟨sink.val, h_sink⟩ := by
  sorry

/-- The returned `value` equals the flow value of the returned matrix. -/
theorem min_cost_flow_value
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_cost : cost.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    value = flow_value (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
                       ⟨source.val, h_source⟩ := by
  sorry

/-- The returned `total_cost` equals the flow cost of the returned matrix. -/
theorem min_cost_flow_cost
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_cost : cost.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    total_cost = flow_cost (toMatrix cost n.val h_cost)
                           (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow) := by
  sorry

/-- Maximality: no feasible `source`–`sink` flow has value larger than the
    returned flow. -/
theorem min_cost_flow_max
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix cap n.val h_cap i j)
    (h_cost : cost.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    ∀ (g : Fin n.val → Fin n.val → Int),
      is_flow (toMatrix cap n.val h_cap) g
              ⟨source.val, h_source⟩ ⟨sink.val, h_sink⟩ →
      flow_value g ⟨source.val, h_source⟩ ≤
        flow_value (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
                   ⟨source.val, h_source⟩ := by
  sorry

/-- Optimality: among all feasible flows of the same (maximum) value, the
    returned flow has minimum cost. -/
theorem min_cost_flow_optimal
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (value total_cost : Int) (flow : alloc.vec.Vec Int)
    (h_cap : cap.length = n.val * n.val)
    (h_nonneg : ∀ i j : Fin n.val, 0 ≤ toMatrix cap n.val h_cap i j)
    (h_cost : cost.length = n.val * n.val)
    (h_flow : (alloc.vec.Vec.deref flow).length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Ok (value, total_cost, flow))) :
    ∀ (g : Fin n.val → Fin n.val → Int),
      is_flow (toMatrix cap n.val h_cap) g
              ⟨source.val, h_source⟩ ⟨sink.val, h_sink⟩ →
      flow_value g ⟨source.val, h_source⟩ =
        flow_value (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow)
                   ⟨source.val, h_source⟩ →
      flow_cost (toMatrix cost n.val h_cost)
                (toMatrix (alloc.vec.Vec.deref flow) n.val h_flow) ≤
        flow_cost (toMatrix cost n.val h_cost) g := by
  sorry

/-- Negative-cycle detection: on `Err`, the graph has a negative-cost cycle. -/
theorem min_cost_flow_negative_cycle
    (n : Std.Usize) (cap cost : Slice Int) (source sink : Std.Usize)
    (h_cap : cap.length = n.val * n.val)
    (h_cost : cost.length = n.val * n.val)
    (h_source : source.val < n.val)
    (h_sink : sink.val < n.val)
    (h : min_cost_flow n cap cost source sink =
           ok (core.result.Result.Err ())) :
    has_negative_cycle n.val (toMatrix cap n.val h_cap)
                              (toMatrix cost n.val h_cost) := by
  sorry
