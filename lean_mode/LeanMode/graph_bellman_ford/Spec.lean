import LeanMode.graph_bellman_ford.GraphBellmanFord
import Mathlib

open Aeneas Aeneas.Std Result
open graph_bellman_ford

set_option linter.unusedVariables false

/-!
# Specification: Bellman-Ford Shortest Paths

`bellman_ford n from_list to_list weight_list start` returns
`ok (core.result.Result.Ok (dist, reachable))` on success, and
`ok (core.result.Result.Err ())` when a negative cycle reachable from `start`
is detected.  The directed, weighted graph is read directly off the concrete
CSR edge list.
-/

/-- A directed weighted graph as parallel `from_list` / `to_list` /
    `weight_list` slices. -/
structure CsrGraph where
  from_list : Slice Std.Usize
  to_list : Slice Std.Usize
  weight_list : Slice Int
  h_len : from_list.length = to_list.length
  h_len_w : from_list.length = weight_list.length

/-- View an edge-list slice of length `m` as `Fin m → Nat`. -/
def toEdgeVec (s : Slice Std.Usize) (m : Nat) (h : s.length = m) : Fin m → Nat :=
  λ k =>
    have hb : k.val < s.length := by rw [h]; exact k.is_lt
    (s[k.val]'hb).val

/-- View a weight slice of length `m` as `Fin m → Int`. -/
def toWeightVec (s : Slice Int) (m : Nat) (h : s.length = m) : Fin m → Int :=
  λ k =>
    have hb : k.val < s.length := by rw [h]; exact k.is_lt
    s[k.val]'hb

/-- Edge from `u` to `v` with weight `w`. -/
def edge_in (g : CsrGraph) (u v : Nat) (w : Int) : Prop :=
  ∃ k : Fin g.from_list.length,
    toEdgeVec g.from_list g.from_list.length rfl k = u ∧
    toEdgeVec g.to_list g.from_list.length g.h_len.symm k = v ∧
    toWeightVec g.weight_list g.from_list.length g.h_len_w.symm k = w

/-- Path from `u` to `v` with total weight. -/
inductive has_path (g : CsrGraph) : Nat → Nat → Int → Prop
  | empty (v : Nat) : has_path g v v 0
  | step (u v w : Nat) (w_edge w_rest : Int) :
      edge_in g u v w_edge →
      has_path g v w w_rest →
      has_path g u w (w_edge + w_rest)

/-- Shortest distance from `start` to `v` is `d`. -/
def is_shortest_dist (g : CsrGraph) (start v : Nat) (d : Int) : Prop :=
  has_path g start v d ∧ ∀ w, has_path g start v w → d ≤ w

/-- A negative cycle (non-trivial) reachable from `start`. -/
def has_reachable_negative_cycle (g : CsrGraph) (start : Nat) : Prop :=
  ∃ (v : Nat) (wv wc : Int),
    has_path g start v wv ∧ has_path g v v wc ∧ wc < 0


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
    (g : CsrGraph)
    (start : Std.Usize)
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h : bellman_ford n g.from_list g.to_list g.weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    dist.length = n.val ∧ reachable.length = n.val := by
  sorry

/-- Soundness: on `Ok`, there is no negative cycle reachable from `start`,
    and for every `v` with `reachable[v] = true`, `dist[v]` is the shortest
    distance from `start` to `v`. -/
theorem bellman_ford_sound
    (n : Std.Usize)
    (g : CsrGraph)
    (start : Std.Usize)
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h_dist_len : dist.length = n.val)
    (h_reachable_len : reachable.length = n.val)
    (h : bellman_ford n g.from_list g.to_list g.weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    ¬ has_reachable_negative_cycle g start.val ∧
    ∀ (v : Fin n.val),
      (toReachableVec reachable n.val h_reachable_len v = true) →
      is_shortest_dist g start.val v.val (toDistVec dist n.val h_dist_len v) := by
  sorry

/-- Completeness: on `Ok`, if `reachable[v] = false`, then there is no path
    from `start` to `v`. -/
theorem bellman_ford_complete
    (n : Std.Usize)
    (g : CsrGraph)
    (start : Std.Usize)
    (dist : alloc.vec.Vec Int) (reachable : alloc.vec.Vec Bool)
    (h_dist_len : dist.length = n.val)
    (h_reachable_len : reachable.length = n.val)
    (h : bellman_ford n g.from_list g.to_list g.weight_list start =
           ok (core.result.Result.Ok (dist, reachable))) :
    ∀ (v : Fin n.val),
      (toReachableVec reachable n.val h_reachable_len v = false) →
      ¬ ∃ w, has_path g start.val v.val w := by
  sorry

/-- Negative cycle detection: on `Err`, there exists a negative cycle
    reachable from `start`. -/
theorem bellman_ford_negative_cycle
    (n : Std.Usize)
    (g : CsrGraph)
    (start : Std.Usize)
    (h : bellman_ford n g.from_list g.to_list g.weight_list start =
           ok (core.result.Result.Err ())) :
    has_reachable_negative_cycle g start.val := by
  sorry
