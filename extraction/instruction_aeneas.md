# Aeneas Code Format

All Rust code for aeneas translation must follow these rules.

## 1. No generics

Replace any generic parameter `T` with concrete `f64`.

```
fn gauss_step<T: Copy + num_traits::Float>(data: &mut [T], diag: T, ...)
// becomes:
fn gauss_step(data: &mut [f64], diag: f64, ...)
```

Also replace:
- `T::one()` → `1.0`
- `T::zero()` → `0.0`
- `.is_zero()` → `== 0.0`
- `num_traits::Float` import → remove it

## 2. No Vec or Vec<Vec>

```
fn foo(matrix: Vec<Vec<f64>>) -> f64
// becomes:
fn foo(data: &[f64], nrows: usize, ncols: usize) -> f64
```

For `&mut` access, use `&mut [f64]`:
```
fn foo(matrix: &mut Vec<Vec<f64>>)
// becomes:
fn foo(data: &mut [f64], nrows: usize, ncols: usize)
```

Replace indexing `matrix[i][j]` with `data[i * ncols + j]`.

## 3. No vec! macro

```
let v = vec![0.0; n];
// use fixed array:
let v = [0.0; 16];
// or accept caller-provided storage:
fn foo(data: &mut [f64])  // caller allocates
```

## 4. No return/break/continue in nested loops

If `return`, `break`, or `continue` appears inside a nested loop—or if
aeneas reports "Returns inside of nested loops" / "Continue to outer loops"
/ "Breaks to outer loops"—extract the inner loop into a helper function:

```
// before
for i in 0..n {
    for j in 0..m {
        if cond { return x; }  // return crosses nested boundary
    }
}

// after
fn inner_loop(args) -> Option<f64> {
    for j in 0..m {
        if cond { return Some(x); }
    }
    None
}
for i in 0..n {
    if let Some(x) = inner_loop(args) { return x; }
}
```

## 5. No closures or iterator chains

```
data.iter().flat_map(|x| x.iter()).filter(|x| ...)
// becomes:
for row in &data { for &val in row.iter() { if ... { ... } } }
```

## 6. No mutable branches

If both branches of an `if` update the same `&mut` parameter,
aeneas cannot merge the results:

```
// wrong
if cond {
    update_a(data);  // data mutated
} else {
    update_b(data);  // data also mutated
}

// correct: restructure so only one path mutates
update_a(data);
if cond {
    fixup(data);  // only fixes when cond true
}
```

## 7. Minimal main

```
fn main() {}
```

No test arrays, no `vec!`, no f64 constants in `main`. `main.rs` should
contain ONLY `fn main() {}`. Put ALL algorithm code in `src/lib.rs` and
tests in `tests/` — they will NOT be translated by aeneas.

## 8. Library structure

Put ALL algorithm code in `src/lib.rs` as `pub fn`. Keep `src/main.rs` as
`fn main() {}` only. Tests go in `tests/` as integration tests. Charon
translates the library target (`--lib`); code in `main.rs` is excluded.
