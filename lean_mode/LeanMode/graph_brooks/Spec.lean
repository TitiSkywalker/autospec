import LeanMode.graph_brooks.GraphBrooks
import Mathlib

open Aeneas Aeneas.Std Result
open graph_brooks

set_option linter.unusedVariables false

/-!
# Specification: Brooks' theorem (1941)

`brooks_color n adj` colours the undirected graph encoded by the flat,
row-major `n × n` 0/1 *symmetric* adjacency matrix `adj`
(`adj[u * n + v] = 1` iff `{u, v}` is an edge, and `adj[v * n + v] = 0`) with
at most `Δ + 1` colours, where `Δ` is the maximum degree, and with at most `Δ`
colours whenever the graph is connected, not complete and not an odd cycle.

The mathematical content is **Brooks' theorem** (Brooks, 1941): a connected
graph that is neither complete nor an odd cycle has chromatic number at most
its maximum degree; `K_n` needs `n = Δ + 1` colours and an odd cycle needs
`3 = Δ + 1`.  `brooks_color` is a *constructive* witness: the colouring it
returns is proper, and the bound on the colours it uses is the theorem.

The proofs are `sorry`-ed (the benchmark only checks that the statements
elaborate); every statement below is a true statement about the program.
-/

namespace graph_brooks

noncomputable section

open Classical

/-! ## Viewing the flat matrix as an adjacency relation -/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Nat`. -/
def toMatrix (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Nat :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    (s[i.val * n + j.val]'h_bound).val

/-- The adjacency relation read off the flat 0/1 matrix: `u` and `v` are
    adjacent iff the corresponding entry is `1` (exactly the test the Rust
    program performs). -/
def Adj (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Prop :=
  λ u v => toMatrix s n h u v = 1

/-! ## Colourings -/

/-- A colouring is **proper** when adjacent vertices receive different
    colours. -/
def IsProperColoring {n : Nat} (A : Fin n → Fin n → Prop) (col : Fin n → Nat) : Prop :=
  ∀ u v : Fin n, A u v → col u ≠ col v

/-- The number of colours occurring in `col`. -/
def colorsUsed {n : Nat} (col : Fin n → Nat) : Nat :=
  (Finset.univ.image col).card

/-- The graph admits a proper colouring with at most `k` colours. -/
def Colorable {n : Nat} (A : Fin n → Fin n → Prop) (k : Nat) : Prop :=
  ∃ col : Fin n → Nat, IsProperColoring A col ∧ colorsUsed col ≤ k

/-- **The chromatic number** of the graph is `k`: the graph is `k`-colourable
    and no proper colouring uses fewer than `k` colours. -/
def ChromaticNumber {n : Nat} (A : Fin n → Fin n → Prop) (k : Nat) : Prop :=
  Colorable A k ∧ ∀ j : Nat, Colorable A j → k ≤ j

/-! ## Degrees -/

/-- The degree of `v`. -/
def degree {n : Nat} (A : Fin n → Fin n → Prop) (v : Fin n) : Nat :=
  (Finset.univ.filter (fun u => A v u)).card

/-- The maximum degree `Δ` of the graph. -/
def Delta {n : Nat} (A : Fin n → Fin n → Prop) : Nat :=
  Finset.univ.sup (fun v => degree A v)

/-! ## The three structural predicates of Brooks' theorem -/

/-- The graph is **complete**: every two distinct vertices are adjacent. -/
def IsComplete {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∀ u v : Fin n, u ≠ v → A u v

/-- Walk-reachability in the graph underlying `A`: each step traverses an edge
    in either direction, because the graph is undirected. -/
inductive Reach {n : Nat} (A : Fin n → Fin n → Prop) : Fin n → Fin n → Prop where
  | refl (v : Fin n) : Reach A v v
  | step (u v w : Fin n) : (A u v ∨ A v u) → Reach A v w → Reach A u w

/-- The graph is **connected**: every vertex reaches every other vertex. -/
def IsConnected {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∀ u v : Fin n, Reach A u v

/-- `a` and `b` are consecutive on the `k`-cycle: one is the successor of the
    other modulo `k` (indices live in `Fin k`). -/
def cyclicAdjacent (k : Nat) (a b : Fin k) : Prop :=
  (a.val + 1) % k = b.val ∨ (b.val + 1) % k = a.val

/-- The graph is an **odd cycle**: up to a bijection `e` onto `Fin k` with `k`
    odd and `k ≥ 3`, adjacency is exactly that of the cyclic order — i.e. `u`
    and `v` are adjacent iff their images are consecutive on the `k`-cycle.
    (The bijection forces `n = k`.)  Note that this fixes `Δ = 2`. -/
def IsOddCycle {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∃ (k : Nat) (_ : Odd k) (_ : 3 ≤ k) (e : Fin n ≃ Fin k),
    ∀ u v : Fin n, A u v ↔ cyclicAdjacent k (e u) (e v)

/-! ## Reading the program's output -/

/-- View the returned flat colour vector as `Fin n → Nat`. -/
def vecToFn {n : Nat} (v : alloc.vec.Vec Std.Usize) (h : v.length = n) : Fin n → Nat :=
  fun i =>
    have hb : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'hb).val

/-! ## Specification theorems -/

/-- The returned vector is a colouring of *every* vertex: its length is `n`. -/
theorem brooks_color_length
    (n : Std.Usize) (adj : Slice Std.Usize) (col : alloc.vec.Vec Std.Usize)
    (h : brooks_color n adj = ok col) :
    col.length = n.val := by
  sorry

/-- **Properness**: the returned vector is a proper colouring of the graph. -/
theorem brooks_color_proper
    (n : Std.Usize) (adj : Slice Std.Usize) (col : alloc.vec.Vec Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_col : col.length = n.val)
    (h01 : ∀ u v : Fin n.val,
      toMatrix adj n.val h_len u v = 0 ∨ toMatrix adj n.val h_len u v = 1)
    (hsym : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (hirr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (h : brooks_color n adj = ok col) :
    IsProperColoring (Adj adj n.val h_len) (vecToFn col h_col) := by
  sorry

/-- **`Δ + 1` colours always suffice** (the greedy bound): whatever the shape of
    a `0/1` symmetric irreflexive graph, the colouring returned uses at most
    `Δ + 1` colours. -/
theorem brooks_color_delta_plus_one
    (n : Std.Usize) (adj : Slice Std.Usize) (col : alloc.vec.Vec Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_col : col.length = n.val)
    (h01 : ∀ u v : Fin n.val,
      toMatrix adj n.val h_len u v = 0 ∨ toMatrix adj n.val h_len u v = 1)
    (hsym : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (hirr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (h : brooks_color n adj = ok col) :
    colorsUsed (vecToFn col h_col) ≤ Delta (Adj adj n.val h_len) + 1 := by
  sorry

/-- **Brooks' theorem** (Brooks, 1941), in the constructive form the program
    realises: if the graph is connected, not complete and not an odd cycle,
    then the colouring returned by `brooks_color` uses at most `Δ` colours.
    `h01` is the documented precondition that `adj` encode a `0/1` adjacency
    matrix; without it `is_complete_graph` (testing `= 0`) and `Adj` (testing
    `= 1`) disagree on entries `≥ 2`. -/
theorem brooks_theorem
    (n : Std.Usize) (adj : Slice Std.Usize) (col : alloc.vec.Vec Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_col : col.length = n.val)
    (h01 : ∀ u v : Fin n.val,
      toMatrix adj n.val h_len u v = 0 ∨ toMatrix adj n.val h_len u v = 1)
    (hsym : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (hirr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (hconn : IsConnected (Adj adj n.val h_len))
    (hnc : ¬ IsComplete (Adj adj n.val h_len))
    (hnoc : ¬ IsOddCycle (Adj adj n.val h_len))
    (h : brooks_color n adj = ok col) :
    colorsUsed (vecToFn col h_col) ≤ Delta (Adj adj n.val h_len) := by
  sorry

/-- The returned colouring exhibits the chromatic number: it is at most the
    number of colours the program used. -/
theorem brooks_color_chromatic_number_bound
    (n : Std.Usize) (adj : Slice Std.Usize) (col : alloc.vec.Vec Std.Usize)
    (h_len : adj.length = n.val * n.val)
    (h_col : col.length = n.val)
    (h01 : ∀ u v : Fin n.val,
      toMatrix adj n.val h_len u v = 0 ∨ toMatrix adj n.val h_len u v = 1)
    (hsym : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (hirr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (h : brooks_color n adj = ok col) :
    ∀ k : Nat, ChromaticNumber (Adj adj n.val h_len) k →
      k ≤ colorsUsed (vecToFn col h_col) := by
  sorry

/-- **Brooks' theorem, chromatic-number form**: a connected graph that is
    neither complete nor an odd cycle has chromatic number at most `Δ`. -/
theorem brooks_chromatic_number_le_delta
    {n : Nat} (A : Fin n → Fin n → Prop)
    (hsym : ∀ u v, A u v → A v u)
    (hirr : ∀ v, ¬ A v v)
    (hconn : IsConnected A) (hnc : ¬ IsComplete A) (hnoc : ¬ IsOddCycle A) :
    ∀ k : Nat, ChromaticNumber A k → k ≤ Delta A := by
  sorry

/-- **The complete graph is the extremal case for the `Δ + 1` bound**: `K_n`
    (n ≥ 1) is complete, has maximum degree `n − 1`, and its chromatic number
    is `n = Δ + 1` — so `Δ` colours do not suffice there. -/
theorem complete_graph_needs_delta_plus_one
    {n : Nat} (A : Fin n → Fin n → Prop)
    (hn : 1 ≤ n)
    (hsym : ∀ u v, A u v → A v u)
    (hirr : ∀ v, ¬ A v v)
    (hcomplete : IsComplete A) :
    ChromaticNumber A n ∧ Delta A + 1 = n := by
  sorry

/-- **An odd cycle is the other extremal case**: an odd cycle has chromatic
    number `3 = Δ + 1`, so `Δ = 2` colours do not suffice there. -/
theorem odd_cycle_needs_three
    {n : Nat} (A : Fin n → Fin n → Prop)
    (hcycle : IsOddCycle A) :
    ChromaticNumber A 3 ∧ Delta A + 1 = 3 := by
  sorry

end

end graph_brooks
