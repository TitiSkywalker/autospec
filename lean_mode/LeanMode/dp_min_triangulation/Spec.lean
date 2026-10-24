import LeanMode.dp_min_triangulation.DpMinTriangulation
import Mathlib

open Aeneas Aeneas.Std Result
open dp_min_triangulation

set_option linter.unusedVariables false

/-!
# Specification: minimum-weight triangulation of a convex polygon

`min_triangulation n weight` runs an interval DP over the sub-polygons and
returns the minimum *triangulation cost* of the convex `n`-gon whose edge
weights are read from the flat row-major `n × n` matrix `weight`.

The specification below characterises that cost **declaratively**, with no
mention of the DP, of the interval boundaries, or of the loop order:

* a **triangulation** is a set of `n - 3` pairwise non-crossing diagonals;
* the **faces** it induces are the triples `a < b < c` whose three sides are each
  a polygon side or one of the chosen diagonals (for a non-crossing set this is
  exactly the set of triangular cells — no other vertex lies inside such a
  triangle, since the polygon is convex);
* the **cost** is the sum, over all faces, of the three edge weights of that
  face.

The minimum over all triangulations is `minTriangulationCost`; the theorem says
the DP returns it.
-/

namespace MinTriangulation

/-- Vertices `a < b` are *adjacent* on the `n`-gon (they form a polygon side). -/
abbrev IsSide (n a b : Nat) : Prop :=
  a + 1 = b ∨ (a = 0 ∧ b + 1 = n)

/-- Two diagonals `(a,b)` and `(c,d)` (with `a < b`, `c < d`, all endpoints
distinct) *cross* iff their endpoints interleave around the polygon. -/
abbrev Crosses (a b c d : Nat) : Prop :=
  (a < c ∧ c < b ∧ b < d) ∨ (c < a ∧ a < d ∧ d < b)

/-- Read the weight of the edge `(i, j)` (`i < j`) from the flat row-major
`n × n` weight slice; out-of-range reads fall back to `0`. -/
def edgeWeight (weight : Slice Std.Usize) (n i j : Nat) : Nat :=
  ((weight : List Std.Usize).getD (i * n + j) 0#usize).val

/-- `D` triangulates the convex `n`-gon: it is a set of `n - 3` pairwise
non-crossing diagonals (pairs `a < b < n` that are not polygon sides). -/
abbrev IsTriangulation (n : Nat) (D : Finset (Nat × Nat)) : Prop :=
  (∀ d ∈ D, d.1 < d.2 ∧ d.2 < n) ∧
  (∀ d ∈ D, ¬ IsSide n d.1 d.2) ∧
  D.card = n - 3 ∧
  (∀ d ∈ D, ∀ e ∈ D, d ≠ e → ¬ Crosses d.1 d.2 e.1 e.2)

/-- The triangular faces induced by a diagonal set `D`: the triples `a < b < c`
whose three sides `(a,b)`, `(b,c)`, `(a,c)` are each a polygon side or a
diagonal of `D`.  (`D` is stored as `((a, b), c)`.) -/
def faces (n : Nat) (D : Finset (Nat × Nat)) : Finset ((Nat × Nat) × Nat) :=
  (((Finset.range n).product (Finset.range n)).product (Finset.range n)).filter
    (fun T =>
      T.1.1 < T.1.2 ∧ T.1.2 < T.2 ∧
      (IsSide n T.1.1 T.1.2 ∨ (T.1.1, T.1.2) ∈ D) ∧
      (IsSide n T.1.2 T.2 ∨ (T.1.2, T.2) ∈ D) ∧
      (IsSide n T.1.1 T.2 ∨ (T.1.1, T.2) ∈ D))

/-- The cost of a single triangular face: the sum of its three edge weights. -/
def faceCost (weight : Slice Std.Usize) (n : Nat) (T : (Nat × Nat) × Nat) : Nat :=
  edgeWeight weight n T.1.1 T.1.2 + edgeWeight weight n T.1.2 T.2 +
    edgeWeight weight n T.1.1 T.2

/-- The cost of the triangulation induced by `D`: the sum, over all induced
faces, of their three edge weights. -/
def triangulationCost (weight : Slice Std.Usize) (n : Nat) (D : Finset (Nat × Nat)) : Nat :=
  (faces n D).sum (fun T => faceCost weight n T)

/-- The candidate diagonals of the `n`-gon: all pairs `a < b < n` that are not
sides. -/
def allDiagonals (n : Nat) : Finset (Nat × Nat) :=
  ((Finset.range n).product (Finset.range n)).filter
    (fun p => p.1 < p.2 ∧ ¬ IsSide n p.1 p.2)

/-- All triangulations of the `n`-gon. -/
def allTriangulations (n : Nat) : Finset (Finset (Nat × Nat)) :=
  (allDiagonals n).powerset.filter (IsTriangulation n)

/-- The minimum triangulation cost over all triangulations of the `n`-gon.
(The set is non-empty whenever `3 ≤ n`; that is the hypothesis of the main
theorem.  For `n < 3` the definition defaults to `0`, matching the
implementation.)  `Finset.min` lands in `WithTop Nat` (whose top value stands
for "no triangulation"), and `untopD 0` turns that top value into `0`. -/
noncomputable def minTriangulationCost (weight : Slice Std.Usize) (n : Nat) : Nat :=
  (((allTriangulations n).image (fun D => triangulationCost weight n D)).min).untopD 0

end MinTriangulation

open MinTriangulation

/-- Correctness of `min_triangulation`: for a convex polygon with at least three
vertices, the DP returns the minimum cost over all triangulations, where the cost
of a triangulation is the sum of the three edge weights of each of its `n - 2`
triangles.  `weight` is required to have exactly `n * n` entries (row-major). -/
theorem min_triangulation_correct (n : Std.Usize) (weight : Slice Std.Usize)
    (k : Std.Usize)
    (h_n : 3 ≤ n.val)
    (h_len : weight.length = n.val * n.val)
    (h : min_triangulation n weight = ok k) :
    k.val = minTriangulationCost weight n.val := by
  sorry

/-- Degenerate polygons (`n < 3`) have no triangle, so the cost is `0`. -/
theorem min_triangulation_degenerate (n : Std.Usize) (weight : Slice Std.Usize)
    (k : Std.Usize)
    (h_n : n.val < 3)
    (h : min_triangulation n weight = ok k) :
    k.val = 0 := by
  sorry
