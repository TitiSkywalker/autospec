import LeanMode.graph_matrix_tree.GraphMatrixTree
import Mathlib

open Aeneas Aeneas.Std Result
open graph_matrix_tree

set_option linter.unusedVariables false

noncomputable section

/-!
# Specification: Number of Spanning Trees (Matrix-Tree / Kirchhoff)

`matrix_tree n adj` returns `ok result` where `adj` is a flat, row-major,
symmetric `n × n` 0/1 adjacency matrix of a simple undirected graph (no
self-loops) and `result` is the number of spanning trees.

The theorem asserts that the returned value equals the number of spanning trees
(defined combinatorially: subsets of `n - 1` edges that connect all vertices).
The proof — that the determinant of a Laplace cofactor counts spanning trees —
goes through the Cauchy–Binet formula, which is the subtle part.
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

/-- The set of all undirected edges `(i, j)` with `i < j` and `adj i j = 1`. -/
def all_edges {n : Nat} (adj : Fin n → Fin n → Int) : Finset (Fin n × Fin n) :=
  (Finset.univ : Finset (Fin n × Fin n)).filter (λ p => p.1.val < p.2.val ∧ adj p.1 p.2 = 1)

/-- A path in the edge set `S` (an undirected edge joins `u` and `v` when
    either ordered pair is in `S`). -/
inductive IsPath {n : Nat} (S : Finset (Fin n × Fin n)) : List (Fin n) → Prop
  | single (u v : Fin n) (h : (u, v) ∈ S ∨ (v, u) ∈ S) : IsPath S [u, v]
  | cons (u v : Fin n) (rest : List (Fin n))
      (h : (u, v) ∈ S ∨ (v, u) ∈ S) (hr : IsPath S (v :: rest)) :
      IsPath S (u :: v :: rest)

/-- `S` connects every pair of vertices. -/
def Connects {n : Nat} (S : Finset (Fin n × Fin n)) : Prop :=
  ∀ u v : Fin n, u ≠ v → ∃ p : List (Fin n), IsPath S p ∧ p.head? = some u ∧ p.getLast? = some v

/-- `S` is a spanning tree of `adj`: its edges all come from `adj`, it has
    `n - 1` edges, and it connects all vertices. -/
def is_spanning_tree {n : Nat} (adj : Fin n → Fin n → Int) (S : Finset (Fin n × Fin n)) : Prop :=
  S ⊆ all_edges adj ∧ S.card = n - 1 ∧ Connects S

/-- The number of spanning trees of `adj`. -/
def num_spanning_trees {n : Nat} (adj : Fin n → Fin n → Int) : Int := by
  classical
  exact (((all_edges adj).powerset.filter (λ S => is_spanning_tree adj S)).card : Int)

end


/-! ## Specification theorem -/

/-- `matrix_tree` returns the number of spanning trees (Kirchhoff's theorem). -/
theorem matrix_tree_count
    (n : Std.Usize) (adj : Slice Int) (result : Int)
    (h_adj : adj.length = n.val * n.val)
    (h_symm : ∀ i j : Fin n.val, toMatrix adj n.val h_adj i j =
                                 toMatrix adj n.val h_adj j i)
    (h_adj01 : ∀ i j : Fin n.val,
       toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_diag : ∀ i : Fin n.val, toMatrix adj n.val h_adj i i = 0)
    (h : matrix_tree n adj = ok result) :
    result = num_spanning_trees (toMatrix adj n.val h_adj) := by
  sorry
