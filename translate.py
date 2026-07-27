"""
Translate a Rust project into Lean using Charon + Aeneas.
"""

from __future__ import annotations

import re
import shutil
from pathlib import Path

from utils import CmdRunner

# ── F64 helper crate (separate crate → charon treats as opaque → aeneas axioms) ──

F64_HELPERS_CARGO_TOML = """[package]
name = "f64_helpers"
version = "0.1.0"
edition = "2021"
"""

F64_HELPERS_LIB_RS = """#![allow(dead_code)]
#[inline(never)] pub fn add(a: f64, b: f64) -> f64 { a + b }
#[inline(never)] pub fn sub(a: f64, b: f64) -> f64 { a - b }
#[inline(never)] pub fn mul(a: f64, b: f64) -> f64 { a * b }
#[inline(never)] pub fn div(a: f64, b: f64) -> f64 { a / b }
#[inline(never)] pub fn neg(a: f64) -> f64 { -a }
#[inline(never)] pub fn abs(a: f64) -> f64 { a.abs() }
#[inline(never)] pub fn eq(a: f64, b: f64) -> bool { a == b }
#[inline(never)] pub fn gt(a: f64, b: f64) -> bool { a > b }
#[inline(never)] pub fn lt(a: f64, b: f64) -> bool { a < b }
#[inline(never)] pub fn le(a: f64, b: f64) -> bool { a <= b }
#[inline(never)] pub fn ge(a: f64, b: f64) -> bool { a >= b }
#[inline(never)] pub fn const_0() -> f64 { 0.0 }
#[inline(never)] pub fn const_1() -> f64 { 1.0 }
"""

# Regex rules: (pattern, replacement) — order matters, specific before general.
_F64_PATCH_RULES = [
    (r'(\w+(?:\[[^\]]+\]))\.abs\(\)',
     r'f64_helpers::abs(\1)'),
    (r'(\w+)\s*==\s*0\.0',
     r'f64_helpers::eq(\1, f64_helpers::const_0())'),
    (r'(\w+)\s*>\s*(\w+)\b(?!\s*=)',
     r'f64_helpers::gt(\1, \2)'),
    (r'1\.0\s*/\s*(\w+)',
     r'f64_helpers::div(f64_helpers::const_1(), \1)'),
    (r'(\w+\[[^\]]+\])\s*\*=\s*(\w+)',
     r'\1 = f64_helpers::mul(\1, \2)'),
    (r'(\w+\[[^\]]+\])\s*=\s*\1\s*-\s*(\w+\[[^\]]+\])\s*\*\s*(\w+)',
     r'\1 = f64_helpers::sub(\1, f64_helpers::mul(\2, \3))'),
]


class Translator:
    def __init__(self, charon_path: str | Path | None = None,
                 aeneas_path: str | Path | None = None):
        root = Path(__file__).resolve().parent
        self.charon_path = Path(charon_path or root / "aeneas" / "charon" / "bin" / "charon")
        self.aeneas_path = Path(aeneas_path or root / "aeneas" / "bin" / "aeneas")

    def translate(self, project_dir: str | Path,
                  output_dir: str | Path | None = None,
                  patch_f64: bool = False) -> bool:
        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path

        restore = self._setup_f64_helpers(project_path) if patch_f64 else None
        try:
            runner = CmdRunner()
            runner.cd(project_path.as_posix())

            # Charon: Rust → .llbc
            print("── Charon ──")
            if runner.run(f"{self.charon_path} cargo --preset=aeneas",
                          verbose=True, panic=False) != 0:
                return False

            # Aeneas: .llbc → .lean
            print("── Aeneas ──")
            llbc_files = [f for f in runner.ls_files() if f.endswith(".llbc")]
            if not llbc_files:
                return False

            ok = True
            for f in llbc_files:
                if runner.run(f"{self.aeneas_path} -sequential -backend lean {f}",
                              verbose=True, panic=False) != 0:
                    ok = False

            lean_files = [f for f in runner.ls_files() if f.endswith(".lean")]
            if not lean_files:
                return False

            # Move output
            if output_path != project_path:
                output_path.mkdir(parents=True, exist_ok=True)
                for f in lean_files:
                    (project_path / f).rename(output_path / f)

            # Cleanup
            for f in runner.ls_files():
                if f.endswith(".llbc"):
                    runner.run(f"rm {f}", verbose=False, panic=False)

            print("── Done ──")
            return True
        finally:
            if restore:
                self._teardown_f64_helpers(restore)

    # ── f64 patching ──────────────────────────────────────────────────

    def _setup_f64_helpers(self, project_path: Path) -> dict | None:
        print("── Patch Rust source (f64 → f64_helpers) ──")

        # Create helpers crate
        h = project_path / "f64_helpers"
        h.mkdir(exist_ok=True)
        (h / "src").mkdir(exist_ok=True)
        (h / "Cargo.toml").write_text(F64_HELPERS_CARGO_TOML)
        (h / "src" / "lib.rs").write_text(F64_HELPERS_LIB_RS)

        # Add dependency to Cargo.toml
        cargo_toml = project_path / "Cargo.toml"
        orig_toml = cargo_toml.read_text()
        dep = 'f64_helpers = { path = "f64_helpers" }'
        if "[dependencies]" in orig_toml:
            cargo_toml.write_text(orig_toml.replace("[dependencies]",
                                                     f"[dependencies]\n{dep}", 1))
        else:
            cargo_toml.write_text(orig_toml.rstrip() + f"\n\n[dependencies]\n{dep}\n")

        # Patch .rs files
        backups = {}
        for rs in project_path.rglob("*.rs"):
            if "f64_helpers" in rs.parts:
                continue
            backups[str(rs)] = rs.read_text()
            patched, n = _patch_rust_source(rs.read_text())
            if n:
                rs.write_text(patched)
                print(f"  {rs.name}: {n} replacements")

        return {"project_path": str(project_path), "orig_toml": orig_toml, "rs_backups": backups}

    def _teardown_f64_helpers(self, restore: dict) -> None:
        project_path = Path(restore["project_path"])
        h = project_path / "f64_helpers"
        if h.exists():
            shutil.rmtree(h)
        (project_path / "Cargo.toml").write_text(restore["orig_toml"])
        for path_str, orig in restore["rs_backups"].items():
            Path(path_str).write_text(orig)
        print("  Restored original sources")


def _patch_rust_source(source: str) -> tuple[str, int]:
    total = 0
    for pat, repl in _F64_PATCH_RULES:
        source, n = re.subn(pat, repl, source)
        total += n
    return source, total


# ── CLI ────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Translate a Rust crate into Lean")
    p.add_argument("project_dir")
    p.add_argument("-o", "--output-dir", default=None, help="Output directory")
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    p.add_argument("--patch-f64", action="store_true",
                   help="Replace f64 inline ops with external function calls")
    args = p.parse_args()

    t = Translator(charon_path=args.charon, aeneas_path=args.aeneas)
    raise SystemExit(0 if t.translate(args.project_dir, args.output_dir, args.patch_f64) else 1)
