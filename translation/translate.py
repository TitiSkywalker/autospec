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

        ``src/lib.rs`` (implementation) → ``<Crate>.lean``.
        ``tests/*.rs`` (specs)           → ``Test_<name>.lean``.
        """
        project_path = Path(project_dir).resolve()
        output_path = Path(output_dir).resolve() if output_dir else project_path
        output_path.mkdir(parents=True, exist_ok=True)

        print(f"autospec  \033[1m{project_path.name}\033[0m  →  {output_path}")

        # 1. Translate implementation
        ok = self._translate_inner(project_path, output_path, flatten=False)
        if not ok:
            _warn("aeneas failed, retrying with flatten-loops")
            ok = self._translate_inner(project_path, output_path, flatten=True)
        if not ok:
            return False

        # 2. Translate each test file in tests/ as a separate spec
        tests_dir = project_path / "tests"
        if tests_dir.is_dir():
            for test_file in sorted(tests_dir.glob("*.rs")):
                self._translate_test_file(project_path, output_path, test_file)

        return True

    def _translate_inner(
        self,
        project_path: Path,
        output_path: Path,
        *,
        flatten: bool,
    ) -> bool:
        """Core pipeline.  When *flatten* is True, nested loops are extracted
        into separate functions before charon/aeneas."""
        workdir = Path(tempfile.mkdtemp(prefix="autospec_"))
        try:
            shutil.copytree(
                project_path,
                workdir,
                dirs_exist_ok=True,
                ignore=shutil.ignore_patterns("target", "lean", "*.lean"),
            )

            # Phase 0 (optional): flatten nested loops
            if flatten:
                _try_flatten_nested_loops(workdir, project_path)

            # Phase 1: charon → .llbc (library target, so aeneas sees pub fns)
            result = subprocess.run(
                [self.charon, "cargo", "--preset=aeneas", "--", "--lib"],
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

    # ── test-file translation ────────────────────────────────────────────────

    def _translate_test_file(
        self,
        project_path: Path,
        output_path: Path,
        test_file: Path,
    ) -> None:
        """Append test functions from *test_file* to the main crate's lib.rs,
        translate everything, then extract only the test definitions into
        ``Test_<stem>.lean``.

        This keeps the test translation inside the same crate so that
        dependencies like ``matmul::matmul`` are resolved as local definitions
        (computable) instead of external (noncomputable).
        """
        raw = test_file.read_text()

        # Strip #[test] and make every fn pub
        raw = re.sub(r"#\[test\]\s*\n?", "", raw)
        raw = re.sub(r"\bfn\s+(\w+)", r"pub fn \1", raw)

        # Remove `use` items that refer to the parent crate
        raw = re.sub(r"^use\s+super::.*\n?", "", raw, flags=re.MULTILINE)

        # Replace crate-prefixed calls (e.g. matmul::matmul → matmul):
        # inside the same crate the prefix is unnecessary and invalid
        crate_name = project_path.name.replace("-", "_")
        raw = re.sub(rf"\b{crate_name}::", "", raw)

        stem = test_file.stem

        # Create a temp copy of the main project with test fns appended to lib.rs
        tmp_dir = Path(tempfile.mkdtemp(prefix=f"autospec_test_{stem}_"))
        try:
            shutil.copytree(
                project_path,
                tmp_dir,
                dirs_exist_ok=True,
                ignore=shutil.ignore_patterns("target", "lean", "*.lean", "tests"),
            )
            lib_rs = tmp_dir / "src" / "lib.rs"
            original_lib = lib_rs.read_text()
            lib_rs.write_text(original_lib.rstrip() + "\n\n" + raw + "\n")

            combined_output = Path(tempfile.mkdtemp(prefix="autospec_combined_"))
            try:
                ok = self._translate_inner(tmp_dir, combined_output, flatten=False)
                if not ok:
                    _warn(f"retrying {test_file.name} with flatten-loops")
                    ok = self._translate_inner(tmp_dir, combined_output, flatten=True)
                if not ok:
                    _warn(f"test translation failed for {test_file.name}")
                    return

                # Extract only the test definitions by matching function
                # name prefixes from the test file (avoids charon naming
                # differences between the two runs).
                test_fn_names: set[str] = set(
                    re.findall(r"pub\s+fn\s+(\w+)", raw)
                )

                for lf in sorted(combined_output.glob("*.lean")):
                    combined_text = lf.read_text()
                    # _translate_inner already did F64→Rat + inline, but we
                    # run again (idempotent) in case output was cached.
                    combined_text = _lean_f64_to_rat(combined_text)
                    combined_text = inline_f64_helpers(combined_text)
                    # Deduplicate linter options from the two passes
                    combined_text = re.sub(
                        r"(set_option linter\.style\.nameCheck false\n)+",
                        "set_option linter.style.nameCheck false\n",
                        combined_text,
                    )

                    test_text = _extract_defs_by_prefix(combined_text, test_fn_names)
                    if test_text.strip():
                        # Add import for the implementation file
                        # Compute the module path for the impl file relative
                        # to the Lake package root (where lakefile.toml lives).
                        lake_root = _find_lake_root(project_path)
                        if lake_root:
                            rel = project_path.relative_to(lake_root)
                            # Module path = library root dir (capitalized) +
                            # subdirectories (as-is) + file name (capitalized)
                            parts = [p.replace("-", "_") for p in rel.parts]
                            if parts:
                                parts[0] = _capitalize_first(parts[0])
                            mod_prefix = ".".join(parts)
                            impl_module = f"{mod_prefix}.{_capitalize_first(project_path.name.replace('-', '_'))}"
                        else:
                            impl_module = project_path.name.replace("-", "_").capitalize()
                        test_text = _add_import(test_text, impl_module)
                        test_text = _add_evals(test_text, test_fn_names)
                        target = output_path / f"Test_{stem}.lean"
                        target.write_text(test_text)
                        def_count = len(re.findall(
                            r"^\s*(?:def|theorem|axiom)\s+\S+",
                            test_text, re.MULTILINE,
                        ))
                        _phase("test", f"Test_{stem}  {def_count} defs")
            finally:
                shutil.rmtree(combined_output, ignore_errors=True)
        finally:
            shutil.rmtree(tmp_dir, ignore_errors=True)


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


# ── Lean definition extraction ────────────────────────────────────────────────


def _capitalize_first(s: str) -> str:
    """Uppercase the first character, leave the rest unchanged."""
    if not s:
        return s
    return s[0].upper() + s[1:]


def _find_lake_root(path: Path) -> Path | None:
    """Walk up from *path* to find a directory containing lakefile.toml or lakefile.lean."""
    for parent in [path] + list(path.parents):
        if (parent / "lakefile.toml").exists() or (parent / "lakefile.lean").exists():
            return parent
    return None


def _add_evals(lean_text: str, fn_names: set[str]) -> str:
    """Append ``#eval <fn>`` for each test function, before ``end <ns>``."""
    for name in sorted(fn_names):
        # Insert #eval before the final `end <namespace>`
        lean_text = re.sub(
            rf"(\nend\s+\S+\s*)$",
            rf"\n#eval {name}\n\1",
            lean_text,
            count=1,
        )
    return lean_text


def _add_import(lean_text: str, module: str) -> str:
    """Insert ``import {module}`` after the last existing import line."""
    lines = lean_text.split("\n")
    last_import = -1
    for i, line in enumerate(lines):
        if line.startswith("import "):
            last_import = i
    if last_import >= 0:
        lines.insert(last_import + 1, f"import {module}")
    return "\n".join(lines)


def _extract_defs_by_prefix(combined_text: str, prefixes: set[str]) -> str:
    """Extract definitions whose names start with any prefix in *prefixes*.

    Each extracted definition includes its preceding doc-comment and attributes.
    Also includes framework definitions that are not implementation-specific
    (e.g. RatClone, MaybeUninit) so the output is self-contained.
    """
    lines = combined_text.split("\n")

    # Find preamble (imports, options, namespace, RatClone)
    preamble_end = 0
    for i, line in enumerate(lines):
        if line.startswith("/-- [") or re.match(
            r"^\s*(?:def|theorem|axiom)\s+\S+", line
        ):
            preamble_end = i
            break
    preamble = "\n".join(lines[:preamble_end]).rstrip()

    ns_match = re.search(r"namespace\s+(\S+)", preamble)
    ns = ns_match.group(1) if ns_match else "«crate»"

    # Split into definition blocks
    blocks: list[str] = []
    current: list[str] = []

    for line in lines[preamble_end:]:
        stripped = line.strip()
        # End of namespace — stop collecting
        if stripped.startswith("end ") and re.match(r"^end\s+\S+$", stripped):
            break
        if line.startswith("/-- [") and current:
            blocks.append("\n".join(current).rstrip())
            current = []
        current.append(line)
    if current:
        text = "\n".join(current).rstrip()
        # Drop trailing `end matmul` if present
        text = re.sub(r"\nend\s+\S+\s*$", "", text)
        if text.strip():
            blocks.append(text.strip())

    # Keep blocks whose definition name matches a test prefix.
    # Framework definitions (RatClone, MaybeUninit, etc.) are already
    # available from the imported implementation module.
    kept: list[str] = []

    for block in blocks:
        m = re.search(r"^\s*(?:def|theorem|axiom)\s+(\S+)", block, re.MULTILINE)
        if m:
            name = m.group(1).rstrip(":")
            if any(name.startswith(p) for p in prefixes):
                kept.append(block)

    if not kept:
        return ""

    return preamble + "\n\n" + "\n\n".join(kept) + "\n\nend " + ns + "\n"


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
