---
name: vericoding
description: How to prove the Aeneas-translated Lean goals in RustVerif/Spec.lean — canonical proof skeleton, a compile-verified bridge-lemma prelude, three loop-induction recipes, an Aeneas/List lemma map, and the harness rules that decide pass/fail. Use when the task is to close theorems whose statements are already written.
---

# Vericoding: closing Aeneas-translated Lean goals

You are proving the theorems in `RustVerif/Spec.lean` about functions Aeneas
translated from Rust into `RustVerif/Function.lean`. The statements are given and
must not change. Your job is to supply proof terms.

Two facts about *this* environment decide almost everything below, and both
contradict the guidance bundled in `aeneas/documentation/`:

1. **The statements are equational, not weakest-precondition triples.** A
   benchmark theorem looks like
   `theorem f_correct (x : Std.Usize) (r : Vec Int) (h : f x = ok r) : post r`.
   142 of the 159 challenge specs carry an `= ok …` hypothesis of exactly this
   kind, and **none** of the 159 states a `⦃ ⦄` triple.
   The bundled docs are written for a different codebase whose specs are
   `f x ⦃ r => P r ⦄` and whose proofs are `unfold f; step*`.
2. **Loops are translated with `partial_fixpoint`, not with `-loops-to-rec`.**
   156 of 159 `Function.lean` files use `partial_fixpoint` (147 of those also
   carry `@[rust_loop]`); three cases have no loops at all.
   There is no `loop.spec_decr_nat` anywhere in the benchmark, and in proved
   solutions `step` appears in only 36 of 111 cases. The workhorse is
   `unfold f at h` (79% of proved solutions) followed by manual destruction of
   the `do`-block.

Consequence: **do not follow the bundled aeneas skill files as instructions.**
They describe a codebase whose specs are WP triples and whose proofs are
`step*`; borrowed naively they point you at the wrong tools. (Their content is
also older than the bundled library: the skill files last changed 2026-04-10,
`Aeneas/Std/Vec.lean` 2026-09-25.) In the traces,
runs that went to read `documentation/skills/aeneas-tactics-quickref` mostly
did not prove (30% of unproved runs vs 5% of proved runs read it; the same skew
holds for `aeneas-lean-core`, 13% vs 2%). Use the docs only to look up what a
lemma is called, and verify every tactic claim against the goal in front of you.
In particular the bundled "⛔ BANNED TACTICS — never use `omega`" rule is wrong
here: `omega` appears in 108 of 111 proved solutions. See the arithmetic
section for where `omega` actually stops working.

What *is* worth doing instead of reading prose: ask the library directly.
`#check @name` and `grep -rn 'theorem .*<keyword>'
aeneas/backends/lean/Aeneas/Std` answer "does this lemma exist" in one call;
§7 lists the names that proved runs actually used.

## 1. Finish by checking the kernel, every time

Before you stop, all three must hold:

1. `lean_diagnostic_messages` on `RustVerif/Spec.lean` — no errors, and no
   `declaration uses 'sorry'` warnings.
2. Axiom check. Either `lean_verify` on `RustVerif/Spec.lean`, or the shell
   equivalent, which is what most proved runs used:
   `printf 'import RustVerif.Spec\n#print axioms <theorem>\n' > /tmp/ax.lean && lake env lean /tmp/ax.lean`
   The only permitted axioms are `propext`, `Classical.choice`, `Quot.sound`.
3. `grep -n 'sorry\|admit' RustVerif/*.lean` — returns nothing.

Proved runs called `lean_verify` in 76% of cases (1.26 calls/case); unproved
runs in 9% (0.13/case). The tool is not magic — it is the only thing that
reports what the kernel saw — but the *actionable* rule is "verify by some
kernel-level check", and `#print axioms` counts.

**Never** use `native_decide` or `bv_decide`: both rely on an unchecked native
axiom, and Comparator rejects them as
`Illegal axiom detected: '..._native.bv_decide.ax_1_9'`. For `BitVec` goals use
`decide`, `simp`, `omega` or `bv_omega` instead. Plain
`decide` / `norm_num` are kernel-checked and safe (18 proved solutions use
`decide`).

## 2. Don't touch `Function.lean`

The verifier rebuilds `RustVerif/Function.lean` from the pristine benchmark
translation before checking your `Spec.lean`. An edit there is silently
discarded, and any proof that depended on it fails.

- The one run out of 151 that edited `Function.lean` rewrote a `partial def`
  into a `partial_fixpoint` definition to obtain `.fixpoint_induct`. Its
  solution then died with `Unknown constant '<fn>.fixpoint_induct'` and scored
  BUILD_FAIL.
- Corollary: if a loop is emitted as `partial def` *inside a `mutual` block*
  (one benchmark case), Lean gives it no unfolding lemma
  and no induction principle. You cannot reason about it from `Spec.lean`.
  Recognise this shape early and report it rather than burning the budget.

Keep the challenge's `import` lines, `namespace`s, helper `def`s, and theorem
names byte-identical. Comparator checks fully-qualified names and statements;
add your helper lemmas, do not restructure the file. (69 of 111 proved solutions
keep a trailing `end`, i.e. they only add inside the existing namespace.)

## 3. The canonical proof

`do`-blocks are chains of `bind`s that can `ok`, `fail` or `div`. The dominant
pattern — 79% of proved solutions — unfolds the function **at the hypothesis**
and peels the chain by hand. This works where `step` does not, because it needs
no `@[step]` theorem and no WP postcondition.

```lean
theorem f_correct (a : Slice Std.Usize) (r : Std.Usize) (h : f a = ok r) : P r := by
  unfold f at h                       -- instantiate the body in the hypothesis
  simp only [bind_tc_ok] at h         -- drop the trivially-ok binds
  obtain ⟨i, hi, h⟩ := bind_ok h      -- split a bind into its witness
  have hi' := usize_add_ok hi         -- i = x + y  ↦  i.val = x.val + y.val
  split at h                          -- or: split_ifs at h
  · obtain ⟨j, hj, h⟩ := bind_ok h
    ...
  · simp only [ok.injEq, Prod.mk.injEq] at h
    obtain ⟨rfl, rfl⟩ := h
    ...
```

Rules that come straight out of the proved set:

- `bind_ok` / `bind_eq_ok` is the single most valuable helper; 60 of the 111
  proved solutions declare one (under half a dozen names). Put it in the prelude
  (§4) before writing anything else.
- Destruct a successful return with `simp only [ok.injEq] at h`, and a tuple
  return with `, Prod.mk.injEq`; 54 cases use `ok.injEq`. Then
  `obtain ⟨rfl, rfl⟩ := h` replaces the result variable by its value.
- A pair-returning call (mutable borrow) sometimes arrives uncurried; when
  `obtain` fails on the shape, insert `try simp only [Std.uncurry_apply_pair] at h`
  (20 cases need it). It is harmless when the goal is already curried.
- Split a monadic operator that may fail:
  `cases hx : x + y with | ok z => … | fail e => … | div => …`, then
  `simp only [hx] at h` in each branch.
- `split_ifs at h` (48 cases) for `if`-branches; `by_cases hk : k = i.val`
  (41 cases) for the "did this index get written" case split in loop invariants.
- `exfalso` (39 cases) when a branch is contradictory; `by_contra` (45 cases)
  for set-like conclusions.
- The last line of the top-level theorem is usually a *hand-off*:
  `exact loop_spec … h` or `simpa using loop_spec … h`.

## 4. Write the prelude first

Proved solutions contain a median of **23 theorems / lemmas** and 506 lines; the
recurring bridge lemmas are reinvented from scratch in almost every case
(`usize_add_ok` in 41 cases, `slice_index_ok` 34, `vec_index_ok` 34,
`vec_index_mut_ok` 30, `usize_sub_ok` 25, `usize_mul_ok` 23, `push_ok` 18,
`bind_ok` 17, `from_elem_ok` 16). That is the largest single tax on the budget.

Paste this block near the top of `Spec.lean` and start the real proof from
minute one. It has been compiled against six different `Function.lean`
files covering array, DP, data-structure, string, integer and crypto code.
Delete any name the challenge already declares.

```lean
/-! ## Prelude: Result plumbing, scalar facts, index bridges -/

theorem bind_ok {α β : Type} {x : Result α} {f : α → Result β} {r : β}
    (h : x >>= f = ok r) : ∃ a, x = ok a ∧ f a = ok r := by
  cases x <;> simp_all [Bind.bind, Std.bind]

theorem usize_add_ok {x y z : Std.Usize} (h : x + y = ok z) : z.val = x.val + y.val := by
  have := UScalar.add_equiv x y
  rw [h] at this
  exact this.2.1

theorem usize_sub_ok {x y z : Std.Usize} (h : x - y = ok z) :
    y.val ≤ x.val ∧ z.val = x.val - y.val := by
  have := UScalar.sub_equiv x y
  rw [h] at this
  omega

theorem usize_mul_ok {x y z : Std.Usize} (h : x * y = ok z) : z.val = x.val * y.val := by
  have := UScalar.mul_equiv x y
  rw [show UScalar.mul x y = x * y from rfl, h] at this
  exact this.2.1

theorem usize_div_ok {x y z : Std.Usize} (hy : y.val ≠ 0) (h : x / y = ok z) :
    z.val = x.val / y.val := by
  obtain ⟨z', h1, h2⟩ := UScalar.div_spec x (y := y) hy
  rw [h] at h1; cases h1; exact h2

theorem slice_index_ok {α : Type} {s : Slice α} {i : Std.Usize} {x : α}
    (h : Slice.index_usize s i = ok x) : s.val[i.val]? = some x := by
  unfold Slice.index_usize at h
  split at h <;> simp_all

theorem vec_index_ok {α : Type} {v : alloc.vec.Vec α} {i : Std.Usize} {x : α}
    (h : alloc.vec.Vec.index (core.slice.index.SliceIndexUsizeSlice α) v i = ok x) :
    v.val[i.val]? = some x := by
  rw [alloc.vec.Vec.index_slice_index] at h
  unfold alloc.vec.Vec.index_usize at h
  split at h <;> simp_all

theorem vec_index_mut_ok {α : Type} {v : alloc.vec.Vec α} {i : Std.Usize} {x : α}
    {back : α → alloc.vec.Vec α}
    (h : alloc.vec.Vec.index_mut (core.slice.index.SliceIndexUsizeSlice α) v i
          = ok (x, back)) :
    i.val < v.val.length ∧ ∀ y, (back y).val = v.val.set i.val y := by
  rw [alloc.vec.Vec.index_mut_slice_index] at h
  unfold alloc.vec.Vec.index_mut_usize alloc.vec.Vec.index_usize at h
  split at h
  · rename_i y hy
    split at hy
    · simp at hy
    · rename_i z hz
      simp at hy h
      obtain ⟨_, rfl⟩ := h
      simp at hz
      rw [List.getElem?_eq_some_iff] at hz
      exact ⟨hz.1, fun y => by simp⟩
  · simp at h
  · simp at h

theorem push_ok {α : Type} {v v' : alloc.vec.Vec α} {x : α}
    (h : alloc.vec.Vec.push v x = ok v') : v'.val = v.val ++ [x] := by
  unfold alloc.vec.Vec.push at h
  dsimp only at h
  split at h
  · cases h; simp
  · cases h

theorem from_elem_usize_ok {x n : Std.Usize} {v : alloc.vec.Vec Std.Usize}
    (h : alloc.vec.from_elem core.clone.CloneUsize x n = ok v) :
    v.val = List.replicate n.val x := by
  obtain ⟨v', h1, h2⟩ := WP.spec_imp_exists
    (alloc.vec.from_elem_spec core.clone.CloneUsize x n (by rfl))
  rw [h] at h1; cases h1; exact h2.1

theorem deref_val {α : Type} (v : alloc.vec.Vec α) :
    ((alloc.vec.Vec.deref v : Slice α) : List α) = v.val := by
  simp [alloc.vec.Vec.deref]

theorem getElem!_set_eq {α : Type} [Inhabited α] (l : List α) (i : Nat) (x : α)
    (h : i < l.length) : (l.set i x)[i]! = x := by
  simp [h]

theorem getElem!_set_ne {α : Type} [Inhabited α] (l : List α) (i j : Nat) (x : α)
    (h : i ≠ j) : (l.set i x)[j]! = l[j]! := by
  simp [List.getElem!_eq_getElem?_getD, h]

theorem idx_inj (i j r c cols : Nat) (hj : j < cols) (hc : c < cols)
    (h : i * cols + j = r * cols + c) : i = r ∧ j = c := by
  have e1 : (i * cols + j) / cols = i := by
    rw [Nat.add_comm, Nat.add_mul_div_right _ _ (by omega), Nat.div_eq_of_lt hj]; simp
  have e2 : (r * cols + c) / cols = r := by
    rw [Nat.add_comm, Nat.add_mul_div_right _ _ (by omega), Nat.div_eq_of_lt hc]; simp
  have e3 : (i * cols + j) % cols = j := by
    rw [Nat.add_comm, Nat.add_mul_mod_self_right, Nat.mod_eq_of_lt hj]
  have e4 : (r * cols + c) % cols = c := by
    rw [Nat.add_comm, Nat.add_mul_mod_self_right, Nat.mod_eq_of_lt hc]
  rw [h] at e1 e3
  omega
```

`idx_inj` is for flat 2-D arrays (`buf[i * cols + j]`): it is how you turn
"the index that was written equals the index I am reading" into coordinate
equality. 14 cases need it. Adapt its type to `Std.Usize` where convenient.

Beyond the prelude, each case needs a **model** and an **invariant predicate**.
97 of 111 proved solutions declare their own `def`s. Model the data as
`List Nat` via a converter (`toNatList s := (s : List Std.Usize).map (·.val)`,
`(v : alloc.vec.Vec α) : List α` coerces directly), state the semantics as a
clean recursive `def`, and prove the loop invariant as a predicate over that
model. Do not phrase invariants over `Vec` internals if a `List` model will do.

## 5. Loops

Every loop gets its own theorem. Its postcondition should be exactly what the
caller needs — nested loops compose by having the inner theorem conclude the
outer invariant.

Large design lesson from the failures: **20 of the 23 unproved runs left every
challenge theorem `sorry`** — they never banked a single sub-lemma. If a loop
looks hard, still state its invariant and prove the easy parts; partial progress
is what compounds.

### Route A — `unfold … at h` + recursive theorem + `termination_by` (37 cases)

The proof of the loop theorem applies itself to the recursive call (via `step*`
or an explicit `exact ih …`), so Lean requires a termination measure on the
theorem: without `termination_by` it reports `fail to show termination for
loop_spec` at the declaration, `@[step]` or not. `termination_by` is therefore
mandatory even though the *definition* is a `partial_fixpoint`.

Shape of a loop theorem (a template, not a compiled proof; substitute your
loop's name, parameters and invariant):

```lean
@[step]
theorem loop_spec (n : Std.Usize) (buf : alloc.vec.Vec Std.Usize) (i : Std.Usize)
    (hlen : buf.length = n.val) (hi : i.val ≤ n.val)
    (hinv : ∀ k, k < i.val → INV k buf) :
    my_loop n buf i ⦃ res => res.length = buf.length ∧ ∀ k, k < n.val → INV k res ⦄ := by
  unfold my_loop
  step*
  -- one bullet per side-condition / invariant-rebuild goal that `step*` leaves:
  · omega                    -- linear bound goals
  · scalar_tac               -- type-bound / overflow goals
  · -- invariant rebuild: the freshly written cell vs the untouched ones
    intro k hk
    simp only [alloc.vec.Vec.set_val_eq]
    by_cases hki : k = i.val
    · subst hki; simp [getElem!_set_eq, ...]        -- the written index
    · rw [getElem!_set_ne _ _ _ _ (Ne.symm hki)]    -- an untouched index
      exact hinv k (by omega)
termination_by n.val - i.val
decreasing_by scalar_tac
```

Lessons that carried over between cases (these matter more than the template):

- Goals of the form `a * c ≤ b * c` or `a * c < b * c` are products, so `omega`
  rejects them. State the monotonicity fact first
  (`Nat.mul_le_mul_right`, `Nat.mul_lt_mul_of_pos_right`) and then call `omega`.
- State every index-bound `have` **before** `intro k hk`. Once `k` is
  introduced, `omega` no longer sees the length hypothesis in the shape it needs.
- For flat 2-D arrays (`buf[i * cols + j]`), "the index I wrote equals the index
  I am reading" becomes coordinate equality through `idx_inj` from the prelude.
- Unfold the length wrappers (`simp only [alloc.vec.Vec.length] at hlen ⊢`)
  before any length arithmetic.

If `step*` does not accept the loop body, the manual form is the same recipe as
§3 (`unfold … at h`, `split at h`, `obtain ⟨…⟩ := bind_ok h`, recurse through
`ih`), with `termination_by` unchanged.

Typical measures: `<counter>.val - i.val`, `n.val - k.val`, the length of the
list being chopped, `2 * r + 1` for heap loops. `decreasing_by scalar_tac`
handles almost all of them; use `decreasing_by omega` (or
`all_goals scalar_tac`) when the measure is Nat-valued.

### Route B — fuel / strong induction (40 cases)

When the measure is awkward, state the theorem with an extra `Nat` argument and
induct on it. The theorem then is not recursive, so no `termination_by`:

```lean
theorem loop_spec (…) :
    ∀ (k : Nat) (l r : Usize) (found result : Bool), r.val - l.val = k →
      … → my_loop … = ok result → post := by
  intro k
  induction k using Nat.strong_induction_on with
  | _ k ih =>
  intro l r found result hk …
  unfold my_loop at h
```

Note the counter-`val` equality (`r.val - l.val = k`) threaded into the
hypotheses, and that each recursive call is discharged with
`exact ih (by omega) …`. This style is friendlier when there are several
mutually decreasing indices.

### Route C — `.fixpoint_induct` (12 cases)

For loops whose body is a long chain and where Route A's invariant rebuild is
unmanageable, the `partial_fixpoint` machinery gives you the fixpoint induction
principle. The bundled docs say to avoid it; the proved set uses it for
pointer-chasing and graph-style algorithms and partition-style loops, where the
invariant is relational. Recipe (adapted from a compiling proof; rename the
loop and its constant parameters):

```lean
set_option hygiene false in
macro "bstep " x:rcasesPat hx:ident : tactic =>
  `(tactic| (have h' := bind_ok h; clear h; obtain ⟨$x, $hx:ident, h⟩ := h'))

-- `True` below stands for your postcondition. `c₁ c₂` are the loop's constant
-- parameters, `(buf, i)` its varying ones.
theorem my_loop_true (c₁ c₂ : Std.Usize) :
    ∀ (buf : alloc.vec.Vec Std.Usize) (i : Std.Usize) (r : alloc.vec.Vec Std.Usize),
      my_loop c₁ c₂ buf i = ok r → True := by
  apply my_loop.fixpoint_induct c₁ c₂ (fun g => ∀ buf i r, g buf i = ok r → True)
  · apply Lean.Order.admissible_pi; intro buf
    apply Lean.Order.admissible_pi; intro i
    apply Lean.Order.admissible_pi; intro r
    apply Lean.Order.admissible_apply
      (fun _ (h : Usize → Result (alloc.vec.Vec Std.Usize)) => h i = ok r → True) buf
    apply Lean.Order.admissible_apply (fun _ fx => fx = ok r → True) i
    apply Lean.Order.admissible_flatOrder
    simp
  · intro g ih buf i r h
    beta_reduce at h
    split at h
    · bstep idx hidx
      bstep ⟨_, back⟩ hm
      bstep i1 hi1
      exact ih (back i) i1 r h
    · simp at h; subst h; trivial
```

Replace `True` by your postcondition everywhere and the last line becomes the
base-case proof. `bstep` saves repeating `obtain ⟨…⟩ := bind_ok h` with a fresh
hypothesis name at each bind. The inner `bstep` lines and the `ih` call must
match your loop's actual binds; this block is a shape to adapt, not paste.

Three things are easy to get wrong:

- the first argument(s) of `.fixpoint_induct` are the loop's **constant**
  parameters (`c₁ c₂`); the motive abstracts the *function*, so arguments that
  are still in scope stay in the goal;
- the admissibility chain must be written one `admissible_apply` per
  non-constant argument, innermost argument last, ending in
  `admissible_flatOrder; simp`;
- `beta_reduce at h` before `split at h`, otherwise the goal keeps the motive
  as a beta-redex and `split` misses.

For a mutually recursive pair use `f.mutual_fixpoint_induct` (one proved case).

### Which route

Default to A. Switch to B when the decreasing measure is not a simple
difference. Switch to C only when the body has ≥ 4 monadic calls *and* the
invariant is relational rather than a conjunction of per-index facts — C lets
you keep the whole hypothesis `h` and recurse, instead of rebuilding an
invariant.

## 6. Arithmetic: where each tactic stops

The bundled docs ban `omega`; that is wrong for this toolchain. Measured on
proved solutions: `omega` 108/111 cases, `ring` 51, `scalar_tac` 44,
`simp_all` 53, `nlinarith` 30, `linarith` 21, `agrind` 1, `simp_lists` 1.

Probe results on this exact toolchain (Aeneas + Lean 4.31, `Usize = UScalar Usize`):

| Goal | Works |
|---|---|
| `x.val < y.val ⊢ x < y`, and the converse | `omega`, `scalar_tac` |
| `x.val < y.val ⊢ x.val + 1 ≤ y.val` | `omega`, `scalar_tac`, `simp_scalar` |
| `x.val ≤ Usize.max`, `x.val < 256` for `U8` | **`scalar_tac` only** — `omega` rejects |
| `s.length = s.val.length` (Slice) | `simp [Slice.length]` |
| `v.length = …` reasoning (Vec) | `simp only [alloc.vec.Vec.length] at h ⊢` then `omega` |
| `(l.set i x).length = l.length` | `List.length_set` |
| `i.val * cols.val ≤ (n.val+1) * cols.val` | **not** `omega`/`scalar_tac`; give it `Nat.mul_le_mul_right` then `omega`, or `nlinarith` |
| `l[i]! = l[i]` with `i < l.length` | `simp [getElem!_pos, h]` |

Distilled rule:

- `omega` is the default for **linear Nat-level** goals. It sees `UScalar.val`
  and list lengths *as Nats*; it does not know `Usize.max` or type bounds.
- `scalar_tac` for anything involving type bounds, `Usize.max`, or overflow
  side-conditions left by `step`. It is `scalar_tac_preprocess; omega`.
- Unfold the length wrappers first. `v.length` and `s.length` are not Nats
  until `alloc.vec.Vec.length` / `Slice.length` is simplified, and `omega` will
  fail without that step. This one habit removes most "omega could not prove
  the goal" dead ends.
- Products need a monotonicity lemma or `nlinarith`; neither `omega` nor
  `scalar_tac` does nonlinear reasoning.
- `ring` for algebra in `Int`/`ℝ` (matrix and string cases); `norm_num` for
  concrete numerals; `decide` for finite concrete facts.
- `simp_all` is acceptable here (53 proved cases) but it deletes hypotheses;
  when a hypothesis disappears that you still need, switch to
  `simp only [*]` or list the lemmas explicitly.

## 7. Lemma map

Where things live in `aeneas/backends/lean/` (read only the file whose types
appear in the goal; proved runs concentrated on `Std/Vec.lean` 41%,
`Std/Slice.lean` 41%, `Std/WP.lean` 35%, `Std/Primitives.lean` 30%,
`Std/Scalar/Ops/{Add,Sub,Mul}.lean` 12–26%):

| Need | File | Verified names |
|---|---|---|
| Monad plumbing | `Aeneas/Std/Primitives.lean` | `bind_tc_ok`, `bind_tc_fail`, `bind_tc_div`, `bind_assoc_eq`, `Std.bind`, `Std.uncurry_apply_pair`, `ok.injEq` |
| WP / spec API | `Aeneas/Std/WP.lean` | `WP.spec_ok`, `WP.spec_imp_exists`, `WP.spec_bind`, `WP.spec_mono`, `WP.spec`, `WP.dspec` |
| Scalar arithmetic | `Aeneas/Std/Scalar/Ops/{Add,Sub,Mul,Div,Rem}.lean` | `UScalar.add_equiv`, `UScalar.sub_equiv`, `UScalar.mul_equiv` (match-shaped), `UScalar.add_spec`, `sub_spec`, `mul_spec`, `div_spec`, `rem_spec` (WP-shaped, need precondition) |
| Scalar comparisons | same | `UScalar.lt_equiv`, `le_equiv`, `eq_equiv`, `neq_to_neq_val`, `UScalar.max_USize_eq` |
| `Vec` | `Aeneas/Std/Vec.lean` | `Vec.index_slice_index`, `Vec.index_mut_slice_index`, `Vec.index_usize_spec`, `Vec.index_mut_usize_spec`, `Vec.set_val_eq`, `Vec.set_length`, `Vec.length`, `Vec.deref`, `Vec.push_spec`, `Vec.getElem?_Nat_eq` |
| `Slice` | `Aeneas/Std/Slice.lean`, `SliceDef.lean` | `Slice.index_usize_spec`, `Slice.getElem?_Usize_eq`, `Slice.len_val`, `Slice.length`, `Slice.set_length`, `Slice.set_val_eq`, `core.slice.Slice.swap_spec`, `alloc.slice.Slice.to_vec_spec` |
| `List` bridges | Mathlib | `getElem!_pos`, `List.getElem!_eq_getElem?_getD`, `List.getD_eq_getElem`, `List.getElem?_eq_getElem`, `List.getElem?_eq_none`, `List.getElem?_set`, `List.getElem_set_self`, `List.set_getElem!_ne`, `List.length_set`, `List.length_append`, `List.getElem!_append_left`, `List.getElem?_replicate` |
| `from_elem` | `Aeneas/Std/Vec.lean` | `alloc.vec.from_elem_spec` (needs the clone instance and its correctness proof) |

`Vec` and `Slice` are backed by `List`; almost every `Vec` fact you need is a
`List` fact after `alloc.vec.Vec.set_val_eq` / `Vec.deref` / `Slice` coercion.
That is why the prelude's `getElem!_set_eq` / `getElem!_set_ne` / `idx_inj`
exist and why `simp only [alloc.vec.Vec.set_val_eq]` appears in 31 proved cases.

Writing your own `@[step]` theorem: the conclusion must be a `⦃ ⦄` triple over a
`Result`. `@[step]` on a plain `Prop` statement fails at declaration time with
`LE.le is not a supported spec statement` — a shape error you can catch
immediately by compiling.

### Reading discipline

A controlled ablation on DeepSeek-Harness measured the cost/benefit of file
guidance directly: telling the agent which Aeneas files are relevant to the
case (generated from Opus-5.5's own trace read sets) and telling it not to
browse others flipped **3 of 39** baseline failures to PROVED (7.7%), made 2
others partially better and 1 worse. So: read `RustVerif/Spec.lean` and
`RustVerif/Function.lean` in full, read only the Std file whose *type* appears
in the goal (the table above), and stop. Everything else is budget spent
without information.

## 8. Budget discipline

Proved runs: median **59** tool calls. Unproved: median **100** (the cap), with
13/23 at the cap and 22/23 at ≥ 90. Per case, proved vs unproved:
`Bash` 35 vs 55, `lean_diagnostic_messages` 12 vs 25, `lean_goal` 4.7 vs 7.6,
`multi_attempt` 1.3 vs 1.7. More queries is not more progress — the unproved
runs asked the LSP *more*, and still never left a single theorem proved.
The trap is not a single expensive call; it is 50 `Bash` calls that each rewrite
the same file. Proved runs still used `Bash` a lot (35/case, mostly `cat`/`grep`
/`sed` on `Spec.lean` and `lake env lean` on a scratch file), so do not avoid
the shell — avoid *redundant* shell.

Concrete rules:

- Bank easy theorems first (base cases, length lemmas, helper arithmetic), then
  attack the hard invariant. Twenty of the twenty-three failures ended with a
  `Spec.lean` identical in structure to the input (every challenge theorem still
  `sorry`).
- If the same goal has survived three different tactic attempts: stop editing
  it. Re-read the goal with `lean_goal` and re-read `Function.lean` for that
  loop. Ask whether the invariant is wrong rather than the tactic.
- If a goal is a linear arithmetic fact, `omega` (or `scalar_tac`) and move on;
  do not unfold further.
- Iterate in a scratch file inside the package (`RustVerif/Scratch.lean`, so the
  LSP and `lake env lean` both resolve imports), then transplant the finished
  lemmas into `Spec.lean`. Rewriting `Spec.lean` wholesale on every attempt
  burns calls and loses the diagnostics cache.
- `lake build` of the whole project is not the feedback loop; use
  `lean_diagnostic_messages`. A full build is for the final check only.
- The multi-line here-doc (`python3 - <<'EOF' … EOF` doing
  `s.replace(old, new); open(p,'w').write(s)`) is a legitimate edit tool when a
  `sed` one-liner would need quoting — proved runs use it — but each such call
  costs a full round trip. Batch several edits into one call.

## 9. What not to do

- Do not read Mathlib sources hunting for the answer; runs that touched a
  `Mathlib/` path were 35% of BUILD_FAIL vs 3% of PROVED. Write the tactic and
  let `simp`/`omega`/`ring` find the lemma.
- Do not treat `aeneas/documentation/skills/*` as a rulebook for this codebase
  (see the two environment facts at the top of this file). Its examples do not
  compile against this translation.
- Do not restate a library lemma as a local `have` before checking
  `lean_local_search` / `grep` for the name.
- Do not plan a big refactor of the challenge's definitions. The statements
  must be preserved; only add.

## 10. Evidence and caveats

Primary sample: Claude-Opus-5.5 on 151 benchmark cases — **111 PROVED, 23
UNPROVED, 17 BUILD_FAIL** (73.5% proved). Secondary: an earlier Claude-Opus run
(84/7/1 of 92) and DeepSeek-Harness (23 proved of 159, used only for
qualitative comparison). Tactic/lemma/idiom counts are over the 111 proved
`Spec.lean` files; the prelude and both loop recipes were compiled against this
repo's Aeneas checkout (`lean-toolchain: v4.31.0`) as a check that the numbers
correspond to buildable Lean.

Honest limits:

- Tool-usage and doc-reading differences are correlations, not causal effects:
  a run that is losing tends to re-query the goal and go reading documentation.
  The parts that are causal are the compile-verified recipes and the
  harness rules (Function.lean restored, `native_decide` rejected,
  `partial def`-in-`mutual` has no induction principle).
- The 17 BUILD_FAIL cases split into two groups you should not confuse. Twelve
  failed with `PANIC … Constant <name> not found in environment` while exporting
  the *challenge*: that is a verification-script artifact (theorems nested in a
  `namespace` were exported by bare name), not a proof failure. Reading their
  `Spec.lean` directly, four are sorry-free and would have been PROVED and eight
  still contain `sorry`. The other five BUILD_FAILs are genuine: an edited
  `Function.lean`, `@[step]` on a non-WP theorem, a nonexistent lemma name,
  unsolved goals, and a failed `rewrite`.
- So the honest headline for Opus-5.5 is 111 proved by the harness, 115 proved
  by kernel when the four mis-exported solutions are counted — still 73–76%.
  The remaining cases are genuinely harder (existence proofs in graph theory,
  matrix factorisations, automata constructions), not budget accidents.
- The `termination_by` vs fuel split (37 vs 40) has no measurable winner; pick
  by shape as in §5.
- The prelude's lemma names were all confirmed with `#check` against this
  checkout. Plausible-looking names that **do not exist** here include
  `UScalar.ofNat_val_eq`, `UScalar.val_add`, `List.getElem?_eq_getElem?`,
  `List.getElem!_set_self`, `alloc.vec.Vec.push_val`,
  `alloc.vec.Vec.getElem_set_self`. `#check @name` costs one LSP call and saves
  a wasted proof attempt.
