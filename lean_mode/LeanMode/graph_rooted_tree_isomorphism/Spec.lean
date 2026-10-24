import LeanMode.graph_rooted_tree_isomorphism.GraphRootedTreeIsomorphism
import Mathlib

open Aeneas Aeneas.Std Result
open graph_rooted_tree_isomorphism

set_option linter.unusedVariables false

/-!
# Specification: Rooted Tree Isomorphism (AHU)

`graph_rooted_tree_isomorphism n1 parent1 root1 n2 parent2 root2` returns `ok result`
where each tree is given by a parent array (`parent[v]` is the parent of `v`,
root satisfies `parent[root] = root`) and a root index.

The AHU algorithm assigns each vertex a canonical label by induction on depth:
a node's label is the rank (within its level, over *both* trees jointly) of the
sorted multiset of its children's labels.  Two rooted trees are isomorphic iff
their roots get the same label.  The subtle part of the proof is the induction
that this re-ranking is a complete isomorphism invariant (canonical-form
induction).
-/

/-- View a length-`n` parent slice as `Fin n → Nat`. -/
def toParentNat (s : Slice Std.Usize) (n : Nat) (h : s.length = n) : Fin n → Nat :=
  λ i =>
    have hb : i.val < s.length := by rw [h]; exact i.is_lt
    (s[i.val]'hb).val

/-- View a length-`n` parent slice as `Fin n → Fin n`, given that every entry is
    a valid vertex index. -/
def toParent (s : Slice Std.Usize) (n : Nat) (h : s.length = n)
    (hb : ∀ i : Fin n, toParentNat s n h i < n) : Fin n → Fin n :=
  λ i => ⟨toParentNat s n h i, hb i⟩

/-- `par` is a rooted tree: the root is its own parent, and every vertex reaches
    the root by iterating the parent map. -/
def is_rooted_tree {n : Nat} (par : Fin n → Fin n) (r : Fin n) : Prop :=
  par r = r ∧ ∀ u : Fin n, ∃ k : Nat, Nat.iterate par k u = r

/-- Two rooted trees are isomorphic when there is a vertex bijection that sends
    root to root and commutes with the parent map. -/
def rooted_isomorphic {n1 n2 : Nat} (par1 : Fin n1 → Fin n1) (r1 : Fin n1)
    (par2 : Fin n2 → Fin n2) (r2 : Fin n2) : Prop :=
  ∃ φ : Fin n1 ≃ Fin n2, φ r1 = r2 ∧ ∀ u : Fin n1, φ (par1 u) = par2 (φ u)


/-! ## Specification theorem -/

/-- `graph_rooted_tree_isomorphism` returns `true` iff the two rooted trees are
    isomorphic. -/
theorem graph_rooted_tree_isomorphism_correct
    (n1 n2 : Std.Usize) (parent1 parent2 : Slice Std.Usize) (root1 root2 : Std.Usize) (result : Bool)
    (h_p1 : parent1.length = n1.val)
    (h_p2 : parent2.length = n2.val)
    (h_r1 : root1.val < n1.val)
    (h_r2 : root2.val < n2.val)
    (h_p1b : ∀ i : Fin n1.val, toParentNat parent1 n1.val h_p1 i < n1.val)
    (h_p2b : ∀ i : Fin n2.val, toParentNat parent2 n2.val h_p2 i < n2.val)
    (h_tree1 : is_rooted_tree (toParent parent1 n1.val h_p1 h_p1b) ⟨root1.val, h_r1⟩)
    (h_tree2 : is_rooted_tree (toParent parent2 n2.val h_p2 h_p2b) ⟨root2.val, h_r2⟩)
    (h : graph_rooted_tree_isomorphism n1 parent1 root1 n2 parent2 root2 = ok result) :
    (result = true) ↔ rooted_isomorphic
        (toParent parent1 n1.val h_p1 h_p1b) ⟨root1.val, h_r1⟩
        (toParent parent2 n2.val h_p2 h_p2b) ⟨root2.val, h_r2⟩ := by
  sorry
