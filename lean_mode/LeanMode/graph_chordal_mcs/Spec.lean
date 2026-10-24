import LeanMode.graph_chordal_mcs.GraphChordalMcs
import Mathlib

open Aeneas Aeneas.Std Result
open graph_chordal_mcs

set_option linter.unusedVariables false

/-!
# Specification: Maximum Cardinality Search, perfect elimination orders, and chordal graphs

`chordal_mcs n adj` reads an undirected graph off the flat, row-major `n × n`
symmetric `0/1` adjacency matrix `adj` and returns the MCS ordering `pi`
(reverse selection order: the vertex chosen first is written to the last slot,
i.e. MCS numbers the vertices from `n` down to `1`).

The theory is the Fulkerson–Gross (1965) / Rose–Tarjan–Lueker (1976)
characterisation of chordal graphs by perfect elimination orderings:

* `IsPEO Adj pi` — `pi` lists every vertex once and, for every vertex `v`, the
  neighbours of `v` that appear *later* in `pi` form a clique;
* `IsChordal Adj` — every cycle of length `≥ 4` has a chord;
* `mcs_peo_iff_chordal` — the MCS ordering `pi` is a perfect elimination
  ordering **iff** the graph is chordal.
-/

namespace graph_chordal_mcs

/-- View a flat `n × n` matrix (row-major: element `(row, col)` lives at
    `row * n + col`) as `Fin n → Fin n → Nat`. -/
def toMatrix (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) :
    Fin n → Fin n → Nat :=
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

/-- View a flat, row-major `n × n` matrix (element `(row, col)` lives at
    `row * n + col`) as a binary relation `Fin n → Fin n → Prop`
    (`Adj i j` holds iff the entry is the sentinel-free value `1`). -/
def toAdj (s : Slice Std.Usize) (n : Nat) (h : s.length = n * n) :
    Fin n → Fin n → Prop :=
  λ i j => toMatrix s n h i j = 1

/-- View a `Vec Std.Usize` of length `n` as `Fin n → Nat`. -/
def toVecNat (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n) :
    Fin n → Nat :=
  λ i =>
    have h_bound : i.val < v.length := by rw [h]; exact i.is_lt
    (v[i.val]'h_bound).val

/-- View a `Vec Std.Usize` of length `n`, all of whose entries are `< n`, as an
    `n`-element permutation-like map `Fin n → Fin n`. -/
def toPerm (v : alloc.vec.Vec Std.Usize) (n : Nat) (h : v.length = n)
    (hb : ∀ i : Fin n, toVecNat v n h i < n) : Fin n → Fin n :=
  λ i => ⟨toVecNat v n h i, hb i⟩

/-- The cyclic successor `i + 1 (mod k)` of a position in a `k`-cycle. -/
def csucc (k : Nat) (hk : 0 < k) (i : Fin k) : Fin k :=
  ⟨(i.val + 1) % k, Nat.mod_lt (i.val + 1) hk⟩

/-- `pi` is a perfect elimination ordering of `Adj`: it is a bijection
    (every vertex appears exactly once) and, for every vertex `pi i`, the
    neighbours of `pi i` that are listed *later* in `pi` form a clique. -/
def IsPEO {n : Nat} (Adj : Fin n → Fin n → Prop) (pi : Fin n → Fin n) : Prop :=
  Function.Bijective pi ∧
  ∀ i j k : Fin n, i < j → i < k → j ≠ k →
    Adj (pi i) (pi j) → Adj (pi i) (pi k) → Adj (pi j) (pi k)

/-- `Adj` is chordal: every simple cycle of length `k ≥ 4` has a chord.  A
    cycle is an injective map `c : Fin k → Fin n` whose cyclically consecutive
    images are adjacent; a chord is a pair of non-consecutive cycle positions
    that are adjacent. -/
def IsChordal {n : Nat} (Adj : Fin n → Fin n → Prop) : Prop :=
  ∀ (k : Nat) (hk : 4 ≤ k) (c : Fin k → Fin n),
    Function.Injective c →
    (∀ i : Fin k, Adj (c i) (c (csucc k (by omega) i))) →
    ∃ i j : Fin k,
      j.val ≠ i.val ∧
      j.val ≠ (i.val + 1) % k ∧
      j.val ≠ (i.val + k - 1) % k ∧
      Adj (c i) (c j)

end graph_chordal_mcs

open graph_chordal_mcs

/-! ## Specification theorems -/

/-- The returned vector `pi` is a permutation of `0 … n-1`: its length is `n`,
    every entry is a valid vertex, and no vertex occurs twice. -/
theorem chordal_mcs_is_perm
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : alloc.vec.Vec Std.Usize)
    (h_len : pi.length = n.val)
    (h : chordal_mcs n adj = ok pi) :
    (∀ i : Fin n.val, toVecNat pi n.val h_len i < n.val) ∧
    (∀ i j : Fin n.val, i ≠ j →
      toVecNat pi n.val h_len i ≠ toVecNat pi n.val h_len j) := by
  sorry

/-- **Fulkerson–Gross / Rose–Tarjan–Lueker**: for a simple undirected graph,
    the MCS ordering `pi` is a perfect elimination ordering if and only if the
    graph is chordal.  `h01` is the documented precondition that `adj` encode a
    `0/1` adjacency matrix; without it the program's `≠ 0` edge test and the
    spec's `= 1` relation `toAdj` disagree on entries `≥ 2`. -/
theorem mcs_peo_iff_chordal
    (n : Std.Usize) (adj : Slice Std.Usize) (pi : alloc.vec.Vec Std.Usize)
    (h_adj : adj.length = n.val * n.val)
    (h_len : pi.length = n.val)
    (hb : ∀ i : Fin n.val, toVecNat pi n.val h_len i < n.val)
    (h01 : ∀ i j : Fin n.val,
      toMatrix adj n.val h_adj i j = 0 ∨ toMatrix adj n.val h_adj i j = 1)
    (h_symm : ∀ i j : Fin n.val,
      toAdj adj n.val h_adj i j ↔ toAdj adj n.val h_adj j i)
    (h_irrefl : ∀ i : Fin n.val, ¬ toAdj adj n.val h_adj i i)
    (h : chordal_mcs n adj = ok pi) :
    IsPEO (toAdj adj n.val h_adj) (toPerm pi n.val h_len hb) ↔
    IsChordal (toAdj adj n.val h_adj) := by
  sorry
