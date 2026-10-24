import LeanMode.ds_lct.DsLct
import Mathlib

open Aeneas Aeneas.Std Result
open ds_lct

set_option linter.unusedVariables false

/-!
# Specification: Link-Cut Tree

`lct_build` creates `n` isolated nodes whose node `i` (1-indexed) has value
`vals[i-1]`.  `lct_link` connects two nodes in different trees, `lct_cut`
removes an edge, `lct_findroot` returns the root of a node's tree, and
`lct_path_sum` returns the sum of the node values on the unique tree path.
Theorems below state the path-sum and root behavior for small trees built from
an initially-empty forest.
-/

/-- The value of 1-indexed node `i` (0 if out of range). -/
def valAt (vals : Slice Int) (i : Nat) : Int :=
  (vals : List Int).getD (i - 1) 0

/-- In an empty forest every node is its own root. -/
theorem findroot_isolated (n : Std.Usize) (vals : Slice Int) (t t' : Lct) (x r : Std.Usize)
    (h_build : lct_build n vals = ok t)
    (h_q : lct_findroot t x = ok (t', r)) :
    r = x := by
  sorry

/-- Linking two isolated nodes `x`, `y` makes the `x`–`y` path exactly the new
    edge, so its value sum is `val[x] + val[y]`. -/
theorem link_path_sum_2 (n : Std.Usize) (vals : Slice Int) (t t' t'' : Lct)
    (x y : Std.Usize) (s : Int)
    (h_build : lct_build n vals = ok t)
    (h_xy : x ≠ y)
    (h_link : lct_link t x y = ok t')
    (h_q : lct_path_sum t' x y = ok (t'', s)) :
    s = valAt vals x.val + valAt vals y.val := by
  sorry

/-- Linking `a`–`b` then `b`–`c` (all distinct) makes the `a`–`c` path the
    three-node chain `a`–`b`–`c`, whose value sum is
    `val[a] + val[b] + val[c]`. -/
theorem chain_path_sum_3 (n : Std.Usize) (vals : Slice Int) (t t1 t2 t3 : Lct)
    (a b c : Std.Usize) (s : Int)
    (h_build : lct_build n vals = ok t)
    (h_ab : a ≠ b) (h_ac : a ≠ c) (h_bc : b ≠ c)
    (h_link1 : lct_link t a b = ok t1)
    (h_link2 : lct_link t1 b c = ok t2)
    (h_q : lct_path_sum t2 a c = ok (t3, s)) :
    s = valAt vals a.val + valAt vals b.val + valAt vals c.val := by
  sorry

/-- Cutting the edge `x`–`y` disconnects the two nodes: afterwards `findroot`
    distinguishes them. -/
theorem cut_disconnects (n : Std.Usize) (vals : Slice Int) (t t1 t2 t3 t4 : Lct)
    (x y : Std.Usize) (rx ry : Std.Usize)
    (h_build : lct_build n vals = ok t)
    (h_link : lct_link t x y = ok t1)
    (h_cut : lct_cut t1 x y = ok t2)
    (h_rx : lct_findroot t2 x = ok (t3, rx))
    (h_ry : lct_findroot t3 y = ok (t4, ry)) :
    rx ≠ ry := by
  sorry
