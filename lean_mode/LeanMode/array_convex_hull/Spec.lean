import LeanMode.array_convex_hull.ArrayConvexHull
import Mathlib

open Aeneas Aeneas.Std Result
open array_convex_hull

set_option linter.unusedVariables false

/-!
# Specification: convex hull (Andrew's monotone chain)

`convex_hull points n` returns the extreme points in counter-clockwise order:
a convex polygon whose vertices are a subset of the input and that contains
every input point.
-/

/-- Read point `i` from a flat `[x0, y0, x1, y1, …]` coordinate list. -/
def pt (l : List Rat) (i : Nat) : Rat × Rat := (l.getD (2 * i) 0, l.getD (2 * i + 1) 0)

/-- Cross product `(b - a) × (c - a)` of three points. -/
def orient (a b c : Rat × Rat) : Rat :=
  (b.1 - a.1) * (c.2 - a.2) - (b.2 - a.2) * (c.1 - a.1)

/-- `p` is inside or on the CCW convex polygon with vertices `hull` (a flat list of `m` points). -/
def insideConvex (hull : List Rat) (m : Nat) (p : Rat × Rat) : Prop :=
  ∀ e, e < m → 0 ≤ orient (pt hull e) (pt hull ((e + 1) % m)) p

/-- The hull is convex (all consecutive turns are strictly left, CCW) and contains every input point. -/
theorem convex_hull_correct (points : Slice Rat) (n : Std.Usize)
    (result : alloc.vec.Vec Rat) (h : convex_hull points n = ok result)
    (h_len : (points : List Rat).length = 2 * n.val) :
    let hull := (alloc.vec.Vec.deref result : List Rat)
    let m := hull.length / 2
    (∀ v, v < m → ∃ i, i < n.val ∧ pt hull v = pt (points : List Rat) i) ∧
    (∀ e, e < m → 0 ≤ orient (pt hull e) (pt hull ((e + 1) % m)) (pt hull ((e + 2) % m))) ∧
    (∀ i, i < n.val → insideConvex hull m (pt (points : List Rat) i)) := by
  sorry
