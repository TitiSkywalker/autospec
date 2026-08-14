# lean_mode

This project implements a benchmark for Rust verification using `Aeneas` translation pipeline. Each testcase is a Rust `cargo` project with the follwing structure:
- `src/lib.rs`: the implemented algorithm as a Rust function, without using external crates. 
```
.
├── MatrixMul.lean
├── Spec.lean
├── Test_test.lean
├── Cargo.lock
├── Cargo.toml
├── src
│   └── lib.rs
└── tests
    └── test.rs
```