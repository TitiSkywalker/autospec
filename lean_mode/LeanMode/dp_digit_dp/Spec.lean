import LeanMode.dp_digit_dp.DpDigitDp
import Mathlib

open Aeneas Aeneas.Std Result
open dp_digit_dp

set_option linter.unusedVariables false

/-!
# Specification: digit DP

`digit_count n k` sweeps the decimal digits of `N = decode n` from the most
significant one, tracking `(tight, rem)`, and returns the number of
`x ∈ [0, N]` whose decimal digits contain no `4` and whose digit sum is
divisible by `k`.

The specification below is a plain counting statement — the cardinality of a
`Finset` of numbers satisfying a decidable predicate about their decimal
digits — with no mention of the DP table, of the `tight` flag, or of the sweep
order.
-/

namespace DigitDP

/-- View a flat `usize` slice as a `List Nat` via `.val`. -/
def toNatList (s : Slice Std.Usize) : List Nat :=
  (s : List Std.Usize).map (·.val)

/-- The decimal digits of `x`, least significant first, with **no** leading
zeros (`0` has no digits at all).  `fuel` bounds the recursion; `x + 1` is
always enough. -/
def digitsAux : Nat → Nat → List Nat
  | 0, _ => []
  | _ + 1, 0 => []
  | fuel + 1, n + 1 => (n + 1) % 10 :: digitsAux fuel ((n + 1) / 10)

/-- The decimal digits of `x`, least significant first. -/
def digits (x : Nat) : List Nat :=
  digitsAux (x + 1) x

/-- Sum of the decimal digits of `x` (`0` for `x = 0`). -/
def digSum (x : Nat) : Nat :=
  (digits x).sum

/-- No decimal digit of `x` is `4`.  (Leading zeros are not digits of `x`, and
the digit `0` is allowed, so both conventions agree.) -/
def NoFour (x : Nat) : Prop :=
  ∀ d ∈ digits x, d ≠ 4

/-- `x` is a good number for the modulus `k`. -/
def Good (k x : Nat) : Prop :=
  NoFour x ∧ k ∣ digSum x

/-- The number whose decimal digits are the list `ds`, most significant first.
Leading zeros do not change the value. -/
def decodeAux : List Nat → Nat
  | [] => 0
  | d :: ds => d * 10 ^ ds.length + decodeAux ds

/-- The number written by the digit list `n`, most significant first. -/
def decode (n : List Nat) : Nat :=
  decodeAux n

/-- The number of good numbers in `[0, N]`. -/
noncomputable def digitCount (N k : Nat) : Nat := by
  classical
  exact ((Finset.range (N + 1)).filter (fun x => Good k x)).card

end DigitDP

open DigitDP

/-- Correctness of the digit DP: whenever `digit_count n k` returns `c`, the
value `c` is the number of integers `x` with `0 ≤ x ≤ decode n` whose decimal
expansion contains no `4` and whose digit sum is a multiple of `k`.

The hypotheses say that `n` really is a decimal expansion (every digit is
`< 10`, `h_dig`) and that the modulus is positive (`h_k`, so that "divisible"
is a meaningful condition and the DP never divides by zero). -/
theorem digit_count_correct (n : Slice Std.Usize) (k : Std.Usize) (c : Std.Usize)
    (h_k : 1 ≤ k.val)
    (h_dig : ∀ d ∈ toNatList n, d < 10)
    (hres : digit_count n k = ok c) :
    c.val = digitCount (decode (toNatList n)) k.val := by
  sorry
