"""
Translate a Rust project into Lean using Charon + Aeneas.

Pipeline:
  Rust source → [flatten-loops] → charon → .llbc
  → [LLBC adapter: f64→f64_helpers] → aeneas → .lean
  → [F64→Real post-process] → output

The original project is never modified — all transformations happen on a
temporary copy.
"""

from __future__ import annotations

import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# ── tool paths ────────────────────────────────────────────────────────────────

_ROOT = Path(__file__).resolve().parent

_LLBC_ADAPTER = _ROOT / "tools" / "llbc-adapter" / "adapter.py"

_FLATTEN_TOOL = (
    _ROOT / "tools" / "flatten-loops" / "target" / "release" / "flatten-loops"
)


# ── public API ────────────────────────────────────────────────────────────────

def translate(
    project_dir: str | Path,
    output_dir: str | Path | None = None,
    *,
    charon_bin: str | Path | None = None,
    aeneas_bin: str | Path | None = None,
    flatten_loops: bool = False,
    llbc_adapter: bool = True,
) -> bool:
    """Translate a Rust project to Lean.

    Returns True if at least one .lean file was produced.
    """
    project_path = Path(project_dir).resolve()
    output_path = Path(output_dir).resolve() if output_dir else project_path
    charon = str(charon_bin or _ROOT / "aeneas" / "charon" / "bin" / "charon")
    aeneas = str(aeneas_bin or _ROOT / "aeneas" / "bin" / "aeneas")

    workdir = Path(tempfile.mkdtemp(prefix="autospec_"))
    try:
        _log("Copy project to", workdir)
        shutil.copytree(project_path, workdir, dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("target", "lean", "*.lean"))

        if flatten_loops:
            _flatten_nested_loops(workdir)

        # ── Phase 1: charon → .llbc ─────────────────────────────────────────
        _log("Charon")
        if subprocess.run([charon, "cargo", "--preset=aeneas"],
                          cwd=workdir).returncode != 0:
            return False

        # ── Phase 2: LLBC adapter ───────────────────────────────────────────
        if llbc_adapter:
            _log("LLBC adapter")
            for f in sorted(os.listdir(str(workdir))):
                if not f.endswith(".llbc"):
                    continue
                llbc_path = str(workdir / f)
                result = subprocess.run(
                    [sys.executable, str(_LLBC_ADAPTER), llbc_path,
                     "-o", llbc_path],
                    capture_output=True, text=True,
                )
                if result.returncode != 0:
                    _log(f"  adapter error: {result.stderr.strip()[:300]}")
                else:
                    for line in result.stderr.strip().split("\n"):
                        if line.strip():
                            _log(f"  {line.strip()}")

        # ── Phase 3: aeneas → .lean ─────────────────────────────────────────
        _log("Aeneas")
        llbc_files = sorted(
            f for f in os.listdir(str(workdir)) if f.endswith(".llbc")
        )
        if not llbc_files:
            return False

        for f in llbc_files:
            result = subprocess.run(
                [aeneas, "-sequential", "-backend", "lean", str(workdir / f)],
                cwd=str(workdir), capture_output=True, text=True,
            )
            for line in (result.stdout + result.stderr).split("\n"):
                line = re.sub(r'\x1b\[[0-9;]*m', '', line).strip()
                if line and ("Error" in line or "Warn" in line
                             or "Generated" in line):
                    _log(f"  {line}")

        # ── Phase 4: post-process → output ──────────────────────────────────
        lean_files = sorted(
            f for f in os.listdir(str(workdir)) if f.endswith(".lean")
        )
        if not lean_files:
            return False

        output_path.mkdir(parents=True, exist_ok=True)
        for f in lean_files:
            content = (workdir / f).read_text()
            content = _lean_f64_to_rat(content)
            (output_path / f).write_text(content)

        _log(f"Done ({len(lean_files)} .lean file(s))")
        return True
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


# ── helpers ───────────────────────────────────────────────────────────────────

def _log(*args):
    print("──", *args)


# ── Lean post-processing: F64 → Rat ──────────────────────────────────────────

def _lean_f64_to_rat(content: str) -> str:
    """Replace F64 opaque type with Rat (computable rational arithmetic)."""
    if "F64" not in content:
        return content

    const_names = set(re.findall(r"f64_helpers\.(const_\w+)", content))
    content = content.replace("F64.Insts.CoreCloneClone.clone", "RatClone.clone")
    content = content.replace("F64.Insts.CoreCloneClone", "RatClone")

    lines = content.split("\n")
    skip: set[int] = set()
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.startswith("/-- [") or (
            line.startswith("/-- Trait") and "Clone for f64" in line
        ):
            if any([
                "f64_helpers::" in line,
                "core::clone::impls" in line
                and "core::clone::Clone for f64" in line,
                "Trait implementation:" in line
                and "core::clone::Clone for f64" in line,
                "core::f64::" in line,
            ]):
                start = i
                while i < len(lines) and not lines[i].rstrip().endswith("-/"):
                    i += 1
                i += 1
                while i < len(lines) and lines[i].startswith("@["):
                    i += 1
                while i < len(lines) and lines[i].strip() != "":
                    i += 1
                if i < len(lines) and lines[i].strip() == "":
                    i += 1
                skip.update(range(start, i))
                continue
        i += 1

    out = []
    for i, line in enumerate(lines):
        if i in skip:
            continue
        out.append(line.replace("F64", "Rat"))
    result = "\n".join(out)
    # Rat operations are fully computable.  Remove the blanket
    # noncomputable section and mark only the remaining axioms.
    # Replace remaining calls to core.f64.X.abs with f64_helpers.abs
    result = re.sub(
        r"core\.f64\.\S+\.abs\b", "f64_helpers.abs", result,
    )
    # Mark remaining axioms as noncomputable
    result = re.sub(
        r"\baxiom \b(?!.*noncomputable)", "noncomputable axiom ",
        result,
    )
    result = result.replace("noncomputable section\n", "")

    # Only add defs that don't already exist
    existing = set(re.findall(
        r"^def (f64_helpers\.\w+|RatClone\S*)", result, re.MULTILINE
    ))
    defs = _make_rat_defs(const_names, existing)
    if defs:
        result = re.sub(r"(namespace \w+\n)", r"\1\n" + defs, result, count=1)
    return result


def _float_str_to_rat(val: str) -> str:
    """Convert a float string to a Rat expression.  E.g. 0.001 → (1/1000 : Rat)."""
    if val.startswith("-"):
        return f"-({_float_str_to_rat(val[1:])})"
    if "." not in val:
        return f"({val} : Rat)"
    int_part, frac_part = val.split(".")
    denom = 10 ** len(frac_part)
    num = int(int_part) * denom + int(frac_part) if int_part else int(frac_part)
    return f"({num}/{denom} : Rat)"


def _make_rat_defs(const_names: set[str],
                    existing: set[str] | None = None) -> str:
    """Generate Rat-based Lean definitions replacing f64_helpers."""
    if existing is None:
        existing = set()
    parts = []
    if "RatClone.clone" not in existing:
        parts.extend([
            "/- F64 replaced with Rat by autospec -/",
            "def RatClone.clone (x : Rat) : Result Rat := ok x",
            "",
            "@[reducible]",
            "def RatClone : core.clone.Clone Rat := {",
            "  clone := RatClone.clone",
            "}",
            "",
        ])
    for name, sig, body in [
        ("add", "(a b : Rat) : Result Rat", "ok (a + b)"),
        ("sub", "(a b : Rat) : Result Rat", "ok (a - b)"),
        ("mul", "(a b : Rat) : Result Rat", "ok (a * b)"),
        ("div", "(a b : Rat) : Result Rat", "ok (a / b)"),
        ("neg", "(a : Rat) : Result Rat",  "ok (-a)"),
        ("abs", "(a : Rat) : Result Rat",  "ok (|a|)"),
        ("eq",  "(a b : Rat) : Result Bool", "ok (a = b)"),
        ("gt",  "(a b : Rat) : Result Bool", "ok (a > b)"),
        ("lt",  "(a b : Rat) : Result Bool", "ok (a < b)"),
        ("ge",  "(a b : Rat) : Result Bool", "ok (a ≥ b)"),
        ("le",  "(a b : Rat) : Result Bool", "ok (a ≤ b)"),
    ]:
        if f"f64_helpers.{name}" not in existing:
            parts.append(f"def f64_helpers.{name} {sig} := {body}")

    if "f64_helpers.from_usize" not in existing:
        parts.append("")
        parts.append(
            "def f64_helpers.from_usize (x : Std.Usize) : Result Rat "
            ":= ok ((x.val : Rat))"
        )

    new_consts = const_names - {
        n for n in const_names if f"f64_helpers.{n}" in existing
    }
    if new_consts:
        parts.append("")
        for name in sorted(new_consts):
            val = _float_str_to_rat(_const_name_to_float_str(name))
            parts.append(
                f"def f64_helpers.{name} : Result Rat := ok ({val})"
            )

    return "\n".join(parts) + "\n" if parts else ""


def _const_name_to_float_str(name: str) -> str:
    """Convert const_0_001 → "0.001", const_neg_1_0 → "-1.0"."""
    v = name[len("const_"):]
    if v.startswith("neg_"):
        return "-" + v[len("neg_"):].replace("_", ".")
    return v.replace("_", ".")


# ── nested loop flattening ────────────────────────────────────────────────────

def _flatten_nested_loops(project_path: Path) -> None:
    if not _FLATTEN_TOOL.exists():
        _log(f"Warning: flatten-loops not found at {_FLATTEN_TOOL}")
        _log("  Build: cargo build --release "
             "--manifest-path tools/flatten-loops/Cargo.toml")
        return

    _log("Flatten nested loops")
    for rs in project_path.rglob("*.rs"):
        if "f64_helpers" in rs.parts or "target" in rs.parts:
            continue
        result = subprocess.run(
            [str(_FLATTEN_TOOL), str(rs), "--in-place"],
            capture_output=True, text=True, timeout=30,
        )
        if result.returncode != 0:
            _log(f"  Warning: flatten-loops failed on {rs.name}: "
                 f"{result.stderr.strip()}")
        elif result.stderr.strip():
            _log(f"  {rs.name}: {result.stderr.strip()}")


# ── CLI ───────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Translate a Rust crate into Lean")
    p.add_argument("project_dir")
    p.add_argument("-o", "--output-dir", default=None)
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    p.add_argument("--flatten-loops", action="store_true",
                   help="Extract nested inner for-loops into separate functions")
    p.add_argument("--no-llbc-adapter", action="store_true",
                   help="Skip LLBC-level f64 transformation")
    args = p.parse_args()

    ok = translate(
        args.project_dir, args.output_dir,
        charon_bin=args.charon, aeneas_bin=args.aeneas,
        flatten_loops=args.flatten_loops,
        llbc_adapter=not args.no_llbc_adapter,
    )
    raise SystemExit(0 if ok else 1)
