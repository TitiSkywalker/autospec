# autospec — De-OOP Rust Code into Self-Contained Pure Functions

## Core Principle

Eliminate OOP. Extract algorithmic computation from `impl` blocks into standalone
`pub fn` that operate on `Vec<T>` / `&[T]` / `[T; N]` / primitives only.
No structs, no method calls, no wrappers.

**Crate-internal types must be recursively decomposed** — open the struct, flatten
every field until only std types remain. Only types from external crates are
unresolvable. Every extracted function must compilable in an empty Rust file
with zero dependencies beyond `std`.

## Overall Strategy (must follow)

You are the orchestrator. Do NOT try to process every file yourself.

1. **Plan:** run `cargo check`, then use grep to find every `.rs` file that
   contains `#[cfg(test)]` or `#[test]`. Create a todo list with one entry per
   such file.

2. **Delegate:** for EACH file in the todo list, call the `task` tool:

   ```
   task(
     agent="deoop",
     task="<Sub-agent Instructions below, with the filename filled in>"
   )
   ```

   Call `task` for ALL files before waiting for results — they can run in
   parallel.

3. **Monitor:** after all sub-agents finish, run `cargo test` one final time
   to confirm everything passes.

4. **Collect:** gather every extracted pure function and save them to the
   output directory as described below.

## Sub-agent Instructions

When spawning a sub-agent for a specific file, give it these instructions:

---
Extract pure functions from this ONE file, adapt its tests, and verify
with `cargo test`.

Rules:
1. Never change test assertions.
2. No wrappers — delete the original struct method.
3. **Self-contained output** — each extracted fn must compile with zero
   dependencies beyond `std`. To achieve this, recursively decompose ALL types:
   - Crate-internal structs → open their definition, flatten all fields
   - `Vec<T>` → `&[T]` / `&mut [T]`
   - Scalars → direct params
   - Arrays → `[T; N]` or flatten individual elements
   - Tuple structs → flatten each element
   - ONLY skip when you hit a type from an **external crate** (e.g. `num::Complex`)
4. **Open struct definitions.** Before extracting, read the struct definition.
   Map every field to a parameter. Example:
   `struct Matrix { data: Vec<f64>, nrows: usize, ncols: usize }`
   → `data: &[f64], nrows: usize, ncols: usize`
5. Skip trait impls, getters, setters, constructors.
6. Preserve comments. Do not refactor the algorithm body.
7. Naming: `<struct_snake>_<method_name>`.
8. **One function per crate.** Each extracted `pub fn` must live in its own
   standalone crate under `<output_dir>/<function_name>/`. If a file has N
   methods, you create N crates. Never batch multiple functions into one crate.

```
BEFORE:                                 AFTER:
impl Matrix {                           pub fn matrix_transpose(
    pub fn transpose(&self) -> Matrix {     data: &[f64],
        ...algorithm...                     rows: usize,
    }                                       cols: usize,
}                                       ) -> Vec<f64> {
                                            ...identical algorithm...
                                        }
```

Workflow:
1. read_file the target file.
2. Extract eligible methods as pure functions.
3. Adapt tests: `Struct::new(a,b)` → raw Vec/primitives;
   `m.method(args)` → `function(data, dims, args)`.
4. `cargo check` → fix errors → `cargo test` → fix failures → repeat
   until `test result: ok.`
5. Report back: how many functions extracted, any functions skipped (and why).
---

## Output — Save Extracted Functions

After ALL sub-agents finish and `cargo test` passes, create the output
directory provided at the end of this prompt. Write exactly ONE Cargo crate
per extracted function:

```
<output_dir>/
├── matrix_transpose/
│   ├── Cargo.toml
│   └── src/main.rs
├── matrix_multiply/
│   ├── Cargo.toml
│   └── src/main.rs
└── manifest.txt
```

**`Cargo.toml`:**
```toml
[package]
name = "matrix_transpose"
version = "0.1.0"
edition = "2021"
```

**`src/main.rs`:**
- The `pub fn` with its doc comment
- A minimal `fn main() {}` at the bottom
- Zero dependencies beyond std

**`manifest.txt`** — one line per function:
```
matrix_transpose(data: &[f64], rows: usize, cols: usize) -> Vec<f64>
...
```

Only declare completion when `cargo test` passes and the output directory
is fully populated.
