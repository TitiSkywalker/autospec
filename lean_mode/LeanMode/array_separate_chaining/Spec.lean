import LeanMode.array_separate_chaining.ArraySeparateChaining
import Mathlib

open Aeneas Aeneas.Std Result
open array_separate_chaining

set_option linter.unusedVariables false

/-!
# Specification: separate-chaining hash table

`ct_insert` / `ct_contains` search the bucket chain `head[key mod nb]` by
following `next` pointers — so membership is defined by *chain reachability*,
not by the flat `keys` array.  (The flat `keys` array records every stored key,
but a key is only "present" if it is actually reachable from its bucket's head.)
-/

/-- The head array as a `List (Option Std.Usize)`. -/
def headList (t : ChainTable) : List (Option Std.Usize) :=
  (alloc.vec.Vec.deref t.head : List (Option Std.Usize))

/-- The next array as a `List (Option Std.Usize)`. -/
def nextList (t : ChainTable) : List (Option Std.Usize) :=
  (alloc.vec.Vec.deref t.next : List (Option Std.Usize))

/-- The keys array as a `List Nat`. -/
def keyList (t : ChainTable) : List Nat :=
  (alloc.vec.Vec.deref t.keys : List Std.Usize).map (·.val)

/-- The key stored at node `node` (0 if out of bounds). -/
def keyOf (t : ChainTable) (node : Nat) : Nat :=
  (keyList t).getD node 0

/-- The next node in the chain from `node`, as `Option Nat`. -/
def nextOf (t : ChainTable) (node : Nat) : Option Nat :=
  match (nextList t).getD node none with
  | none => none
  | some n => some n.val

/-- The head of bucket `b`, as `Option Nat`. -/
def headOf (t : ChainTable) (b : Nat) : Option Nat :=
  match (headList t).getD b none with
  | none => none
  | some n => some n.val

/-- `key` is reachable from `start` by following `next` pointers within `fuel` steps. -/
def chainHas (t : ChainTable) (start : Option Nat) (key : Nat) (fuel : Nat) : Prop :=
  match fuel with
  | 0 => False
  | fuel' + 1 =>
    match start with
    | none => False
    | some node => keyOf t node = key ∨ chainHas t (nextOf t node) key fuel'

/-- `key` is present in the table: reachable from its bucket's head. -/
def contains_spec (t : ChainTable) (key : Nat) : Prop :=
  chainHas t (headOf t (key % (headList t).length)) key ((keyList t).length + 1)

/-! ## Specification theorems -/

/-- A `true` insert adds a key that was not reachable in its bucket's chain. -/
theorem ct_insert_true (t : ChainTable) (key : Std.Usize) (r : ChainTable)
    (h_buckets : 0 < (headList t).length)
    (h : ct_insert t key = ok (r, true)) :
    ¬ contains_spec t key.val ∧
    Multiset.ofList (keyList r) = Multiset.ofList (keyList t) + {key.val} := by
  sorry

/-- A `false` insert leaves the stored keys unchanged. -/
theorem ct_insert_false (t : ChainTable) (key : Std.Usize) (r : ChainTable)
    (h_buckets : 0 < (headList t).length)
    (h : ct_insert t key = ok (r, false)) :
    keyList r = keyList t := by
  sorry

/-- `ct_contains` agrees with `contains_spec`. -/
theorem ct_contains_correct (t : ChainTable) (key : Std.Usize) (res : Bool)
    (h_buckets : 0 < (headList t).length)
    (h : ct_contains t key = ok res) :
    (res = true) ↔ contains_spec t key.val := by
  sorry
