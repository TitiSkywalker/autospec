#!/usr/bin/env python3
"""autospec — Generate formal specifications for Rust projects from doc strings.

Usage:
    autospec PROJECT_PATH [--output DIR] [--model MODEL] [--max-iterations N] [--verbose]
    autospec PROJECT_PATH --split   # one agent per documented function

Examples:
    autospec ~/my-rust-lib
    autospec ~/my-rust-lib --output specs/ --model deepseek-v4-pro --verbose
    autospec ~/my-rust-lib --split --max-iterations 5
"""

import argparse
import os
import signal
import sys
import time
from pathlib import Path


def _split_detail(detail: str) -> list[dict]:
    """Split read_rustdoc detail output into per-item sections.

    Each section starts with ``## kind: name`` and extends until the
    next ``## `` header or end of text. Only sections that actually
    contain doc comments (beyond the header line) are returned.
    """
    import re

    sections: list[dict] = []
    lines = detail.split("\n")

    current: dict | None = None  # {kind, name, start}

    for i, line in enumerate(lines):
        # Match any ## kind: name header as a section boundary
        m = re.match(r"^## (\w+): (.+)", line)
        if m:
            kind = m.group(1)
            if current is not None:
                text = "\n".join(lines[current["start"] : i])
                if _section_has_docs(text):
                    sections.append(
                        {"kind": current["kind"], "name": current["name"], "text": text}
                    )
            raw_name = m.group(2).strip()
            # Strip re-export suffixes: "(via path)", "(re-export)"
            clean_name = re.sub(r"\s*\(via .+\)", "", raw_name).strip()
            clean_name = re.sub(r"\s*\(re-export\)", "", clean_name).strip()
            current = {"kind": kind, "name": clean_name, "start": i}

    if current is not None:
        text = "\n".join(lines[current["start"] :])
        if _section_has_docs(text):
            sections.append(
                {"kind": current["kind"], "name": current["name"], "text": text}
            )

    return sections


def _section_has_docs(text: str) -> bool:
    """Check whether a detail section has real doc comments beyond the header."""
    for line in text.split("\n")[1:]:  # skip the ## header
        stripped = line.strip()
        # Real doc content is a non-empty line that isn't a sub-header or impl stub
        if stripped and not stripped.startswith("##") and not stripped.startswith("[impl") and not stripped.startswith("  fn "):
            return True
    return False


def _parse_functions_from_section(doc_section: str) -> list[dict]:
    """Parse individual documented functions from a doc section.

    Handles two formats:
    1. Type sections (struct/enum/trait): extracts ``fn name(...) -> T — doc``
       lines from ``[impl ...]`` blocks.
    2. Standalone function sections: ``## function: name`` headers followed by
       ``fn sig`` and doc text.

    Returns a list of dicts with keys: name, sig, doc, impl, module,
    type_kind, type_name, is_standalone.
    """
    import re

    mod_match = re.search(r"// Module: (.+)", doc_section)
    type_match = re.search(r"// Type: (\w+) (.+)", doc_section)
    is_standalone = "// Standalone functions" in doc_section

    module = mod_match.group(1).strip() if mod_match else ""
    type_kind = type_match.group(1) if type_match else ""
    type_name = type_match.group(2) if type_match else ""

    # Extract struct fields / enum variants so agents don't need read_file
    fields_text = ""
    fm = re.search(r"\nFields:\n((?:  .+\n)+)", doc_section)
    if fm:
        fields_text = fm.group(1).strip()

    functions: list[dict] = []

    if is_standalone:
        # Standalone functions: each has "## function: name" header,
        # followed by "fn sig" line and doc text.
        parts = re.split(r"\n## function: ", doc_section)
        for part in parts[1:]:  # skip header portion before first function
            lines = part.strip().split("\n")
            name = lines[0].strip()
            sig = ""
            doc_lines: list[str] = []
            for line in lines[1:]:
                if line.startswith("fn ") and not sig:
                    sig = line.strip()
                elif sig and line.strip() and not line.startswith("["):
                    doc_lines.append(line.strip())
                elif line.startswith("["):
                    break
            doc = " ".join(doc_lines).strip()
            if doc and sig and name:
                functions.append({
                    "name": name,
                    "sig": sig,
                    "doc": doc,
                    "impl": "standalone",
                    "module": module,
                    "type_kind": "",
                    "type_name": "",
                    "is_standalone": True,
                    "fields": "",
                })
    else:
        # Struct/enum/trait methods from impl blocks
        current_impl = "inherent"
        for line in doc_section.split("\n"):
            impl_m = re.match(r"^  \[impl (.+)\]$", line)
            if impl_m:
                current_impl = impl_m.group(1)
                continue

            # Return type may contain spaces: Result<T, E>, & Type, (A, B)
            fn_m = re.match(r"^    fn (\w+)\(([^)]*)\) -> (.+?) — (.+)$", line)
            if fn_m:
                name = fn_m.group(1)
                params = fn_m.group(2)
                ret = fn_m.group(3)
                doc = fn_m.group(4).strip()
                if not doc:
                    continue
                functions.append({
                    "name": name,
                    "sig": f"fn {name}({params}) -> {ret}",
                    "doc": doc,
                    "impl": current_impl,
                    "module": module,
                    "type_kind": type_kind,
                    "type_name": type_name,
                    "is_standalone": False,
                    "fields": fields_text,
                })

    return functions


def _merge_specs(output_dir: str) -> list[str]:
    """Merge individual per-function spec files into per-type combined files.

    Reads ``output_dir/individual/*.rs``, groups by ``{module}__{type}``
    prefix (everything before the last ``__`` in the stem), and writes
    combined files to ``output_dir/`` for compilation.

    Idempotent — safe to call on already-merged output.
    """
    import re

    output = Path(output_dir)
    indiv_dir = output / "individual"

    if not indiv_dir.is_dir():
        return []

    indiv_files = sorted(indiv_dir.glob("*.rs"))
    if not indiv_files:
        return []

    # Group by merge key: stem before the last "__"
    groups: dict[str, list[Path]] = {}
    for fp in indiv_files:
        stem = fp.stem
        if "__" not in stem:
            continue
        last_dd = stem.rfind("__")
        merge_key = stem[:last_dd]
        groups.setdefault(merge_key, []).append(fp)

    merged_files: list[str] = []

    for merge_key, files in sorted(groups.items()):
        contents: list[str] = []
        module = ""
        type_kind = ""
        type_name = ""
        is_standalone = False
        func_names: list[str] = []

        for i, fp in enumerate(files):
            text = fp.read_text(encoding="utf-8")
            contents.append(text.strip())

            # Extract metadata from first file
            if i == 0:
                mod_match = re.search(r"// Module: (.+)", text)
                if mod_match:
                    module = mod_match.group(1).strip()
                type_match = re.search(r"// Type: (\w+) (.+)", text)
                if type_match:
                    type_kind = type_match.group(1)
                    type_name = type_match.group(2)
                if "// Standalone functions" in text:
                    is_standalone = True

            spec_match = re.search(r"// Spec-for: (.+)", text)
            if spec_match:
                func_names.append(spec_match.group(1).strip())

        # Reconstruct header
        header_lines = []
        if module:
            header_lines.append(f"// Module: {module}")
        if is_standalone:
            header_lines.append(f"// Standalone functions")
        elif type_kind and type_name:
            header_lines.append(f"// Type: {type_kind} {type_name}")
        header_lines.append(
            f"// Expected-fns: {', '.join(func_names) if func_names else '(none)'}"
        )
        header_lines.append("// Kani proof harnesses generated from doc comments.")
        header_lines.append("")

        combined = "\n".join(header_lines) + "\n" + "\n\n".join(contents) + "\n"

        out_path = output / f"{merge_key}.rs"
        out_path.write_text(combined, encoding="utf-8")
        merged_files.append(merge_key)

    print(f"  Merged {len(indiv_files)} individual files → {len(merged_files)} type files")
    return sorted(merged_files)


def _run_func_agent(
    project_path: str,
    output_dir: str,
    filename: str,
    func_info: dict,
    api_key: str,
    base_url: str,
    model: str,
    max_iterations: int,
    max_tokens: int,
    verbose: bool,
) -> bool:
    """Run a slim agent to write ONE Kani harness for a single function.

    Returns True on success, False on failure.
    """
    from agent import SpecAgent
    from prompt import FUNC_SYSTEM_PROMPT

    f = func_info

    context_parts = ["Write ONE Kani proof harness for:"]
    context_parts.append(f"  Function: {f['name']}")
    context_parts.append(f"  Signature: {f['sig']}")
    context_parts.append(f"  Documentation: {f['doc']}")

    if f["type_name"]:
        context_parts.append(f"  Defined in: impl {f['impl']}")
        context_parts.append(f"  Type: {f['type_kind']} {f['type_name']}")
        if f.get("fields"):
            context_parts.append(f"  Struct fields:\n{f['fields']}")
    context_parts.append(f"  Module: {f['module']}")

    context_parts.append("")
    if not f.get("fields"):
        context_parts.append(
            "Use read_file to inspect source files if you need to check "
            "field types or function bodies."
        )
    else:
        context_parts.append(
            "Struct fields are listed above — you should NOT need read_file. "
            "Only use read_file if the function body has hidden preconditions "
            "(unwrap, array indexing) not mentioned in the docs."
        )
    context_parts.append(
        f"Then call write_spec(filename=\"{filename}\", content=...) "
        "with exactly ONE harness."
    )
    context_parts.append(
        "Include file header comments at the top of your content: "
        f"// Module: {f['module']}, // Type: {f['type_kind']} {f['type_name']}, "
        f"// Spec-for: {f['name']}"
    )

    task = "\n".join(context_parts)

    try:
        agent = SpecAgent(
            project_path=project_path,
            output_dir=output_dir,
            model=model,
            max_iterations=max(3, max_iterations),
            max_tokens=max_tokens,
            verbose=verbose,
            api_key=api_key,
            base_url=base_url,
            system_prompt=FUNC_SYSTEM_PROMPT,
            tool_filter=["write_spec", "read_file", "search_code"],
        )
        agent.run(task=task, quiet=True)
        return True
    except Exception as exc:
        print(f"  [FAIL] {f['name']}: {type(exc).__name__}: {exc}", file=sys.stderr)
        return False


# ——— Standard trait names whose properties are well-known ———
_STANDARD_TRAITS: set[str] = {
    "Clone", "Copy", "Debug", "Display", "Default",
    "PartialEq", "Eq", "PartialOrd", "Ord", "Hash",
    "Borrow", "BorrowMut", "AsRef", "AsMut", "Deref", "DerefMut",
    "From", "Into", "TryFrom", "TryInto",
    "Serialize", "Deserialize", "Zeroize", "ToOwned", "FromStr",
    "Iterator", "IntoIterator", "FromIterator", "DoubleEndedIterator",
    "ExactSizeIterator", "Index", "IndexMut",
    "Add", "Sub", "Mul", "Div", "Rem", "Neg", "Not",
    "AddAssign", "SubAssign", "MulAssign", "DivAssign",
    "BitAnd", "BitOr", "BitXor", "Shl", "Shr",
    "Sum", "Product", "FusedIterator",
    "Send", "Sync", "Unpin", "Sized",
    "ConditionallySelectable", "CtOption",
    "Any", "Freeze", "RefUnwindSafe", "UnwindSafe", "UnsafeUnpin",
    "CloneToUninit",
}


def _strip_standard_trait_impls(detail: str) -> str:
    """Remove impl blocks for standard traits from a detail section.

    Standard trait impls (Clone, Debug, Default, From, etc.) have
    well-known semantics and don't need specifications.
    """
    import re

    lines = detail.split("\n")
    result: list[str] = []
    i = 0

    while i < len(lines):
        line = lines[i]
        # Check if this is an impl block header
        m = re.match(r"^  \[impl (.+)\]$", line)
        if m:
            label = m.group(1)
            # Determine if this is a standard trait impl
            # Format: "impl TraitName for TypeName" or "impl TypeName"
            skip = False
            if " for " in label:
                trait_part = label.split(" for ")[0]
                if trait_part in _STANDARD_TRAITS:
                    skip = True

            if skip:
                # Skip lines until the next blank line or non-impl content
                i += 1
                while i < len(lines):
                    if lines[i].strip() == "":
                        break
                    if re.match(r"^  \[impl ", lines[i]):
                        break
                    if re.match(r"^## \w+:", lines[i]):
                        break
                    i += 1
                # Skip trailing blank lines
                while i < len(lines) and lines[i].strip() == "":
                    i += 1
                continue

        result.append(line)
        i += 1

    return "\n".join(result)


def _get_spec_targets(project_path: str, output_dir: str) -> list[tuple[str, str]]:
    """Generate per-type spec targets from documented modules.

    Returns list of (safe_filename, doc_section) where doc_section is
    the full pre-parsed docs for one type or a group of standalone functions.
    """
    import tools  # noqa: F401 — triggers tool registration
    from tool_registry import set_context, dispatch

    abs_project = str(Path(project_path).resolve())
    abs_output = str(Path(output_dir).resolve())
    set_context(abs_project, abs_output)

    # Generate JSON if needed
    result = dispatch("generate_doc_json", {"features": "all"})
    if result.startswith("Error:") and "already exists" not in result:
        print(f"Warning: generate_doc_json: {result}", file=sys.stderr)

    # Get module summary
    summary = dispatch("read_rustdoc", {})
    if summary.startswith("Error:"):
        print(f"Error: {summary}", file=sys.stderr)
        sys.exit(1)

    # Parse candidate modules from summary
    import re
    candidates: list[str] = []
    for line in summary.split("\n"):
        stripped = line.strip()
        if not stripped.startswith("📁"):
            continue
        m = re.match(r"📁\s+(\S+?)/?\s+\((\d+)/(\d+)\s+documented\)", stripped)
        if not m:
            continue
        mod_path = m.group(1)
        numerator = int(m.group(2))
        if numerator > 0:
            candidates.append(mod_path)

    # Build per-type targets from each candidate module
    targets: list[tuple[str, str]] = []

    for mod_path in candidates:
        if "test_vector" in mod_path.lower():
            print(f"  (skip) {mod_path} — test data")
            continue

        detail = dispatch("read_rustdoc", {"module": mod_path})
        if detail.startswith("Error:"):
            print(f"  (skip) {mod_path} — {detail[:80]}")
            continue

        # Strip standard trait impl blocks (Clone, Debug, From, etc.)
        detail = _strip_standard_trait_impls(detail)

        sections = _split_detail(detail)
        safe_mod = mod_path.replace("::", "_") or "crate_root"

        type_sections = [s for s in sections if s["kind"] in ("struct", "enum", "trait")]
        func_sections = [s for s in sections if s["kind"] == "function"]

        def _doc_functions(text: str) -> list[str]:
            """Extract names of documented functions from doc-section text."""
            # Format: "fn name(...) -> Type — doc" or "fn name(...) -> Type\n"
            return sorted(set(re.findall(r"fn (\w+)\([^)]*\) -> \S+", text)))

        for s in type_sections:
            name = s["name"]
            safe_filename = f"{safe_mod}__{name}.rs"
            expected = _doc_functions(s["text"])
            header = (
                f"// Module: {mod_path}\n"
                f"// Type: {s['kind']} {name}\n"
                f"// Expected-fns: {', '.join(expected) if expected else '(none)'}\n"
                f"// Kani proof harnesses generated from doc comments.\n\n"
            )
            targets.append((safe_filename, header + s["text"]))
            print(f"  (target) {safe_filename}")

        if func_sections:
            safe_filename = f"{safe_mod}__functions.rs"
            expected = _doc_functions("\n".join(s["text"] for s in func_sections))
            header = (
                f"// Module: {mod_path}\n"
                f"// Standalone functions\n"
                f"// Expected-fns: {', '.join(expected) if expected else '(none)'}\n"
                f"// Kani proof harnesses generated from doc comments.\n\n"
            )
            merged = header + "\n\n".join(s["text"] for s in func_sections)
            targets.append((safe_filename, merged))
            print(f"  (target) {safe_filename}  ({len(func_sections)} functions)")

    return targets


def _check_specs(
    project_path: str, output_dir: str, return_errors: bool = False
) -> bool | tuple[bool, str, list[str]]:
    """Check generated Kani harnesses by compiling with the original crate.

    If *return_errors* is False, returns a bool (True = all ok).
    If *return_errors* is True, returns (ok, stderr_text, merged_file_stems)
    where *merged_file_stems* is used to map harness_N back to spec files.
    """
    import re
    import shutil
    import subprocess
    import tempfile
    import tomllib

    output = Path(output_dir)
    project = Path(project_path)

    # Prefer individual/ files when available (per-function harnesses)
    indiv_dir = output / "individual"
    if indiv_dir.is_dir():
        spec_files = sorted(indiv_dir.glob("*.rs"))
    else:
        spec_files = sorted(f for f in output.glob("*.rs") if f.name != "INDEX")
    spec_file_stems = [sf.stem for sf in spec_files]
    if not spec_files:
        print("  No spec files found to check.")
        return (True, "", []) if return_errors else True

    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)

    # Copy the source crate to a temp directory
    tmp = Path(tempfile.mkdtemp(prefix="autospec_kani_"))
    shutil.copytree(project / "src", tmp / "src")
    shutil.copy2(project / "Cargo.toml", tmp / "Cargo.toml")
    # Copy Cargo.lock if it exists (important for dependency resolution)
    lock = project / "Cargo.lock"
    if lock.exists():
        shutil.copy2(lock, tmp / "Cargo.lock")

    # Read original lib.rs
    lib_rs = tmp / "src" / "lib.rs"
    lib_content = lib_rs.read_text() if lib_rs.exists() else ""

    # Inject harness modules into lib.rs
    harness_mod_names: list[str] = []
    for i, sf in enumerate(spec_files, 1):
        mod_name = f"autospec_harness_{i}"
        harness_mod_names.append(mod_name)

        content = sf.read_text()

        # Determine module path from header, then filename
        mod_match = re.search(r"// Module: (.+)", content)
        mod_path = mod_match.group(1).strip() if mod_match else ""
        if not mod_path:
            stem = sf.stem
            if "__" in stem:
                mod_path = stem.split("__")[0]

        # Inject use imports before the first harness
        use_lines = ["#![allow(dead_code, non_snake_case, unused_variables)]"]
        if mod_path:
            # {self, *} lets agents use both bare fn() and module::fn() forms
            use_lines.append(f"use crate::{mod_path}::{{self, *}};")
        use_lines.append("")

        # Find end of header comments and inject use statements there
        first_non_comment = 0
        for m in re.finditer(
            r"^(?://.*\n)*(?:\n)*", content, re.MULTILINE
        ):
            first_non_comment = m.end()
            break

        if first_non_comment > 0:
            content = (
                content[:first_non_comment]
                + "\n".join(use_lines)
                + "\n"
                + content[first_non_comment:]
            )
        else:
            content = "\n".join(use_lines) + "\n\n" + content

        # Rename to match module name: geometry__Point.rs → autospec_harness_1.rs
        (tmp / "src" / f"{mod_name}.rs").write_text(content)

    # Append harness mod declarations to lib.rs
    mod_decls = "\n// Kani harness modules (generated by autospec)\n"
    mod_decls += "\n".join(
        f"#[cfg(kani)] pub mod {name};" for name in harness_mod_names
    )
    (lib_rs).write_text(lib_content + mod_decls + "\n")

    # Phase 1: fast compilation check (--only-codegen)
    print(f"  Compilation check (cargo kani --only-codegen) ...")
    try:
        result = subprocess.run(
            ["cargo", "kani", "--only-codegen", "--no-assertion-reach-checks"],
            cwd=str(tmp),
            capture_output=True,
            text=True,
            timeout=120,
        )
    except subprocess.TimeoutExpired:
        print("  FAIL: compilation check timed out (2 minutes)")
        shutil.rmtree(tmp)
        return (False, "TIMEOUT after 2 minutes", spec_file_stems) if return_errors else False

    # Show compilation errors/warnings
    stderr_text = result.stderr + "\n" + result.stdout
    for line in stderr_text.split("\n"):
        if any(kw in line for kw in ("error[", "warning[", "aborting",
                                       "could not compile", "Compiling")):
            print(f"    {line}")

    if result.returncode != 0:
        print(f"  FAIL: compilation failed with exit code {result.returncode}")
        shutil.rmtree(tmp)
        return (False, stderr_text, spec_file_stems) if return_errors else False

    print(f"  OK: {len(spec_files)} harness files compile successfully.")
    shutil.rmtree(tmp)
    return (True, "", spec_file_stems) if return_errors else True


def _parse_compilation_errors(
    stderr: str, file_stems: list[str]
) -> dict[str, list[dict]]:
    """Parse cargo kani output into errors grouped by spec file stem.

    Recognises ``error[EXXXX]: message`` blocks with
    ``--> src/autospec_harness_N.rs`` location lines.
    Returns ``{file_stem: [{code, message, harness_file, line, body}]}``
    where *file_stem* is the stem of the individual or merged spec file.
    """
    import re

    # Split into error blocks (each starts with "error[E" at beginning of line)
    error_blocks = re.split(r"\n(?=error\[E\d+\])", stderr)

    errors_by_file: dict[str, list[dict]] = {}

    for block in error_blocks:
        if not block.strip():
            continue

        header_m = re.match(
            r"error\[(E\d+)\]: (.+?)\n\s+-->\s+\S*(autospec_harness_(\d+))\.rs"
            r"(?::(\d+))?(?::(\d+))?",
            block,
        )
        if not header_m:
            continue

        code = header_m.group(1)
        message = header_m.group(2).strip()
        harness_file = header_m.group(3) + ".rs"
        harness_num = int(header_m.group(4))
        line = int(header_m.group(5)) if header_m.group(5) else 0
        col = int(header_m.group(6)) if header_m.group(6) else 0

        # Map harness_N → spec file stem (1:1 for individual files)
        if 1 <= harness_num <= len(file_stems):
            file_stem = file_stems[harness_num - 1]
        else:
            continue

        # Body is everything after the header line
        header_end = block.index("\n") + 1
        body = block[header_end:].strip()

        errors_by_file.setdefault(file_stem, []).append({
            "code": code,
            "message": message,
            "harness_file": harness_file,
            "line": line,
            "col": col,
            "body": body,
        })

    return errors_by_file


def _run_fix_agent(
    project_path: str,
    output_dir: str,
    indiv_filename: str,
    file_content: str,
    errors: list[dict],
    api_key: str,
    base_url: str,
    model: str,
    max_tokens: int,
    verbose: bool,
) -> bool:
    """Run a slim fix agent to repair one individual spec file.

    The agent receives the current file content and compilation errors.
    Returns True if the agent ran without exception.
    """
    import re

    from agent import SpecAgent
    from prompt import FIX_SYSTEM_PROMPT

    # Extract metadata from file headers
    func_name = ""
    mod_path = ""
    type_kind = ""
    type_name = ""
    for line in file_content.split("\n"):
        m = re.search(r"// Spec-for: (\S+)", line)
        if m:
            func_name = m.group(1)
        m = re.search(r"// Module: (.+)", line)
        if m:
            mod_path = m.group(1).strip()
        m = re.search(r"// Type: (\w+) (.+)", line)
        if m:
            type_kind = m.group(1)
            type_name = m.group(2)

    # Format errors
    error_text_parts = []
    for err in errors:
        error_text_parts.append(
            f"error[{err['code']}]: {err['message']}\n"
            f"  location: {err.get('harness_file', '?')}:{err.get('line', '?')}\n"
            f"  details:\n{err.get('body', '').strip()}"
        )
    error_block = "\n\n---\n\n".join(error_text_parts)

    task_parts = [
        "FIX compilation errors in this Kani harness:",
        "",
        f"  Function: {func_name or '(unknown)'}",
        f"  Module: {mod_path or '(unknown)'}",
    ]
    if type_kind:
        task_parts.append(f"  Type: {type_kind} {type_name}")
    task_parts += [
        f"  File: {indiv_filename}",
        "",
        "=== COMPILATION ERRORS ===",
        error_block,
        "",
        "=== CURRENT HARNESS ===",
        "```rust",
        file_content,
        "```",
        "",
        "Fix ONLY the compilation errors above. Preserve all correct parts.",
        "Use read_file to check source code for types/signatures if needed.",
        f"Then call write_spec(filename=\"{indiv_filename}\", content=...) "
        "with the FIXED harness (include all header comments).",
    ]
    task = "\n".join(task_parts)

    try:
        indiv_output = str(Path(output_dir) / "individual")
        agent = SpecAgent(
            project_path=project_path,
            output_dir=indiv_output,
            model=model,
            max_iterations=5,
            max_tokens=max_tokens,
            verbose=verbose,
            api_key=api_key,
            base_url=base_url,
            system_prompt=FIX_SYSTEM_PROMPT,
            tool_filter=["write_spec", "read_file", "search_code"],
        )
        agent.run(task=task, quiet=True)
        return True
    except Exception as exc:
        print(f"  [FIX-FAIL] {indiv_filename}: {type(exc).__name__}: {exc}",
              file=sys.stderr)
        return False


def _auto_fix_checks(
    project_path: str,
    output_dir: str,
    max_rounds: int,
    api_key: str,
    base_url: str,
    model: str,
    max_tokens: int,
    verbose: bool,
) -> bool:
    """Run compilation check with auto-fix retry loop.

    Each individual harness file is compiled as a separate module, so
    ``harness_N`` maps 1:1 to the Nth individual file.  Fix agents write
    directly to ``individual/`` — no merge needed in the hot path.

    Returns True if compilation passes (on first try or after fixing).
    """
    import threading
    from concurrent.futures import ThreadPoolExecutor, as_completed

    for round_num in range(1, max_rounds + 1):
        if round_num == 1:
            print(f"\n[2] Compilation check (round {round_num}/{max_rounds}) ...")
        else:
            print(f"\n[2] Compilation check (retry round {round_num}/{max_rounds}) ...")

        ok, stderr, file_stems = _check_specs(
            project_path, output_dir, return_errors=True
        )

        if ok:
            print(f"  OK: compilation passed.")
            return True

        # Parse errors — harness_N maps 1:1 to individual file stem
        errors_by_file = _parse_compilation_errors(stderr, file_stems)
        if not errors_by_file:
            print("  Could not parse compilation errors from stderr.")
            return False

        total_errors = sum(len(v) for v in errors_by_file.values())
        print(f"  Found {total_errors} error(s) in {len(errors_by_file)} file(s):")
        for stem, errs in errors_by_file.items():
            codes = sorted(set(e["code"] for e in errs))
            print(f"    {stem}.rs: {len(errs)} error(s) {codes}")

        if round_num == max_rounds:
            print(f"  FAIL: compilation still broken after {max_rounds} round(s).")
            return False

        # Build fix tasks directly — no reverse-mapping needed
        fix_tasks: list[tuple[str, list[dict]]] = [
            (f"{stem}.rs", errs) for stem, errs in errors_by_file.items()
        ]

        print(f"\n  Running fix agents for {len(fix_tasks)} file(s)...")

        print_lock = threading.Lock()
        fixed = 0
        fix_failed = 0

        def run_fix(indiv_fn, errs):
            indiv_path = Path(output_dir) / "individual" / indiv_fn
            if not indiv_path.is_file():
                return indiv_fn, False
            content = indiv_path.read_text()
            ok_fix = _run_fix_agent(
                project_path=project_path,
                output_dir=output_dir,
                indiv_filename=indiv_fn,
                file_content=content,
                errors=errs,
                api_key=api_key,
                base_url=base_url,
                model=model,
                max_tokens=max_tokens,
                verbose=verbose,
            )
            return indiv_fn, ok_fix

        workers = min(len(fix_tasks), 5)
        with ThreadPoolExecutor(max_workers=workers) as pool:
            futures = {
                pool.submit(run_fix, ifn, errs): ifn
                for ifn, errs in fix_tasks
            }
            for future in as_completed(futures):
                ifn, ok_fix = future.result()
                with print_lock:
                    if ok_fix:
                        fixed += 1
                        print(f"    [{fixed + fix_failed}/{len(fix_tasks)}] {ifn}  FIXED")
                    else:
                        fix_failed += 1
                        print(f"    [{fixed + fix_failed}/{len(fix_tasks)}] {ifn}  FAILED")

    return False


def _verify_specs(project_path: str, output_dir: str, timeout: int = 120) -> tuple[bool, int, int, int]:
    """Run full Kani model-checking verification (SLOW)."""
    import re
    import shutil
    import subprocess
    import tempfile
    import tomllib

    output = Path(output_dir)
    project = Path(project_path)

    indiv_dir = output / "individual"
    if indiv_dir.is_dir():
        spec_files = sorted(indiv_dir.glob("*.rs"))
    else:
        spec_files = sorted(f for f in output.glob("*.rs") if f.name != "INDEX")
    if not spec_files:
        print("  No spec files found.")
        return True

    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)

    tmp = Path(tempfile.mkdtemp(prefix="autospec_kani_"))
    shutil.copytree(project / "src", tmp / "src")
    shutil.copy2(project / "Cargo.toml", tmp / "Cargo.toml")
    lock = project / "Cargo.lock"
    if lock.exists():
        shutil.copy2(lock, tmp / "Cargo.lock")

    lib_rs = tmp / "src" / "lib.rs"
    lib_content = lib_rs.read_text() if lib_rs.exists() else ""

    harness_mod_names: list[str] = []
    for i, sf in enumerate(spec_files, 1):
        mod_name = f"autospec_harness_{i}"
        harness_mod_names.append(mod_name)
        content = sf.read_text()
        mod_match = re.search(r"// Module: (.+)", content)
        mod_path = mod_match.group(1).strip() if mod_match else ""
        if not mod_path:
            stem = sf.stem
            if "__" in stem:
                mod_path = stem.split("__")[0]
        use_lines = ["#![allow(dead_code, non_snake_case, unused_variables)]"]
        if mod_path:
            # {self, *} lets agents use both bare fn() and module::fn() forms
            use_lines.append(f"use crate::{mod_path}::{{self, *}};")
        use_lines.append("")
        first_non_comment = 0
        for m in re.finditer(r"^(?://.*\n)*(?:\n)*", content, re.MULTILINE):
            first_non_comment = m.end()
            break
        if first_non_comment > 0:
            content = content[:first_non_comment] + "\n".join(use_lines) + "\n" + content[first_non_comment:]
        else:
            content = "\n".join(use_lines) + "\n\n" + content
        (tmp / "src" / f"{mod_name}.rs").write_text(content)

    mod_decls = "\n// Kani harness modules (generated by autospec)\n"
    mod_decls += "\n".join(
        f"#[cfg(kani)] pub mod {name};" for name in harness_mod_names
    )
    lib_rs.write_text(lib_content + mod_decls + "\n")

    # Extract harness function names from spec files
    import re as _re
    harnesses: list[tuple[str, str, str]] = []  # (source_file, mod_name, harness_name)
    for i, sf in enumerate(spec_files, 1):
        mod_name = harness_mod_names[i - 1]
        for m in _re.finditer(r"#\[kani::proof\]\s*pub fn (\w+)\(", sf.read_text()):
            harnesses.append((sf.stem, mod_name, m.group(1)))

    if not harnesses:
        print("  No harness functions found.")
        shutil.rmtree(tmp)
        return False

    # Run harnesses one at a time so fast ones pass and only slow ones timeout.
    # Use process groups to cleanly kill CBMC subprocesses on timeout.
    total = len(harnesses)
    passed = 0
    failed = 0
    timed_out = 0

    for i, (source, mod_name, fn_name) in enumerate(harnesses, 1):
        fq_name = f"{mod_name}::{fn_name}"
        print(f"    [{i}/{total}] {source}::{fn_name} ... ", end="", flush=True)
        proc = subprocess.Popen(
            ["cargo", "kani", "--harness", fq_name, "--default-unwind", "3"],
            cwd=str(tmp),
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            start_new_session=True,  # process group for clean kill
        )

        import select
        deadline = time.time() + timeout
        killed = False

        try:
            while proc.poll() is None:
                if time.time() > deadline:
                    os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
                    killed = True
                    timed_out += 1
                    print(f"TIMEOUT ({timeout}s)")
                    break

                ready, _, _ = select.select([proc.stdout], [], [], 1.0)
                if ready:
                    # Drain stdout silently
                    proc.stdout.readline()
        except Exception:
            if not killed:
                os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
                failed += 1
        finally:
            # Drain any remaining output before waiting
            try:
                while True:
                    if not proc.stdout.readline():
                        break
            except Exception:
                pass
            try:
                proc.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(os.getpgid(proc.pid), signal.SIGKILL)
                proc.wait()

        if not killed:
            if proc.returncode == 0:
                passed += 1
                print("PASS")
            else:
                failed += 1
                print(f"FAIL (exit={proc.returncode})")

        # Failsafe: kill any CBMC processes leaked from this run
        try:
            import subprocess as _sp
            _sp.run(["pkill", "-f", f"cbmc.*{tmp.name}"], timeout=5,
                    stdout=_sp.DEVNULL, stderr=_sp.DEVNULL)
        except Exception:
            pass

    shutil.rmtree(tmp)

    print(f"\n  Results: {passed} passed, {failed} failed, {timed_out} timed out")
    return failed == 0 and timed_out == 0, passed, failed, timed_out


def _generate_report(
    project_path: str,
    output_dir: str,
    run_verify: bool = False,
    kani_timeout: int = 120,
    verify_counts: tuple[int, int, int] | None = None,
) -> None:
    """Generate a structured report: doc coverage → specs → Kani results.

    If *verify_counts* is provided as (passed, failed, timed_out), those
    values are used instead of re-running verification.
    """
    import json
    import tomllib

    from rustdoc import RustDoc, RustItem

    project = Path(project_path)
    output = Path(output_dir)

    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)

    # ── 1. Doc coverage from rustdoc JSON ──────────────────────────────
    from tools.doc_json import _find_doc_json

    json_path = _find_doc_json(project, crate_name)
    if not json_path:
        print("Error: No rustdoc JSON found. Run with --check or --check-only first.")
        return

    doc = RustDoc(str(json_path))

    # Walk modules and count documented public items
    total_documented = 0
    total_public = 0
    documented_by_kind: dict[str, int] = {}
    public_by_kind: dict[str, int] = {}
    documented_modules = 0
    total_modules = 0
    documented_methods = 0
    total_methods = 0

    def walk_module(item_id: int) -> None:
        nonlocal total_documented, total_public, documented_modules, total_modules
        nonlocal documented_methods, total_methods
        for child in doc.children(item_id):
            if child.visibility != "public":
                continue
            if child.kind == "module":
                total_modules += 1
                if child.docs:
                    documented_modules += 1
                walk_module(child.id)
            elif child.kind in ("struct", "enum", "trait", "function", "const",
                                 "type", "static", "union"):
                total_public += 1
                kind = child.kind
                public_by_kind[kind] = public_by_kind.get(kind, 0) + 1
                if child.docs:
                    total_documented += 1
                    documented_by_kind[kind] = documented_by_kind.get(kind, 0) + 1
                # Count documented methods from impl blocks (skip external traits)
                for trait_name, methods, is_external in doc.get_impls(child.id):
                    if is_external:
                        continue
                    for mname, mdocs, _m_item in methods:
                        if mname is not None:
                            total_methods += 1
                            if mdocs:
                                documented_methods += 1

    walk_module(doc.root_id)

    # Source line counts
    src_dir = project / "src"
    source_lines = 0
    source_files = 0
    if src_dir.is_dir():
        for rs in sorted(src_dir.rglob("*.rs")):
            # Skip harness files injected by earlier runs
            if "autospec_harness" in rs.name:
                continue
            source_files += 1
            source_lines += len(rs.read_text().splitlines())

    # ── 2. Spec generation from specs.json ─────────────────────────────
    specs_json = output / "specs.json"
    spec_files_count = 0
    total_harnesses = 0
    spec_funcs: set[str] = set()
    spec_entries = []
    spec_status_ok = 0
    spec_status_failed = 0

    if specs_json.is_file():
        manifest = json.loads(specs_json.read_text())
        spec_entries = manifest.get("entries", [])
        spec_files_count = len(spec_entries)
        spec_status_ok = manifest.get("succeeded", 0)
        spec_status_failed = manifest.get("failed", 0)
        for entry in spec_entries:
            total_harnesses += 1
            func = entry.get("source_function", "")
            mod = entry.get("module", "")
            if func and mod:
                spec_funcs.add(f"{mod}::{func}")
            elif func:
                spec_funcs.add(func)

    # ── 3. Kani verification ──────────────────────────────────────────
    verify_passed = verify_failed = verify_timeout = -1  # -1 = not run
    if verify_counts is not None:
        verify_passed, verify_failed, verify_timeout = verify_counts
    elif run_verify and spec_files_count > 0:
        print("\n  Running Kani verification ...\n")
        _, verify_passed, verify_failed, verify_timeout = _verify_specs(
            project_path, output_dir, timeout=kani_timeout
        )

    # ── 4. Print report ────────────────────────────────────────────────
    type_docs = documented_by_kind.get('struct', 0) + documented_by_kind.get('enum', 0) + documented_by_kind.get('trait', 0)
    type_pub = public_by_kind.get('struct', 0) + public_by_kind.get('enum', 0) + public_by_kind.get('trait', 0)
    func_docs = documented_by_kind.get('function', 0)
    func_pub = public_by_kind.get('function', 0)

    total_documentable = total_documented + documented_methods
    total_pub = total_public + total_methods
    doc_pct = (total_documentable / total_pub * 100) if total_pub > 0 else 0.0

    # Coverage: only functions and methods produce harnesses, not types
    spec_targets = func_docs + documented_methods
    spec_cov_pct = (len(spec_funcs) / spec_targets * 100) if spec_targets > 0 else 0.0

    print()
    print("=" * 60)
    print(f"  Autospec Report — {crate_name}")
    print("=" * 60)

    print(f"\n── Documentation")
    print(f"  Modules with docs:  {documented_modules}/{total_modules}")
    print(f"  Documented items:   {total_documentable}/{total_pub}  ({doc_pct:.0f}%)")
    print(f"    Types:            {type_docs}/{type_pub}")
    print(f"    Functions:        {func_docs}/{func_pub}")
    print(f"    Methods:          {documented_methods}/{total_methods}")
    print(f"  Source files:       {source_files}  ({source_lines} lines)")

    print(f"\n── Specifications")
    print(f"  Spec files:         {spec_files_count}")
    print(f"  Total harnesses:    {total_harnesses}")
    print(f"  Coverage:           {len(spec_funcs)}/{spec_targets} functions & methods  ({spec_cov_pct:.0f}%)")
    print(f"  Generation status:  {spec_status_ok} ok, {spec_status_failed} failed")

    if verify_passed >= 0:
        total_verified = verify_passed + verify_failed + verify_timeout
        verify_pct = (verify_passed / total_verified * 100) if total_verified > 0 else 0.0
        print(f"\n── Kani Verification")
        print(f"  Total harnesses:    {total_verified}")
        print(f"  Passed:             {verify_passed}")
        print(f"  Failed:             {verify_failed}")
        print(f"  Timed out:          {verify_timeout}")
        print(f"  Pass rate:          {verify_pct:.0f}%")
    else:
        print(f"\n── Kani Verification  (not run — use --verify to include)")

    print()
    print("=" * 60)


def _structural_check(output_dir: str) -> list[str]:
    """Basic structural validation of Kani harness files without compilation.

    Returns a list of warnings (empty = all good).
    """
    import re

    output = Path(output_dir)
    warnings: list[str] = []

    # Scan individual/ when it exists, otherwise scan merged files
    indiv_dir = output / "individual"
    if indiv_dir.is_dir():
        spec_files = sorted(indiv_dir.glob("*.rs"))
    else:
        spec_files = sorted(
            f for f in output.glob("*.rs") if f.name != "INDEX"
        )

    for sf in spec_files:
        content = sf.read_text()
        size = sf.stat().st_size

        if size < 200:
            warnings.append(f"{sf.name}: too small ({size} bytes), likely empty")
            continue

        # Count harness blocks (comment + #[kani::proof] + pub fn)
        spec_blocks = len(re.findall(r"/\*\nDescription:", content))
        kani_proofs = len(re.findall(r"#\[kani::proof\]", content))
        spec_for_entries = len(re.findall(r"Spec-for:", content))

        if spec_blocks == 0:
            warnings.append(f"{sf.name}: no Description blocks found")
        if kani_proofs == 0:
            warnings.append(f"{sf.name}: no #[kani::proof] harnesses")
        if spec_for_entries == 0:
            warnings.append(f"{sf.name}: no Spec-for: mappings (function→harness links)")
        if spec_blocks != kani_proofs:
            warnings.append(
                f"{sf.name}: mismatch — {spec_blocks} Description blocks but "
                f"{kani_proofs} #[kani::proof] harnesses"
            )
        if spec_for_entries < kani_proofs:
            warnings.append(
                f"{sf.name}: only {spec_for_entries} Spec-for: entries for "
                f"{kani_proofs} harnesses"
            )

        # Check no unimplemented/todo placeholders
        if "unimplemented!" in content:
            warnings.append(f"{sf.name}: contains unimplemented!() calls")
        if "todo!(" in content:
            warnings.append(f"{sf.name}: contains todo!() calls")

        # Check Expected-fns coverage vs Spec-for entries
        expected_match = re.search(r"// Expected-fns: (.+)", content)
        if expected_match:
            expected_str = expected_match.group(1).strip()
            if expected_str != "(none)":
                expected_fns = set(n.strip() for n in expected_str.split(","))
                covered = set(re.findall(r"Spec-for: (\S+)", content))
                missing = expected_fns - covered
                extra = covered - expected_fns
                if missing:
                    warnings.append(
                        f"{sf.name}: missing harnesses for: {', '.join(sorted(missing))}"
                    )
                if extra:
                    warnings.append(
                        f"{sf.name}: unexpected Spec-for entries (not in Expected-fns): "
                        f"{', '.join(sorted(extra))}"
                    )

        # Check for duplicate harness function names
        fn_names = re.findall(r"#\[kani::proof\]\s*pub fn (\w+)\(", content)
        seen: set[str] = set()
        for name in fn_names:
            if name in seen:
                warnings.append(f"{sf.name}: duplicate harness — pub fn {name}() defined more than once")
            seen.add(name)

        # Count assert! calls (Kani treats them as proof obligations)
        proof_asserts = len(re.findall(r"assert!?\(", content))
        if proof_asserts == 0:
            warnings.append(
                f"{sf.name}: no assert!() calls — harness may be empty"
            )

    return warnings


def _write_spec_json(
    output_dir: str,
    crate_name: str,
    func_results: list[tuple[str, str, str, bool]],
) -> None:
    """Write a structured specs.json manifest with per-function entries."""
    import json
    import re

    entries: list[dict] = []
    for indiv_filename, merged_filename, func_name, ok in func_results:
        indiv_path = Path(output_dir) / indiv_filename
        size = indiv_path.stat().st_size if indiv_path.is_file() else 0

        desc = ""
        mod_path = ""
        if indiv_path.is_file():
            content = indiv_path.read_text()
            desc_match = re.search(r"Description: (.+)", content)
            if desc_match:
                desc = desc_match.group(1).strip()
            mod_match = re.search(r"// Module: (.+)", content)
            if mod_match:
                mod_path = mod_match.group(1).strip()

        entries.append({
            "file": indiv_filename,
            "merged_file": merged_filename,
            "source_function": func_name,
            "module": mod_path,
            "description": desc,
            "size_bytes": size,
            "status": "ok" if ok else "failed",
        })

    manifest = {
        "crate": crate_name,
        "total_harnesses": len(func_results),
        "succeeded": sum(1 for _, _, _, ok in func_results if ok),
        "failed": sum(1 for _, _, _, ok in func_results if not ok),
        "entries": entries,
    }

    json_path = Path(output_dir) / "specs.json"
    json_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Wrote specs.json → {json_path}")


def _run_split(
    project: Path,
    output: Path,
    api_key: str,
    base_url: str,
    model: str,
    max_iterations: int,
    max_tokens: int,
    verbose: bool,
    jobs: int = 5,
) -> None:
    """Divide and conquer: one slim agent per documented function.

    Agents run in parallel (*jobs* concurrent workers) since each function
    is independent.
    """
    import threading
    from concurrent.futures import ThreadPoolExecutor, as_completed

    project_path = str(project)
    output_dir = str(output)

    print("\nPhase 1: Generating rustdoc JSON and scanning for documented types...")
    targets = _get_spec_targets(project_path, output_dir)

    # Parse individual functions from all targets
    all_funcs: list[tuple[str, str, list[dict]]] = []  # (filename, header, [funcs])
    for filename, doc_section in targets:
        functions = _parse_functions_from_section(doc_section)
        if not functions:
            continue
        # Extract the header (everything before the first ## section)
        header_end = doc_section.find("\n## ")
        header = doc_section[:header_end].strip() + "\n\n" if header_end > 0 else ""
        all_funcs.append((filename, header, functions))

    total_funcs = sum(len(funcs) for _, _, funcs in all_funcs)
    print(f"\nParsed {total_funcs} documented functions across {len(all_funcs)} files.")

    if not all_funcs:
        print("No documented functions found. Nothing to do.")
        return

    # Spawn agents in parallel
    func_iterations = max(3, max_iterations)
    print(f"\nPhase 3: Generating harnesses — {jobs} parallel agents "
          f"({func_iterations} iterations max each, {total_funcs} functions)...")

    # Flatten into (indiv_filename, merged_filename, func_info) tasks
    tasks: list[tuple[str, str, dict]] = []
    for merged_filename, _header, functions in all_funcs:
        for func_info in functions:
            module = func_info.get("module", "").replace("::", "_") or "crate_root"
            type_part = func_info.get("type_name") or "functions"
            indiv_filename = f"individual/{module}__{type_part}__{func_info['name']}.rs"
            tasks.append((indiv_filename, merged_filename, func_info))

    func_results: list[tuple[str, str, str, bool]] = []  # (indiv, merged, name, ok)
    print_lock = threading.Lock()
    completed = 0

    def run_one(indiv_fn: str, merged_fn: str, func_info: dict) -> tuple[str, str, str, bool]:
        """Run a single function agent. Returns (indiv_fn, merged_fn, name, ok)."""
        ok = _run_func_agent(
            project_path=project_path,
            output_dir=output_dir,
            filename=indiv_fn,
            func_info=func_info,
            api_key=api_key,
            base_url=base_url,
            model=model,
            max_iterations=func_iterations,
            max_tokens=max_tokens,
            verbose=verbose,
        )
        return indiv_fn, merged_fn, func_info["name"], ok

    with ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = {pool.submit(run_one, ifn, mfn, fi): (ifn, mfn, fi)
                   for ifn, mfn, fi in tasks}

        for future in as_completed(futures):
            indiv_fn, merged_fn, fn_name, ok = future.result()
            func_results.append((indiv_fn, merged_fn, fn_name, ok))
            with print_lock:
                completed += 1
                status = "OK" if ok else "FAIL"
                print(f"  [{completed}/{total_funcs}] {indiv_fn} :: {fn_name}  {status}")

    succeeded = sum(1 for _, _, _, ok in func_results if ok)
    failed = sum(1 for _, _, _, ok in func_results if not ok)

    # Phase 4: Merge individual files by type
    print(f"\nPhase 4: Merging individual files by type...")
    _merge_specs(output_dir)

    # Write INDEX
    index_lines = [
        "# Specification index",
        f"# Generated from {project_path}",
        f"# {succeeded} succeeded, {failed} failed",
        f"# {total_funcs} harnesses across {len(all_funcs)} target types",
        "",
    ]
    # Group by merged file
    merged_groups: dict[str, list[tuple[str, str, str, bool]]] = {}
    for indiv_fn, merged_fn, name, ok in func_results:
        merged_groups.setdefault(merged_fn, []).append((indiv_fn, merged_fn, name, ok))

    for merged_fn in sorted(merged_groups):
        merged_path = output / merged_fn
        merged_size = merged_path.stat().st_size if merged_path.is_file() else 0
        index_lines.append(f"\n## {merged_fn} ({merged_size:,} bytes)")
        for indiv_fn, _, name, ok in merged_groups[merged_fn]:
            indiv_path = output / indiv_fn
            indiv_size = indiv_path.stat().st_size if indiv_path.is_file() else 0
            status = "OK" if ok else "FAIL"
            index_lines.append(
                f"  - [{indiv_fn}](./{indiv_fn}) :: {name}  {status}  ({indiv_size:,} bytes)"
            )
    index_path = output / "INDEX"
    index_path.write_text("\n".join(index_lines) + "\n")
    print(f"\nWrote INDEX → {index_path}")
    print(f"Done: {succeeded} succeeded, {failed} failed "
          f"({total_funcs} harnesses total)")

    # Write structured JSON manifest
    import tomllib
    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)
    _write_spec_json(output_dir, crate_name, func_results)


def main() -> None:
    parser = argparse.ArgumentParser(
        prog="autospec",
        description="Generate formal specifications for Rust projects from doc strings",
    )
    parser.add_argument(
        "project_path",
        help="Path to the Rust project root (must contain Cargo.toml)",
    )
    parser.add_argument(
        "--output", "-o",
        default="specs",
        help="Output directory (default: specs/)",
    )
    parser.add_argument(
        "--model", "-m",
        default="claude-sonnet-4-6-20250514",
        help="Anthropic model ID (default: claude-sonnet-4-6-20250514)",
    )
    parser.add_argument(
        "--max-iterations", "-n",
        type=int,
        default=30,
        help="Maximum tool-calling iterations per agent (default: 30, 5 in --split)",
    )
    parser.add_argument(
        "--max-tokens",
        type=int,
        default=4096,
        help="Maximum tokens per model response (default: 4096)",
    )
    parser.add_argument(
        "--verbose", "-v",
        action="store_true",
        help="Print detailed progress to stderr",
    )
    parser.add_argument(
        "--split", "-s",
        action="store_true",
        help="Divide and conquer: run one slim agent per documented function, "
             "each writing one harness via write_spec(append=True).",
    )
    parser.add_argument(
        "--jobs", "-j",
        type=int,
        default=5,
        help="Number of parallel agents in --split mode (default: 5).",
    )
    parser.add_argument(
        "--check", "-c",
        action="store_true",
        help="After generation, compile specs against the original crate "
             "and run structural validation.",
    )
    parser.add_argument(
        "--check-only",
        action="store_true",
        help="Skip generation; only run checks on existing specs in --output.",
    )
    parser.add_argument(
        "--auto-fix",
        type=int,
        const=2,
        nargs="?",
        default=None,
        metavar="N",
        help="Auto-fix compilation errors with LLM fix agents, up to N rounds "
             "(default: 2 if --auto-fix is given without a value). "
             "Requires ANTHROPIC_API_KEY.",
    )
    parser.add_argument(
        "--verify",
        action="store_true",
        help="Run full Kani model-checking verification.",
    )
    parser.add_argument(
        "--kani-timeout",
        type=int,
        default=120,
        help="Timeout in seconds for each Kani verification run (default: 120).",
    )
    parser.add_argument(
        "--report",
        action="store_true",
        help="Generate a structured report: doc coverage → specs → Kani results.",
    )
    parser.add_argument(
        "--merge",
        action="store_true",
        help="Merge individual/ per-function files into per-type bundled files and exit.",
    )

    args = parser.parse_args()

    project = Path(args.project_path).resolve()
    if not project.is_dir():
        print(f"Error: {project} is not a directory", file=sys.stderr)
        sys.exit(1)
    if not (project / "Cargo.toml").is_file():
        print(f"Error: Cargo.toml not found in {project}", file=sys.stderr)
        sys.exit(1)

    output = Path(args.output).resolve()
    print(f"Project:  {project}")
    print(f"Output:   {output}")
    print(f"Model:    {args.model}")
    print(f"Mode:     {'split (per-type agents)' if args.split else 'monolithic'}")
    print()

    # --merge: just merge individual files into bundled output
    if args.merge:
        _merge_specs(str(output))
        return

    # --check-only: skip generation, just validate
    if args.check_only:
        api_key = ""
        base_url = ""
        auto_fix = args.auto_fix or 0
        if auto_fix > 0:
            api_key = os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")
            if not api_key:
                print("Error: --auto-fix requires ANTHROPIC_API_KEY", file=sys.stderr)
                sys.exit(1)
            base_url = os.environ.get("ANTHROPIC_BASE_URL", "")
        vcounts = _run_checks(
            str(project), str(output),
            verify=args.verify,
            kani_timeout=args.kani_timeout,
            auto_fix_rounds=auto_fix,
            api_key=api_key,
            base_url=base_url,
            model=args.model,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
        )
        if args.report:
            _generate_report(str(project), str(output),
                             verify_counts=vcounts, kani_timeout=args.kani_timeout)
        return

    api_key = os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("ANTHROPIC_AUTH_TOKEN")
    if not api_key:
        print("Error: ANTHROPIC_API_KEY or ANTHROPIC_AUTH_TOKEN not set. "
              "Set it in the environment or in ~/.autospec/.env",
              file=sys.stderr)
        sys.exit(1)
    base_url = os.environ.get("ANTHROPIC_BASE_URL", "")

    if args.split:
        _run_split(
            project=project,
            output=output,
            api_key=api_key,
            base_url=base_url,
            model=args.model,
            max_iterations=args.max_iterations,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
            jobs=args.jobs,
        )
    else:
        from agent import SpecAgent

        agent = SpecAgent(
            project_path=str(project),
            output_dir=str(output),
            model=args.model,
            max_iterations=args.max_iterations,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
            api_key=api_key,
            base_url=base_url,
        )
        agent.run()

    print(f"\nDone. Output written to {output}/")

    if args.check or args.auto_fix:
        vcounts = _run_checks(
            str(project), str(output),
            verify=args.verify,
            kani_timeout=args.kani_timeout,
            auto_fix_rounds=args.auto_fix or 0,
            api_key=api_key,
            base_url=base_url,
            model=args.model,
            max_tokens=args.max_tokens,
            verbose=args.verbose,
        )
    else:
        vcounts = None

    if args.report:
        if args.check:
            _generate_report(str(project), str(output),
                             verify_counts=vcounts, kani_timeout=args.kani_timeout)
        else:
            _generate_report(str(project), str(output),
                             run_verify=args.verify, kani_timeout=args.kani_timeout)


def _run_checks(
    project_path: str,
    output_dir: str,
    verify: bool = False,
    kani_timeout: int = 120,
    auto_fix_rounds: int = 0,
    api_key: str = "",
    base_url: str = "",
    model: str = "",
    max_tokens: int = 4096,
    verbose: bool = False,
) -> tuple[int, int, int] | None:
    """Run all spec validation checks.  Returns (passed, failed, timed_out) or None.

    If *auto_fix_rounds* > 0, compilation failures trigger fix agents and
    retry up to that many rounds.
    """
    print("=" * 60)
    print("Specification validation")
    print("=" * 60)

    # 1. Structural checks
    print("\n[1] Structural checks ...")
    warnings = _structural_check(output_dir)
    if warnings:
        for w in warnings:
            print(f"  WARN: {w}")
    else:
        print("  OK: no structural issues detected.")

    # 2. Compilation check (with optional auto-fix)
    if auto_fix_rounds > 0:
        compilation_ok = _auto_fix_checks(
            project_path=project_path,
            output_dir=output_dir,
            max_rounds=auto_fix_rounds,
            api_key=api_key,
            base_url=base_url,
            model=model,
            max_tokens=max_tokens,
            verbose=verbose,
        )
    else:
        print("\n[2] Compilation check ...")
        compilation_ok, _, _ = _check_specs(project_path, output_dir, return_errors=True)

    if compilation_ok:
        if verify:
            print(f"\n[3] Kani verification (timeout={kani_timeout}s) ...")
            _, passed, failed, timed_out = _verify_specs(project_path, output_dir, timeout=kani_timeout)
            return (passed, failed, timed_out)
    return None


if __name__ == "__main__":
    main()
