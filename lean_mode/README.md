# lean_mode

This project implements a benchmark for Rust verification using the `Aeneas` translation pipeline. Each testcase is a Rust `cargo` project with the following structure:

```
.
├── MatrixMul.lean      # generated: Aeneas translation of src/lib.rs
├── Spec.lean           # hand-written specification (theorems stated with `sorry`)
├── Test_test.lean      # generated: translation of tests/test.rs as `#eval`s
├── Cargo.lock
├── Cargo.toml          # [package.metadata.autospec] selects int-model / float-model
├── src
│   └── lib.rs
└── tests
    └── test.rs
```

Re-translate every case with `scripts/translate_all.sh`, and build-check all of them with `python3 scripts/check_all.py`.
