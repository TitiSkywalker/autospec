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
python3 translation/translate.py lean_mode/LeanMode/<name>

# 4. Hand-write Spec.lean (see section 6)

# 5. Verify the Lean compiles
cd lean_mode && lake build LeanMode.<name>.<Crate>
lake build LeanMode.<name>.Spec
```

Batch commands:
- `python3 scripts/translate_all.py` — re-translate every testcase (skips `failure/`).
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

- **Default `--target Rat`**: `f64 → Rat` (computable, `#eval` works).
- **When the code uses `sqrt / exp / sin` etc.**, `lean_f64_to_rat` raises
  `NonComputableError`, and `translate.py` **auto-switches to `--target Real`**:
  - `f64 → ℝ`, `sqrt` becomes `Real.sqrt`.
  - The generated file gains `import Mathlib` and `noncomputable section` at the top.
- **Under the Real target, `tests/test.rs` is NOT translated** (`translate.py` skips it,
  because `Real.sqrt` cannot be `#eval`'d). So sqrt-based testcases have only
  `<Crate>.lean` + `Spec.lean` — **no `Test_test.lean`** (cf. `vector_norm`,
  `tensor_softmax_1d`, `matrix_qr`).

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
5. **Encode preconditions with `assert!`, not `Result`/empty output.** If the theorem only
   holds under some condition (e.g. `det ≠ 0` for invertibility), `assert!` in Rust so that
   an `ok` result *implies* the condition. Then the Spec can state it unconditionally on the
   `ok` result, instead of carrying the condition as a separate hypothesis. This is the
   `matrix_inverse`/`matrix_inverse_2x2` pattern.

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
cd ../../.. && python3 translation/translate.py lean_mode/LeanMode/<name>
cd lean_mode && lake build LeanMode.<name>.<Crate>    # impl translation compiles
lake build LeanMode.<name>.Spec                       # spec compiles
python3 ../scripts/check_all.py                       # (optional) full regression
```

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
- **Function pointers / closures as function inputs are unsupported** (e.g. a `diff(f, x)`
  taking a function `f`) — such testcases were abandoned.
- **Nested `for` loops** can leave `sorry` bodies in AENEAS; the pipeline auto-retries with
  loop flattening, but flat `while` loops are safest.
- **`f64` comparison inside an `if` condition** can trigger binop errors — hoist it:
  `let cond = a > b; if cond { ... }`.
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

---

## 10. Implemented testcases by target

- **Rat (has `Test_test.lean`)**: `matrix_lu`, `matrix_ldlt`, `matrix_det`,
  `vector_gram_schmidt`, `graph_*`, `array_*`, … (pure arithmetic).
- **Real (no `Test_test.lean`)**: `vector_norm`, `tensor_softmax_1d`, `tensor_softmax_2d`,
  `matrix_qr` (use `sqrt`/`exp`).
- **`failure/`**: testcases the translator can't handle (e.g. `matrix_bidiagonal`) — excluded
  from `translate_all.py` / `check_all.py`.
- **Avoid `sqrt` by reformulating**: Cholesky was switched to LDLᵀ (no `sqrt`) so it stays
  in the computable Rat target — prefer this when possible.

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
