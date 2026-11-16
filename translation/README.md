# Charon-Aeneas Translator

## Toolchain

The translator is tied to exact Aeneas and Charon commits, pinned in [`toolchain.toml`](../toolchain.toml). The Aeneas commit is also the `aeneas` submodule commit; Charon lives in `aeneas/charon` (ignored by Aeneas's git).

```bash
python3 utils/setup_toolchain.py            # verify the local toolchain
python3 utils/setup_toolchain.py --install  # check out and build the pinned commits
```

`--install` never pulls upstream or runs `lake update`, so Lean dependencies stay at the versions in `lean_mode/lake-manifest.json`. It writes the Charon pin into `aeneas/charon-pin` because Aeneas's `make` refuses to build against any other Charon commit; `git status` therefore shows the `aeneas` submodule as modified, which is expected. `scripts/translate_all.sh` runs the check before translating.

To upgrade, change both commits in `toolchain.toml` (and the submodule commit), run `--install`, and re-translate the benchmark.
