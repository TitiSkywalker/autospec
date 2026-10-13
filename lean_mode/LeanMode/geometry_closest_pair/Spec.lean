import LeanMode.geometry_closest_pair.GeometryClosestPair
import Mathlib

open Aeneas Aeneas.Std Result
open geometry_closest_pair

set_option linter.unusedVariables false

/-!
# Specification: closest pair (divide and conquer)

`closest_pair points n` returns the minimum squared Euclidean distance over
all unordered pairs of the `n` points.
-/

/-- Read point `i` from a flat `[x0, y0, x1, y1, …]` coordinate list. -/
def pt (l : List Rat) (i : Nat) : Rat × Rat := (l.getD (2 * i) 0, l.getD (2 * i + 1) 0)

/-- Squared Euclidean distance between two points. -/
def sqDist (p q : Rat × Rat) : Rat :=
  let dx := p.1 - q.1
  let dy := p.2 - q.2
  dx * dx + dy * dy

/-- `closest_pair` returns the minimum squared distance, attained by some pair. -/
theorem closest_pair_correct (points : Slice Rat) (n : Std.Usize) (result : Rat)
    (h : closest_pair points n = ok result)
    (h_ge : 2 ≤ n.val) :
    (∀ i j, i < j → j < n.val →
       result ≤ sqDist (pt (points : List Rat) i) (pt (points : List Rat) j)) ∧
    (∃ i j, i < j ∧ j < n.val ∧
       result = sqDist (pt (points : List Rat) i) (pt (points : List Rat) j)) := by
  sorry
