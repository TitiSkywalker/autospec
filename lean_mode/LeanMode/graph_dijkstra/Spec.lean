import LeanMode.graph_dijkstra.GraphDijkstra
import Mathlib

open Aeneas Aeneas.Std Result
open graph_dijkstra

set_option linter.unusedVariables false

/-!
# Specification: Dijkstra's Shortest Paths

`dijkstra n from_list to_list weight_list start` returns shortest distances
from `start` to all vertices 0..n-1.
-/

/-- View the result vec as `Fin n → Int`. -/
def toVec (v : alloc.vec.Vec Int) (len : Nat) (h : v.length = len) : Fin len → Int :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound

/-- Edge from `u` to `v` with weight `w`. -/
def has_edge (edges : Finset (Nat × Nat × Int)) (u v : Nat) (w : Int) : Prop :=
  (u, v, w) ∈ edges

/-- Path from `u` to `v` with total weight. -/
inductive has_path (edges : Finset (Nat × Nat × Int)) : Nat → Nat → Int → Prop
  | empty (v : Nat) : has_path edges v v 0
  | step (u v w : Nat) (w_edge w_rest : Int) :
      has_edge edges u v w_edge →
      has_path edges v w w_rest →
      has_path edges u w (w_edge + w_rest)

/-- Shortest distance (or unreachable). -/
def is_shortest_dist (edges : Finset (Nat × Nat × Int)) (start v : Nat) (d : Int) : Prop :=
  has_path edges start v d ∧ ∀ w, has_path edges start v w → d ≤ w

/-- All edge weights ≥ 0. -/
def weights_non_negative (edges : Finset (Nat × Nat × Int)) : Prop :=
  ∀ u v w, has_edge edges u v w → w ≥ 0

/-- Every vertex is reachable from `start`. -/
def connected (edges : Finset (Nat × Nat × Int)) (start n : Nat) : Prop :=
  ∀ v, v < n → ∃ d, is_shortest_dist edges start v d


/-! ## Specification theorems -/

/-- `dijkstra` returns a vec of length `n`. -/
theorem dijkstra_length
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (weight_list : Slice Int)
    (start : Std.Usize)
    (result : alloc.vec.Vec Int)
    (h : dijkstra n from_list to_list weight_list start = ok result) :
    result.length = n.val := by
  sorry

/-- Correctness: under connectivity + non-negative weights, each vertex
    gets its shortest distance. -/
theorem dijkstra_correct
    (n : Std.Usize)
    (edges : Finset (Nat × Nat × Int))
    (h_wf : weights_non_negative edges)
    (h_conn : connected edges start n.val)
    (start : Nat)
    (result : alloc.vec.Vec Int)
    (h_result : result.length = n.val) :
    let dist := toVec (alloc.vec.Vec.deref result) n.val h_result
    ∀ (v : Fin n.val), is_shortest_dist edges start v.val (dist v) := by
  sorry
