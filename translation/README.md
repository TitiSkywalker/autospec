# Translator

The translator turns a Rust crate into Lean 4 code that you can prove theorems about. It builds on [Charon](https://github.com/AeneasVerif/charon) and [Aeneas](https://github.com/AeneasVerif/aeneas) and extends them in two ways: it represents machine numbers with mathematical types, which makes floating-point code translatable, and it simplifies the generated code so that Mathlib can reason about it.

Before you start, install the toolchain as described in the [main README](../README.md).

## Usage

Run these commands from the repository root:

```bash
(cd translation/translator && cargo build --release)
translation/translator/target/release/translator path/to/my_crate   # one crate
scripts/translate_all.sh                                            # the whole benchmark
```

For each crate, the translator writes two files next to its Cargo.toml. MyCrate.lean is the translation of src/lib.rs. Test_test.lean is the translation of tests/test.rs, where each test becomes an #eval, so you can check that the Lean code gives the same results as the Rust code. Your source files are never modified.

## Numeric models

You can choose how numbers are represented, separately for each crate, in its Cargo.toml:

```toml
[package.metadata.autospec]
int-model = "nat"      # int (default), nat or bitvec
float-model = "Real"   # Rat (default) or Real
```

Floating-point numbers become rationals by default. If the code calls a function such as sqrt that has no rational equivalent, they become real numbers instead. Lean cannot evaluate real-number code, so the tests of such a crate are not translated.

Signed integers always become Int. Unsigned integers become Int by default, Nat with the nat model, or BitVec with the bitvec model. BitVec keeps the wrap-around behaviour of machine integers, which suits code that relies on overflow, such as hashing or cryptography. usize is never converted, because Aeneas relies on it for array bounds checks.

Aeneas supports only a subset of Rust, so code translates best when it is written in a plain style. Avoid traits, closures and iterator chains, and prefer while loops, slices, Vec and simple structs. The benchmark cases in [lean_mode/LeanMode](../lean_mode/LeanMode/) follow this style and are good starting points.

## Toolchain

The translator depends on specific versions of Aeneas and Charon, which are pinned in [toolchain.toml](../toolchain.toml).

```bash
python3 utils/setup_toolchain.py             # check that your toolchain matches the pinned versions
python3 utils/setup_toolchain.py --install   # check out and build the pinned versions
```

The install step records the pinned Charon version in aeneas/charon-pin, so git reports the aeneas submodule as modified. This is expected.
