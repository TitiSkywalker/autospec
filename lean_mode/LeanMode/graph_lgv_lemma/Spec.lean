import LeanMode.graph_lgv_lemma.GraphLgvLemma
import Mathlib

open Aeneas Aeneas.Std Result
open graph_lgv_lemma
open Matrix

set_option linter.unusedVariables false

/-!
# Specification: Lindström–Gessel–Viennot (LGV) lemma

`graph_lgv_lemma n adj k src snk` returns `ok result` where `adj` is a flat,
row-major, `n × n` 0/1 adjacency matrix of a DAG whose edges only go *forward*
(`adj u v = 1 ⟹ u < v`), `src`/`snk` are length-`k` slices of distinct source /
sink vertices, and `result` is the determinant of the `k × k` matrix
`M[i,j] = e(src[i], snk[j])` = number of (unweighted) paths from source `i` to
sink `j`.

The LGV lemma says this determinant equals the *signed* sum over vertex-disjoint
path families (intersecting families cancel).  The subtle part of the proof is
that cancellation — a determinant expansion plus a sign-reversing involution on
intersecting families (Cauchy–Binet style).
-/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Int`. -/
def toMatrix (s : Slice Int) (n : Nat) (h : s.length = n * n) : Fin n → Fin n → Int :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    s[i.val * n + j.val]'h_bound

/-- View a length-`k` vertex-index slice as `Fin k → Nat`. -/
def toVertexVec (s : Slice Std.Usize) (k : Nat) (h : s.length = k) : Fin k → Nat :=
  λ i =>
    have hb : i.val < s.length := by rw [h]; exact i.is_lt
    (s[i.val]'hb).val

noncomputable section

/-- `S` is a path of the forward DAG `adj`: at least two vertices, and
    consecutive vertices (in increasing order) are joined by an edge.  The
    endpoints of the path are its minimum and maximum elements. -/
def IsPathSet (adj : Fin n → Fin n → Int) (S : Finset (Fin n)) : Prop :=
  2 ≤ S.card ∧
  ∀ x y : Fin n, x ∈ S → y ∈ S → x.val < y.val →
    (∀ z : Fin n, x.val < z.val → z.val < y.val → z ∉ S) → adj x y = 1

/-- The set of all paths from `u` to `v` (a finite set of vertex sets). -/
def paths (adj : Fin n → Fin n → Int) (u v : Fin n) : Finset (Finset (Fin n)) := by
  classical
  exact (Finset.univ : Finset (Finset (Fin n))).filter
    (λ S => u ∈ S ∧ v ∈ S ∧ IsPathSet adj S ∧
            (∀ x : Fin n, x ∈ S → u.val ≤ x.val ∧ x.val ≤ v.val))

/-- The path-count matrix `e(A_i, B_j)`. -/
def e_matrix {n k : Nat} (adj : Fin n → Fin n → Int) (src snk : Fin k → Fin n)
    : Matrix (Fin k) (Fin k) Int :=
  λ i j => ((paths adj (src i) (snk j)).card : Int)

/-- A vertex-disjoint path family for the permutation `σ`: path `i` goes from
    `src i` to `snk (σ i)`, and all paths are pairwise disjoint. -/
def disjoint_family {n k : Nat} (adj : Fin n → Fin n → Int) (src snk : Fin k → Fin n)
    (σ : Equiv.Perm (Fin k)) (P : Fin k → Finset (Fin n)) : Prop :=
  (∀ i : Fin k, P i ∈ paths adj (src i) (snk (σ i))) ∧
  (∀ i j : Fin k, i ≠ j → Disjoint (P i) (P j))

/-- Number of vertex-disjoint path families realising the permutation `σ`. -/
def disjoint_family_count {n k : Nat} (adj : Fin n → Fin n → Int) (src snk : Fin k → Fin n)
    (σ : Equiv.Perm (Fin k)) : Nat := by
  classical
  exact ((Finset.univ : Finset (Fin k → Finset (Fin n))).filter
    (λ P => disjoint_family adj src snk σ P)).card

/-- The sign of a permutation, as an integer. -/
def sign_int {k : Nat} (σ : Equiv.Perm (Fin k)) : Int :=
  (Equiv.Perm.sign σ).val

/-- The signed sum over vertex-disjoint path families. -/
def lgv_rhs {n k : Nat} (adj : Fin n → Fin n → Int) (src snk : Fin k → Fin n) : Int :=
  ∑ σ : Equiv.Perm (Fin k), sign_int σ * (disjoint_family_count adj src snk σ : Int)

end


/-! ## Specification theorems -/

/-- The LGV lemma: the determinant of the path-count matrix equals the signed
    sum over vertex-disjoint path families (requires the source and sink sets to
    be disjoint, so no trivial empty path can occur). -/
theorem lgv_identity {n k : Nat} (adj : Fin n → Fin n → Int) (src snk : Fin k → Fin n)
    (h_disjoint : ∀ i j : Fin k, src i ≠ snk j) :
    Matrix.det (e_matrix adj src snk) = lgv_rhs adj src snk := by
  sorry

/-- `graph_lgv_lemma` returns the determinant of the path-count matrix. -/
theorem graph_lgv_lemma_correct
    (n : Std.Usize) (adj : Slice Int) (k : Std.Usize) (src snk : Slice Std.Usize) (result : Int)
    (h_adj : adj.length = n.val * n.val)
    (h_src : src.length = k.val)
    (h_snk : snk.length = k.val)
    (h_forward : ∀ u v : Fin n.val, toMatrix adj n.val h_adj u v = 1 → u.val < v.val)
    (h_adj01 : ∀ u v : Fin n.val,
       toMatrix adj n.val h_adj u v = 0 ∨ toMatrix adj n.val h_adj u v = 1)
    (h_src_lt : ∀ i : Fin k.val, toVertexVec src k.val h_src i < n.val)
    (h_snk_lt : ∀ i : Fin k.val, toVertexVec snk k.val h_snk i < n.val)
    (h_src_ne_snk : ∀ i j : Fin k.val,
       toVertexVec src k.val h_src i ≠ toVertexVec snk k.val h_snk j)
    (h : graph_lgv_lemma n adj k src snk = ok result) :
    result = Matrix.det (e_matrix (toMatrix adj n.val h_adj)
        (λ i : Fin k.val => ⟨toVertexVec src k.val h_src i, h_src_lt i⟩)
        (λ i : Fin k.val => ⟨toVertexVec snk k.val h_snk i, h_snk_lt i⟩)) := by
  sorry
