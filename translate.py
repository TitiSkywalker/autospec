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

_PATCH_F64_TOOL = (
    Path(__file__).resolve().parent
    / "tools" / "patch-f64" / "target" / "release" / "patch-f64"
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
#[inline(never)] pub fn from_usize(x: usize) -> f64 { x as f64 }
"""

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

    all_literals: set[str] = set()

    # Phase 1: syn-based AST transformation
    if _PATCH_F64_TOOL.exists():
        for rs in project_path.rglob("*.rs"):
            if "f64_helpers" in rs.parts:
                continue
            result = subprocess.run(
                [str(_PATCH_F64_TOOL), str(rs)],
                capture_output=True, text=True, timeout=30,
            )
            if result.returncode != 0:
                print(f"  Warning: patch-f64 failed on {rs.name}: {result.stderr.strip()}")
                continue
            # Parse float literals from stderr
            for line in result.stderr.strip().split("\n"):
                if line.startswith("CONST:"):
                    parts = line.split(":")
                    if len(parts) >= 3:
                        all_literals.add(parts[1])
            rs.write_text(result.stdout)
            print(f"  {rs.name}: patched")

        # Phase 2: regex fallback for float literals inside macros (e.g., vec![])
        _FLOAT_RE = re.compile(r'(?<![.\w])(\d+\.\d+)(?![.\w])')
        for rs in project_path.rglob("*.rs"):
            if "f64_helpers" in rs.parts:
                continue
            content = rs.read_text()
            # Only replace float literals that are still present
            # (syn tool already handled standalone ones, these are in macros)
            rem = set(_FLOAT_RE.findall(content))
            all_literals.update(rem)
    else:
        print(f"  Warning: patch-f64 not found at {_PATCH_F64_TOOL}")
        print(f"  Build: cargo build --release --manifest-path tools/patch-f64/Cargo.toml")

    # Generate helpers crate with builtin + discovered const helpers
    helpers = _generate_float_helpers(all_literals)
    h = project_path / "f64_helpers"
    h.mkdir(exist_ok=True)
    (h / "src").mkdir(exist_ok=True)
    (h / "Cargo.toml").write_text(F64_HELPERS_CARGO_TOML)
    (h / "src" / "lib.rs").write_text(F64_HELPERS_LIB_RS + helpers)

    # Phase 3: regex replace remaining float literals (inside macros)
    if all_literals:
        _FLOAT_RE = re.compile(r'(?<![.\w])(\d+\.\d+)(?![.\w])')
        for rs in project_path.rglob("*.rs"):
            if "f64_helpers" in rs.parts:
                continue
            content = rs.read_text()
            patched, n = _FLOAT_RE.subn(
                lambda m: _float_to_helper(m.group(1)), content
            )
            if n:
                rs.write_text(patched)
                print(f"  {rs.name}: {n} float literal(s) in macros")

    # Add dependency
    cargo_toml = project_path / "Cargo.toml"
    content = cargo_toml.read_text()
    dep = 'f64_helpers = { path = "f64_helpers" }'
    if "[dependencies]" in content:
        cargo_toml.write_text(content.replace("[dependencies]",
                                               f"[dependencies]\n{dep}", 1))
    else:
        cargo_toml.write_text(content.rstrip() + f"\n\n[dependencies]\n{dep}\n")


def _float_to_helper(value: str) -> str:
    """Convert a float literal like '58.0' to 'f64_helpers::const_58_0()'."""
    name = "const_" + value.replace(".", "_").replace("-", "neg_")
    return f"f64_helpers::{name}()"


def _generate_float_helpers(literals: set[str]) -> str:
    """Generate #[inline(never)] helper functions for each float literal."""
    if not literals:
        return ""
    funcs = []
    for val in sorted(literals, key=lambda v: (len(v), v)):
        name = "const_" + val.replace(".", "_").replace("-", "neg_")
        funcs.append(f'#[inline(never)] pub fn {name}() -> f64 {{ {val} }}')
    return "\n" + "\n".join(funcs) + "\n"


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
