import LeanMode.array_fnv.ArrayFnv
import Mathlib

open Aeneas Aeneas.Std Result
open array_fnv

set_option linter.unusedVariables false

/-!
# Specification: FNV-1a 64-bit hash

`fnv1a` is the FNV-1a fold: start from the 64-bit offset basis and, for each
symbol (an integer-encoded byte), XOR it into the hash and multiply by the FNV
prime — all in `BitVec 64` (wrapping) arithmetic.
-/

/-- The FNV-1a 64-bit fold over a list of symbols. -/
def fnv_go : BitVec 64 → List (BitVec 64) → BitVec 64
  | h, [] => h
  | h, x :: xs => fnv_go ((h ^^^ x) * FNV_PRIME) xs

/-- FNV-1a of a symbol list. -/
def fnv_spec (l : List (BitVec 64)) : BitVec 64 :=
  fnv_go FNV_OFFSET l

/-! ## Specification theorems -/

/-- `fnv1a` equals the FNV-1a fold. -/
theorem fnv1a_correct (data : Slice (BitVec 64)) (result : BitVec 64)
    (h : fnv1a data = ok result) :
    result = fnv_spec (data : List (BitVec 64)) := by
  sorry
