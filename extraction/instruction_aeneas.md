# Aeneas Code Format

All extracted Rust code must comply with these rules. The code will be
translated to Lean via Charon + Aeneas; non-compliant patterns cause
translation failures.

## Types

1. **No generics.** Use concrete `f64`, not `T: Float`.
   ```rust
   // wrong
   fn foo<T: Float>(x: &[T]) -> T

   // correct
   fn foo(x: &[f64]) -> f64
   ```

2. **No `Vec` or `Vec<Vec>`.** Use flat slices.
   ```rust
   // wrong
   fn foo(m: Vec<Vec<f64>>) -> f64

   // correct
   fn foo(data: &[f64], nrows: usize, ncols: usize) -> f64
   ```

3. **No `vec!` macro.** Use fixed arrays or caller-provided storage.
   ```rust
   // wrong
   let v = vec![0.0; n];

   // correct
   let v = [0.0; 16];
   // or: accept &mut [f64] from caller
   ```

## Control flow

4. **No return/break/continue crossing nested loop boundaries.**
   ```rust
   // wrong
   for i in 0..n {
       for j in 0..m {
           return x;      // return inside nested loop
           break 'outer;   // labeled break from inner loop
       }
   }

   // correct: extract inner loop to helper function
   for i in 0..n {
       inner_helper(args);
   }
   ```

5. **No closures or iterator chains.** Replace with plain `for` loops.
   ```rust
   // wrong
   data.iter().flat_map(|x| ...).filter(|x| ...)

   // correct
   for x in &data { ... }
   ```

6. **No `if`-`else` that updates a mutable reference in both branches.**
   Aeneas cannot join the borrowed value from two paths.
   ```rust
   // wrong
   if cond {
       gauss_step_swap(data, ...);  // data updated
   } else {
       gauss_step(data, ...);       // data also updated
   }
   // data is now "bottom" (unknown which branch)

   // correct: restructure to avoid conditional mutation
   gauss_step(data, ...);  // unconditional
   ```

## main()

7. **Minimal `main`.** No f64 array constants.
   ```rust
   // correct
   fn main() {}
   ```
