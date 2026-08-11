import LeanMode.random_integer.RandomInteger
import Mathlib.Probability.ProbabilityMassFunction.Basic
import Mathlib.Probability.ProbabilityMassFunction.Constructions
import Mathlib.Probability.Distributions.Uniform

open Aeneas Aeneas.Std Result
open random_integer
open PMF
open scoped ProbabilityTheory

set_option linter.unusedVariables false

/-!
# Specification: Random Integer in `[0, n)`

`random_integer` 现在是泛型 monad 的函数。给 `PMF` 配上 `MonadRandom`
instance 之后，`random_integer n` 直接就是 `PMF Usize`。

核心命题：已知 `random` 在 `Usize` 上均匀分布，
且 `n | Usize.size`，
则 `random_integer n` 在 `[0,n)` 上也均匀分布。
-/

/-! ### 必要的实例 -/

/-- `UScalar ty ≃ BitVec ty.numBits ≃ Fin (2^numBits)`，因此是 Fintype。 -/
instance (ty : UScalarTy) : Fintype (UScalar ty) :=
  Fintype.ofEquiv (Fin (2 ^ ty.numBits)) {
    toFun := λ x => UScalar.mk (BitVec.ofFin x)
    invFun := λ us => us.bv.toFin
    left_inv := λ x => rfl
    right_inv := λ us => by
      cases us; simp
  }

/-- PMF instance: random 在 `Usize` 上均匀分布 -/
noncomputable instance : MonadRandom PMF where
  random := PMF.uniformOfFintype Usize

/-! ### 辅助定义与引理 -/

/-- `random_integer` 中用到的纯余数表达式 -/
def usize_rem (r n : Usize) : Usize := ⟨BitVec.umod r.bv n.bv⟩

/--
关键引理：`random_integer n` 在 `PMF` monad 中等价于
`uniformOfFintype Usize` 经 `usize_rem` 的 pushforward（n=0 时退化）。
-/
theorem random_integer_pmf_eq (n : Usize) :
    (random_integer n : PMF Usize) =
    if n = 0#usize then pure 0#usize
    else (PMF.uniformOfFintype Usize).map (λ r => usize_rem r n) := by
  unfold random_integer usize_rem
  by_cases hn : n = 0#usize
  · subst hn; rfl
  · simp [hn]
    rfl

/--
核心定理：已知 `random` 对任意 `x, y` 取到概率相同（均匀），
`n | Usize.size`，`n > 0`，则 `random_integer n` 在 `[0, n)` 上也均匀。
即：`∀ x y, x.val < n.val → y.val < n.val → P(output=x) = P(output=y)`
-/
theorem random_integer_uniform (n : Usize) (h_div : n.val ∣ Usize.size) (h_pos : n.val > 0)
    (x y : Usize) (hx : x.val < n.val) (hy : y.val < n.val) :
    (random_integer n : PMF Usize) x = (random_integer n : PMF Usize) y := by
  rw [random_integer_pmf_eq n]
  have hn0 : n ≠ 0#usize := by
    intro h; have hval := congrArg UScalar.val h; simp [h_pos.ne'] at hval
  simp [hn0, usize_rem]
  -- Goal: ((uniformOfFintype Usize).map (λ r => ⟨BitVec.umod r.bv n.bv⟩)) x = ... y
  sorry
