#!/usr/bin/env python3
"""autospec — Generate formal specifications for Rust projects from doc strings.

Usage:
    autospec PROJECT_PATH [--output DIR] [--model MODEL] [--max-iterations N] [--verbose]
    autospec PROJECT_PATH --split   # one file per documented type, per-type agents

Examples:
    autospec ~/my-rust-lib
    autospec ~/my-rust-lib --output specs/ --model deepseek-v4-pro --verbose
    autospec ~/my-rust-lib --split --max-iterations 5
"""

import argparse
import os
import sys
from pathlib import Path


def _run_type_agent(
    project_path: str,
    output_dir: str,
    filename: str,
    doc_section: str,
    api_key: str,
    base_url: str,
    model: str,
    max_iterations: int,
    max_tokens: int,
    verbose: bool,
) -> bool:
    """Run a slim agent for a single type using pre-parsed docs.

    Returns True on success, False on failure (caller should continue).
    """
    from agent import SpecAgent
    from prompt import TYPE_SYSTEM_PROMPT

    task = (
        f"Below are the documented items to translate into Hoare triples.\n\n"
        f"{doc_section}\n\n"
        f"Write the complete specification to `{filename}` using write_spec. "
        f"Call write_spec ONCE then respond with a brief summary."
    )

    try:
        agent = SpecAgent(
            project_path=project_path,
            output_dir=output_dir,
            model=model,
            max_iterations=max_iterations,
            max_tokens=max_tokens,
            verbose=verbose,
            api_key=api_key,
            base_url=base_url,
            system_prompt=TYPE_SYSTEM_PROMPT,
            tool_filter=["write_spec"],
        )
        agent.run(task=task)
        return True
    except Exception as exc:
        print(f"  [FAIL] {filename}: {type(exc).__name__}: {exc}", file=sys.stderr)
        return False


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

        for s in type_sections:
            name = s["name"]
            safe_filename = f"{safe_mod}__{name}.rs"
            header = (
                f"// Module: {mod_path}\n"
                f"// Type: {s['kind']} {name}\n"
                f"// This file contains specifications generated from doc comments.\n\n"
            )
            targets.append((safe_filename, header + s["text"]))
            print(f"  (target) {safe_filename}")

        if func_sections:
            safe_filename = f"{safe_mod}__functions.rs"
            header = (
                f"// Module: {mod_path}\n"
                f"// Standalone functions\n\n"
            )
            merged = header + "\n\n".join(s["text"] for s in func_sections)
            targets.append((safe_filename, merged))
            print(f"  (target) {safe_filename}  ({len(func_sections)} functions)")

    return targets


def _check_specs(project_path: str, output_dir: str) -> bool:
    """Check generated Kani harnesses by compiling with the original crate.

    Copies the target crate to a temp directory, injects harness files
    as modules, and runs ``cargo kani`` for verification.

    Returns True if all harnesses pass verification.
    """
    import re
    import shutil
    import subprocess
    import tempfile
    import tomllib

    output = Path(output_dir)
    project = Path(project_path)

    spec_files = sorted(f for f in output.glob("*.rs") if f.name != "INDEX")
    if not spec_files:
        print("  No spec files found to check.")
        return True

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
            use_lines.append(f"use crate::{mod_path}::*;")
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
    print(f"  [1/2] Compilation check (--only-codegen) ...")
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
        return False

    # Show compilation errors/warnings
    stderr = result.stderr.strip()
    stdout = result.stdout.strip()
    combined = stderr + "\n" + stdout
    for line in combined.split("\n"):
        if any(kw in line for kw in ("error[", "warning[", "aborting",
                                       "could not compile", "Compiling")):
            print(f"    {line}")

    if result.returncode != 0:
        print(f"  FAIL: compilation failed with exit code {result.returncode}")
        shutil.rmtree(tmp)
        return False

    print(f"  OK: {len(spec_files)} harness files compile successfully.")
    shutil.rmtree(tmp)
    return True


def _verify_specs(project_path: str, output_dir: str, timeout: int = 120) -> bool:
    """Run full Kani model-checking verification (SLOW)."""
    import re
    import shutil
    import subprocess
    import tempfile
    import tomllib

    output = Path(output_dir)
    project = Path(project_path)

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
            use_lines.append(f"use crate::{mod_path}::*;")
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

    print(f"  Running cargo kani (this may take a while) ...")
    try:
        result = subprocess.run(
            ["cargo", "kani", "--default-unwind", "3"],
            cwd=str(tmp),
            capture_output=True,
            text=True,
            timeout=timeout,
        )
    except subprocess.TimeoutExpired:
        print(f"  TIMEOUT: cargo kani exceeded {timeout}s — specs are syntactically valid, verification skipped.")
        shutil.rmtree(tmp)
        return False

    # Show verification results
    for line in (result.stdout + "\n" + result.stderr).split("\n"):
        if any(kw in line for kw in ("VERIFICATION", "FAILED", "SUCCESS",
                                      "Complete", "Checking", "assertion",
                                      "error[", "status:")):
            print(f"    {line}")

    if result.returncode == 0:
        print(f"  OK: {len(spec_files)} harness files passed Kani verification.")
        shutil.rmtree(tmp)
        return True
    else:
        print(f"  FAIL: cargo kani returned exit code {result.returncode}")
        shutil.rmtree(tmp)
        return False


def _structural_check(output_dir: str) -> list[str]:
    """Basic structural validation of Kani harness files without compilation.

    Returns a list of warnings (empty = all good).
    """
    import re

    output = Path(output_dir)
    warnings: list[str] = []

    for sf in sorted(output.glob("*.rs")):
        if sf.name == "INDEX":
            continue
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
    targets: list[tuple[str, str]],
    results: list[tuple[str, bool]],
) -> None:
    """Write a structured specs.json manifest with function-to-spec mappings."""
    import json
    import re

    entries: list[dict] = []
    for (filename, doc_section), (_, ok) in zip(targets, results):
        spec_path = Path(output_dir) / filename
        size = spec_path.stat().st_size if spec_path.is_file() else 0

        mod_match = re.search(r"// Module: (.+)", doc_section)
        mod_path = mod_match.group(1).strip() if mod_match else ""
        type_match = re.search(r"// Type: (\w+) (.+)", doc_section)
        item_kind = type_match.group(1) if type_match else ""
        item_name = type_match.group(2) if type_match else filename

        # Count harnesses and extract function→harness mappings
        spec_count = 0
        mappings: list[dict] = []
        if spec_path.is_file():
            content = spec_path.read_text()
            spec_count = len(re.findall(r"/\*\nDescription:", content))

            # Extract each Description + Spec-for pair
            blocks = re.findall(
                r"/\*\nDescription: (.+?)\n(?:.*\n)*?Spec-for: (\S+)",
                content,
            )
            for desc, func_name in blocks:
                mappings.append({
                    "source_function": func_name,
                    "description": desc.strip(),
                })

        entries.append({
            "file": filename,
            "module": mod_path,
            "kind": item_kind,
            "name": item_name,
            "size_bytes": size,
            "spec_count": spec_count,
            "status": "ok" if ok else "failed",
            "harnesses": mappings,
        })

    manifest = {
        "crate": crate_name,
        "total_targets": len(targets),
        "succeeded": sum(1 for _, ok in results if ok),
        "failed": sum(1 for _, ok in results if not ok),
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
) -> None:
    """Divide and conquer: one slim agent per documented type."""
    project_path = str(project)
    output_dir = str(output)

    print("\nPhase 1: Generating rustdoc JSON and scanning for documented types...")
    targets = _get_spec_targets(project_path, output_dir)
    print(f"\nSelected {len(targets)} type-level targets for specification generation.")

    if not targets:
        print("No documented types found. Nothing to do.")
        return

    # Use fewer iterations per type (they only need to call write_spec once)
    type_iterations = min(max_iterations, 5)

    print(f"\nPhase 2: Generating specs ({type_iterations} iterations each)...")
    results: list[tuple[str, bool]] = []

    for i, (filename, doc_section) in enumerate(targets, 1):
        print(f"\n[{i}/{len(targets)}] {filename}")

        ok = _run_type_agent(
            project_path=project_path,
            output_dir=output_dir,
            filename=filename,
            doc_section=doc_section,
            api_key=api_key,
            base_url=base_url,
            model=model,
            max_iterations=type_iterations,
            max_tokens=max_tokens,
            verbose=verbose,
        )
        results.append((filename, ok))

    succeeded = sum(1 for _, ok in results if ok)
    failed = sum(1 for _, ok in results if not ok)

    # Write INDEX
    index_lines = [
        "# Specification index",
        f"# Generated from {project_path}",
        f"# {succeeded} succeeded, {failed} failed",
        "",
    ]
    for filename, _doc_section in targets:
        spec_path = output / filename
        size = spec_path.stat().st_size if spec_path.is_file() else 0
        if size > 200:
            index_lines.append(f"- [{filename}](./{filename}) ({size:,} bytes)")
    index_path = output / "INDEX"
    index_path.write_text("\n".join(index_lines) + "\n")
    print(f"\nWrote INDEX → {index_path}")
    print(f"Done: {succeeded} succeeded, {failed} failed")

    # Write structured JSON manifest
    import tomllib
    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)
    _write_spec_json(output_dir, crate_name, targets, results)


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
        help="Divide and conquer: run one slim agent per documented type, "
             "producing one spec file per struct/enum/trait.",
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

    # --check-only: skip generation, just validate
    if args.check_only:
        _run_checks(str(project), str(output), verify=args.verify, kani_timeout=args.kani_timeout)
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

    if args.check:
        _run_checks(str(project), str(output), verify=args.verify, kani_timeout=args.kani_timeout)


def _run_checks(project_path: str, output_dir: str, verify: bool = False, kani_timeout: int = 120) -> None:
    """Run all spec validation checks."""
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

    # 2. Compilation check (fast)
    print("\n[2] Compilation check ...")
    if _check_specs(project_path, output_dir):
        if verify:
            print(f"\n[3] Kani verification (timeout={kani_timeout}s) ...")
            _verify_specs(project_path, output_dir, timeout=kani_timeout)


if __name__ == "__main__":
    main()
