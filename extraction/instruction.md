# autospec — Worker Agent

Extract ONE specific method into a standalone pure function.

## Rules

1. Never change the algorithm body — the logic must stay identical.
2. **Remove all `unsafe`.** Replace `get_unchecked` with `[]`, raw pointer
   access with slice indexing. Use `copy_from_slice` instead of
   `ptr::copy_nonoverlapping`.
3. No wrappers — delete the original method from the impl block.
4. **Self-contained output** — the extracted fn must compile with zero
   dependencies beyond `std`. Recursively decompose ALL types:
   - Crate-internal structs → open their definition, flatten all fields
   - `Vec<T>` → `&[T]` / `&mut [T]`
   - `Vec<Vec<T>>` → flatten to `&mut [T]` + `nrows` + `ncols`
   - Scalars → direct params
   - Arrays → `[T; N]` or flatten individual elements
   - ONLY skip when you hit a type from an **external crate**
5. **Open struct definitions.** Before extracting, read the struct.
   Map every field to a parameter:
   `struct Matrix { data: Vec<f64>, nrows: usize, ncols: usize }`
   → `data: &[f64], nrows: usize, ncols: usize`
6. Skip trait impls, getters, setters, constructors.
7. Preserve doc comments. Naming: `<struct_snake>_<method_name>`.
8. **Follow instruction_aeneas.md** for all Rust code you generate. The code
   will be translated to Lean via Charon + Aeneas; non-compliant patterns
   will cause translation failures.
9. **Write at least one basic test.** A smoke test that calls the function
   and checks the output is reasonable. Reuse existing test vectors if available.

## Workflow

1. read_file the target file. Open struct definitions and flatten all fields.
2. Extract the method as a `pub fn` with std-only params. Remove all `unsafe`.
3. Write the output to the provided output directory:

```
<output_dir>/<function_name>/src/main.rs        ← pub fn + doc + fn main() {}
<output_dir>/<function_name>/tests/test_<fn>.rs ← at least 1 smoke test (Rule 9)
<output_dir>/<function_name>/Cargo.toml          ← [package] name="..."
                                                     version="0.1.0"
                                                     edition="2021"
```

4. Run `cargo test` in the output directory to verify all tests pass.

Then report: "Extracted: <function_signature> (N tests pass)".

After extraction, a refine agent will fix aeneas-incompatible patterns
(see instruction_refine.md) and write to a separate output directory.
Do NOT apply aeneas fixes yourself — keep the original algorithm intact.
