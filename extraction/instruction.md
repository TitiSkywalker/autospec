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
9. **Write non-trivial tests.** Tests must verify correctness, not just "it runs".
   - **Known-answer test**: expected output from a trusted source (specification,
     reference implementation, or hand-computed). For hashing: published test
     vector (RFC, NIST). For linear algebra: verify invariants (e.g. P*A = L*U).
     For graph algorithms: hand-computed shortest path / MST weight.
   - **Property test**: verify a mathematical invariant (e.g. `abs(x) >= 0`,
     `L * U == original`, `roundtrip(input) == input`).
   - **Edge case**: empty input, identity matrix, single element, etc.
   Tests that only check "output changed" are NOT acceptable.
10. **Library structure.** Put algorithm code in `src/lib.rs` as `pub fn`.
    Keep `src/main.rs` minimal — just `fn main() {}`. This allows tests in
    `tests/` (integration tests) and Charon translates the library target.

## Workflow

1. read_file the target file. Open struct definitions and flatten all fields.
2. Extract the method as a `pub fn` with std-only params. Remove all `unsafe`.
3. Write the output to the provided output directory:

```
<output_dir>/src/lib.rs     ← ALL algorithm code (pub fn ...)
<output_dir>/src/main.rs    ← fn main() {} ONLY (empty, no logic)
<output_dir>/tests/         ← integration tests (can use crate::*)
<output_dir>/Cargo.toml     ← [package] name="..."
                                  version="0.1.0"
                                  edition="2021"
```

Integration tests go in `tests/`:
```rust
// tests/test_<fn>.rs
use <crate_name>::*;

#[test]
fn test_known_answer() { ... }
#[test]
fn test_property() { ... }
#[test]
fn test_edge_case() { ... }
```

4. Run `cargo build` — zero errors.
5. Run `cargo test` — all tests pass.

Then report: "Extracted: <function_signature> (N tests pass)".

After extraction, a refine agent will fix aeneas-incompatible patterns
(see instruction_refine.md) and write to a separate output directory.
Do NOT apply aeneas fixes yourself — keep the original algorithm intact.
