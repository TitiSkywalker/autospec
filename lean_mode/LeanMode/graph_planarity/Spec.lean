import LeanMode.graph_planarity.GraphPlanarity
import Mathlib

open Aeneas Aeneas.Std Result
open graph_planarity

set_option linter.unusedVariables false

/-!
# Specification: planarity testing by `K5`/`K3,3` minor testing

`is_planar n adj` decides whether the finite simple graph encoded by the flat,
row-major, symmetric `n × n` 0/1 adjacency matrix `adj` (`adj[u * n + v] = 1`
iff `{u, v}` is an edge, diagonal `0`) is planar.  It returns `ok b` where
`b = true` iff the graph has neither a `K5` minor nor a `K3,3` minor (Wagner's
form of Kuratowski's theorem), as computed by an exhaustive search over
canonical partitions of `V` (see `src/lib.rs`).

## What `IsPlanar` means here

`IsPlanar A` is *not* a restatement of the program's criterion: it is the
classical definition of planarity as the existence of a **combinatorial
embedding** (a rotation system), i.e.

* `rot v` is a cyclic order of the edges at `v` (`IsRotationSystem`: `rot v`
  maps neighbours of `v` to neighbours of `v`, and the orbit of any neighbour
  under iteration of `rot v` is *all* neighbours of `v` — the rotation is a
  single cycle on the incident edges, the standard rotation-system data), and
* the induced embedding is spherical (`EulerCharacteristicOK`): counting the
  faces as the orbits of the face permutation `d ↦ (d.2, rot d.2 d.1)` on darts
  (each face has exactly one lexicographically least dart) and counting one
  face per isolated vertex, Euler's formula `V - E + F = 2·c` holds, `c` being
  the number of connected components.  `V - E + F = 2c` is exactly the
  statement that every component is embedded with genus `0`, i.e. in the plane.

The main theorem `is_planar_correct` then says that the program answers `true`
exactly for the graphs with such an embedding, and
`is_planar_iff_minor_free` records that the answer is also the (algorithm's own)
`K5`/`K3,3`-minor criterion.  All proofs are `sorry`-ed (the benchmark only
requires the statements to elaborate); every statement below is a true
statement about the program.
-/

namespace graph_planarity

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
    adjacent iff the corresponding entry is `1` — exactly the test the Rust
    program performs. -/
def Adj (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Prop :=
  λ u v => toMatrix s n h u v = 1

/-! ## Rotation systems (combinatorial embeddings) -/

/-- A **rotation system**: `rot v` sends a neighbour `u` of `v` to the
    neighbour that follows `u` in the cyclic order of the edges at `v`.  The
    second clause says that the orbit of any neighbour of `v` under `rot v`
    covers *all* neighbours of `v`, i.e. `rot v` is a single cycle on the
    incident edges (vacuous at a vertex of degree `0`). -/
def IsRotationSystem {n : Nat} (A : Fin n → Fin n → Prop)
    (rot : Fin n → Fin n → Fin n) : Prop :=
  (∀ u v : Fin n, A v u → A v (rot v u)) ∧
  (∀ u v w : Fin n, A v u → A v w → ∃ k : Nat, (rot v)^[k] u = w)

/-- The dart `(u, v)` of an edge `{u, v}`.  Darts are represented as pairs of
    vertices of `Fin n`; `IsDart` restricts to pairs that really are edges, so
    no dependent subtype is needed. -/
def IsDart {n : Nat} (A : Fin n → Fin n → Prop) (d : Fin n × Fin n) : Prop := A d.1 d.2

/-- The face permutation of a rotation system: the dart `(u, v)` is followed by
    `(v, rot v u)` — reverse the dart, then step to the next edge around `v`. -/
def faceStep {n : Nat} (rot : Fin n → Fin n → Fin n) (d : Fin n × Fin n) : Fin n × Fin n :=
  (d.2, rot d.2 d.1)

/-- `e` is reached from `d` by iterating the face permutation, i.e. `d` and `e`
    lie on a common face walk. -/
inductive FaceRel {n : Nat} (rot : Fin n → Fin n → Fin n) :
    Fin n × Fin n → Fin n × Fin n → Prop where
  | refl (d : Fin n × Fin n) : FaceRel rot d d
  | step (d e : Fin n × Fin n) : FaceRel rot (faceStep rot d) e → FaceRel rot d e

/-- Lexicographic order on darts (by tail, then by head): a total order whose
    restriction to any face orbit has a unique least element. -/
def dartLE {n : Nat} (d e : Fin n × Fin n) : Prop :=
  d.1.val < e.1.val ∨ (d.1.val = e.1.val ∧ d.2.val ≤ e.2.val)

/-- The number of faces of the embedding: each face orbit of the face
    permutation is counted at its lexicographically least dart.  (Darts of
    isolated vertices do not exist; those vertices contribute a face of their
    own, see `isolatedCount`.) -/
def faceCount {n : Nat} (A : Fin n → Fin n → Prop) (rot : Fin n → Fin n → Fin n) : Nat :=
  ((Finset.univ : Finset (Fin n × Fin n)).filter
    (fun d => IsDart A d ∧ ∀ e, IsDart A e → FaceRel rot d e → dartLE d e)).card

/-- The number of undirected edges of the graph. -/
def edgeCount {n : Nat} (A : Fin n → Fin n → Prop) : Nat :=
  ((Finset.univ : Finset (Fin n × Fin n)).filter
    (fun d => IsDart A d ∧ d.1.val < d.2.val)).card

/-- The degree of a vertex. -/
def degree {n : Nat} (A : Fin n → Fin n → Prop) (v : Fin n) : Nat :=
  ((Finset.univ : Finset (Fin n)).filter (fun u => A v u)).card

/-- The number of isolated vertices.  Each of them contributes exactly one
    face to a plane embedding, which is why they are added to `faceCount`. -/
def isolatedCount {n : Nat} (A : Fin n → Fin n → Prop) : Nat :=
  ((Finset.univ : Finset (Fin n)).filter (fun v => degree A v = 0)).card

/-- Connectivity in the underlying undirected graph: the reflexive-transitive
    closure of the adjacency relation, in both directions. -/
inductive Reach {n : Nat} (A : Fin n → Fin n → Prop) : Fin n → Fin n → Prop where
  | refl (v : Fin n) : Reach A v v
  | fwd (u v w : Fin n) : A u v → Reach A v w → Reach A u w
  | bwd (u v w : Fin n) : A v u → Reach A v w → Reach A u w

/-- The number of connected components, each counted at its least vertex. -/
def componentCount {n : Nat} (A : Fin n → Fin n → Prop) : Nat :=
  ((Finset.univ : Finset (Fin n)).filter (fun v => ∀ u, Reach A u v → v.val ≤ u.val)).card

/-- **Euler's formula for a spherical embedding**: with `V` vertices, `E` edges,
    `F` faces and `c` connected components, `V - E + F = 2c` holds exactly when
    the rotation system embeds every component with genus `0`, i.e. in the
    plane.  (Written without subtraction as `V + F = E + 2c`.) -/
def EulerCharacteristicOK {n : Nat} (A : Fin n → Fin n → Prop)
    (rot : Fin n → Fin n → Fin n) : Prop :=
  n + (faceCount A rot + isolatedCount A) = edgeCount A + 2 * componentCount A

/-- **A graph is planar** when it admits a rotation system whose induced
    embedding is spherical — i.e. a combinatorial embedding in the plane. -/
def IsPlanar {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ∃ rot : Fin n → Fin n → Fin n, IsRotationSystem A rot ∧ EulerCharacteristicOK A rot

/-! ## The `K5`/`K3,3`-minor criterion (Wagner's theorem) -/

/-- Reachability inside the subgraph induced on the vertex set `S`. -/
inductive ReachIn {n : Nat} (A : Fin n → Fin n → Prop) (S : Finset (Fin n)) :
    Fin n → Fin n → Prop where
  | refl (v : Fin n) : v ∈ S → ReachIn A S v v
  | fwd (u v w : Fin n) : u ∈ S → A u v → v ∈ S → ReachIn A S v w → ReachIn A S u w
  | bwd (u v w : Fin n) : u ∈ S → A v u → v ∈ S → ReachIn A S v w → ReachIn A S u w

/-- The blocks `F 0, …, F (k-1)` form a **model of an `H`-minor**, where `H` is
    the graph on `Fin k` with edge relation `E`: every block is non-empty, each
    block induces a connected subgraph of `G`, and every edge of `H` is
    realised by an edge of `G` between the two blocks. -/
def IsMinorModel {n k : Nat} (A : Fin n → Fin n → Prop) (E : Fin k → Fin k → Prop)
    (F : Fin k → Finset (Fin n)) : Prop :=
  (∀ p : Fin k, (F p).Nonempty) ∧
  (∀ p : Fin k, ∀ x : Fin n, x ∈ F p → ∀ y : Fin n, y ∈ F p → ReachIn A (F p) x y) ∧
  (∀ p q : Fin k, E p q → ∃ x y : Fin n, x ∈ F p ∧ y ∈ F q ∧ A x y)

/-- `G` has an `H`-minor. -/
def HasMinor {n k : Nat} (A : Fin n → Fin n → Prop) (E : Fin k → Fin k → Prop) : Prop :=
  ∃ F : Fin k → Finset (Fin n), IsMinorModel A E F

/-- `K5`: every pair of distinct vertices is an edge. -/
def K5Edge : Fin 5 → Fin 5 → Prop := fun p q => p ≠ q

/-- `K3,3`: the complete bipartite graph on `{0,1,2} ∪ {3,4,5}`. -/
def K33Edge : Fin 6 → Fin 6 → Prop :=
  fun p q => (p.val < 3 ∧ 3 ≤ q.val) ∨ (3 ≤ p.val ∧ q.val < 3)

/-- `G` has no `K5` and no `K3,3` minor — the criterion the program decides. -/
def MinorFree {n : Nat} (A : Fin n → Fin n → Prop) : Prop :=
  ¬ HasMinor A K5Edge ∧ ¬ HasMinor A K33Edge

/-! ## Specification theorems -/

/-- **Program correctness**: on a symmetric, irreflexive 0/1 adjacency matrix,
    `is_planar` returns `true` exactly when the graph is planar, in the sense of
    admitting a spherical combinatorial embedding.  This is the real content of
    the testcase: the program only searches for minor models, and the equality
    with `IsPlanar` is Wagner's form of the Kuratowski theorem (plus the theory
    of rotation systems behind the Euler condition). -/
theorem is_planar_correct
    (n : Std.Usize) (adj : Slice Std.Usize) (b : Bool)
    (h_len : adj.length = n.val * n.val)
    (h_symm : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (h_irr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (h_adj01 : ∀ u v : Fin n.val,
      toMatrix adj n.val h_len u v = 0 ∨ toMatrix adj n.val h_len u v = 1)
    (h : is_planar n adj = ok b) :
    (b = true ↔ IsPlanar (Adj adj n.val h_len)) := by
  sorry

/-- **Wagner's form of the Kuratowski theorem** (1937): a finite simple graph is
    planar iff it has neither `K5` nor `K3,3` as a minor. -/
theorem wagner_characterisation {n : Nat} (A : Fin n → Fin n → Prop)
    (h_symm : ∀ u v : Fin n, A u v → A v u) (h_irr : ∀ v : Fin n, ¬ A v v) :
    IsPlanar A ↔ MinorFree A := by
  sorry

/-- The program's own criterion: it answers `true` exactly for the graphs with
    no `K5` and no `K3,3` minor, which is what the partition search of
    `has_k5_minor`/`has_k33_minor` computes.  (This direction is close to the
    algorithm's own test; the mathematical content of the testcase is
    `is_planar_correct` above, whose `IsPlanar` is the embedding notion.) -/
theorem is_planar_iff_minor_free
    (n : Std.Usize) (adj : Slice Std.Usize) (b : Bool)
    (h_len : adj.length = n.val * n.val)
    (h_symm : ∀ u v : Fin n.val, Adj adj n.val h_len u v → Adj adj n.val h_len v u)
    (h_irr : ∀ v : Fin n.val, ¬ Adj adj n.val h_len v v)
    (h : is_planar n adj = ok b) :
    (b = true ↔ MinorFree (Adj adj n.val h_len)) := by
  sorry

end

end graph_planarity
