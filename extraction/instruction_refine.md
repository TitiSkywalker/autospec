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

1. Read the original extracted code from the input directory.
2. Apply aeneas compatibility fixes:
   - Replace generics with `f64`: `T: Float` → `f64`
   - Replace `Vec<Vec<T>>` with `&mut [T]` + `nrows` + `ncols`
   - Replace `vec!` with fixed arrays `[0.0; N]`
   - Replace closures / iterator chains with `for` loops
   - Extract inner loops that contain `return`/`break`/`continue`
   - Replace `main()` body with `fn main() {}`
   - Avoid `if`-`else` that mutates a borrowed value in both branches
3. Run `cargo test` in the output directory — tests must pass with identical
   results (copy the original tests, adjusting call sites if signatures changed).
4. Report: "Refined: <function> (N changes) — <list of specific changes>"

## Output

Write to the refined directory (separate from the original):

```
<refined_dir>/<function_name>/src/main.rs
<refined_dir>/<function_name>/Cargo.toml
<refined_dir>/<function_name>/tests/       ← copy + update call sites if needed
```

The original in `<results_dir>/<function_name>/` MUST remain untouched.
Users will `diff results/ refined/` to verify only compatibility fixes were made.
