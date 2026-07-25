# autospec — De-OOP Rust Code into Self-Contained Pure Functions

## Core Principle

Eliminate OOP. Extract algorithmic computation from `impl` blocks into standalone
`pub fn` that operate on `Vec<T>` / `&[T]` / primitives only. No structs, no
method calls, no wrappers. Every extracted function must be copy-paste compilable
in an empty Rust file with zero external dependencies (std only).

## Rules (must follow, no exceptions)

1. **Never change test assertions.**
2. **No wrappers.** Delete the original struct method, do not keep a delegating stub.
3. **Self-contained output.** Each extracted function must compile with `rustc` alone
   — no `use crate::`, no external crate types, no external macros. If a dependency
   cannot be resolved to std, skip that function.
4. **Resolve dependencies recursively.** If the extracted function calls another
   method, extract that too. Repeat until the call chain is pure std.
5. **Vec-based data.** Struct fields become function parameters. `Vec<T>` fields →
   `&[T]` or `&mut [T]` params. Scalar fields → direct params. Delete the struct
   if nothing else references it.
6. **Skip trait impls.** `impl Add for Foo`, `impl Display for Foo` etc. are off-limits.
7. **Preserve comments.** Attach original doc comments to the new function.
8. **Do not refactor the algorithm body.** The logic must stay identical.
9. **Skip trivial methods.** Getters, setters, constructors — delete without replacement.

## Transformation Pattern

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

Naming: `<struct_snake>_<method_name>` (e.g. `Matrix::transpose` → `matrix_transpose`).

## Workflow

1. Grep for files with `#[cfg(test)]` or `#[test]`.
2. Read each such file. Identify structs → methods with computation.
3. Extract eligible methods as pure functions, resolving dependencies recursively.
4. Adapt tests: `Struct::new(a, b)` → raw `Vec` / primitives; `m.method(args)` →
   `function(data, dims, args)`.
5. `cargo check` → fix compile errors → `cargo test` → fix failures → repeat
   until `test result: ok.`
6. Move to the next file.

## Important

Do NOT stop after processing one file. Keep going until every test-containing
file in the repo is done. Only declare completion when `cargo test` passes
for the entire project and every test file has been processed.
