"""
Translate a Rust project into Lean using Charon + Aeneas.

The original project is never modified — all transformations happen on a
temporary copy.
"""

from __future__ import annotations

import re
import shutil
import subprocess
import tempfile
from pathlib import Path

from utils import CmdRunner

# ── flatten-loops tool ────────────────────────────────────────────────────────

_FLATTEN_TOOL = (
    Path(__file__).resolve().parent
    / "tools" / "flatten-loops" / "target" / "release" / "flatten-loops"
)

# ── f64 helper crate ─────────────────────────────────────────────────────────

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

# Regex rules — order matters, specific before general.
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
    (r'(\w+\[[^\]]+\])\s*=\s*\1\s*\+\s*(\w+\[[^\]]+\])\s*\*\s*(\w+\[[^\]]+\])',
     r'\1 = f64_helpers::add(\1, f64_helpers::mul(\2, \3))'),
]


# ── Translator ───────────────────────────────────────────────────────────────

class Translator:
    def __init__(self, charon_path: str | Path | None = None,
                 aeneas_path: str | Path | None = None):
        root = Path(__file__).resolve().parent
        self.charon_path = Path(charon_path or root / "aeneas" / "charon" / "bin" / "charon")
        self.aeneas_path = Path(aeneas_path or root / "aeneas" / "bin" / "aeneas")

    def translate(self, project_dir: str | Path,
                  output_dir: str | Path | None = None,
                  patch_f64: bool = False,
                  flatten_loops: bool = False) -> bool:
        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path

        # Work on a temporary copy — never touch the original
        workdir = Path(tempfile.mkdtemp(prefix="autospec_"))
        try:
            print(f"── Copy project to {workdir} ──")
            shutil.copytree(project_path, workdir, dirs_exist_ok=True,
                            ignore=shutil.ignore_patterns("target"))

            # Apply transformations to the copy
            if patch_f64:
                _setup_f64_helpers(workdir)
            if flatten_loops:
                _flatten_nested_loops(workdir)

            runner = CmdRunner()
            runner.cd(workdir.as_posix())

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

            for f in llbc_files:
                if runner.run(f"{self.aeneas_path} -sequential -backend lean {f}",
                              verbose=True, panic=False) != 0:
                    pass  # aeneas can produce partial output on error

            lean_files = [f for f in runner.ls_files() if f.endswith(".lean")]
            if not lean_files:
                return False

            # Copy output
            output_path.mkdir(parents=True, exist_ok=True)
            for f in lean_files:
                shutil.copy2(workdir / f, output_path / f)

            print(f"── Done ({len(lean_files)} .lean file(s)) ──")
            return True
        finally:
            shutil.rmtree(workdir, ignore_errors=True)


# ── f64 patching (operates on a work-copy, never the original) ────────────────

def _setup_f64_helpers(project_path: Path) -> None:
    print("── Patch f64 operations ──")

    # Create helpers crate
    h = project_path / "f64_helpers"
    h.mkdir(exist_ok=True)
    (h / "src").mkdir(exist_ok=True)
    (h / "Cargo.toml").write_text(F64_HELPERS_CARGO_TOML)
    (h / "src" / "lib.rs").write_text(F64_HELPERS_LIB_RS)

    # Add dependency
    cargo_toml = project_path / "Cargo.toml"
    content = cargo_toml.read_text()
    dep = 'f64_helpers = { path = "f64_helpers" }'
    if "[dependencies]" in content:
        cargo_toml.write_text(content.replace("[dependencies]",
                                               f"[dependencies]\n{dep}", 1))
    else:
        cargo_toml.write_text(content.rstrip() + f"\n\n[dependencies]\n{dep}\n")

    # Patch .rs files
    for rs in project_path.rglob("*.rs"):
        if "f64_helpers" in rs.parts:
            continue
        patched, n = _patch_rust_source(rs.read_text())
        if n:
            rs.write_text(patched)
            print(f"  {rs.name}: {n} replacements")


def _patch_rust_source(source: str) -> tuple[str, int]:
    total = 0
    for pat, repl in _F64_PATCH_RULES:
        source, n = re.subn(pat, repl, source)
        total += n
    return source, total


# ── nested loop flattening ────────────────────────────────────────────────────

def _flatten_nested_loops(project_path: Path) -> None:
    """Extract inner for-loops so aeneas can translate them (aeneas rejects
    for-loops nested inside other for-loops within a single function)."""
    if not _FLATTEN_TOOL.exists():
        print(f"  Warning: flatten-loops not found at {_FLATTEN_TOOL}")
        print(f"  Build: cargo build --release --manifest-path tools/flatten-loops/Cargo.toml")
        return

    print("── Flatten nested loops ──")
    for rs in project_path.rglob("*.rs"):
        if "f64_helpers" in rs.parts or "target" in rs.parts:
            continue
        result = subprocess.run(
            [str(_FLATTEN_TOOL), str(rs), "--in-place"],
            capture_output=True, text=True, timeout=30,
        )
        if result.returncode != 0:
            print(f"  Warning: flatten-loops failed on {rs.name}: {result.stderr.strip()}")
        elif result.stderr.strip():
            print(f"  {rs.name}: {result.stderr.strip()}")


# ── CLI ───────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Translate a Rust crate into Lean")
    p.add_argument("project_dir")
    p.add_argument("-o", "--output-dir", default=None, help="Output directory")
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    p.add_argument("--patch-f64", action="store_true",
                   help="Replace f64 inline ops with external function calls")
    p.add_argument("--flatten-loops", action="store_true",
                   help="Extract nested inner for-loops into separate functions")
    args = p.parse_args()

    t = Translator(charon_path=args.charon, aeneas_path=args.aeneas)
    raise SystemExit(0 if t.translate(args.project_dir, args.output_dir,
                                      args.patch_f64, args.flatten_loops) else 1)
