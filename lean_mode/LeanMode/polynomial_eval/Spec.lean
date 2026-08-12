import LeanMode.polynomial_eval.PolynomialEval
import Mathlib.Algebra.Polynomial.Basic
import Mathlib.Algebra.Polynomial.Eval.Defs

open Aeneas Aeneas.Std Result
open polynomial_eval
open Polynomial

set_option linter.unusedVariables false

/-!
# Specification: Polynomial Evaluation (Horner's method)

The Rust `polynomial_eval` function evaluates a polynomial represented as
coefficients in ascending order at a given point `x`. This specification
connects it to Mathlib's `Polynomial.eval`.
-/

/-- Build a Mathlib `Polynomial ℚ` from a list of coefficients in ascending order.
    `[a₀, a₁, a₂, ...]` → `a₀ + a₁·X + a₂·X² + ...`, using Horner recursion. -/
noncomputable def toPolynomial : List Rat → Polynomial Rat
  | [] => 0
  | a :: cs => C (a : Rat) + X * toPolynomial cs

/-- Correctness of `polynomial_eval`:
    If the coefficient slice is non-empty (enforced by `massert` in the impl)
    and the function returns `ok result`, then `result` equals
    Mathlib's `Polynomial.eval x p` where `p` is the polynomial constructed
    from the coefficients. -/
theorem polynomial_eval_eq
    (coeffs : Slice Rat) (x : Rat) (result : Rat)
    (h : polynomial_eval coeffs x = ok result) :
    eval x (toPolynomial (coeffs : List Rat)) = result :=
by
  sorry
