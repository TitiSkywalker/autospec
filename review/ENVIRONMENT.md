# Running the translator and the Lean environment

How to translate a Rust testcase into Lean and build the result. The deep details — the Rust style subset AENEAS can translate, the Rat/Real float model, the int/nat/bitvec models, how to write a `Spec.lean`, and the long list of hard-won pitfalls — live in [`lean_mode/TESTCASE_GUIDE.md`](../lean_mode/TESTCASE_GUIDE.md). Read that before touching a testcase; this file is the quick reference.

## Layout of one testcase

```
lean_mode/LeanMode/<name>/
├── Cargo.toml          # crate name = <name>; holds [package.metadata.autospec]
├── src/lib.rs          # algorithm implementation (AENEAS-compatible subset)
├── tests/test.rs       # plain Rust tests (translated to #eval when computable)
├── <Crate>.lean        # auto-generated translation of lib.rs
├── Test_test.lean      # auto-generated translation of tests/test.rs + #eval (Rat only)
└── Spec.lean           # hand-written specification (theorem ... := by sorry)
```

`<Crate>.lean` is the CamelCase of the directory name (`matrix_qr` → `MatrixQr.lean`). `Spec.lean` is the only hand-written file; the other two are produced by the translator and must never be edited by hand.

## Translating (Rust → Lean)

The translator is a Rust binary (`translation/translator`). Build it once:

```bash
cd translation/translator && cargo build --release
```

Single testcase:

```bash
translation/translator/target/release/translator lean_mode/LeanMode/<name>
```

Batch (all testcases, concurrent):

```bash
scripts/translate_all.sh            # 8 workers by default
scripts/translate_all.sh -w 16      # 16 workers
```

Per-testcase options are read from `Cargo.toml` (no CLI flags needed):

```toml
[package.metadata.autospec]
int-model = "nat"      # int (default) | nat | bitvec
float-model = "Real"   # Rat (default) | Real (needed for sqrt/transcendentals)
```

The translator auto-switches `float-model` to `Real` when the code uses `sqrt`/`exp`/… and then skips translating `tests/test.rs` (no `Test_test.lean`). See `TESTCASE_GUIDE.md` §4–5 for the full rules.

## Building Lean (the environment)

The Lean project is `lean_mode/` (a Lake project):

- Toolchain: `leanprover/lean4:v4.31.0` (see `lean_mode/lean-toolchain`).
- `lean_mode/lakefile.toml` requires the `aeneas` Lean backend at `../aeneas/backends/lean` (already wired via `[[require]] path = …`).
- `lake`/`lean` come from elan (`~/.elan/bin`).

```bash
cd lean_mode
lake build LeanMode.<name>.<Crate>   # impl translation compiles
lake build LeanMode.<name>.Spec      # spec compiles
```

`lake build` must be run from the `lean_mode/` directory. `Test_test.lean` files (and their `#eval` results) are checked by the regression gate:

```bash
python3 scripts/check_all.py         # lake build every .lean + check #eval results
```

Rust-side correctness is checked with:

```bash
cd lean_mode/LeanMode/<name> && cargo test
```

## Full loop for a new testcase

1. Write `src/lib.rs` / `tests/test.rs` (respect the subset in `TESTCASE_GUIDE.md` §4).
2. `cd lean_mode/LeanMode/<name> && cargo test` — the Rust is correct.
3. `translation/translator/target/release/translator lean_mode/LeanMode/<name>`.
4. Hand-write `Spec.lean` (`TESTCASE_GUIDE.md` §6).
5. `cd lean_mode && lake build LeanMode.<name>.Spec`.
6. `python3 scripts/check_all.py` for the full regression.
