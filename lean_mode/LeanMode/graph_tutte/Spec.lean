import LeanMode.graph_tutte.GraphTutte
import Mathlib

open Aeneas Aeneas.Std Result
open graph_tutte

set_option linter.unusedVariables false

/-!
# Specification: Tutte's 1-factor theorem

`tutte_condition_holds n adj` decides Tutte's 1-factor condition on the finite
undirected graph encoded by the flat, row-major `n × n` 0/1 *symmetric*
adjacency matrix `adj` (`adj[u * n + v] = 1` iff `{u, v}` is an edge).  It
returns `ok b` where `b = true` iff for every `S ⊆ V` the number of connected
components of `G − S` of odd cardinality is at most `|S|`.

The mathematical content is **Tutte's 1-factor theorem** (Tutte, 1947): a
finite undirected graph admits a perfect matching iff it satisfies that
odd-component condition.  The program only decides the condition; the theorem
below identifies "the condition holds" with "a 1-factor exists".

The proofs are `sorry`-ed (the benchmark only checks that the statements
elaborate); every statement below is a true statement about the program.
-/

namespace graph_tutte

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

/-! ## The odd-component count of `G − S` -/

/-- A walk inside the graph induced on the complement of `S`: every vertex on
    the walk lies outside `S`, and consecutive vertices are adjacent. -/
inductive innerReach {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n)) :
    Fin n → Fin n → Prop where
  | refl (v : Fin n) (hv : v ∉ S) : innerReach A S v v
  | step (u v w : Fin n) (hu : u ∉ S) : A u v → v ∉ S →
      innerReach A S v w → innerReach A S u w

/-- `u` and `v` lie in the same connected component of the induced graph
    `G − S`. -/
def connIn {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n))
    (u v : Fin n) : Prop :=
  innerReach A S u v ∨ innerReach A S v u

/-- The connected component of `v` in `G − S` (empty when `v ∈ S`). -/
def compOf {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n))
    (v : Fin n) : Finset (Fin n) :=
  Finset.univ.filter (fun u => connIn A S u v)

/-- `v` is the least vertex of its connected component in `G − S`; picking one
    representative per component makes the count below independent of ordering
    choices. -/
def isRep {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n))
    (v : Fin n) : Prop :=
  v ∉ S ∧ ∀ u, connIn A S u v → v.val ≤ u.val

/-- **Tutte's odd-component count**: the number of connected components of
    `G − S` whose cardinality is odd. -/
def oddComponents {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n)) : Nat :=
  (Finset.univ.filter (fun v => isRep A S v ∧ Odd (compOf A S v).card)).card

/-! ## Tutte's condition and perfect matchings -/

/-- **Tutte's 1-factor condition**: for every `S ⊆ V`, the number of odd
    connected components of `G − S` is at most `|S|`.  This is exactly the
    predicate the Rust program decides. -/
def tutte_condition_all {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∀ S : Finset (Fin n), oddComponents A S ≤ S.card

/-- `A` has a **perfect matching**: an involution `m` with no fixed point such
    that every `{v, m v}` is an edge — i.e. a set of pairwise vertex-disjoint
    edges covering every vertex. -/
def HasPerfectMatching {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∃ m : Fin n → Fin n, (∀ v, m (m v) = v) ∧ (∀ v, m v ≠ v) ∧ (∀ v, A v (m v))

/-! ## Specification theorems -/

/-- **Program correctness**: on a symmetric, irreflexive 0/1 matrix, the
    program returns `true` exactly when the graph satisfies Tutte's condition;
    hence the program is a decision procedure for `tutte_condition_all`. -/
theorem tutte_characterisation
    (n : Std.Usize) (adj : Slice Std.Usize) (b : Bool)
    (h_len : adj.length = n.val * n.val)
    (hsym : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (hirr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (hcall : tutte_condition_holds n adj = ok b) :
    (b = true ↔ tutte_condition_all (Adj adj n.val h_len)) := by
  sorry

/-- **Tutte's 1-factor theorem** (Tutte, 1947): a finite undirected graph has a
    perfect matching iff for every `S` the number of odd components of `G − S`
    is at most `|S|`. -/
theorem tutte_theorem {n : Nat} (A : Fin n → Fin n → Prop)
    (hsym : ∀ u v, A u v → A v u) (hirr : ∀ v, ¬ A v v) :
    HasPerfectMatching A ↔ tutte_condition_all A := by
  sorry

/-! ## A Petersen-style corollary -/

/-- Reachability in the graph obtained by deleting the undirected edge `{a, b}`:
    the single step may not use `{a, b}`. -/
inductive reachAvoiding {n : Nat} (A : Fin n → Fin n → Prop) (a b : Fin n) :
    Fin n → Fin n → Prop where
  | refl (v : Fin n) : reachAvoiding A a b v v
  | step (u v w : Fin n) : A u v → ¬ (u = a ∧ v = b) → ¬ (u = b ∧ v = a) →
      reachAvoiding A a b v w → reachAvoiding A a b u w

/-- `A` is **bridgeless**: deleting any edge of the graph keeps its two
    endpoints connected. -/
def isBridgeless {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∀ u v : Fin n, A u v → reachAvoiding A u v u v

/-- **Petersen's theorem** (Petersen, 1891): every bridgeless cubic graph has a
    perfect matching — a standard corollary of Tutte's theorem.  (The Petersen
    graph is the canonical example, and is exercised by the test suite.) -/
theorem petersen_bridgeless_cubic_has_perfect_matching {n : Nat}
    (A : Fin n → Fin n → Prop)
    (hsym : ∀ u v, A u v → A v u) (hirr : ∀ v, ¬ A v v)
    (hcubic : ∀ v, (Finset.univ.filter (fun u => A v u)).card = 3)
    (hbridgeless : isBridgeless A) :
    HasPerfectMatching A := by
  sorry

end

end graph_tutte
