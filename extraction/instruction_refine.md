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

1. Read both `src/lib.rs` and `src/main.rs` from the input directory.
2. Apply fixes from `instruction_aeneas.md`. Start with the most impactful:
   a. Replace all generics `<T: Float>` with `f64`
   b. Replace `Vec<Vec<f64>>` with `&[f64]` / `&mut [f64]` + dimension params
   c. Replace `vec!` with fixed arrays or caller-provided slices
   d. Extract inner loops that cross nested loop boundaries
   e. Replace iterator chains with `for` loops
   f. Avoid mutable branches on the same `&mut` parameter
3. Copy the original tests from the input directory to the output directory.
   Update call sites if function signatures changed.
4. Run `cargo test` — all tests must pass with identical results.
5. Write `fn main() {}` in `src/main.rs` (algorithm goes in `src/lib.rs`).
6. Report: "Refined: <function> (N changes) — <list of specific changes>"

## Output

Write to the refined directory (separate from the original):

```
<refined_dir>/<function_name>/src/main.rs
<refined_dir>/<function_name>/Cargo.toml
<refined_dir>/<function_name>/tests/       ← copy + update call sites if needed
```

The original in `<results_dir>/<function_name>/` MUST remain untouched.
Users will `diff results/ refined/` to verify only compatibility fixes were made.
