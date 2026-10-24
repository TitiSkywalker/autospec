import LeanMode.graph_chu_liu_edmonds.GraphChuLiuEdmonds
import Mathlib

open Aeneas Aeneas.Std Result
open graph_chu_liu_edmonds

set_option linter.unusedVariables false

/-!
# Specification: Minimum-weight spanning arborescence (Chu–Liu / Edmonds)

`optimum_branching nv from to w r` computes a **minimum-weight spanning
arborescence** (branching) rooted at `r` of the directed graph on `nv`
vertices `0, …, nv-1` whose edges are the flat lists `from` / `to` / `w`
(parallel edges allowed, self-loops ignored).  It returns `(exists, weight)`,
where `exists = false` means that no spanning arborescence rooted at `r`
exists (and then `weight = 0`).

The algorithm is Chu–Liu / Edmonds (1965 / 1967): repeatedly choose the
minimum-weight incoming edge of every non-root vertex; if those choices close
a directed cycle, contract the cycle into a single supernode (re-weighting the
edges that enter it by subtracting the minimum in-edge weight of their head)
and iterate.

The mathematical content is the **optimum branching theorem**: the weight
returned is attained by an arborescence `E`, and no arborescence has smaller
weight.  The predicate `IsArborescence` spells out "spanning arborescence
rooted at `r`": every vertex other than `r` has **exactly one** incoming edge
from `E`, `r` has none, edges are not self-loops, and following the (then
uniquely determined) parent pointers from any vertex reaches `r`.
-/

namespace graph_chu_liu_edmonds

/-- View a `Slice` of vertices of length `n` as the function `Fin n → Nat`
    (the vertex label stored at each position). -/
def sliceFun (s : Slice Std.Usize) (n : Nat) (h : s.length = n) : Fin n → Nat :=
  λ i =>
    have hb : i.val < s.length := by rw [h]; exact i.is_lt
    (s[i.val]'hb).val

/-- Follow the parent pointers determined by the edge set `E` for at most `k`
    steps: `reachesIn E src tgt r k v` holds iff `v = r`, or `v` has an
    in-edge `i ∈ E` and the source of `i` reaches `r` in `k` steps. -/
def reachesIn {m : Nat} (E : Finset (Fin m)) (src tgt : Fin m → Nat) (r : Nat) :
    Nat → Nat → Prop
  | 0, v => v = r
  | k + 1, v =>
      v = r ∨ ∃ i : Fin m, i ∈ E ∧ tgt i = v ∧ reachesIn E src tgt r k (src i)

/-- `E` is a spanning arborescence (branching) of the graph `(src, tgt)` on the
    `nv` vertices `0, …, nv-1`, rooted at `r`:
    every selected edge stays inside the vertex set and is not a self-loop,
    every vertex other than `r` has exactly one selected in-edge, `r` has none,
    and every vertex reaches `r` by following the selected in-edges. -/
def IsArborescence {nv m : Nat} (E : Finset (Fin m))
    (src tgt : Fin m → Nat) (r : Nat) : Prop :=
  (∀ i : Fin m, i ∈ E → src i < nv ∧ tgt i < nv) ∧
  (∀ i : Fin m, i ∈ E → src i ≠ tgt i) ∧
  (∀ i : Fin m, i ∈ E → tgt i ≠ r) ∧
  (∀ v : Fin nv, v.val ≠ r → ∃! i : Fin m, i ∈ E ∧ tgt i = v.val) ∧
  (∀ v : Fin nv, v.val ≠ r → reachesIn E src tgt r nv v.val)

/-- The total weight of the selected edge set. -/
def weight {m : Nat} (E : Finset (Fin m)) (w : Fin m → Nat) : Nat :=
  ∑ e ∈ E, w e

/-- A spanning arborescence rooted at `r` exists. -/
def exists_arborescence (nv m : Nat) (src tgt : Fin m → Nat) (r : Nat) : Prop :=
  ∃ E : Finset (Fin m), IsArborescence (nv := nv) E src tgt r

/-! ## Specification theorems -/

/-- **Existence**: the `exists` flag returned by `optimum_branching` is `true`
    if and only if a spanning arborescence rooted at `r` exists. -/
theorem optimum_branching_exists
    (nv : Std.Usize) (esrc etgt ew : Slice Std.Usize) (r : Std.Usize)
    (ex : Bool) (wt : Std.Usize) (m : Nat)
    (h_src : esrc.length = m) (h_tgt : etgt.length = m) (h_w : ew.length = m)
    (h_r : r.val < nv.val)
    (hcall : optimum_branching nv esrc etgt ew r = ok (ex, wt)) :
    (ex = true ↔
      exists_arborescence nv.val m
        (sliceFun esrc m h_src) (sliceFun etgt m h_tgt) r.val) := by
  sorry

/-- **Optimality** (the Chu–Liu / Edmonds optimum branching theorem): when the
    algorithm reports success there is an arborescence `E` rooted at `r` whose
    weight is exactly the returned weight, and no arborescence rooted at `r` has
    strictly smaller weight. -/
theorem optimum_branching_optimal
    (nv : Std.Usize) (esrc etgt ew : Slice Std.Usize) (r : Std.Usize)
    (ex : Bool) (wt : Std.Usize) (m : Nat)
    (h_src : esrc.length = m) (h_tgt : etgt.length = m) (h_w : ew.length = m)
    (h_r : r.val < nv.val)
    (hcall : optimum_branching nv esrc etgt ew r = ok (ex, wt)) :
    ex = true →
      ∃ E : Finset (Fin m),
        IsArborescence (nv := nv.val) E
          (sliceFun esrc m h_src) (sliceFun etgt m h_tgt) r.val ∧
        weight E (sliceFun ew m h_w) = wt.val ∧
        (∀ E' : Finset (Fin m),
          IsArborescence (nv := nv.val) E'
            (sliceFun esrc m h_src) (sliceFun etgt m h_tgt) r.val →
          wt.val ≤ weight E' (sliceFun ew m h_w)) := by
  sorry

/-- **Failure branch**: when the algorithm reports `exists = false` (no
    spanning arborescence), the returned weight is `0`. -/
theorem optimum_branching_zero
    (nv : Std.Usize) (esrc etgt ew : Slice Std.Usize) (r : Std.Usize)
    (ex : Bool) (wt : Std.Usize)
    (hcall : optimum_branching nv esrc etgt ew r = ok (ex, wt)) :
    ex = false → wt.val = 0 := by
  sorry

end graph_chu_liu_edmonds
