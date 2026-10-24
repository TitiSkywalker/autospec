import LeanMode.dp_subset_convolution.DpSubsetConvolution
import Mathlib

open Aeneas Aeneas.Std Result
open dp_subset_convolution

set_option linter.unusedVariables false

/-!
# Specification: subset convolution (ranked zeta/Möbius transforms)

`subset_convolution a b` runs the `O(2ⁿ n²)` SOS-DP algorithm and returns the
array `c` of length `2ⁿ` with

```text
c[S] = Σ_{T ⊆ S} a[T] · b[S \ T] .
```

The specification below states exactly that closed form — a sum over the
submasks of `S` — with no mention of the rank layers, of the zeta/Möbius
transforms, or of the intermediate tables.
-/

namespace SubsetConvolution

/-- View a flat `usize` slice as a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The `i`-th entry of a flat `usize` slice, read as a `Nat` (`0` out of range). -/
def entry (s : Slice Std.Usize) (i : Nat) : Nat :=
  (toNatList s).getD i 0

/-- `T` is a submask of `S` (as bit masks). -/
abbrev Submask (T S : Nat) : Prop :=
  T &&& S = T

/-- The subset convolution at `S`: the sum of `a[T] * b[S \ T]` over the
submasks `T` of `S`.  For a submask, the natural difference `S - T` is the
bitwise complement of `T` inside `S`, i.e. `S \ T`. -/
def subsetConvAt (a b : Nat → Nat) (S : Nat) : Nat :=
  ((Finset.range (S + 1)).filter (fun T => Submask T S)).sum
    (fun T => a T * b (S - T))

end SubsetConvolution

open SubsetConvolution

/-- Correctness of the ranked-transform subset convolution: whenever
`subset_convolution a b` returns `c`, the `S`-th entry of `c` is
`Σ_{T ⊆ S} a[T] · b[S \ T]`, for every `S < 2ⁿ`.

The hypotheses say that `a` and `b` are both indexed by the subsets of an
`n`-element set, i.e. they both have length `2ⁿ`. -/
theorem subset_convolution_correct (a b : Slice Std.Usize)
    (c : alloc.vec.Vec Std.Usize) (n : Nat)
    (h_a : a.length = 2 ^ n)
    (h_b : b.length = 2 ^ n)
    (hres : subset_convolution a b = ok c) :
    ∀ S, S < 2 ^ n →
      entry (alloc.vec.Vec.deref c) S
        = subsetConvAt (entry a) (entry b) S := by
  sorry
