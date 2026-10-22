import LeanMode.string_runs.StringRuns
import Mathlib

open Aeneas Aeneas.Std Result
open string_runs

set_option linter.unusedVariables false

/-!
# Specification: runs (maximal periodicities)

A *run* of a word `s` is a half-open interval `[start, end)` together with its
**minimal period** `period` such that

* `2 * period ≤ end - start` (the interval contains at least two periods), and
* the interval is maximal: neither `[start - 1, end)` nor `[start, end + 1)` has
  period `period`.

`runs s` returns every run of `s`, exactly once, without duplicates.

Correctness is stated as four facts about the concrete returned buffer
(`runsOf rs` decodes the `Vec Run` into a list of `(start, end, period)`
triples):

* `runs_sound`    — every recorded triple really is a run;
* `runs_complete` — every run is recorded;
* `runs_theorem`  — the number of runs is strictly smaller than `|s|`
  (Kolpakov–Kucherov / Bannai et al.);
* `runs_nodup`    — the recorded triples are pairwise distinct.

Characters are `usize`-encoded bytes in `0..=255` (`AllBytes`).

The periodicity predicate is deliberately *weak* (`s[k] = s[k + p]` for all `k`
with `k + p < end`), so it encodes every restriction the implementation's
period test uses.  It is written with `List.drop` and a recursive helper: the
`List.get?`-based formulation does not elaborate in this toolchain.
-/

/-- Convert a list of `Std.Usize` to a list of `Nat` (`x.val`). -/
def toNatList (l : List Std.Usize) : List Nat := l.map (fun x => x.val)

/-- Every element of a byte string is a byte value (`≤ 255`). -/
def AllBytes (s : Slice Std.Usize) : Prop :=
  ∀ x ∈ toNatList (s : List Std.Usize), x ≤ 255

/-- View the returned vector as a list of `(start, end, period)` triples. -/
def runsOf (rs : alloc.vec.Vec Run) : List (Nat × Nat × Nat) :=
  ((alloc.vec.Vec.deref rs) : List Run).map
    (fun r => (r.start.val, r.end.val, r.period.val))

/-- `u` and `v` agree on their first `n` characters (and both must have at
    least `n` of them). -/
def agreeN : Nat → List Nat → List Nat → Prop
  | 0, _, _ => True
  | _ + 1, [], _ => False
  | _ + 1, _, [] => False
  | n + 1, x :: xs, y :: ys => x = y ∧ agreeN n xs ys

/-- The half-open interval `[start, stop)` of `s` has period `period`.

    This is weak periodicity: `s[k + period] = s[k]` for every `k` with
    `start ≤ k` and `k + period < stop`.  There are exactly
    `stop - start - period` such positions, so comparing `s.drop start` with
    `s.drop (start + period)` on that many characters is the whole condition.

    (The second endpoint is called `stop` because `end` is a Lean keyword.) -/
def IsPeriodic (s : List Nat) (start stop period : Nat) : Prop :=
  0 < period ∧ start ≤ stop ∧ stop ≤ s.length ∧ start + period ≤ stop ∧
    agreeN (stop - start - period) (s.drop start) (s.drop (start + period))

/-- `period` is the **minimal** period of `[start, stop)`: it is a period and no
    period is smaller. -/
def IsMinPeriod (s : List Nat) (start stop period : Nat) : Prop :=
  IsPeriodic s start stop period ∧ ∀ q, IsPeriodic s start stop q → period ≤ q

/-- `[start, stop)` with minimal period `period` is a maximal periodic interval
    (a *run*) of `s`: it spans at least two periods, and neither `[start - 1,
    stop)` nor `[start, stop + 1)` has period `period`. -/
def IsRun (s : List Nat) (start stop period : Nat) : Prop :=
  IsMinPeriod s start stop period ∧ 2 * period ≤ stop - start ∧
  (start = 0 ∨ ¬ IsPeriodic s (start - 1) stop period) ∧
  (stop = s.length ∨ ¬ IsPeriodic s start (stop + 1) period)

/-- Extensional characterisation of `run`: it is the *runs* of `s`. -/
def IsRunsOf (s : List Nat) (rs : List (Nat × Nat × Nat)) : Prop :=
  (∀ r ∈ rs, IsRun s r.1 r.2.1 r.2.2) ∧
  (∀ start stop period, IsRun s start stop period → (start, stop, period) ∈ rs) ∧
  rs.Nodup

/-! ## Specification theorems -/

/-- **Soundness.**  Every recorded triple is a run of `s`. -/
theorem runs_sound (s : Slice Std.Usize) (rs : alloc.vec.Vec Run)
    (h : runs s = ok rs) (hb : AllBytes s) :
    ∀ r ∈ runsOf rs, IsRun (toNatList (s : List Std.Usize)) r.1 r.2.1 r.2.2 := by
  sorry

/-- **Completeness.**  Every run of `s` is recorded.

    This is where the `l == i` guard and the independent minimal-period recheck
    earn their keep: a run `[start, end)` with minimal period `p` is always
    rediscovered from the scan position `i = start` at window length
    `end - start` (then `L - pi[L - 1] = p`), and the recheck rules out the
    spurious candidates produced by shorter windows. -/
theorem runs_complete (s : Slice Std.Usize) (rs : alloc.vec.Vec Run)
    (h : runs s = ok rs) (hb : AllBytes s) :
    ∀ start stop period, IsRun (toNatList (s : List Std.Usize)) start stop period →
      (start, stop, period) ∈ runsOf rs := by
  sorry

/-- **The runs theorem (Kolpakov–Kucherov, Bannai et al.).**  A nonempty word
    has strictly fewer than `|s|` runs. -/
theorem runs_theorem (s : Slice Std.Usize) (rs : alloc.vec.Vec Run)
    (h : runs s = ok rs) (hb : AllBytes s)
    (h_pos : 0 < (s : List Std.Usize).length) :
    (runsOf rs).length < (s : List Std.Usize).length := by
  sorry

/-- **No duplicates.**  Each run is recorded once.  (`AllBytes` is not needed:
    the implementation compares the recorded triples directly.) -/
theorem runs_nodup (s : Slice Std.Usize) (rs : alloc.vec.Vec Run)
    (h : runs s = ok rs) :
    (runsOf rs).Nodup := by
  sorry

/-- The three facts above combine into the extensional characterisation of the
    returned list. -/
theorem runs_correct (s : Slice Std.Usize) (rs : alloc.vec.Vec Run)
    (h : runs s = ok rs) (hb : AllBytes s) :
    IsRunsOf (toNatList (s : List Std.Usize)) (runsOf rs) := by
  sorry
