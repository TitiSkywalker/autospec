#!/usr/bin/env python3
"""Print a concise overview of the LeanMode benchmark suite.

For each test case it reports the integer model and the float model (both
declared in Cargo.toml ``[package.metadata.autospec].int-model`` /
``.float-model``), plus a category derived from the directory-name prefix
(``integer_*``, ``matrix_*``, ...). The float model is inferred from whether
``Test_test.lean`` was generated — Real testcases use sqrt/transcendentals and
have no computable tests.
"""

import argparse
import tomllib
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN_MODE = ROOT / "lean_mode" / "LeanMode"


def int_model(d: Path) -> str:
    """Return the declared int-model, defaulting to ``int`` (translate.py's fallback)."""
    cargo = d / "Cargo.toml"
    if cargo.exists():
        try:
            data = tomllib.loads(cargo.read_text())
            m = (
                data.get("package", {})
                .get("metadata", {})
                .get("autospec", {})
                .get("int-model")
            )
            if m in ("int", "nat", "bitvec"):
                return m
        except Exception:
            pass
    return "int"


def float_model(d: Path) -> str:
    """Infer the float-model: Real testcases have no ``Test_test.lean`` (sqrt is noncomputable)."""
    return "Real" if not (d / "Test_test.lean").exists() else "Rat"


def category(name: str) -> str:
    """Category from the directory-name prefix (``integer_gcd`` → ``integer``)."""
    return name.split("_")[0] if "_" in name else "misc"


def main() -> None:
    parser = argparse.ArgumentParser(description="Overview of the LeanMode benchmark suite")
    parser.add_argument(
        "-l", "--list", action="store_true", help="Also list every test case"
    )
    args = parser.parse_args()

    dirs = sorted(
        d
        for d in LEAN_MODE.iterdir()
        if d.is_dir() and d.name != "failure" and (d / "Cargo.toml").exists()
    )
    failure = sorted(d.name for d in (LEAN_MODE / "failure").iterdir() if d.is_dir())

    int_models: Counter[str] = Counter()
    float_models: Counter[str] = Counter()
    categories: Counter[str] = Counter()
    rows = []
    for d in dirs:
        m = int_model(d)
        t = float_model(d)
        int_models[m] += 1
        float_models[t] += 1
        categories[category(d.name)] += 1
        rows.append((d.name, m, t))

    print(f"Benchmark overview: {len(dirs)} testcases in LeanMode/")
    print()
    print("category:")
    for c, n in sorted(categories.items()):
        print(f"  {c:<12} {n}")
    print()
    print("int-model:")
    for m in ("nat", "int", "bitvec"):
        print(f"  {m:<8} {int_models.get(m, 0)}")
    print()
    print("float-model:")
    for t in ("Rat", "Real"):
        print(f"  {t:<8} {float_models.get(t, 0)}")
    if failure:
        print()
        print(f"failure/ (excluded): {len(failure)}")

    if args.list:
        print()
        for name, m, t in rows:
            print(f"  {name:<32} int-model={m:<6} float-model={t}")


if __name__ == "__main__":
    main()
