import LeanMode.array_open_addressing.ArrayOpenAddressing
import Mathlib

open Aeneas Aeneas.Std Result
open array_open_addressing

set_option linter.unusedVariables false

/-!
# Specification: open-addressing hash table (linear probing)

Membership is the usual *extensional* notion — a key is present iff it occurs in
some occupied slot.  The theorems hold under the linear-probing *well-formedness*
invariant `WellFormed`: for every occupied slot `i` holding key `k`, all slots on
the probe path from `k mod cap` up to (but excluding) `i` are occupied — i.e. no
empty slot lies between a key's home slot and the slot it actually occupies.
-/

/-- The keys array as a `List Nat`. -/
def keyList (t : OpenTable) : List Nat :=
  (alloc.vec.Vec.deref t.keys : List Std.Usize).map (·.val)

/-- The occupied flags as a `List Bool`. -/
def occList (t : OpenTable) : List Bool :=
  (alloc.vec.Vec.deref t.occupied : List Bool)

/-- The keys actually stored: `keys[i]` where `occupied[i]` is set. -/
def storedKeysAux : List Nat → List Bool → List Nat
  | [], _ => []
  | _, [] => []
  | k :: ks, o :: os => if o then k :: storedKeysAux ks os else storedKeysAux ks os

def storedKeys (t : OpenTable) : List Nat :=
  storedKeysAux (keyList t) (occList t)

/-- Cyclic forward distance from `home` to `i` around a cycle of length `cap`. -/
def cyclicDist (cap home i : Nat) : Nat :=
  (i + cap - home) % cap

/-- `key` is present in the table: it occurs in some occupied slot. -/
def contains_spec (t : OpenTable) (key : Nat) : Prop :=
  key ∈ storedKeys t

/-- Linear-probing well-formedness: the arrays have equal length, and every
    occupied slot's key sits at the first free position of its own probe
    sequence (all slots from its home `k mod cap` up to it are occupied). -/
def WellFormed (t : OpenTable) : Prop :=
  let cap := (keyList t).length
  (occList t).length = cap ∧
  ∀ i, i < cap → (occList t).getD i false = true →
    let home := (keyList t).getD i 0 % cap
    ∀ j, j < cap → cyclicDist cap home j < cyclicDist cap home i →
      (occList t).getD j false = true

/-! ## Specification theorems -/

/-- A `true` insert adds a previously-absent key and changes nothing else. -/
theorem ot_insert_true (t : OpenTable) (key : Std.Usize) (r : OpenTable)
    (h_cap : 0 < (keyList t).length)
    (h_wf : WellFormed t)
    (h : ot_insert t key = ok (r, true)) :
    ¬ contains_spec t key.val ∧
    Multiset.ofList (storedKeys r) = Multiset.ofList (storedKeys t) + {key.val} := by
  sorry

/-- A `false` insert leaves the stored keys unchanged. -/
theorem ot_insert_false (t : OpenTable) (key : Std.Usize) (r : OpenTable)
    (h_cap : 0 < (keyList t).length)
    (h_wf : WellFormed t)
    (h : ot_insert t key = ok (r, false)) :
    Multiset.ofList (storedKeys r) = Multiset.ofList (storedKeys t) := by
  sorry

/-- Inserting preserves well-formedness. -/
theorem ot_insert_preserves_wf (t : OpenTable) (key : Std.Usize) (r : OpenTable) (b : Bool)
    (h_cap : 0 < (keyList t).length)
    (h_wf : WellFormed t)
    (h : ot_insert t key = ok (r, b)) :
    WellFormed r := by
  sorry

/-- `ot_contains` agrees with the extensional membership `contains_spec`. -/
theorem ot_contains_correct (t : OpenTable) (key : Std.Usize) (res : Bool)
    (h_cap : 0 < (keyList t).length)
    (h_wf : WellFormed t)
    (h : ot_contains t key = ok res) :
    (res = true) ↔ contains_spec t key.val := by
  sorry
