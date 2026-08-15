"""Extract and translate test files (``tests/*.rs``) as separate Lean specs.

Test functions are appended to a copy of the main crate so that all
definitions are local (computable) rather than external (noncomputable).
"""

import re
import shutil
import tempfile
from pathlib import Path

from .log import phase, warn
from .lean_post import lean_f64_to_rat, lean_f64_to_real, lean_integers_to_model, NonComputableError
from .utils import read_crate_name, capitalize_first, find_lake_root

# Import from sibling tools package
import sys
_tools_dir = str(Path(__file__).resolve().parent.parent / "tools")
if _tools_dir not in sys.path:
    sys.path.insert(0, _tools_dir)
from lean_inline import inline_f64_helpers, inline_i64_helpers  # noqa: E402


def translate_test_file(
    translate_inner,
    project_path: Path,
    output_path: Path,
    test_file: Path,
    *,
    target: str = "Rat",
    int_model: str = "int",
) -> int | None:
    """Append test functions to the main crate, translate, extract test defs.

    Parameters
    ----------
    translate_inner:
        Callable ``(project_path, output_path, *, flatten) -> bool`` that
        runs the core translation pipeline.
    """
    raw = test_file.read_text()
    crate_name = read_crate_name(project_path)

    # Record #[test]-annotated function names (these get #eval later).
    # Handles bare #[test] and #[test] + #[should_panic(...)] chains.
    eval_names: set[str] = set(
        re.findall(
            r"#\[test\]\s*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*fn\s+(\w+)",
            raw,
        )
    )

    # Strip #[test] and make every fn pub
    raw = re.sub(r"#\[test\]\s*\n?", "", raw)
    raw = re.sub(r"\bfn\s+(\w+)", r"pub fn \1", raw)

    # Remove `use` items that refer to the parent crate or the crate itself
    raw = re.sub(
        rf"^use\s+(?:super|{crate_name})::.*\n?",
        "",
        raw,
        flags=re.MULTILINE,
    )
    # Replace crate-prefixed calls (e.g. matmul::matmul → matmul)
    raw = re.sub(rf"\b{crate_name}::", "", raw)

    stem = test_file.stem

    tmp_dir = Path(tempfile.mkdtemp(prefix=f"autospec_test_{stem}_"))
    try:
        shutil.copytree(
            project_path,
            tmp_dir,
            dirs_exist_ok=True,
            ignore=shutil.ignore_patterns("target", "lean", "*.lean", "tests"),
        )
        lib_rs = tmp_dir / "src" / "lib.rs"
        lib_rs.write_text(lib_rs.read_text().rstrip() + "\n\n" + raw + "\n")

        combined_output = Path(tempfile.mkdtemp(prefix="autospec_combined_"))
        try:
            outcome = translate_inner(tmp_dir, combined_output, flatten=False)
            if outcome is None:
                warn(f"retrying {test_file.name} with flatten-loops")
                outcome = translate_inner(tmp_dir, combined_output, flatten=True)
            if outcome is None:
                warn(f"test translation failed for {test_file.name}")
                return None

            test_fn_names: set[str] = set(
                re.findall(r"pub\s+fn\s+(\w+)", raw)
            )

            _use_real = target == "Real"

            for lf in sorted(combined_output.glob("*.lean")):
                combined_text = lf.read_text()
                if _use_real:
                    combined_text = lean_f64_to_real(combined_text)
                else:
                    try:
                        combined_text = lean_f64_to_rat(combined_text)
                    except NonComputableError:
                        raise
                combined_text = lean_integers_to_model(combined_text, int_model)
                combined_text = inline_f64_helpers(combined_text)
                combined_text = inline_i64_helpers(combined_text)
                if _use_real:
                    combined_text = re.sub(r"\bRat\b", "ℝ", combined_text)
                    combined_text = re.sub(r"\bRatClone\b", "RealClone", combined_text)
                combined_text = re.sub(
                    r"(set_option linter\.style\.nameCheck false\n)+",
                    "set_option linter.style.nameCheck false\n",
                    combined_text,
                )

                test_text = _extract_defs_by_prefix(combined_text, test_fn_names)
                if test_text.strip():
                    impl_module = _compute_module_path(project_path)
                    test_text = _add_import(test_text, impl_module)
                    test_text = _add_evals(test_text, eval_names)
                    target = output_path / f"Test_{stem}.lean"
                    target.write_text(test_text)
                    def_count = len(re.findall(
                        r"^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+",
                        test_text, re.MULTILINE,
                    ))
                    return def_count
        finally:
            shutil.rmtree(combined_output, ignore_errors=True)
    finally:
        shutil.rmtree(tmp_dir, ignore_errors=True)


# ── helpers ───────────────────────────────────────────────────────────────────


def _snake_to_camel(s: str) -> str:
    """Convert snake_case to CamelCase (e.g. ``matrix_kronecker`` → ``MatrixKronecker``)."""
    return "".join(part[0].upper() + part[1:] for part in s.split("_") if part)


def _compute_module_path(project_path: Path) -> str:
    """Compute the Lean module path for the impl file relative to the Lake root."""
    lake_root = find_lake_root(project_path)
    if lake_root:
        rel = project_path.relative_to(lake_root)
        parts = [p.replace("-", "_") for p in rel.parts]
        if parts:
            parts[0] = capitalize_first(parts[0])
        mod_prefix = ".".join(parts)
        crate_stem = project_path.name.replace("-", "_")
        return f"{mod_prefix}.{_snake_to_camel(crate_stem)}"
    return _snake_to_camel(project_path.name.replace("-", "_"))


def _add_import(lean_text: str, module: str) -> str:
    """Insert ``import {module}`` after the last ``import`` line."""
    lines = lean_text.split("\n")
    last_import = -1
    for i, line in enumerate(lines):
        if line.startswith("import "):
            last_import = i
    if last_import >= 0:
        lines.insert(last_import + 1, f"import {module}")
    return "\n".join(lines)


def _add_evals(lean_text: str, fn_names: set[str]) -> str:
    """Append ``#eval <fn>`` for each test function, before ``end <ns>``.

    Skips eval when the file or the specific def is ``noncomputable``
    (``#eval`` cannot execute noncomputable definitions).
    """
    if "noncomputable section" in lean_text:
        return lean_text
    for name in sorted(fn_names):
        # Skip if this specific def is marked noncomputable
        if re.search(
            rf"noncomputable\s+def\s+{re.escape(name)}\b", lean_text
        ):
            continue
        lean_text = re.sub(
            rf"(\nend\s+\S+\s*)$",
            rf"\n#eval {name}\n\1",
            lean_text,
            count=1,
        )
    return lean_text


def _extract_defs_by_prefix(combined_text: str, prefixes: set[str]) -> str:
    """Extract definitions whose names start with any prefix in *prefixes*."""
    lines = combined_text.split("\n")

    # Find preamble
    preamble_end = 0
    for i, line in enumerate(lines):
        if line.startswith("/-- [") or re.match(
            r"^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+", line
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
        if stripped.startswith("end ") and re.match(r"^end\s+\S+$", stripped):
            break
        if line.startswith("/-- [") and current:
            blocks.append("\n".join(current).rstrip())
            current = []
        current.append(line)
    if current:
        text = "\n".join(current).rstrip()
        text = re.sub(r"\nend\s+\S+\s*$", "", text)
        if text.strip():
            blocks.append(text.strip())

    _KEEP_PREFIXES = set(prefixes)
    kept: list[str] = []
    for block in blocks:
        m = re.search(r"^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+(\S+)", block, re.MULTILINE)
        if m:
            name = m.group(1).rstrip(":")
            if any(name.startswith(p) for p in _KEEP_PREFIXES):
                kept.append(block)

    if not kept:
        return ""

    return preamble + "\n\n" + "\n\n".join(kept) + "\n\nend " + ns + "\n"
