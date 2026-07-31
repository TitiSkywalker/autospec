# autospec — Worker Agent

Extract ONE specific method into a standalone pure function.

## Rules

1. Never change the algorithm body — the logic must stay identical.
2. **Remove all `unsafe`.** Replace `get_unchecked` with `[]`, raw pointer access with slice indexing. Use `copy_from_slice` instead of `ptr::copy_nonoverlapping`.
3. No wrappers — delete the original method from the impl block.
3. **Self-contained output** — the extracted fn must compile with zero dependencies beyond `std`. Recursively decompose ALL types:
   - Crate-internal structs → open their definition, flatten all fields
   - `Vec<T>` → `&[T]` / `&mut [T]`
   - `Vec<Vec<T>>` → flatten to `&mut [T]` + `nrows` + `ncols`
   - Scalars → direct params
   - Arrays → `[T; N]` or flatten individual elements
   - **Never use `vec!`** (Aeneas cannot translate it). Use `[0.0; N]` or pre-allocated slices.
   - ONLY skip when you hit a type from an **external crate**
4. **Open struct definitions.** Before extracting, read the struct.
   Map every field to a parameter:
   `struct Matrix { data: Vec<f64>, nrows: usize, ncols: usize }`
   → `data: &[f64], nrows: usize, ncols: usize`
5. Skip trait impls, getters, setters, constructors.
6. Preserve doc comments. Naming: `<struct_snake>_<method_name>`.
7. **One function per crate** — write to `<output_dir>/<function_name>/`.
8. **Write at least one basic test.** Add a test that calls the function   and checks the output is reasonable (smoke test). A known-answer test is   preferred if you have reference data, but a smoke test is enough. Reuse   existing test vectors from the original code if available.

## Workflow

1. read_file the target file. Open struct definitions and flatten all fields.
2. Extract the method as a `pub fn` with std-only params. Remove all `unsafe`.
3. Write the output to the provided output directory:

```
<output_dir>/<function_name>/src/main.rs        ← pub fn + doc + fn main() {}
<output_dir>/<function_name>/tests/test_<fn>.rs ← at least 1 smoke test (Rule 8)
<output_dir>/<function_name>/Cargo.toml          ← [package] name="..."
                                                     version="0.1.0"
                                                     edition="2021"
```

   The `tests/test_<fn>.rs` file MUST contain at least 1 test (see Rule 8).

4. Run `cargo test` in the output directory to verify all tests pass.

Then report: "Extracted: <function_signature> (N tests pass)".

## Aeneas Compatibility (CRITICAL)

The extracted code will be translated to Lean via Charon + Aeneas.
Follow these rules.

1. **No generics.** Use concrete types: `fn foo(x: &[f64]) -> f64`, not
   `fn foo<T: Float>(x: &[T]) -> T`.

2. **No `Vec` or `Vec<Vec>`.** Use flat slices with dimensions:
   `fn foo(data: &mut [f64], nrows: usize, ncols: usize)`.

3. **No `vec!` macro.** Use fixed arrays `[0.0; N]` or accept a pre-allocated slice from the caller.

4. **No control flow crossing nested loop boundaries.** `return`, `break 'outer`,
   `continue 'outer` inside a nested loop are banned. Extract the inner loop
   into a helper function instead.

5. **No closures or iterator chains.** Replace `.flat_map()`, `.filter()`,
   `.map()` with plain `for` loops.

6. **Minimal `main`.** Use `fn main() {}` with no f64 array constants.
