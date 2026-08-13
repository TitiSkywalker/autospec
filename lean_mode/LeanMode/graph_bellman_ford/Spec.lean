import LeanMode.graph_bellman_ford.GraphBellmanFord
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bellman_ford

set_option linter.unusedVariables false

/-!
# Specification: Bellman-Ford Shortest Paths

`bellman_ford n from_list to_list weight_list start` returns
`ok (core.result.Result.Ok (dist, reachable))` where:
- `dist[v]` = shortest distance from `start` to `v` (0 for unreachable vertices)
- `reachable[v]` = true iff `v` is reachable from `start`

It returns `ok (core.result.Result.Err ())` when a negative cycle reachable
from `start` is detected.
-/

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

/-- Shortest distance from `start` to `v` is `d`. -/
def is_shortest_dist (edges : Finset (Nat × Nat × Int)) (start v : Nat) (d : Int) : Prop :=
  has_path edges start v d ∧ ∀ w, has_path edges start v w → d ≤ w

/-- A negative cycle (non-trivial) reachable from `start`. -/
def has_reachable_negative_cycle (edges : Finset (Nat × Nat × Int)) (start : Nat) : Prop :=
  ∃ (v : Nat) (wv wc : Int),
    has_path edges start v wv ∧ has_path edges v v wc ∧ wc < 0

/-! ## View helpers -/

/-- View the distance vec as `Fin n → Int`. -/
def toDistVec (v : alloc.vec.Vec Int) (len : Nat) (h : v.length = len) : Fin len → Int :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound

/-- View the reachable vec as `Fin n → Bool`. -/
def toReachableVec (v : alloc.vec.Vec Bool) (len : Nat) (h : v.length = len) : Fin len → Bool :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    v[i.val]'h_bound

/-! ## Specification theorems -/

/-- `bellman_ford` returns two vecs, each of length `n`, on success. -/
theorem bellman_ford_length
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (weight_list : Slice Int)
    (start : Std.Usize)
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h : bellman_ford n from_list to_list weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    dist.length = n.val ∧ reachable.length = n.val := by
  sorry

/-- Soundness: on `Ok`, there is no negative cycle reachable from `start`,
    and for every `v` with `reachable[v] = true`, `dist[v]` is the shortest
    distance from `start` to `v`. -/
theorem bellman_ford_sound
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (weight_list : Slice Int)
    (start : Std.Usize)
    (edges : Finset (Nat × Nat × Int))
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h_dist_len : dist.length = n.val)
    (h_reachable_len : reachable.length = n.val)
    (h : bellman_ford n from_list to_list weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    ¬ has_reachable_negative_cycle edges start.val ∧
    ∀ (v : Fin n.val),
      (toReachableVec reachable n.val h_reachable_len v = true) →
      is_shortest_dist edges start.val v.val (toDistVec dist n.val h_dist_len v) := by
  sorry

/-- Completeness: on `Ok`, if `reachable[v] = false`, then there is no path
    from `start` to `v`. -/
theorem bellman_ford_complete
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (weight_list : Slice Int)
    (start : Std.Usize)
    (edges : Finset (Nat × Nat × Int))
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h_dist_len : dist.length = n.val)
    (h_reachable_len : reachable.length = n.val)
    (h : bellman_ford n from_list to_list weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    ∀ (v : Fin n.val),
      (toReachableVec reachable n.val h_reachable_len v = false) →
      ¬ ∃ w, has_path edges start.val v.val w := by
  sorry

/-- Negative cycle detection: on `Err`, there exists a negative cycle
    reachable from `start`. -/
theorem bellman_ford_negative_cycle
    (n : Std.Usize)
    (from_list to_list : Slice Std.Usize)
    (weight_list : Slice Int)
    (start : Std.Usize)
    (edges : Finset (Nat × Nat × Int))
    (h : bellman_ford n from_list to_list weight_list start =
           ok (core.result.Result.Err ())) :
    has_reachable_negative_cycle edges start.val := by
  sorry
