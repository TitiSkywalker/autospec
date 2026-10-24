import LeanMode.graph_stable_match.GraphStableMatch
import Mathlib

open Aeneas Aeneas.Std Result
open graph_stable_match

set_option linter.unusedVariables false

/-!
# Specification: Stable Marriage (Gale–Shapley)

`stable_match n rank_m rank_w` returns `ok matched` where `rank_m` / `rank_w`
are flat, row-major `n × n` rank matrices (`rank_m m w` is the rank of woman
`w` in man `m`'s preference order, `0` = most preferred) and `matched` is a
flat `n × n` 0/1 matrix with `matched u v = 1` when man `u` is matched to
woman `v`.

A matching is *stable* when there is no man–woman pair that both strictly
prefer each other over their assigned partner.  The theorem asserts that the
returned matching is stable (the subtle part of the proof is the Gale–Shapley
stability argument).
-/

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Nat`. -/
def toMatrixNat (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n)
    : Fin n → Fin n → Nat :=
  λ i j =>
    have h_bound : i.val * n + j.val < s.length := by
      rw [h]
      have hi : i.val < n := i.is_lt
      have hj : j.val < n := j.is_lt
      calc
        i.val * n + j.val < i.val * n + n := Nat.add_lt_add_left hj (i.val * n)
        _ = (i.val + 1) * n := by ring
        _ ≤ n * n := Nat.mul_le_mul_right n (Nat.succ_le_of_lt hi)
    (s[i.val * n + j.val]'h_bound).val

/-- `matched` is a matching: 0/1 entries, at most one partner per man and per
    woman. -/
def is_matching {n : Nat} (matched : Fin n → Fin n → Nat) : Prop :=
  (∀ u v : Fin n, matched u v = 0 ∨ matched u v = 1) ∧
  (∀ u : Fin n, (∑ v : Fin n, matched u v) ≤ 1) ∧
  (∀ v : Fin n, (∑ u : Fin n, matched u v) ≤ 1)

/-- Man `m` strictly prefers woman `w1` over `w2`. -/
def prefers_m {n : Nat} (rank_m : Fin n → Fin n → Nat) (m w1 w2 : Fin n) : Prop :=
  rank_m m w1 < rank_m m w2

/-- Woman `w` strictly prefers man `m1` over `m2`. -/
def prefers_w {n : Nat} (rank_w : Fin n → Fin n → Nat) (w m1 m2 : Fin n) : Prop :=
  rank_w w m1 < rank_w w m2

/-- `(m, w)` is a blocking pair: both prefer each other over their current
    partner (or are single). -/
def blocking_pair {n : Nat} (rank_m rank_w : Fin n → Fin n → Nat)
    (matched : Fin n → Fin n → Nat) (m w : Fin n) : Prop :=
  ((∀ w' : Fin n, matched m w' = 0) ∨
     (∃ w' : Fin n, matched m w' = 1 ∧ prefers_m rank_m m w w')) ∧
  ((∀ m' : Fin n, matched m' w = 0) ∨
     (∃ m' : Fin n, matched m' w = 1 ∧ prefers_w rank_w w m m'))

/-- `matched` is a stable matching: it is a matching with no blocking pair. -/
def is_stable {n : Nat} (rank_m rank_w : Fin n → Fin n → Nat)
    (matched : Fin n → Fin n → Nat) : Prop :=
  is_matching matched ∧ ∀ m w : Fin n, ¬ blocking_pair rank_m rank_w matched m w


/-! ## Specification theorems -/

/-- `stable_match` returns a matching matrix of length `n * n`. -/
theorem stable_match_length
    (n : Std.Usize) (rank_m rank_w : Slice Std.Usize) (matched : alloc.vec.Vec Std.Usize)
    (h : stable_match n rank_m rank_w = ok matched) :
    (alloc.vec.Vec.deref matched).length = n.val * n.val := by
  sorry

/-- Soundness: the returned matrix is a matching. -/
theorem stable_match_is_matching
    (n : Std.Usize) (rank_m rank_w : Slice Std.Usize) (matched : alloc.vec.Vec Std.Usize)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : stable_match n rank_m rank_w = ok matched) :
    is_matching (toMatrixNat (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- Stability: the returned matching has no blocking pair. -/
theorem stable_match_is_stable
    (n : Std.Usize) (rank_m rank_w : Slice Std.Usize) (matched : alloc.vec.Vec Std.Usize)
    (h_rank_m : rank_m.length = n.val * n.val)
    (h_rank_w : rank_w.length = n.val * n.val)
    (h_perm_m : ∀ (m : Fin n.val) (k : Fin n.val),
       ∃ w : Fin n.val, toMatrixNat rank_m n.val h_rank_m m w = k.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : stable_match n rank_m rank_w = ok matched) :
    is_stable (toMatrixNat rank_m n.val h_rank_m)
              (toMatrixNat rank_w n.val h_rank_w)
              (toMatrixNat (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry

/-- Man-optimality: for every other stable matching, every man weakly prefers
    his partner in `matched` over his partner in that matching. -/
def man_optimal {n : Nat} (rank_m rank_w : Fin n → Fin n → Nat)
    (matched : Fin n → Fin n → Nat) : Prop :=
  ∀ (matched' : Fin n → Fin n → Nat),
    is_stable rank_m rank_w matched' →
    ∀ (m w w' : Fin n),
      matched m w = 1 → matched' m w' = 1 → rank_m m w ≤ rank_m m w'

/-- Gale–Shapley returns the man-optimal stable matching. -/
theorem stable_match_man_optimal
    (n : Std.Usize) (rank_m rank_w : Slice Std.Usize) (matched : alloc.vec.Vec Std.Usize)
    (h_rank_m : rank_m.length = n.val * n.val)
    (h_rank_w : rank_w.length = n.val * n.val)
    (h_perm_m : ∀ (m : Fin n.val) (k : Fin n.val),
       ∃ w : Fin n.val, toMatrixNat rank_m n.val h_rank_m m w = k.val)
    (h_perm_w : ∀ (w : Fin n.val) (k : Fin n.val),
       ∃ m : Fin n.val, toMatrixNat rank_w n.val h_rank_w w m = k.val)
    (h_matched : (alloc.vec.Vec.deref matched).length = n.val * n.val)
    (h : stable_match n rank_m rank_w = ok matched) :
    man_optimal (toMatrixNat rank_m n.val h_rank_m)
                (toMatrixNat rank_w n.val h_rank_w)
                (toMatrixNat (alloc.vec.Vec.deref matched) n.val h_matched) := by
  sorry
