# LeanMode Testcase Guide

This document records the full workflow — and the hard-won pitfalls — for adding a new testcase to `LeanMode/`. The goal is to translate a Rust algorithm into Lean and attach a specification theorem (proved with `sorry`), so that `scripts/check_all.py` passes.

---

## 1. What a testcase looks like

```
LeanMode/<name>/
├── Cargo.toml          # crate name = <name> (snake_case)
├── Cargo.lock
├── src/lib.rs          # algorithm implementation (AENEAS-compatible style, see section 4)
├── tests/test.rs       # plain Rust tests (run by cargo test; not always translated)
├── <Crate>.lean        # auto-generated: translation of lib.rs (e.g. MatrixQr.lean)
├── Test_test.lean      # auto-generated: translation of tests/test.rs + #eval (Rat-only)
└── Spec.lean           # hand-written: specification theorem (A = Q·R style)
```

- The `<Crate>.lean` filename is the CamelCase of the crate name (`matrix_qr` → `MatrixQr.lean`).
- `Spec.lean` is the only hand-written Lean file; the other two are produced by the translator.

---

## 2. Roles of the three Lean files

| File | Source | Contents | Present when |
|------|--------|----------|--------------|
| `<Crate>.lean` | auto | function-by-function translation of `src/lib.rs` | always |
| `Test_test.lean` | auto | translation of `tests/test.rs` with trailing `#eval <test>` | Rat-computable only (no sqrt/exp) |
| `Spec.lean` | hand-written | `toMatrix`-style helpers + correctness `theorem ... := by sorry` | always |

`check_all.py` runs `lake build` on every `.lean` file and, for `Test_test.lean`,
asserts the `#eval` results (`#[should_panic]` tests are expected to return `Result.fail`).

---

## 3. Standard workflow

```bash
# 1. Create directory and write Rust
mkdir -p lean_mode/LeanMode/<name>/src lean_mode/LeanMode/<name>/tests
# write Cargo.toml / src/lib.rs / tests/test.rs

# 2. Verify the Rust itself is correct
cd lean_mode/LeanMode/<name> && cargo test

# 3. Translate (generates <Crate>.lean, possibly Test_test.lean)
cd ../../../..  # back to repo root
./translation/translator/target/release/translator lean_mode/LeanMode/<name>
#    (older docs said `python3 translation/translate.py`; that script is gone —
#     the translator is this Rust binary, whose CLI is `translator <project-dir>`)

# 4. Hand-write Spec.lean (see section 6)

# 5. Verify the Lean compiles
cd lean_mode && lake build LeanMode.<name>.<Crate>
lake build LeanMode.<name>.Spec
```

Batch commands:
- `./scripts/translate_all.sh -w 8` — re-translate every testcase (skips `failure/`).
- `python3 scripts/check_all.py` — `lake build` every `.lean` individually and check `#eval`.

If the translator fails on a construct, see section 11 — diagnose, fix what's in-scope, and
surface real limitations for a human to decide.

---

## 4. Rust style constraints (what AENEAS can translate)

The pipeline is `charon → LLBC adapter → aeneas → lean`, and it imposes hard limits on
the Rust. **Write `src/lib.rs` using only this subset:**

1. **Use only `while` loops.** No `for .. in ..`, no `.iter()/.fold()/.map()/.all()`, no
   closures.
2. **No external crates** (keep `[dependencies]` empty in `Cargo.toml`).
3. **No `.abs()/.min()/.signum()/.max()` method calls** — expand them by hand with `if`:
   ```rust
   let m = if x < 0.0 { -x } else { x };          // |x|
   let s = if x < 0.0 { -1.0 } else { 1.0 };      // sign
   let k = if a < b { a } else { b };             // min
   ```
   (`simplify.py` helps with `.min()`, but `abs/signum/sqrt` are up to you.)
4. **A `usize` counting-down `while` loop is fine**: `while ii > 0 { ii -= 1; ... }`.
   But **`isize` (signed) + `continue` will fail to translate** (see section 9 — this is why
   `matrix_bidiagonal` ended up in `failure/`).
5. **`&mut T` borrows are modeled as pass-by-value + return.** So a shape like
   ```rust
   fn clear_column(mut m: Vec<f64>, ...) -> (f64, Vec<f64>) { ...; (norm, m) }
   ```
   (consume the `Vec`, return it at the end) is the least surprising; `&mut Vec<f64>` /
   `&mut [f64]` (slices) also work — AENEAS threads the updated value automatically.
6. Allocate with `vec![0.0; n]` (→ `alloc.vec.from_elem`), copy with `matrix.to_vec()`
   (→ `alloc.slice.Slice.to_vec`), append with `v.push(x)` (→ `alloc.vec.Vec.push`).

---

## 5. Numeric types: f64 → Rat/Real, integers → Int (important)

During translation, `f64` is replaced:

- **Default `--float-model Rat`**: `f64 → Rat` (computable, `#eval` works).
- **When the code uses `sqrt / exp / sin` etc.**, `lean_f64_to_rat` raises
  `NonComputableError`, and `translate.py` **auto-switches to `--float-model Real`**:
  - `f64 → ℝ`, `sqrt` becomes `Real.sqrt`.
  - The generated file gains `import Mathlib` and `noncomputable section` at the top.
- **Under the Real float-model, `tests/test.rs` is NOT translated** (`translate.py` skips it,
  because `Real.sqrt` cannot be `#eval`'d). So sqrt-based testcases have only
  `<Crate>.lean` + `Spec.lean` — **no `Test_test.lean`** (cf. `array_norm`,
  `tensor_softmax_1d`, `matrix_qr`).
- **Mark Real testcases explicitly**: set `float-model = "Real"` per testcase in
  `Cargo.toml` (mirrors `int-model` below); `translate.py` reads it and skips the
  auto-switch:

```toml
[package.metadata.autospec]
float-model = "Real"
```

Rule of thumb: does the algorithm use `sqrt`/transcendental functions? No → Rat (has
`Test_test.lean`); yes → Real (impl + Spec only).

### Integers: `Int` vs `Nat` (the `--int-model` flag)

The translator has `--int-model {int,nat,bitvec}`:

- **`int` (default)** — every integer type (`i32`/`i64`/`u32`/`u64`/…) → `Int`
  (unbounded signed), sidestepping bit-width / overflow. `usize` is **not**
  replaced (it is used for array indexing).
- **`nat`** — **unsigned** types (`u32`/`u64`/…) → `Nat`; **signed** (`i32`/`i64`) → `Int`.
- **`bitvec`** — unsigned → `BitVec w` (fixed-width wrapping); signed → `Int`.

**Prefer `nat` for purely non-negative algorithms** (gcd, lcm, factorial, totient,
`pow`, `mod_pow`, …): the spec is then stated over `Nat` (`result = Nat.factorial n`,
`result = Nat.gcd a b`) instead of `Int` + `.toNat`, avoiding negative-value edge cases
and the `Int.toNat` truncation footgun. Set it per testcase in `Cargo.toml` — no CLI
flag needed, `translate.py` reads it and `translate_all.py` respects it:

```toml
[package.metadata.autospec]
int-model = "nat"
```

Mixed directions work: Möbius `μ(n)` uses `u64 → Nat` for its input but
`i64 → Int` for its `-1/0/1` result.

Existing examples: `array_bubble_sort` / `array_binary_search` use `i32`,
`graph_dijkstra` / `graph_bellman_ford` use `i64`; the `integer_*` testcases use
`u64` + `nat` except `extended_gcd` / `mod_inverse` / `div_mod`, which need `i64`
(signed Bézout coefficients / intermediate negatives).

**`bitvec`** — `bitvec_ops` uses `u32` + `bitvec` (→ `BitVec 32`), covering rotate,
wrapping add/sub/mul/neg, bitwise `& | ^ !`, shifts `<< >>`, and popcount. Notes:

- Aeneas's named scalar methods are rewritten by `lean_post._bitvec_ops_to_bv`:
  `x.rotate_left(n)` → `BitVec.rotateLeft x n.toNat` (the shift arg stays `BitVec w`
  and must be `.toNat`-coerced), `a.wrapping_add/sub/mul(b)` → `a +/ -/ * b` (BitVec
  arithmetic already wraps). `usize` is untouched (stays `Std.Usize` for indexing).
- Aeneas only scalar-lowers `wrapping_add/sub/mul/shl/shr`; `wrapping_neg`, `div`,
  `rem`, `count_ones`, `leading_zeros` have **no** scalar lowering — they become
  opaque `rust_fun` axioms (noncomputable, no `#eval`). Write them out instead, e.g.
  `wrapping_neg(x)` → `0u32.wrapping_sub(x)`.
- Plain bitwise/shift ops (`a & b`, `a << b`, …) come through Aeneas's *symbolic*
  path (`a &&& b`, `a <<< b`, …) which is already valid `BitVec` notation; the
  typings are the `(n : BitVec w)` literal form, so `_convert_let_line`'s operands
  accept parenthesized literals.
- `_mark_noncomputable_defs` strips doc comments before reference-matching, so a
  name appearing only in a neighboring docstring doesn't over-mark `noncomputable`.
- Scalar `assert_eq!` on a `BitVec` works (`massert (a = b)` via `DecidableEq (BitVec w)`).
- **`as` casts between unsigned integers are now supported under `bitvec`** — `u8 as usize`,
  `u8 as u32`, `u32 as u8` lower to `UScalar.cast`/`IScalar.cast`, which
  `_bitvec_ops_to_bv` rewrites to `BitVec.ofNat` / `BitVec.ofInt` / `UScalar.mk`
  conversions.  This means table indexing (`SBOX[x as usize]`) works.
- **`const [u8; N]` / `[[u8; N]; M]` fixed tables do translate** (→ `Array (BitVec 8) N#usize`,
  indexed via `Array.index_usize`).  Combined with cast support, table-driven crypto kernels
  are now viable — `crypto_aes128` uses the official S-box / Rcon tables, `crypto_des128` uses
  permutation and S-box tables (both `#eval` in Lean).

This is the model the crypto kernels (`rotate_right`/`wrapping_add`/bitwise) need.

---

## 6. How to write Spec.lean

Template (column-major QR):

```lean
import LeanMode.matrix_qr.MatrixQr
import Mathlib

open Aeneas Aeneas.Std Result
open matrix_qr
open Matrix

set_option linter.unusedVariables false

-- View a flat Slice as a Matrix (column-major: element (row,col) is at row + col*rows)
def toMatrix (s : Slice ℝ) (rows cols : Nat)
    (h : s.length = rows * cols) : Matrix (Fin rows) (Fin cols) ℝ :=
  λ i j =>
    have h_bound : i.val + j.val * rows < s.length := by
      rw [h]
      have hi : i.val < rows := i.is_lt
      have hj : j.val < cols := j.is_lt
      calc
        i.val + j.val * rows < rows + j.val * rows := Nat.add_lt_add_right hi (j.val * rows)
        _ = (j.val + 1) * rows := by ring
        _ ≤ cols * rows := Nat.mul_le_mul_right rows (Nat.succ_le_of_lt hj)
        _ = rows * cols := Nat.mul_comm cols rows
    s[i.val + j.val * rows]'h_bound

theorem qr_decomposition_correct
    (matrix : Slice ℝ) (nrows ncols : Std.Usize) (qr_result : Qr)
    (h : qr_decomposition matrix nrows ncols = ok qr_result)
    (h_len : matrix.length = nrows.val * ncols.val)
    (h_q : (alloc.vec.Vec.deref qr_result.q).length = nrows.val * (min nrows.val ncols.val))
    (h_r : (alloc.vec.Vec.deref qr_result.r).length = (min nrows.val ncols.val) * ncols.val) :
    let k := min nrows.val ncols.val
    let Q : Matrix (Fin nrows.val) (Fin k) ℝ :=
      toMatrix (alloc.vec.Vec.deref qr_result.q) nrows.val k h_q
    let R : Matrix (Fin k) (Fin ncols.val) ℝ :=
      toMatrix (alloc.vec.Vec.deref qr_result.r) k ncols.val h_r
    Qᵀ * Q = 1 ∧
    (∀ i j, j.val < i.val → R i j = 0) ∧
    (toMatrix matrix nrows.val ncols.val h_len) = Q * R :=
by
  sorry
```

Key points:

1. **`toMatrix`**: `Slice ℝ → Matrix (Fin rows) (Fin cols) ℝ`, taking
   `h : s.length = rows*cols` and a bounds proof. Row-major uses `i.val * cols + j.val`;
   column-major uses `i.val + j.val * rows`.
2. **Extract concrete data from the result — do NOT fall back on an `∃` existential**
   (that reads as a hack). The Rust function should return concrete factors like
   `Qr { q, r }`, and the Spec pulls them out via `alloc.vec.Vec.deref qr_result.q`
   into a `Slice ℝ` for `toMatrix`. If the Rust return shape is inconvenient to extract
   from, **change the Rust**.
3. `Vec → Slice` is `alloc.vec.Vec.deref`; length is `(alloc.vec.Vec.deref v).length`.
4. Proving with `sorry` is the convention (`check_all.py` only requires it to compile).
5. **Preconditions: prefer an explicit hypothesis for simple constraints; fall back on
   `assert!` only when the condition is awkward to state.** Two cases:
   - **Simple mathematical constraint** (e.g. `nrows ≥ ncols` for upper bidiagonalization) →
     state it explicitly as a theorem hypothesis (`h_ge : nrows.val >= ncols.val`). It is
     self-documenting, needs no knowledge of `massert` semantics, and still works if the Rust
     changes. The `assert!` in Rust remains as a runtime double-check but the Spec does not
     rely on it. See `matrix_bidiagonal`.
   - **Hard-to-write condition** (e.g. `det ≠ 0` for invertibility) → encode it with `assert!`
     in Rust so an `ok` result *implies* the condition, and state the theorem unconditionally
     on the `ok` result instead of carrying the condition as a separate hypothesis. This is the
     `matrix_inverse` pattern.

---

## 7. Naming and imports

- crate name = directory name (snake_case); the generated file is CamelCase.
- `Spec.lean` starts with `import LeanMode.<name>.<Crate>`; the namespace inside
  `<Crate>.lean` is the crate name.
- `open Aeneas Aeneas.Std Result` + `open <crate>` + (when using matrices) `open Matrix`.

---

## 8. Verification checklist

```bash
cd lean_mode/LeanMode/<name> && cargo test            # Rust logic is correct
cd ../../.. && ./translation/translator/target/release/translator lean_mode/LeanMode/<name>
cd lean_mode && lake build LeanMode.<name>.<Crate>    # impl translation compiles
lake build LeanMode.<name>.Spec                       # spec compiles
python3 ../scripts/check_all.py                       # (optional) full regression
python3 review/review.py --agent claude --only <name>  # audit the spec's *truth*
```

All of the above prove only that things *compile* — `sorry` compiles whatever it is
attached to.  The last step is not optional for a new testcase: `review/review.py`
launches an agent that hunts for counterexamples to the theorems, and on the eight
research-level graph cases it found four defective specs (two missing 0/1 hypotheses,
one `∧`-for-`∨` in `IsAntichain`, one missing edge-endpoint range hypothesis) plus one
genuinely broken algorithm (a matroid-intersection BFS whose source/sink tests were
paired with the opposite matroid from its exchange arcs, so it selected self-loops).
Re-run it after every spec fix; it writes `review/reports/<name>.md` with a
`Verdict: PASS|FAIL|CONCERNS` line.

---

## 9. Pitfalls (from real experience)

- **`isize` counting-down + `continue` → translation failure**: `failure/matrix_bidiagonal`'s
  `build_u`/`build_vt` used `let mut ii: isize = ...; while ii >= 0 { ...; continue; }`,
  which AENEAS couldn't translate — the body was left as `sorry`. Use `usize` with a plain
  `while ii > 0 { ii -= 1; }`.
- **Reference extraction code can carry numeric/sign bugs**: `extraction/results/nalgebra/qr_new`'s
  test `extract_r` reads the R diagonal as `-diag[i]`, which is wrong (its own
  `test_edge_case_identity_2x2` FAILs with `-1 != 1`). The correct value is `|diag[i]|`
  (a Householder reflection maps the sub-column to `‖x‖·e₁`). Validate with `cargo test`
  before translating anything you copy.
- **Don't get the orthogonality check wrong**: to verify Q is orthogonal, compute
  `Qᵀ·Q` (`Σ_k Q[k][i]·Q[k][j]`), not `Q·Q`.
- **LSP diagnostics can lag behind `lake build`**: after regenerating `<Crate>.lean`
  (changed struct fields, etc.), the IDE may briefly report `Invalid field ...`; trust
  `lake build` (optionally `touch` both files to force a recompile).
- **`.lean` files are generated**: never hand-edit `<Crate>.lean` / `Test_test.lean`;
  change `src/lib.rs` / `tests/test.rs` and re-run the translator.
- **Nested `Vec<Vec<T>>`** is hard for AENEAS — use a flat `Vec<T>` plus manual index
  arithmetic (`row + col * nrows`).
- **`print!`/`println!` → `noncomputable`**: printing in tests (or impl) pulls in
  `Display`/string constants (`Aeneas.Std.Usize.Insts.CoreFmtDisplay`) that AENEAS can't
  handle, and the whole test becomes noncomputable. Don't print in `src/lib.rs` or `tests/test.rs`.
- **Don't `assert_eq!` two `Vec`/`Slice` values directly** — AENEAS has no `PartialEq` on
  `Vec` (`Rat.Insts.CoreCmpPartialEqRat` error). Compare element-by-element in a loop.
- **Don't call `.is_empty()` on a *struct field* in `tests/test.rs`** — `r.l.is_empty()` gets
  mis-translated to `(alloc.vec.Vec.deref r).length = 0.l` (the field name `.l` is dropped and
  re-attached to the `0` literal), so `#eval` aborts with "depends on `sorry`". `.is_empty()` on
  a plain `Vec` *variable* is fine. Fix: use `r.l.len() == 0` (translates to
  `alloc.vec.Vec.len r.l`). See `matrix_lblt`.
- **Function pointers / closures as function inputs are unsupported** (e.g. a `diff(f, x)`
  taking a function `f`) — such testcases were abandoned.
- **Nested `for` loops** can leave `sorry` bodies in AENEAS; the pipeline auto-retries with
  loop flattening, but flat `while` loops are safest.
- **Rolling scalar DP state → "Could not compute a loop fixed point in 2 iterations"**:
  a `while` loop whose only loop-carried state is several mutually-assigned local scalars
  (the two-variable `prev2 = prev1; prev1 = cur;` Fibonacci-style recurrence in
  `dp_house_robber`) made AENEAS abandon the *entire* function body as `sorry`, so
  `#eval` aborted with "depends on `sorry`".  The translator still reported
  `aeneas ok` / `output 1 defs` — the `sorry` is only visible in the generated
  `<Crate>.lean`.  **Fix:** reformulate as an explicit DP array (`vec![0usize; n + 2]`)
  indexed by `usize`, exactly like `dp_knapsack`.  (A single scalar accumulator loop
  like `integer_pow`'s `result/base/exp` is fine; it's the mutually-assigned pair that
  trips the fixed-point computation.)
- **"Find the first / any index with a property" scan → "Could not compute a loop fixed
  point in 2 iterations"**.  A pivot search written as
  `let mut r = k; let mut found = false; while i < n { if !found { let x = m[...]; if x != 0
  { r = i; found = true; } } i += 1; }` (the `found` flag with a *nested* `if`, no `else`)
  breaks AENEAS's loop-fixed-point analysis.  The stoer-wagner / min-cost-flow "pick the
  best" scans are fine only because their `!found` arm sets the result *unconditionally*
  and is always paired with an `else`.  **Fix:** use a sentinel + "min/max index" scan
  instead — `let mut r = n; while i < n { let x = m[...]; if x != 0 { if i < r { r = i; } }
  i += 1; } if r == n { return 0; }` (see `lgv_lemma`'s `det_int`).  Same for a toggling
  scalar like `sign = 0 - sign`: prefer a `swaps += 1` counter and apply the parity once at
  the end via `swaps % 2`.
- **`f64` comparison inside an `if` condition** can trigger binop errors — hoist it:
  `let cond = a > b; if cond { ... }`.
- **Don't mix `||`/`&&` with `<=`/`>=` on `f64`.** `x <= 0.0 || y <= t` short-circuits into
  `if … then ok true else f64_helpers.le y t`; the `f64_helpers.le` in the `else` branch is
  not inlined by `lean_inline.py` (its regex only catches `let ← f64_helpers.le …` and
  tail-position calls), leaving an `Unknown identifier 'f64_helpers.le'`. Rewrite with `>`/`<`
  and flip the branch — e.g. `od == 0.0 || od <= threshold` → `let is_large = od > threshold;`
  then branch on `is_large` (see `matrix_symmetric_eigen`).
- **`0.0 - v` inside an `if` branch → `f64_helpers.sub`** (same class as the `f64_helpers.le`
  pitfall): a binary `0.0 - v` in an `if`-then-else branch lowers to `f64_helpers.sub _f64_const_0 v`,
  which `lean_inline.py` does not inline → `Unknown identifier 'f64_helpers.sub'`. A top-level
  `let x = 0.0 - t;` is fine (`_f64_const_0 - t`), but in a branch use **unary negation**
  `-v` instead (`if v < 0.0 { -v } else { v }` — see `matrix_jordan`'s `row_sum_bound`).
- **Rust `%`/`/` (truncating) vs Lean `%`/`/` (Euclidean)**: Rust's `%` and `/` truncate
  toward zero (sign follows the dividend); Lean's `Int` `%`/`/` are Euclidean
  (`emod`/`ediv`, remainder always `≥ 0`). They agree on non-negative operands but differ
  on negatives — Rust `(-5) % 3 == -2`, Lean `(-5) % 3 == 1`. This is why the Daan Leijen
  `div_floor`/`mod_floor`/`div_ceil` formulas and the `(x % m + m) % m` normalization are
  written to survive both semantics. When in doubt, keep inputs non-negative (use `nat`).
- **Integer spec helpers** (`Int`/`Nat` gcd, division): `Int.gcd : Int → Int → Nat`
  returns `Nat` (so under `int` model `result = Int.gcd a b` coerces the RHS to `Int`;
  under `nat` use `Nat.gcd`). Core Lean also exposes `Int.fdiv` (floor) / `Int.tdiv`
  (trunc) / `Int.ediv` (Euclid) with matching `fmod`/`tmod`/`emod`, plus `Int.natAbs`,
  `Nat.gcd`, `Nat.factorial`, `Nat.totient`. Ceiling division: `⌈a/b⌉ = -(Int.fdiv (-a) b)`.
- **Indexed write with an arithmetic RHS fails** (`usize`/`Std.Usize`): `v[i] = x + y`,
  `v[i] = x * y`, `v[i] = x - y` (add/sub/mul directly in a `Vec` index-mut write) raise
  Aeneas "Unimplemented binary operation" — these lower to the *checked* ops
  (`AddChecked`/`SubChecked`/`MulChecked`) which the symbolic evaluator can't handle.
  `v[i] = x / y` (division) and `v[i] = <var>`/`<literal>` are fine. Fix: hoist the
  arithmetic into a `let` first — `let t = x + y; v[i] = t;`. (This is why
  `integer_euler_totient_sieve` writes `let pm1 = p - 1; phi[p] = pm1;`.)
- **`assert_eq!` on `Vec<u64>` fails** (`PartialEqNat` unknown): comparing two `Vec<u64>`
  (→ `alloc.vec.Vec Nat`) with `assert_eq!` in `tests/test.rs` fails to compile — `Nat`'s
  `PartialEq` isn't imported under that name. `Vec<usize>` (→ `Vec Std.Usize`) compares
  fine. Fix: compare element-by-element — `assert_eq!(f.len(), n)` + `assert_eq!(f[0], x)`
  etc. (see `integer_factors` / `integer_prime_factors` tests).
- **`assert_eq!` on a tuple `(T, T)` fails** (`Pair.Insts.CoreCmpPartialEqPair.eq` unknown):
  comparing a tuple-returning call with `assert_eq!(f(...), (a, b))` emits an unresolved
  `Pair.Insts…PartialEq` instance and the whole test becomes noncomputable (`#eval` aborts).
  Fix: destructure then compare scalars — `let (x, y) = f(...); assert_eq!(x, a); assert_eq!(y, b);`
  (see `crypto_hill`; `crypto_mpc_gc` returns a 3-tuple and already destructures).
- **Bare `%` (modulo) as a function's final expression is not `ok`-wrapped**: a function
  whose tail is `x % CONST` (constant divisor) with only `massert`/pure `let`s — no `←`
  bind, no `if` tail, no `while` loop — translates to `Result Nat := do … x % CONST`, but
  `x % CONST` has type `Nat` (not `Result Nat`) so `lake build` fails with a type mismatch.
  AENEAS *does* `ok`-wrap the tail when the body has a `←` bind (e.g. `affine_decrypt`,
  `mod_inverse`) or an `if` tail (`integer_div_mod`'s `mod_floor`), and `-` (subtraction)
  tails (`crypto_hill`'s `determinant`) — it's the bare `%`-by-constant that's special-cased
  wrong. Fix: route through a Result-returning helper, e.g. a `while`-based reduction
  `fn mod26(x) { let mut r = x; while r >= 26 { r = r - 26; } r }` and return `mod26(expr)`
  (see `crypto_affine`).
- **Integer casts (`as`) between unsigned types are supported under `bitvec`** —
  `u8 as usize`, `u8 as u32`, `u32 as u8` etc. lower to `UScalar.cast`/`IScalar.cast`
  and are rewritten to `BitVec` conversions by `_bitvec_ops_to_bv` (see §5 `bitvec`).
  `crypto_aes128` and `crypto_des128` exercise this with table lookups (e.g. `SBOX[x as usize]`).
- **`wrapping_sub` on unsigned types is scalar-lowered by AENEAS** — under `bitvec`,
  `a.wrapping_sub(b)` → `a - b` (BitVec subtraction wraps). This is used in `crypto_tea`'s
  decrypt and works correctly.
- **Feistel cipher half-combination pattern** — `(low as u64) | ((high as u64) << 32u32)`
  combines two `u32` halves into a `u64` block. Under `bitvec` the `u32 as u64` zero-extends
  via `UScalar.cast`, and `<<`/`|` on `BitVec 64` works. This pattern is used in
  `crypto_tea` and `crypto_des128` (`(r << 32u32) | l`).
- **Pass key as separate parameters, not arrays** — `[u64; 2]` array parameters may not
  translate cleanly; split into `key0: u64, key1: u64` (see `crypto_tea`).
- **Verify crypto test vectors against the authoritative source** — a failing `cargo test` /
  `#eval` can be your memory of the vector, not the code. During `crypto_aes128` I swapped
  SP 800-38A block-2's ciphertext for block-4's plaintext (they coincide as `f69f2445…`);
  the code was right, the vector wasn't. Web-check the FIPS/NIST table before debugging.
  When using Wikipedia test vectors for TEA, verify byte-endianness — the reference's
  `to_block`/`from_block` is little-endian; raw `u64` values are not portable.
- **Early `return` inside a deeply-nested loop turns the whole function into a
  `noncomputable axiom`** — `graph_bipartite_check`'s first version used `return None`
  inside `while k < m` inside `while head < tail` inside `if color[s] == 2` inside
  `while s < n`; AENEAS emitted `noncomputable axiom is_bipartite …` (0 `sorry`, but
  the body was gone), and the translator still reported `aeneas ok / output 1 defs`.
  A single top-level `return Err(())` (like `graph_bellman_ford`'s negative-cycle loop)
  is fine; the deep nesting is what breaks it.  **Fix:** thread a `bool` flag
  (`conflict = true;`) instead of early-returning, and branch on it once at the end.
- **`.to_vec()` on a slice of `Option`/other ADT elements pulls in a `noncomputable
  axiom` (the `Clone` instance) and makes the whole function noncomputable, so `#eval`
  aborts** — `graph_floyd_warshall`'s first `Option<i64>` version used
  `weight.to_vec()`, and AENEAS emitted `noncomputable axiom
  core.option.Option.Insts.CoreCloneClone.clone`; every `floyd_warshall_*` def was
  marked `noncomputable` (0 `sorry`, but noncomputable = not `#eval`-able).  **Fix:**
  copy element-by-element with a `while` + `Vec.push` loop instead of `.to_vec()`.
- **A recursive function returning a primitive int (`u64`/`usize`) whose result is
  used in arithmetic at tail position drops the `ok` wrapper** — `ds_linked_list`'s
  `length` (`1 + length(*tail)`) generated `let i ← length tail; 1 + i` (no `ok`),
  which fails with `HAdd ℕ ℤ (Result ℤ)` (`1` elaborates as `Nat`).  **Fix:** thread
  the count as an accumulator parameter (`fn length_acc(list, n) { … length_acc(*tail, n+1) }`)
  and return `n` at the base case, or accumulate into a `Vec` and return `.len()`.
- **A private `&mut Vec` helper in `src/lib.rs` is re-emitted into `Test_test.lean`,
  colliding with the copy in `<Crate>.lean`** ("already been declared") —
  `ds_linked_list`'s `collect_into(list, acc: &mut Vec<i32>)` was duplicated into the
  test file (which imports the impl), while a by-value helper (`reverse_acc`) was not.
  **Fix:** avoid `&mut Vec` helpers in `lib.rs`; pass the accumulator by value and return
  it.  (`&mut Vec` helpers in `tests/test.rs` are fine — they're translated once.)
- **A helper you define in `tests/test.rs` must not share a name with ANY function in
  `src/lib.rs` (public or private)** — charon re-emits the lib's private helpers into the
  test translation, so a same-named test helper collides ("`f` redefined here"; the test
  then fails to compile even though `cargo test` is fine).  `ds_avl`'s test checker reused
  the names `height`/`max_u64`/`taller_by_two`, which are private `lib.rs` helpers.  **Fix:**
  give test-local helpers distinct names (e.g. `subtree_height`, `too_tall`).
- **`assert_eq!(opt, Some(x))` / `assert_eq!(opt, None)` on an `Option` value fails**
  with `Unknown identifier core.option.Option.Insts.CoreCmpPartialEqOption.eq`.  **Fix:**
  unwrap with a sentinel and compare the scalar — `let h = match v { Some(x) => x, None => -1 };
  assert_eq!(h, 1);` — or `let b = match v { Some(_) => false, None => true }; assert!(b);`.
- **`assert!(false)` in a test body turns the whole test function into `sorry`**
  (`ds_queue`'s `None => assert!(false)` branch).  **Fix:** compute a `bool` via
  `match` and `assert!` it, instead of asserting a literal `false`.
- **Arithmetic (`%`, `+`, `-`, `*`) on *locally-derived* `usize` values in a
  straight-line function body gets mis-classified as `Int` and converted to pure `:=`**
  (`ds_queue`'s `enqueue`/`dequeue`: `let idx := i % cap` where `i`/`cap` came from field
  reads / `Vec.len`).  The `monadic_ops_to_pure` pass tracks `usize`-ness of function
  *parameters* and `#usize` literals only — not locals derived from `Vec.len` or struct
  fields.  Arithmetic *inside* a `while` loop is fine (the loop body becomes a `rust_loop`
  whose params are tracked).  **Fix:** replace `% cap` with `if x >= cap { x - cap } else { x }`
  (exact when `x < 2·cap`), or hoist the arithmetic into a `rust_loop`/helper with `usize` params.
- **The same pass tracks *only parameters and `#usize` literals*, so a binary op whose
  *two* operands are both computed locals becomes a pure `let x := A op B` — and
  `Std.Usize`'s `HMul`/`HAdd`/… return a `Result`, so the next use of `x` fails to
  elaborate with `failed to synthesize instance of type class
  HAdd Usize (Result (UScalar UScalarTy.Usize))`.**  This bites in the *implementation*
  too, not just tests, and inside `while` bodies: `let p = f[j * stride + t] *
  g[(r - j) * stride + t];` (two slice reads) and `let p = a[t] * b[rest];` both become
  `let p := i2 * i6`, and the following `acc + p` then breaks.  The op survives only when
  at least one operand is a parameter, a `#usize` literal, or a variable that an earlier
  line already tracked as `usize` (which is why `cur + dist[i]` is fine: `cur` is a
  `rust_loop` parameter).  **Fix:** route the operation through a `usize`-parameter helper
  so both operands become *call arguments* — `fn mul_usize(x: usize, y: usize) -> usize
  { x * y }`, then `let p = mul_usize(a[t], b[rest]);` (the `iadd`/`flat_index` trick, but
  needed for `+ - * / %` alike, in `src/lib.rs` and `tests/test.rs`).  Grep the generated
  file for a pure `let … := … op …` whose operands are results of `Slice.index_usize`.
- **An unannotated integer literal defaults to `i32`, even where a `usize` is required** —
  under `int-model = nat` that becomes `Nat` and poisons the expression.  In particular a
  shift amount need not be `usize`: Rust implements `Shl<i32> for usize`, so
  `let n = 3; … 1usize << n` infers `n : i32`, and the generated `1#usize <<< 3` fails with
  `failed to synthesize instance of type class HShiftLeft (UScalar UScalarTy.Usize) ℕ ?m`.
  **Fix:** annotate the variable — `let n: usize = 3;`, `let mut count: usize = 0;` —
  charon then inlines the literal as `3#usize`; or write the shift amount `3usize`.
  Worst in `tests/test.rs` brute-force helpers, like the `count: usize` entry above.
- **A broken test/impl file does not fail the translator — only `lake build` does.**  The
  translator still prints `aeneas ok` / `output N defs` when the generated file has type
  errors, and the `#eval` lines then abort with "expression depends on the `sorry`
  axiom".  When that happens: `lake build` the generated file, grep it for `sorry`, and
  look for the two type-class failures above — the Rust is usually fine as far as
  `cargo test` is concerned.
- **A field access (`st.k`) inside arithmetic isn't matched by the operand regex
  (`\w+`), so its result is untracked and a downstream `+ j` is mis-classified as
  `Int`** (`ds_sparse_table`'s `range_min`: `l * st.k` → `idx1 := i1 + j`).  Hoisting
  `let k = st.k` does **not** help — rustc/charon inlines it back.  **Fix:** route the
  arithmetic through a helper `fn flat_index(row, k, col)` so the field access becomes a
  call *argument* (not arithmetic) and the helper's `usize` params are tracked.
- **Rust `==` translates to Lean `=` (`Prop`), but `!=` translates to `Bool`** — so
  `let ok = s[a] == s[b];` assigns a `Prop` to a `bool`-typed variable and the whole
  function breaks with `Application type mismatch … has type Prop but is expected to have
  type Bool`.  `==` is only safe as an `if`/`while` *condition*; `!=` is safe both as a
  condition and as a value.  **Fix:** for a boolean you'll reuse, write the test as `!=`
  in an `if` (e.g. `if s[hi] != s[lo] { stop } else { … }`) instead of `==` → `bool`
  (see `string_z_algorithm`, `string_manacher`).
- **Integer `as` casts (`u8 as u64` etc.) only lower cleanly under `bitvec`** — under
  `nat`/`int` they emit `lift (UScalar.cast .U64 b)` with `b : Nat`, which does **not**
  typecheck (`UScalar.cast` expects a `UScalar`).  **Fix:** avoid casts by encoding
  characters/values directly in the target type (e.g. `string_rabin_karp` uses `u64` chars,
  `string_aho_corasick` uses `usize` chars — the plan's "整数编码字符" option).
- **`i - j + 1` evaluates left-to-right in Rust**, so when `j > i` (e.g. a palindrome radius
  `j = i+1`) the `i - j` underflows before the `+ 1` rescues it.  Rewrite as `i + 1 - j`
  (mathematically identical, no underflow since `j ≤ i+1`).  Hit in `string_manacher`'s
  `l = i - j + 1`.
- **`if let … else if let …` chains with a mutable `case` accumulator are mangeled by
  Aeneas** — `ds_red_black_tree`'s first `balance` used
  `let mut case = 0; if let Node(Red, l1, ..) = &l { if let Node(Red, ..) = &**l1 { case = 1; }
  else if … }`; the generated Lean was garbage (`ok (Tree.Node …)` fragments, `case1` used
  before binding, `1#iscalar` type errors).  **Fix:** move the shape *detection* into a
  separate non-recursive helper that is a pure nested `match` returning an `i32`/enum (no
  `if let`, no mutation), then branch on the returned code with `if case == 1 { … }` (see
  `balance_case` / `right_case` in `ds_red_black_tree`).  The move-out-of-`Box` concern is
  avoided by inspecting `&Tree` in the detector and destructuring by value only in the
  reconstruction arms.
- **`match <i32> { <literal> => … }` (integer-literal *patterns*) emit `1#iscalar`** — the
  translator lowers an `Int`-typed scrutinee's literal patterns to `IScalar` literals, which
  fail to typecheck against `Int` (`1#iscalar has type IScalar but is expected to have type
  ℕ`).  `match` on an *enum* (e.g. `Color.Red`) is fine; it is specifically `match x with
  | 1 => … | 2 => …` on a Rust integer.  **Fix:** use `if x == 1 { … } else if x == 2 { … }`
  (the `==` in an `if` condition translates to `x = 1`, an `Int` equality).  Hit in
  `ds_red_black_tree`'s `balance`.
- **A `>=`/`<=`/`>`/`<` comparison whose operands are *struct-field accesses* raises
  "Could not match the contexts"** — `ds_treap`'s first `is_heap` used
  `n.priority >= l.priority` (fields of `&Box<Node>`), which made Aeneas's loop-join
  (`InterpJoin.ml`) fail on the whole test file.  Comparisons on `Vec` elements / plain
  locals (e.g. `keys[i-1] >= keys[i]`, `if a >= b`) are fine; it is the `n.field OP m.field`
  shape that breaks.  **Fix:** route the comparison through a helper so the field accesses
  become *call arguments*: `fn ge(a: i32, b: i32) -> bool { if a >= b { true } else { false } }`,
  then `ge(n.priority, l.priority)`.  (Same root cause as the `ds_sparse_table` field-in-
  arithmetic pitfall, but for comparison operators.)
- **Long `&&`/`||` chains (esp. of bool-returning calls with field-access args) raise
  "Could not match the contexts"** — `ds_kdtree`'s query used
  `let disjoint = a || b || c || d;` and `let contained = a && b && c && d;` (each `a`/`b`
  an `ile(x, n.minx)`-style call), which broke Aeneas's loop-join.  A 1–2 term `&&` on
  plain `bool` *variables* is fine; it is the long chain of *calls* that trips it.  **Fix:**
  fold through binary helpers so every operand is a call/var: `fn both(a b : Bool) :=
  if a then b else false`, `fn either(a b : Bool) := if a then true else b`, then
  `both (both a b) (both c d)` / `either (either a b) (either c d)`.
- **`Int` addition with a struct-field operand (or a bare `+`/`x + 0` tail) is not
  classified pure / not `ok`-wrapped** — `ds_kdtree`'s `let sum = n.sum + val` generated
  `let sum ← n.sum + val` (field access in arithmetic) and the query's tail
  `node_contrib + left + right` generated a bare non-monadic tail.  **Fix:** route addition
  through a helper `fn iadd(a b : Int) := a + b` so fields/partial sums become *call
  arguments*, and return a `let`-bound variable rather than a bare arithmetic tail
  (an `if cond then iadd(..) else <var>` tail is also fine).
- **A "present/absent" stamp array must never reuse a valid data value as its sentinel.**
  `matrix_sparse_cholesky`'s `symbolic_fill` used `vec![n; n]` as an "absent" marker while
  also storing the *global* `l_row_idx.len()` as the "present" marker; once the fill pattern
  grew to `≥ n` nonzeros, `l_row_idx.len() == n` collided with the sentinel, the
  `if pos[i] == n` de-dup guard re-added already-present rows, the output CSR got duplicate
  column indices, and `L·Lᵀ ≠ A` for perfectly valid SPD inputs (caught by the review agent,
  not by the original tests, because they only used `< n` nonzeros).  **Fix:** stamp with the
  *column index* `pos[i] = j` and test `pos[i] != j`, with sentinel `n` (which no column stamp
  can equal); no reset pass is then needed between columns.
- **An unconstrained integer literal in Rust defaults to `i32`** — `let mut count = 0;` is
  `i32`, which under `int-model = nat` becomes `Int`, so a later `count += 1usize`-flavoured
  use emits `HAdd ℤ ℕ (Result ℤ)` and the whole function degrades to `sorry` (the failing
  `#eval` then aborts with "depends on the `sorry` axiom").  This bites hardest in
  `tests/test.rs` brute-force helpers, which look innocuous.  **Fix:** annotate every
  counter: `let mut count: usize = 0;` (see `string_palindromic_tree`).
- **In `Spec.lean`, `Std.Usize` has no `OfNat` instance** — a plain `0`/`1` literal where a
  `Std.Usize` is expected fails with `failed to synthesize OfNat Usize 0`.  Use the Aeneas
  literal form `0#usize` / `1#usize` (e.g. as the default in `(… : List Std.Usize).getD i 0#usize`).
- **`alloc.vec.Vec.deref v` is a `Slice`, not a `List`** — `(alloc.vec.Vec.deref v).map f`
  fails with "no field `map`".  Coerce explicitly:
  `((alloc.vec.Vec.deref v) : List T).map f` (see `string_lz77`'s `factorsOf`).
- **`List.get?` / `List.getElem?` do not exist in this toolchain (Lean v4.31.0)** — a
  `∀ i f, fs.get? i = some f → …` statement fails to elaborate.  Reformulate positional
  properties as recursive predicates that thread the running state
  (e.g. `allMaximal s pos fs`, `insideHistory pos fs` in `string_lz77`), which also avoids
  bounds proofs entirely.
- **Spec helper names can collide with Mathlib** — a root-level `def rotation` clashes with
  Mathlib's `rotation : Circle →* ℂ ≃ₗᵢ[ℝ] ℂ` (`'rotation' has already been declared`, plus
  downstream `expected to have type Circle` errors).  Wrap such helpers in a
  `namespace <CaseName>` + `open <CaseName>` so the theorem statements still read
  unqualified (see `string_min_rotation`).
- **`if <prop> then …` inside a `def` needs the instance to be unfoldable** — a
  `def isPalindrome … : Prop` used in `if isPalindrome w then …` fails instance search
  (`Decidable (isPalindrome w)`) because the definition is opaque to it.  Declare the
  predicate as `abbrev` (same statement, still a `Prop`) or pass a `Decidable` instance.
  Same for a `Prop` predicate used in `Finset.filter`: `def Submask T S : Prop := T &&& S = T`
  makes `(Finset.range (S+1)).filter (fun T => Submask T S)` fail with
  `failed to synthesize instance of type class DecidablePred fun T => Submask T S`;
  `abbrev` fixes it (see `dp_subset_convolution`).
- **A theorem about a *helper* must carry every precondition the helper relies on — not
  just the ones the caller happens to satisfy.**  `string_suffix_array`'s
  `suffix_array_lcp_correct` originally hypothesised only that `sa` is in non-decreasing
  suffix order, because that is all the *caller* (`build_sa`) knows.  But `kasai_lcp`
  scatters `rankpos[sa[j]] = j`, so a repeated/out-of-range index silently overwrites
  entries: on `s = [97, 98]`, `sa = [0, 0]` is trivially "sorted" and `lcp[0]` comes back
  `0` where the theorem claims `2`.  Add the missing `IsPerm (toNatList sa) s.length`.
  (Found by the `review/` agent, not by compilation — `sorry` hides it.)
- **A validity predicate used as the hypothesis of an optimality theorem must encode
  *every* restriction of the algorithm's variant.**  `string_lz77`'s
  `IsValidFactorization` required only `offset ≤ history length`, silently allowing
  *self-overlapping* copies; `lz77_optimal` was then false, because `"aaa"` factors as
  `[(0,1), (1,2)]` (2 factors, overlapping) while greedy non-overlapping returns
  `[(0,1), (1,1), (2,1)]` (3).  The non-overlap clause `f.2 ≤ f.1` for copy factors was
  missing.  Rule of thumb: for `lz77_optimal`-style claims ("greedy ≤ any valid
  factorization"), write the predicate from the *implementation's inner-loop guard*
  (`ql < pos` here) rather than from a prose description.
- **Aeneas's `Vec` is `ListN`-backed, so *every* `v[i]` read and `v[i] = x` write is
  O(length)** (each one does `Slice.val = list.toList` and then `Slice.from = fromList`).
  A 256-wide table written with random writes makes `#eval` quadratic-ish: building the
  FM-index `occ` table this way took **~75 s at n = 12**.  **Fix: build big tables by
  appending whole rows with `Vec::extend_from_slice`** — it translates cleanly to
  `alloc.vec.Vec.extend_from_slice` + `core.clone.CloneUsize` (no `sorry`, no
  `noncomputable`) and took the same n = 12 case from **80 s to 0.96 s (~80×)**; the whole
  `Test_test.lean` then evaluates in ~9 s.  `extend_from_slice` is a plain `Vec` method
  (no external crate), so it is inside the supported subset even though §4 does not list it.
  If a testcase's `#eval` is mysteriously slow, look for a hot indexed-write loop first.
- **`as` is a reserved keyword in Lean 4** — a pattern `| a :: as, b :: bs => …` in
  `Spec.lean` fails with `unexpected token 'as'`.  Use `xs`/`ys`.
- **A test-local helper whose name is a *prefix* of a `src/lib.rs` function re-emits that
  library function into `Test_test.lean`** — `graph_dilworth`'s test helper `cover`
  pulled in the lib's private `cover_size` (`extract_test::extract_defs_by_prefix`
  selects defs by name prefix of any test-file `fn`), so `lake build` failed with
  `` `graph_dilworth.cover_size` has already been declared`` while *both* `cargo test`
  and the translator stayed green.  **Fix:** make test-local helper names unrelated to
  the lib's (`usize_diff`, `brute_*`, `bf_*`, `t_*`) — being *different* is not enough,
  it must not be a prefix.  Related: §9's `&mut Vec` helper and same-name-helper entries.
- **Several loop-carried scalars assigned inside a nested `if` → "Could not compute a loop
  fixed point in 2 iterations".**  `graph_brooks`'s `find_brooks_pair` stored `(v,u,w)`
  in three locals (`fv = v; fu = u; fw = w;`) inside a doubly-nested `if` and Aeneas
  abandoned the body.  A single-scalar min/max scan and array writes both translate fine,
  so **store the tuple in a small `Vec<usize>` instead of parallel scalars.**  Same class
  as the `dp_house_robber` rolling-state entry above, but triggered by *any* multi-scalar
  assignment under nested `if`s, not just a `prev2/prev1` recurrence.
- **`%` (`Rem`) on two `usize` operands is unreliable in test files** — `UScalar.rem`
  exists in Aeneas's `Std/Scalar/Ops/Rem.lean` and `q % 2#usize` translates fine in an
  *impl* body (`graph_tutte`'s `tt_bit`), but `graph_matroid_intersection`'s test-file
  brute force hit `Unknown identifier Aeneas.Std.Usize.rem`.  **Fix:** compute the low bit
  as `x - (x / 2) * 2` (hoisted into a `let`), which is unambiguous.  If a `%`-using
  helper refuses to build, this is the first thing to change.
- **`∑ i in s, f` (binder notation) does not parse in this toolchain** (`unexpected token
  'in'`).  Use the explicit `Finset.sum`/`(Finset.range k).sum f` forms; `List.ofFn` is
  also available for turning `Fin n → Nat` into a degree list, whereas `List.finRange` is
  not used anywhere in the repo.
- **The spec's adjacency relation must be the *same predicate* as the implementation's edge
  test.**  `graph_chordal_mcs` defined `Adj i j := entry == 1` while the algorithm bumps
  weights on `a != 0`, so for a matrix entry `≥ 2` the algorithm uses an edge the spec
  cannot see and `mcs_peo_iff_chordal` is **false** (the review found it with a hand-built
  matrix); `graph_brooks` had the mirror form (`is_complete_graph` tests `a == 0` as "no
  edge", so an entry `2` reads as an edge there while `Adj`/`Delta` only count `== 1`,
  making `Δ+1` and Brooks' bound false).  Both were fixed by carrying the 0/1 hypothesis
  — `h_adj01 : ∀ u v, toMatrix … u v = 0 ∨ toMatrix … u v = 1`, the
  `graph_hopcroft_karp` convention.  Defining `Adj` as `entry ≠ 0` (matching the impl) is
  the alternative; either way, write the spec relation from the *implementation's*
  comparison, not from the prose in the doc comment.
- **In `Spec.lean`, a `Slice` index proof sees `(↑s).length`, not `s.length`, and
  `rw` will not close the gap.**  `row_ptr[i.val]'(by …)` demands
  `i.val < (↑row_ptr).length` (the *coerced* form), while `h_rp : row_ptr.length = n + 1`
  is stated with the projection — and `rw [h_rp]` fails with *"Did not find an
  occurrence of the pattern `row_ptr.length`"*, and `omega` fails too (it abstracts
  `↑(↑row_ptr).length` as an atom and never learns `= n + 1`).  **Fix:** force the
  syntactic form first — `by show i.val < row_ptr.length; rw [h_rp]; exact …` —
  because `show` checks up to definitional equality (the two forms *are* defeq, which is
  why the `have h_lo : i.val < row_ptr.length := by rw [h_rp]; …` style used by
  `csrDecode` works).  See `matrix_sparse_cholesky_symbolic`.
- **A doc-proposed spec can be mathematically false — check the literature's exact
  statement, not the sketch.**  `MATRIX_RESEARCH_TESTCASES.md` proposed "`L[i][j] ≠ 0`
  iff `j` is an ancestor of `i` in the elimination tree" for symbolic Cholesky; that is
  false in both directions (in the 4-cycle `0—1—2—3—0` the fill entry `L[3][1]` exists,
  and the tree there is the path `0→1→2→3`, so `1` is *not* an ancestor of `3`; conversely
  in a tridiagonal matrix `0` *is* an ancestor of `2` while `L[2][0] = 0`).  The true
  statement is the Rose–Tarjan–Lueker **fill path theorem**: `i` must reach `j` through
  interior vertices that are all *smaller than `j`*.  Worth validating any
  characterisation-shaped spec against a brute-force implementation of the *definition*
  before writing it (`matrix_sparse_cholesky_symbolic`: 300 random symmetric patterns,
  Rust vs. a Python clique-completion elimination — 0 mismatches).
- **A fraction-free (Bareiss) determinant *without* pivoting silently returns 0 for a
  nonsingular matrix whose leading pivot vanishes** — `matrix_smith_normal_form`'s test
  helper printed `det Q = 0` for a genuine unimodular `Q = [[1,-2,-2],[0,0,1],[0,1,0]]`
  (`det = -1`, but `Bareiss` hits `m[1][1] = 0`).  If a `det` helper feeds a `±1`
  check, give it row pivoting (with a sign flip) or use the permutation expansion; and
  when a test fails, first ask whether the *helper* is wrong.  When an independent
  brute-force check is easier outside the repo, do it there — the SNF spec's claims
  (`D = P·A·Q`, `P`/`Q` unimodular, invariant factors) were cross-checked on 400 random
  integer matrices against the determinantal-divisor `d_k = Δ_k/Δ_{k-1}` formula.
- **A 5-deep loop nest translates, but aeneas needs ~7 minutes per phase for it** —
  `matrix_smith_normal_form`'s single pass (six inner loops nested inside
  `while t < nm` inside `while !advanced`) produced an 89 kB, 2.7 k-line `.lean`
  file and two aeneas runs of ~7 min each (a `translate_all.sh -w 8`-style batch is
  fine; just don't assume the usual seconds).  `charon`/`aeneas` still reported `ok`
  and no `sorry` appeared (grep for `sorry`/`axiom` in the generated file before
  trusting it).  Structural alternatives that keep such an algorithm translatable are
  in the SNF entry of §10 — sentinel-`Vec` pivot search instead of a `found` flag,
  `if clean { … }` guards instead of early `return`s, and `t = nm` to leave the outer
  loop.
- **Put heavy / randomized differential tests in the *untranslated* `#[cfg(test)] mod tests`
  of `src/lib.rs`, not in `tests/test.rs`.**  Only `tests/test.rs` is translated (into
  `Test_test.lean`, whose `#eval`s `check_all.py` asserts), so it must stay small and fast
  (Lean's `Vec` is `ListN`-backed: every indexed read/write is O(n)).  The crate's own unit-test
  module is *not* picked up by the test translation — verified on `ds_lct`, `ds_splay_tree`,
  `ds_binomial_heap`, `ds_van_emde_boas` and `ds_fibonacci_heap`, whose randomized fuzzers,
  brute-force reference constructions and representation-invariant checkers never appear in
  `<Crate>.lean`.  This module is also where the invariant that a `Spec.lean` hypothesis claims
  must be brute-force checked: assert it — and the *conclusion* of each research-level theorem —
  after **every** operation on randomized inputs (that is how the defects listed in §8's review
  step were found in practice, e.g. an inverted heap-order comparison in a consolidation table).
- **`Multiset.erase` removes exactly *one* occurrence of its argument** in this toolchain
  (`Multiset.count 1 (({1,1,1} : Multiset Nat).erase 1) = 2`), which is the right operator for
  "an extract-min removes one copy of the minimum"; `s - {a}` likewise removes one occurrence.
  (Both differ from `Finset.erase`, which removes the element outright.)  Heaps with duplicate
  minima are the case to test — and note the same distinction for the covering view: a
  `key Multiset` (per-node occurrences) is what multiset laws want, while a `Set` view collapses
  duplicates and would make "`keys` shrinks by one occurrence" unstatable.

---

## 10. Implemented testcases by float-model

- **Rat (has `Test_test.lean`)**: `matrix_lu`, `matrix_ldlt`, `matrix_sparse_lu`
  (sparse LU with partial pivoting on CSR input; factors `L`/`U` returned in CSR,
  decoded back to dense via `csrDecode`; spec = `P · csr_decode A = csr_decode L ·
  csr_decode U`), `matrix_det`,
  `matrix_rank`, `matrix_charpoly` (Faddeev–LeVerrier; spec = coefficients match
  `Matrix.charpoly` + Cayley–Hamilton), `matrix_jordan` (full Jordan normal form:
  char poly → rational-root factoring → generalized-eigenspace ranks → block
  assembly; `success=false` when the char poly does not split over ℚ),
  `array_gram_schmidt`, `array_*`, … (pure arithmetic).  `array_median_of_medians`
  (BFPRT linear-time k-th order statistic; spec = result ∈ input and at least
  `k+1` elements `≤` it / `n-k` elements `≥` it) and `array_floyd_cycle`
  (Floyd's tortoise-and-hare on a `usize` functional graph; spec = the result
  is the first periodic node on the orbit, via explicit tail `μ` + cycle `λ`).
- **Research-level matrix theorems** (Rat; all have `Test_test.lean`, `int` model; the
  matrix category's hard tier, cf. `MATRIX_RESEARCH_TESTCASES.md`):
  `matrix_permanent` (permanent by **Ryser's inclusion–exclusion formula**
  `per(A) = (-1)ⁿ·Σ_{S} (-1)^{|S|}·Π_i Σ_{j∈S} a[i][j]`, with the `2ⁿ` subsets enumerated
  by an explicit binary counter over a flag array and the sum grouped by `|S|` so the
  alternating sign is applied once per subset *size* — `acc[k]` — rather than once per
  subset; spec = `p = Permanent A` with `Permanent A = ∑ σ : Equiv.Perm (Fin n), ∏ i, A i (σ i)`,
  the *definitional* permanent over `n!` permutations, so the theorem is "Ryser = definition"),
  `matrix_smith_normal_form` (invariant factors by elementary row/column operations with
  the two accumulators: every row op is also applied to `P` (left mult.), every column op to
  `Q` (right mult.), the pivot search takes the *first* non-zero of the trailing sub-matrix
  (kept in a 3-element `Vec` — see the §9 fixed-point note), the row/column sweep swaps the
  smaller division remainder into `(t,t)`, `row_t += row_i` is the divisibility fix-up that
  makes it more than a diagonalisation, and at the end the sign of each invariant factor is
  normalised; spec = `D = P·A·Q` with `P`,`Q` **unimodular** (`Matrix.det P = ±1`), `D`
  diagonal, non-negative, `dᵢ ∣ dⱼ` for `i < j`), `matrix_sparse_cholesky_symbolic` (the
  *symbolic* phase alone — no numerical values: CSR→CSC, the left-looking fill closure
  `Struct(L[:,j]) = {j} ∪ {i>j : A[i,j]≠0} ∪ ⋃_{k<j, L[j,k]≠0} Struct(L[:,k])` with the
  column-index stamp array, and the elimination tree `parent[j] = min{i>j : L[i,j]≠0}`
  (sentinel `n` = root); spec = the **Rose–Tarjan–Lueker fill path theorem** — `L i j` is
  present iff `A i j ≠ 0` or there is a path `i = v₀,…,v_k = j` in the graph of `A` whose
  interior vertices are all `< j` — plus the diagonal, lower-triangularity, and the
  min-index characterisation of `parent`; the input's CSR well-formedness and symmetry are
  hypotheses, and the *earlier* "`j` is an ancestor of `i`" formulation is **false**, see §9).
- **Graph algorithms** (Rat; all have `Test_test.lean`, `int` model): single-source
  shortest path `graph_dijkstra` (non-negative weights) and `graph_bellman_ford`
  (negative weights + negative-cycle detection); MST `graph_kruskal` / `graph_prim`;
  DAG ordering `graph_topological_sort` (Kahn's algorithm); 2-coloring
  `graph_bipartite_check` (BFS); all-pairs shortest path `graph_floyd_warshall`
  (dense O(V³) DP, negative weights + negative-cycle detection; distances are
  `Option<i64>` — `None` = no path — so no `INF` sentinel pollutes the integer
  domain); unweighted shortest path `graph_bfs` (BFS, returns `Option<usize>`
  distance-in-edges).  Specs follow the
  `Finset`-of-edges (or weight-matrix) pattern with an inductive `has_path`, then
  "soundness + completeness" theorems; `graph_topological_sort` adds an
  `is_topological_ordering` predicate and `graph_bipartite_check` an
  `is_valid_coloring` predicate, and `graph_bfs` an `is_shortest_distance`
  (edge-count) predicate (`has_path` carries the distance instead of a weight).
- **Research-level graph theorems** (Rat; all have `Test_test.lean`, `int` model; each
  spec is a *named* theorem, added as the graph category's hard tier, cf.
  `GRAPH_RESEARCH_TESTCASES.md`): `graph_chordal_mcs` (MCS ordering; spec =
  `IsPEO pi ↔ IsChordal` — Fulkerson–Gross / Rose–Tarjan–Lueker — so `pi` must be the
  *numbering* order, i.e. reverse selection order; the raw selection order is **not**
  always a PEO, the diamond is a counterexample), `graph_matroid_intersection`
  (exchange-graph BFS over graphic ∩ partition matroids; spec = the support is
  common-independent, of maximum cardinality, plus the Edmonds min–max identity),
  `graph_chu_liu_edmonds` (cycle contraction, returns `(exists, weight)` rather than the
  edge set; spec = ∃ an arborescence of that weight and it is optimal — note the charge
  is `OPT(G′) + Σ_{v∈cycle} w(in v)`, *not* the sum of every round's minima),
  `graph_gomory_hu` (Gusfield's n−1 max-flows, returns `(parent, weight)`; spec =
  `minEdgeOnPath = minCut` for all pairs), `graph_tutte` (O(2ⁿ) enumeration of `S` with
  the odd-component count; spec = Tutte's 1-factor theorem plus Petersen's
  bridgeless-cubic corollary), `graph_dilworth` (Kuhn matching on the split graph; spec =
  min path cover `= n − ν` and Dilworth — `IsPathCover` (real DAG *paths*) is kept
  distinct from `IsChainCover` (Reach-*chains*) because they genuinely differ, smallest
  counterexample `0→1, 0→2, 3→0, 4→0`), `graph_brooks` (Brooks ordering off a vertex with
  two non-adjacent neighbours + an exhaustive-backtracking fallback; spec = proper
  colouring, `≤ Δ+1` always and `≤ Δ` under Brooks' hypotheses), `graph_erdos_gallai`
  (Havel–Hakimi; spec = `IsGraphical ↔ EGCondition`, with `IsGraphical` a *sorted*
  degree-sequence claim and `NonIncreasing` a hypothesis where the two forms differ).
- **Recursive DFS / SCC** (Rat; has `Test_test.lean`, `int` model):
  `graph_scc_tarjan` (Tarjan's strongly-connected-components via recursive DFS).
  The whole mutable state (`index`, `lowlink`, `on_stack`, `stack` + cursor,
  `comp`, `counter`, `comp_count`) is bundled into a `TarjanState` struct threaded
  **by value** through the recursive `strongconnect` (spelling out AENEAS's
  `&mut` = consume-and-return model explicitly), and the inner loops are lowered
  to recursion by `-loops-to-rec`.  Output is a **flat** `Vec<usize>` of dense
  component ids (never `Vec<Vec<usize>>`); the pop-loop's `break` is replaced by
  a `done` flag.  Spec is `strongly_connected` (= `has_path u v ∧ has_path v u`)
  plus `comp u = comp v ↔ strongly_connected u v`.
- **Maximum flow** (Rat; has `Test_test.lean`, `int` model): `graph_max_flow`
  (Edmonds-Karp = Ford-Fulkerson with BFS shortest augmenting paths).  The graph
  is a flat `n × n` row-major capacity matrix (`Vec<i64>` / `Slice Int`), indexed
  via a private `idx(row, n, col)` helper (the `flat_index` pattern) so the
  `row * n + col` arithmetic never appears inline in a slice read/write; the
  reverse residual edge is read off the skew-symmetric flow (`flow[v,u] =
  -flow[u,v]`), so there is no `i ^ 1` reverse-index trick.  The spec is
  `is_flow` (capacity bound + skew symmetry + conservation at non-terminals) and
  `flow_value`, with `max_flow_is_flow` / `max_flow_value` /
  `max_flow_optimal` (`max_flow_optimal` is the max-flow ≤ min-cut direction).
  The `source == sink` case is handled by initialising the outer loop flag to
  `source != sink` rather than an early `return`.
- **Data structures** (Rat; all have `Test_test.lean`, `int` model): `ds_bst`
  (recursive BST — insert / find / delete / `extract_min`), `ds_max_heap` (array
  binary max-heap, push/pop-max), `ds_trie` (binary trie, insert/search),
  `ds_linked_list` (recursive cons list — push/pop/append/reverse/length),
  `ds_queue` (growable ring-buffer FIFO), `ds_deque` (growable ring-buffer
  double-ended queue), `ds_union_find` (disjoint-set — iterative path-compression
  find + union by size), `ds_fenwick` (binary-indexed tree — point update +
  prefix sum), `ds_segment_tree` (iterative range-sum + point update),
  `ds_seg_beats` (Segment Tree Beats / Ji-driver: range `chmin` with
  `second-max` lazy propagation + range max/sum queries, array-based recursive),
  `ds_sparse_table` (static RMQ), `ds_red_black_tree` (persistent Okasaki
  red-black tree via the double-black formulation: insert uses the classic
  4-case `balance`, delete threads a transient `DoubleBlack` color / 
  `DoubleBlackEmpty` leaf through `del` + `fix_db` and re-blackens the root;
  the tree type is a 3-variant enum `Empty | Node Color Tree Int Tree |
  DoubleBlackEmpty`), `ds_treap` (persistent treap: BST on `key` + max-heap on
  an explicit `i32` `priority`; `treap_merge`/`treap_split` are the structural
  primitives and insert/delete are expressed in terms of them; spec is
  `is_bst ∧ is_heap` plus merge/split/insert/delete/find correctness),
  `ds_avl` (persistent AVL tree: stores a `u64` subtree height, rebalances with
  single/double rotations so sibling heights differ by ≤ 1; uses
  `int-model = "nat"` so heights stay `Nat` and `1 + max(…)` avoids the
  `Int + Nat` literal pitfall; spec is `is_bst ∧ is_avl`), `ds_kdtree`
  (2-D KD-tree: insertion alternating the split dimension, per-node bounding
  box + subtree value sum, rectangle sum query with bounding-box pruning; spec
  is `bbox_ok` + query = brute-force `rectSum`), `ds_lct` (array-based Link-Cut
  Tree: splay + `access`/`makeroot`/`link`/`cut`/`findroot`/`path_sum` over a
  dynamic forest; spec states path-sum / findroot behavior on small trees).
- **Research-level data structures** (Rat; all have `Test_test.lean`, `int` model;
  the DS counterpart of the research-level graph batch, cf.
  `DS_RESEARCH_TESTCASES.md` — these target the *global* / *amortized* /
  *decomposition* arguments that the tree-heavy `ds_*` cases above do not):
  `ds_splay_tree` (self-adjusting BST; the classic bottom-up recursive `splay`
  applies zig / zig-zig / zig-zag rotations on the way back up, so a single
  access can leave the tree badly unbalanced — the invariant is *amortized*.
  Spec = `view` / `is_bst` are preserved, the accessed key ends up at the root
  (`h_mem : key ∈ view t` is required — for an absent key the splay only moves
  the last-accessed node up), and `splay_root_is_fixpoint` (re-splaying a
  rooted key is the identity — the structural fixed point that makes the
  case analysis well defined)), `ds_cartesian_tree` (O(n) monotonic-stack
  construction returning the parent array, with the sentinel `a.len()` for the
  root; spec = well-formedness (`IsTree`: one root, no cycles, `n-1` edges —
  with an explicit `0 < n` hypothesis, since the empty tree has no root),
  min-heap, "in-order = `0..n-1`" (`IsInOrder`), uniqueness under distinct
  values, and the centrepiece `cartesian_tree_rmq`: the value at the LCA of
  `l`,`r` is `rangeMin a l r`, i.e. the tree encodes *every* range-minimum
  query), `ds_binomial_heap` (flat-`Vec` pool + `roots` list; consolidation
  via a CLRS degree-bucket array; spec = `binom_tree_size`
  (`IsTree h v r → treeSize h v r = 2^r`, the structural "binomial" theorem),
  `binom_forest_size` (`∑_roots 2^degree = #keys`, the binary-counter
  invariant), insert/merge multiset laws, and extract-min returning the true
  multiset minimum while removing exactly one occurrence; the shared hypothesis
  `WellFormed` pins *all five* pool vectors to length `n+1` — a load-bearing
  clause, because `binom_insert`/`splice` `push` and assume the new node lands
  at logical index `n+1`), `ds_van_emde_boas` (genuinely recursive
  `u = 2^(2^m)` vEB — `Veb { u, half, empty, min, max, summary, clusters }`
  with the clusters as a cons list, so the spec recursion is structural and
  needs no fuel; spec = `veb_member`/`min`/`max`/`successor`/`insert`/`delete`
  against `viewC`, the spec-side value set that mirrors the implementation's
  conventions exactly (min is *not* stored recursively, the `u = 2` leaf keeps
  both endpoints locally, `summary` = indicator of the non-empty clusters) —
  `veb_insert` needs `x < u` (inserting out of range returns `ok` but corrupts
  the tree), while `veb_delete` needs no range hypothesis because an
  out-of-range delete is a no-op), `ds_fibonacci_heap` (flat arena
  (`key`/`degree`/`parent`/`child`/`next`/`marked` + `min` + `used`) with
  *circular* singly-linked root and child lists and CLRS degree-table
  consolidation; spec = the Fredman–Tarjan **`fib_heap_rank_bound`**
  (`Nat.fib (deg + 2) ≤ (subtree v).card`, the pure "global magnitude"
  induction), its global corollary `fib_heap_degree_bound`, insert/merge
  multiset laws, `fib_find_min` (a stored minimum, `none` iff empty) and
  `fib_extract_min` (one occurrence erased — `Multiset.erase` *is*
  single-occurrence removal in this toolchain, unlike `Finset.erase`).  The
  load-bearing hypothesis is `IsFibHeap`, whose *linking invariant* (children of
  a degree-`k` node have degrees exactly `k-1 … 0`) together with acyclicity and
  the parent/child link agreement is what makes the rank bound true — heap order
  alone does not (`Σ_children card (subtree c)` could exceed `card (subtree v)`);
  the two explicit capacity hypotheses (`used + 2 ≤ slots`) are the flat arena's
  side conditions.  No `decrease_key`, so `marked` is invariantly `0`).
  Recursive ADTs use `Option<Box<Node>>` (`Box` erased to a pure inductive in
  Lean); specs use a mutually-recursive `view` (set of stored elements/keys)
  plus a structural invariant (`is_bst` / `is_heap`), with the "set/multiset
  changes by exactly one element" shape (`view t ∪ {v}`, `view t \ {v}`,
  `view h + {v}`, `view h - {max}`).  The linear structures (`linked_list` /
  `queue` / `deque`) use a `toList` order-preserving view; `union_find` uses a
  fuel-bounded `root` + `connected_spec`; `fenwick` / `segment_tree` /
  `sparse_table` state range-sum / RMQ soundness (`query` = prefix sum, point
  update shifts exactly those prefixes ≥ i, `range_min` = the range minimum).
- **DP classics** (Rat; all have `Test_test.lean`): `dp_knapsack` (0/1,
  `nat`), `dp_coin_change` (`nat`), `dp_house_robber` (`nat`),
  `dp_maximum_subarray` (Kadane, `int`), `dp_longest_common_subsequence`
  (`nat`), `dp_jump_game` (`nat`).  Specs are the
  "optimality + attainability over a predicate" shape (`is_valid_strategy` /
  `is_valid_robbery` / `is_subsequence` / `is_valid_jump_path`), except LCS
  whose predicate recurses structurally (see §9).
- **DP hard tier** (Rat; all have `Test_test.lean`, `nat` model; the four remaining
  cases of `DP_HARD_TESTCASES.md`, whose specs are *global construction* statements
  rather than the local "optimality + attainability" shape): `dp_steiner_tree`
  (Dreyfus–Wagner subset DP `dp[S][v]` = merge over the splits of `S` + Bellman–Ford
  relaxation; spec = the minimum total weight over the edge sets connecting the
  terminal set, with `h_sym` making the weight matrix undirected), `dp_subset_convolution`
  (ranked zeta/Möbius SOS DP; spec = `c[S] = Σ_{T ⊆ S} a[T]·b[S \ T]` — the submask sum
  expressed with `Nat.land`, `Submask` declared `abbrev` for the `Finset.filter`), 
  `dp_digit_dp` (sweep over `(tight, rem)`; spec = `#{x ≤ decode n | no digit 4 ∧
  k ∣ digitsum x}`, with an explicit `h_dig : every digit < 10` and `h_k : 1 ≤ k`),
  `dp_count_hamiltonian` (inclusion–exclusion `Σ_{S} (−1)^{n−|S|} W(S)`, the two parities
  accumulated separately; spec = the number of injective vertex orders with adjacent
  consecutive pairs, carrying the `0/1` adjacency hypothesis so the walk counts are path
  counts — the `h_adj01` convention of §9).
- **String algorithms** (Rat; all have `Test_test.lean`, `nat` model):
  `string_z_algorithm` (Z-array), `string_kmp` (prefix function + first-occurrence
  search), `string_manacher` (odd-length palindromes), `string_rabin_karp`
  (rolling-hash search), `string_aho_corasick` (multi-pattern matching via trie +
  failure links), `string_lyndon` (Duval's Lyndon factorization — spec states the
  factorization covers `s`, every factor is a Lyndon word, factors are
  lexicographically non-increasing, and uniqueness), `string_suffix_automaton`
  (SAM — spec states the linear `≤ 2n + 1` state bound, substring recognition, and
  the distinct-substring count).  Characters are `u8` (→ `Nat`) or `usize`/`u64`
  integer-encoded (no `as` casts under `nat`; SAM encodes chars as `usize` so they
  index the transition table directly).
  The “global order / global equivalence class” batch added five more (all
  `usize`-encoded chars, flat `Vec`s, `nat` model, `Test_test.lean` present):
  `string_suffix_array` (prefix doubling with two stable counting sorts + Kasai LCP;
  spec = `sa` is a permutation of `0..n-1`, entries are in non-decreasing suffix
  order — `leSuffix` over `s.drop ·` — and `lcp[i]` is the longest common prefix of
  the two adjacent suffixes), `string_bwt_inverse` (BWT with a sentinel `0`; spec =
  `bwt_roundtrip` `bwt_decode (bwt_encode s) = s` plus a concrete
  `bwtReference s := lastColumn (sortedRotations (s ++ [0]))` characterisation),
  `string_lz77` (greedy non-overlapping LZ77; spec = `IsValidFactorization`
  (replay covers `s`, copies stay inside the history), `lz77_optimal`
  (factor count ≤ any valid factorization) and `lz77_maximal`), `string_min_rotation`
  (Booth over `s ++ s`; spec = the returned index is `< n` and its rotation is
  lexicographically minimal among all `n` rotations), `string_palindromic_tree`
  (eertree with shifted lengths `L(v) = actual + 1` so the odd root's `-1` needs no
  signed type; spec = `sz = #distinct palindromic substrings + 2`, the size bound,
  and `link[v]` = longest proper palindromic suffix).
  A second, research-level batch followed (same conventions, `nat` model, all with
  `Test_test.lean`): `string_runs` (O(n²) maximal periodic intervals — prefix-function
  per suffix, `p = L - pi[L-1]`, extend both ways, dedupe; spec = `IsRun`
  (minimal period + `≥ 2p` + both maximality directions), `runs_sound`,
  `runs_complete`, `runs_nodup` and **`runs_theorem`** `ρ(n) < n`),
  `string_two_way` (faithful Crochemore–Perrin: maximal-suffix scan +
  critical factorization + two-phase search with a `memory` variable; spec includes
  `critical_factorization_correct` — every word of length `≥ 2` has a critical
  factorization whose period is the minimal one), `string_sais` (genuine SA-IS with the
  recursive reduced-string call — recursion does translate; spec = the suffix-array
  perm/sorted/uniqueness theorems, plus `CharsPos` because 0 is the internal sentinel),
  `string_fm_index` (BWT + `c`/`occ` flat tables + backward search; the returned
  interval is **half-open** `[l, r)` so an empty result cannot underflow; spec =
  `fm_count_correct` (`r - l` = number of prefixed suffixes), `fm_interval_exact`
  (iff-characterisation) and `fm_empty_pattern`), `string_maw` (brute-force minimal
  absent words, flat `(data, lens)` output + `decodeFlat`; `IsMaw` carries `a ∈ s ∧ b ∈ s`
  so the alphabet of the notion is finite and the statement is true), and
  `string_ukkonen` (**O(n²) suffix trie** built by inserting every suffix, with suffix
  links computed BFS-style — the real Ukkonen active-point machine was judged
  untranslatable, which is what `STRING_HARD_TESTCASES.md` anticipated; spec =
  `isSubstring ↔ PathExists`, `sz = 1 + #distinct substrings`, the `n(n+1)/2 + 1` bound,
  and `link[v]` = longest proper suffix that still exists in the trie — note
  `link[v] < v` is **false** here, so it is deliberately not asserted).
- **`bitvec`** (Rat, unsigned → `BitVec w`, has `Test_test.lean`): `bitvec_ops`
  (rotate / wrapping arithmetic / bitwise / shifts / popcount), `crypto_aes128`
  (AES-128 encrypt+decrypt, table-driven, conformance + roundtrip), `crypto_des128`
  (DES single-block encrypt+decrypt, FIPS PUB 46-3), `crypto_tea`
  (TEA single-block encrypt+decrypt, 32-round Feistel, pure arithmetic).
- **Real (no `Test_test.lean`)**: `matrix_bidiagonal`, `matrix_cholesky`,
  `matrix_sparse_cholesky` (three-phase sparse Cholesky — CSR→CSC transpose,
  symbolic fill via the left-looking closure, numerical left-looking `sqrt`
  recurrence; spec = `csr_decode L · (csr_decode L)ᵀ = csr_decode A` with `L`
  lower-triangular and positive-diagonal),
  `matrix_hessenberg`, `matrix_qr`, `matrix_svd`, `matrix_symmetric_eigen`,
  `tensor_softmax_1d`, `tensor_softmax_2d`, `array_norm` (use `sqrt`/`exp`; each
  carries `float-model = "Real"` in its `Cargo.toml`).
- **`failure/`**: testcases the translator can't handle — `random_integer` and a stale
  `matrix_bidiagonal` copy (the `isize`+`continue` version, superseded by the working
  top-level `matrix_bidiagonal`) — excluded from `scripts/translate_all.sh` / `scripts/check_all.py`.
- **Avoid `sqrt` by reformulating**: LDLᵀ (`matrix_ldlt`, Rat) is the `sqrt`-free
  alternative to Cholesky (`matrix_cholesky`, Real) — prefer this when a `sqrt`-free
  reformulation exists.

---

## 11. When the translator can't translate — ask a human

The translator has hard limits (section 4, section 5, section 9). When you hit one, **do not silently paper over
it** (by leaving a `sorry` in the impl body, faking the result, or quietly dropping the
testcase). Instead:

1. **Diagnose**: is it (a) a translator limitation, (b) a Rust construct you can rewrite
   within the supported subset, or (c) a bug in the translator?
2. **Fix what's clearly in-scope**: `isize`→`usize`, drop `continue`, flatten nested `Vec`,
   remove `print!`, hand-expand `abs`/`signum`, hoist comparisons out of `if` conditions.
3. **Surface the real blockers and let a human decide.** Examples from history:
   - *Function pointers / higher-order inputs* (`function_diff`) → abandoned (unsupported).
   - *Random / non-determinism* → modeled with `opaque`/`axiom` (skips `#eval`) after
     discussion, or dropped.
   - *`sqrt` (Cholesky)* → reformulated to LDLᵀ to stay in Rat — the human's call.
   - *`isize` + `continue` (bidiagonal)* → moved to `failure/`.
   - *Extending the translator* (i64 adapter, Real auto-switch, loop flattening) — a product
     decision, not something to assume on your own.
