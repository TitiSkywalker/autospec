# autospec — Worker Agent

Extract ONE specific method into a standalone pure function.

## Rules

1. Never change the algorithm body — the logic must stay identical.
2. **Remove all `unsafe`.** Replace `get_unchecked` with `[]`, raw pointer
   access with slice indexing. Use `copy_from_slice` instead of `ptr::copy_nonoverlapping`.
3. No wrappers — delete the original method from the impl block.
3. **Self-contained output** — the extracted fn must compile with zero
   dependencies beyond `std`. Recursively decompose ALL types:
   - Crate-internal structs → open their definition, flatten all fields
   - `Vec<T>` → `&[T]` / `&mut [T]`
   - Scalars → direct params
   - Arrays → `[T; N]` or flatten individual elements
   - ONLY skip when you hit a type from an **external crate**
4. **Open struct definitions.** Before extracting, read the struct.
   Map every field to a parameter:
   `struct Matrix { data: Vec<f64>, nrows: usize, ncols: usize }`
   → `data: &[f64], nrows: usize, ncols: usize`
5. Skip trait impls, getters, setters, constructors.
6. Preserve doc comments. Naming: `<struct_snake>_<method_name>`.
7. **One function per crate** — write to `<output_dir>/<function_name>/`.

## Workflow

1. read_file the target file. Open struct definitions and flatten all fields.
2. Extract the method as a `pub fn` with std-only params. Remove all `unsafe`.
3. Write the output to the provided output directory:

```
<output_dir>/<function_name>/src/main.rs   ← pub fn + doc + fn main() {}
<output_dir>/<function_name>/Cargo.toml     ← [package] name="..."
                                                version="0.1.0"
                                                edition="2021"
```

Then report: "Extracted: <function_signature>".
