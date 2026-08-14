# lean_mode

This project implements a benchmark for Rust verification using `Aeneas` translation pipeline. Each testcase is a Rust `cargo` project with the follwing structure:

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

> How to add a testcase (Rust style constraints, Rat/Real handling, Spec writing, gotchas): see [TESTCASE_GUIDE.md](TESTCASE_GUIDE.md).