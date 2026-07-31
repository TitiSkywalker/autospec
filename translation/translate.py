"""
Translate a Rust project into Lean using Charon + Aeneas.

Pipeline:
  Rust source → [flatten-loops, best-effort] → charon → .llbc
  → [LLBC adapter: f64→f64_helpers] → aeneas → .lean
  → [F64→Rat + inline helpers] → output

The original project is never modified — all transformations happen on a
temporary copy.
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

# ── tool paths ────────────────────────────────────────────────────────────────

_ROOT = Path(__file__).resolve().parent

_LLBC_ADAPTER = _ROOT / "tools" / "llbc_adapter.py"

_FLATTEN_TOOL = (
    _ROOT / "tools" / "flatten-loops" / "target" / "release" / "flatten-loops"
)

from tools.lean_inline import inline_f64_helpers

# ── Translator ────────────────────────────────────────────────────────────────


class Translator:
    """Translate Rust projects to Lean via Charon + Aeneas."""

    def __init__(
        self,
        charon_bin: str | Path | None = None,
        aeneas_bin: str | Path | None = None,
    ):
        self.charon = str(charon_bin or _ROOT.parent / "aeneas" / "charon" / "bin" / "charon")
        self.aeneas = str(aeneas_bin or _ROOT.parent / "aeneas" / "bin" / "aeneas")

    # ── public API ──────────────────────────────────────────────────────────

    def translate(
        self,
        project_dir: str | Path,
        output_dir: str | Path | None = None,
    ) -> bool:
        """Translate a Rust project to Lean.

        Returns True if at least one .lean file was produced.
        """
        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path
        output_path.mkdir(parents=True, exist_ok=True)

        print(f"autospec  \033[1m{project_path.name}\033[0m  →  {output_path}")

        workdir = Path(tempfile.mkdtemp(prefix="autospec_"))
        try:
            shutil.copytree(
                project_path,
                workdir,
                dirs_exist_ok=True,
                ignore=shutil.ignore_patterns("target", "lean", "*.lean"),
            )

            # Phase 0: try flatten nested loops (best-effort)
            _try_flatten_nested_loops(workdir, project_path)

            # Phase 1: charon → .llbc
            result = subprocess.run(
                [self.charon, "cargo", "--preset=aeneas"],
                cwd=workdir,
                capture_output=True,
                text=True,
            )
            if result.returncode != 0:
                _fail("charon", "cargo/charon failed — see stderr below")
                for line in result.stderr.split("\n"):
                    if line.strip():
                        print(f"           {line.strip()}", file=sys.stderr)
                return False
            _phase("charon", "ok")

            # Phase 2: LLBC adapter
            llbc_files = sorted(
                f for f in os.listdir(str(workdir)) if f.endswith(".llbc")
            )
            if not llbc_files:
                _fail("adapter", "no .llbc file found")
                return False

            adapter_stats: dict[str, str] = {}
            for f in llbc_files:
                llbc_path = str(workdir / f)
                result = subprocess.run(
                    [sys.executable, str(_LLBC_ADAPTER), llbc_path, "-o", llbc_path],
                    capture_output=True,
                    text=True,
                )
                if result.returncode != 0:
                    _fail("adapter", result.stderr.strip()[:200])
                    return False
                adapter_stats = _parse_adapter_stderr(result.stderr)

            parts = [f"f64={adapter_stats.get('f64', '?')}"]
            if adapter_stats.get("binop") and adapter_stats["binop"] != "0":
                parts.append(f"{adapter_stats['binop']} binop/neg")
            if adapter_stats.get("consts") and adapter_stats["consts"] != "0":
                parts.append(f"{adapter_stats['consts']} consts")
            _phase("adapter", ",  ".join(parts))

            # Phase 3: aeneas → .lean
            aeneas_warnings: list[str] = []
            for f in llbc_files:
                result = subprocess.run(
                    [self.aeneas, "-sequential", "-backend", "lean", str(workdir / f)],
                    cwd=str(workdir),
                    capture_output=True,
                    text=True,
                )
                for line in (result.stdout + result.stderr).split("\n"):
                    line = re.sub(r"\x1b\[[0-9;]*m", "", line).strip()
                    if not line:
                        continue
                    if "[Error" in line or "error:" in line.lower():
                        _fail("aeneas", line)
                        return False
                    m = re.match(r"\[(?:Warn|Warning)\s*\]\s*(.*)", line)
                    if m:
                        w = m.group(1)
                        if "external, unknown definitions" not in w:
                            aeneas_warnings.append(w)

            lean_files = sorted(
                f for f in os.listdir(str(workdir)) if f.endswith(".lean")
            )
            if not lean_files:
                _fail("aeneas", "no .lean file generated")
                return False

            # Count definitions
            def_count = 0
            for f in lean_files:
                text = (workdir / f).read_text()
                text = text.replace(
                    "set_option linter.unusedVariables false",
                    "set_option linter.unusedVariables false\n"
                    "set_option linter.style.nameCheck false",
                )
                text = _lean_f64_to_rat(text)
                text = inline_f64_helpers(text)
                (output_path / f).write_text(text)
                def_count += len(
                    re.findall(r"^\s*(?:def|theorem|axiom)\s+\S+", text, re.MULTILINE)
                )

            file_list = ", ".join(f.replace(".lean", "") for f in lean_files)
            _phase("aeneas", f"{file_list}  {def_count} defs")

            for w in aeneas_warnings:
                _warn(w)

            _phase("✓ done", f"({len(lean_files)} file(s))")
            return True
        finally:
            shutil.rmtree(workdir, ignore_errors=True)


# ── helpers ───────────────────────────────────────────────────────────────────


def _phase(tag: str, msg: str = "") -> None:
    """Print a pipeline phase status line with consistent alignment."""
    print(f"  {tag:<10} {msg}")


def _fail(tag: str, reason: str) -> None:
    """Print a phase failure and exit."""
    print(f"  {tag:<10} \033[31mFAIL\033[0m  {reason}", file=sys.stderr)


def _warn(msg: str) -> None:
    """Print a warning."""
    print(f"           \033[33m⚠\033[0m  {msg}")


# ── Lean post-processing: F64 → Rat ──────────────────────────────────────────


def _lean_f64_to_rat(content: str) -> str:
    """Replace F64 opaque type with Rat (computable rational arithmetic)."""
    if "F64" not in content:
        return content

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
            if any(
                [
                    "f64_helpers::" in line,
                    "core::clone::impls" in line
                    and "core::clone::Clone for f64" in line,
                    "Trait implementation:" in line
                    and "core::clone::Clone for f64" in line,
                    "core::f64::" in line,
                ]
            ):
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

    result = re.sub(
        r"core\.f64\.\S+\.abs\b",
        "f64_helpers.abs",
        result,
    )
    result = re.sub(
        r"\baxiom \b(?!.*noncomputable)",
        "noncomputable axiom ",
        result,
    )
    result = result.replace("noncomputable section\n", "")

    # Insert RatClone definitions (inline step handles f64_helpers)
    if "def RatClone.clone" not in result:
        rat_clone = (
            "/- F64 replaced with Rat by autospec -/\n"
            "def RatClone.clone (x : Rat) : Result Rat := ok x\n"
            "\n"
            "@[reducible]\n"
            "def RatClone : core.clone.Clone Rat := {\n"
            "  clone := RatClone.clone\n"
            "}\n"
        )
        result = re.sub(r"(namespace \w+\n)", r"\1\n" + rat_clone, result, count=1)

    return result


# ── adapter helpers ─────────────────────────────────────────────────────────


def _parse_adapter_stderr(stderr: str) -> dict[str, str]:
    """Parse adapter stderr lines like ``[adapter] f64=306, usize=311, ...``."""
    stats: dict[str, str] = {}
    for line in stderr.split("\n"):
        m = re.match(r"\[adapter\]\s+(.+)", line.strip())
        if not m:
            continue
        body = m.group(1)
        if "=" in body:
            for part in body.split(","):
                part = part.strip()
                if "=" in part:
                    k, v = part.split("=", 1)
                    stats[k.strip()] = v.strip()
        else:
            m2 = re.match(r"(\d+)\s+float consts?,\s*(\d+)\s+binop/neg", body)
            if m2:
                stats["consts"] = m2.group(1)
                stats["binop"] = m2.group(2)
    return stats


# ── nested loop flattening (best-effort) ──────────────────────────────────────


def _try_flatten_nested_loops(project_path: Path, source_path: Path) -> None:
    """Try to flatten nested loops.  Never fails — if flattening doesn't help,
    we let aeneas handle the nested loops (which it can for simple cases)."""
    if not _FLATTEN_TOOL.exists():
        return

    modified: list[Path] = []
    for rs in project_path.rglob("*.rs"):
        if "f64_helpers" in rs.parts or "target" in rs.parts:
            continue
        try:
            result = subprocess.run(
                [str(_FLATTEN_TOOL), str(rs), "--in-place"],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode != 0:
                _warn(f"flatten-loops failed on {rs.name}: {result.stderr.strip()}")
            elif result.stderr.strip():
                modified.append(rs)
        except subprocess.TimeoutExpired:
            _warn(f"flatten-loops timed out on {rs.name}, skipping")

    if not modified:
        return

    check = subprocess.run(
        ["cargo", "check"],
        cwd=str(project_path),
        capture_output=True,
        text=True,
    )
    if check.returncode == 0:
        _phase("flatten", ", ".join(p.name for p in modified))
    else:
        # Restore original files from source
        for rs in modified:
            rel = rs.relative_to(project_path)
            src = source_path / rel
            if src.exists():
                shutil.copy2(src, rs)
        _warn("flatten-loops broke compilation, restored originals")


# ── CLI ───────────────────────────────────────────────────────────────────────

if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Translate a Rust crate into Lean")
    p.add_argument("project_dir")
    p.add_argument("-o", "--output-dir", default=None)
    p.add_argument("--charon", default=None)
    p.add_argument("--aeneas", default=None)
    args = p.parse_args()

    t = Translator(charon_bin=args.charon, aeneas_bin=args.aeneas)
    ok = t.translate(args.project_dir, args.output_dir)
    raise SystemExit(0 if ok else 1)
