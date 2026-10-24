import LeanMode.graph_erdos_gallai.GraphErdosGallai
import Mathlib

open Aeneas Aeneas.Std Result
open graph_erdos_gallai

set_option linter.unusedVariables false

/-!
# Specification: Erdős–Gallai / Havel–Hakimi graphicality test

`is_graphical d : Result Bool` decides whether the degree sequence `d` is
*graphical*, i.e. whether there is a simple undirected graph whose degree
sequence is `d`.

The mathematics behind the program is

* the **Havel–Hakimi** reduction (which the Rust code literally runs: sort the
  degrees in non-increasing order, take the largest one `d₁`, and subtract `1`
  from the next `d₁` largest degrees), and
* the **Erdős–Gallai theorem** (1960), which characterises the graphical degree
  sequences by a family of linear inequalities.

The two are linked by `erdos_gallai` (the characterisation itself, independent
of the program) and `erdos_gallai_iff_havel_hakimi` (the program decides that
characterisation).
-/

namespace graph_erdos_gallai

/-- The input slice, seen as a list of natural numbers. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (fun x => x.val)

/-- `A` is the adjacency matrix of a simple undirected graph on the vertex set
`Fin n`: entries are `0`/`1`, the matrix is symmetric, and there are no loops. -/
def IsSimpGraph (n : Nat) (A : Fin n → Fin n → Nat) : Prop :=
  (∀ i j, A i j ≤ 1) ∧
  (∀ i j, A i j = A j i) ∧
  (∀ i, A i i = 0)

/-- The degree of vertex `i` in the graph `A`. -/
def adjDegree (n : Nat) (A : Fin n → Fin n → Nat) (i : Fin n) : Nat :=
  ∑ j : Fin n, A i j

/-- The degree sequence of `A`, listed in vertex order. -/
def degreeSeq (n : Nat) (A : Fin n → Fin n → Nat) : List Nat :=
  List.ofFn (fun i : Fin n => adjDegree n A i)

/-- A degree sequence `d` is **graphical** when some simple undirected graph
realises it: there is a symmetric, loopless `0/1` adjacency on `Fin n` whose
degree sequence is `d` up to the order of the degrees.

(For a non-increasing `d` this is the usual notion: the graph's degree sequence,
sorted, is exactly `d`; see `NonIncreasing` and `EGCondition`.) -/
def IsGraphical (d : List Nat) : Prop :=
  ∃ (n : Nat) (A : Fin n → Fin n → Nat),
    IsSimpGraph n A ∧ (degreeSeq n A).Perm d

/-- `d` is sorted in non-increasing order. -/
def NonIncreasing (d : List Nat) : Prop :=
  ∀ i j, i < j → j < d.length → d.getD j 0 ≤ d.getD i 0

/-- The **Erdős–Gallai condition** on a degree sequence `d`: `d` is sorted in
non-increasing order, its degree sum is even, and for every `k`
`∑_{i<k} d_i ≤ k(k-1) + ∑_{i≥k} min(d_i, k)`. -/
def EGCondition (d : List Nat) : Prop :=
  NonIncreasing d ∧
  (2 ∣ d.sum) ∧
  (∀ k, k ≤ d.length →
    (Finset.range k).sum (fun i => d.getD i 0) ≤
      k * (k - 1) +
        ((Finset.range d.length).filter (fun i => k ≤ i)).sum
          (fun i => min (d.getD i 0) k))

/-! ## Specification theorems -/

/-- **Correctness of `is_graphical`.** The program returns `true` exactly for
the graphical degree sequences.  (The program's internal Havel–Hakimi reduction
first sorts the input, so no sortedness hypothesis is needed here.) -/
theorem is_graphical_correct (d : Slice Std.Usize) (b : Bool)
    (h : is_graphical d = ok b) :
    (b = true ↔ IsGraphical (toNatList d)) := by
  sorry

/-- **Erdős–Gallai theorem (1960).** A degree sequence in non-increasing order
is graphical if and only if its sum is even and all the Erdős–Gallai
inequalities hold.  (The sortedness hypothesis is needed because a graphical
sequence in an *arbitrary* order need not be non-increasing.) -/
theorem erdos_gallai (d : List Nat) (h : NonIncreasing d) :
    IsGraphical d ↔ EGCondition d := by
  sorry

/-- The Havel–Hakimi program decides exactly the Erdős–Gallai condition: on
non-increasing input, `is_graphical` returns `true` iff `EGCondition` holds. -/
theorem erdos_gallai_iff_havel_hakimi (d : Slice Std.Usize) (b : Bool)
    (hs : NonIncreasing (toNatList d))
    (h : is_graphical d = ok b) :
    (b = true ↔ EGCondition (toNatList d)) := by
  sorry

end graph_erdos_gallai
