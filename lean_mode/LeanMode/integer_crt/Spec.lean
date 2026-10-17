import LeanMode.integer_crt.IntegerCrt
import Mathlib

open Aeneas Aeneas.Std Result
open integer_crt

set_option linter.unusedVariables false

/-!
# Specification: Chinese Remainder Theorem

`crt residues moduli` returns `x` with `x ≡ residues[i] (mod moduli[i])`
for every index `i` (for pairwise-coprime, positive moduli — asserted).
-/

/-- `x` satisfies every congruence `x ≡ r (mod m)` component-wise. -/
def crt_solution (residues moduli : List Int) (x : Int) : Prop :=
  List.Forall₂ (fun r m => (x - r) % m = 0) residues moduli

theorem crt_correct (residues moduli : Slice Int) (result : Int)
    (h_len : (residues : List Int).length = (moduli : List Int).length)
    (h : crt residues moduli = ok result) :
    crt_solution (residues : List Int) (moduli : List Int) result ∧ 0 ≤ result := by
  sorry
