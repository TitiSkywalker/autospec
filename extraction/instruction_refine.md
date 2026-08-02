# autospec — Refine Agent

Read extracted Rust code and fix only the patterns that block aeneas
translation. Do NOT change the algorithm — the output must produce identical
results to the original.

## Rules

1. **Never change the algorithm logic.** Only fix Rust patterns that aeneas
   rejects. If a function computes `a * b + c`, it must still compute exactly
   `a * b + c`.
2. **Read `instruction_aeneas.md`** — it defines all required code patterns.
3. **Write to a SEPARATE output directory** — never modify the original
   extracted code. This allows diff comparison to verify correctness.

## Workflow

1. Read `src/lib.rs` from the input directory. If the original has `src/main.rs`
   with non-trivial content, move only the `fn main() {}` stub there.
2. Apply fixes from `instruction_aeneas.md`. Start with the most impactful:
   a. Replace all generics `<T: Float>` with `f64`
   b. Replace `Vec<Vec<f64>>` with `&[f64]` / `&mut [f64]` + dimension params
   c. Replace `vec!` with fixed arrays or caller-provided slices
   d. Extract inner loops that cross nested loop boundaries
   e. Replace iterator chains with `for` loops
   f. Avoid mutable branches on the same `&mut` parameter
3. Copy the original `tests/` directory to the output directory.
   Update call sites if function signatures changed.
4. **Add known-answer tests.** Smoke tests (just checking output changed) are NOT enough. Find official test vectors from the algorithm's specification (RFC, NIST, paper) or compute expected results from a trusted implementation. At least one test MUST verify the output against a known correct answer.
5. Run `cargo build` — must compile with zero errors.
6. Run `cargo test` — all tests must pass.
7. Put ALL algorithm code in `src/lib.rs` as `pub fn`. Keep `src/main.rs` as
   `fn main() {}` only. Tests go in `tests/`.
8. Report: "Refined: <function> (N changes, K tests) — <list of specific changes>"

## Output

Write to the refined directory (separate from the original):

```
<refined_dir>/<function_name>/src/lib.rs     ← ALL algorithm code (pub fn)
<refined_dir>/<function_name>/src/main.rs    ← fn main() {} ONLY
<refined_dir>/<function_name>/tests/         ← integration tests
<refined_dir>/<function_name>/Cargo.toml
```

The original in `<extract_dir>/<function_name>/` MUST remain untouched.
Users will `diff <extract_dir>/ <refined_dir>/` to verify correctness.
