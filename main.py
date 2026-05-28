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

from dotenv import load_dotenv

load_dotenv(Path.home() / ".autospec" / ".env")
load_dotenv()


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
    """Check generated specs by compiling them against the original crate.

    Creates a temporary Cargo project that depends on the target crate
    and includes all spec files as modules. Runs ``cargo check`` and
    reports errors.

    Returns True if all specs pass compilation.
    """
    import re
    import shutil
    import subprocess
    import tempfile
    import tomllib

    output = Path(output_dir)
    project = Path(project_path)

    spec_files = sorted(output.glob("*.rs"))
    if not spec_files:
        print("  No spec files found to check.")
        return True

    # Read crate name from Cargo.toml
    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)

    # Create temp Cargo project
    tmp = Path(tempfile.mkdtemp(prefix="autospec_check_"))
    (tmp / "src").mkdir(parents=True)

    (tmp / "Cargo.toml").write_text(
        f'[package]\n'
        f'name = "autospec-check"\n'
        f'version = "0.0.0"\n'
        f'edition = "2021"\n\n'
        f'[dependencies]\n'
        f'{crate_name} = {{ path = "{project}" }}\n'
    )

    # Build main.rs that declares each spec as a module
    main_lines = [
        "#![allow(dead_code, non_snake_case, unused_variables)]",
        "fn main() {}",
        "",
    ]
    for sf in spec_files:
        mod_name = sf.stem.replace("-", "_").replace(".", "_")
        # Also declare the module
        main_lines.append(f"#[path = \"{sf.name}\"]")
        main_lines.append(f"mod {mod_name};")

    # Copy spec files and preprocess them to add imports
    for sf in spec_files:
        content = sf.read_text()

        # Determine module path: first from header, then from filename
        mod_match = re.search(r"// Module: (.+)", content)
        mod_path = mod_match.group(1).strip() if mod_match else ""

        if not mod_path:
            # Infer from filename: "math__functions.rs" → "math"
            stem = sf.stem
            if "__" in stem:
                mod_path = stem.split("__")[0]

        # Build use statements for the target crate's module
        use_imports = []
        if mod_path:
            use_imports.append(f"use {crate_name}::{mod_path}::*;")
        else:
            use_imports.append(f"use {crate_name}::*;")

        # Inject use statements: after header block if present, else at top
        end_of_header = 0
        for m in re.finditer(r"^// (?:Module|Type|This file).*\n", content, re.MULTILINE):
            end_of_header = m.end()

        if end_of_header > 0:
            content = (
                content[:end_of_header]
                + "\n"
                + "\n".join(use_imports)
                + "\n"
                + content[end_of_header:]
            )
        else:
            content = "\n".join(use_imports) + "\n\n" + content

        (tmp / "src" / sf.name).write_text(content)

    (tmp / "src" / "main.rs").write_text("\n".join(main_lines) + "\n")

    print(f"  Running cargo check in {tmp} ...")
    result = subprocess.run(
        ["cargo", "check"],
        cwd=str(tmp),
        capture_output=True,
        text=True,
        timeout=120,
    )

    # Show relevant output
    stderr = result.stderr.strip()
    if stderr:
        # Filter to error/warning lines
        for line in stderr.split("\n"):
            if any(kw in line for kw in ("error", "warning", "-->", "|")):
                print(f"    {line}")

    # Clean up
    shutil.rmtree(tmp)

    if result.returncode == 0:
        print(f"  OK: {len(spec_files)} spec files pass compilation.")
        return True
    else:
        print(f"  FAIL: cargo check returned exit code {result.returncode}")
        return False


def _structural_check(output_dir: str) -> list[str]:
    """Basic structural validation of spec files without compilation.

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

        # Count spec pairs (comment block + precondition + specification)
        spec_blocks = len(re.findall(r"/\*\nDescription:", content))
        prec_fns = len(re.findall(r"pub fn \w+_precondition\(", content))
        spec_fns = len(re.findall(r"pub fn \w+_specification\(", content))

        if spec_blocks == 0:
            warnings.append(f"{sf.name}: no specification blocks found")
        if prec_fns == 0:
            warnings.append(f"{sf.name}: no _precondition functions")
        if spec_fns == 0:
            warnings.append(f"{sf.name}: no _specification functions")
        if prec_fns > spec_blocks:
            warnings.append(
                f"{sf.name}: more preconditions ({prec_fns}) than spec blocks ({spec_blocks})"
            )
        if spec_fns != spec_blocks:
            warnings.append(
                f"{sf.name}: mismatch — {spec_blocks} blocks but "
                f"{spec_fns} spec functions"
            )

        # Check no unimplemented/todo placeholders
        if "unimplemented!" in content:
            warnings.append(f"{sf.name}: contains unimplemented!() calls")
        if "todo!(" in content:
            warnings.append(f"{sf.name}: contains todo!() calls")

        # Check comments that look like missing assertions
        comment_only = len(re.findall(r"^\s*// .*(result|assert|should)", content, re.MULTILINE))
        if comment_only > 3:
            warnings.append(
                f"{sf.name}: {comment_only} comment lines that look like "
                f"missing assertions"
            )

    return warnings


def _write_spec_json(
    output_dir: str,
    crate_name: str,
    targets: list[tuple[str, str]],
    results: list[tuple[str, bool]],
) -> None:
    """Write a structured specs.json manifest alongside the spec files."""
    import json
    import re

    entries: list[dict] = []
    for (filename, doc_section), (_, ok) in zip(targets, results):
        # Parse metadata from doc section header and spec file
        spec_path = Path(output_dir) / filename
        size = spec_path.stat().st_size if spec_path.is_file() else 0

        mod_match = re.search(r"// Module: (.+)", doc_section)
        mod_path = mod_match.group(1).strip() if mod_match else ""
        type_match = re.search(r"// Type: (\w+) (.+)", doc_section)
        item_kind = type_match.group(1) if type_match else ""
        item_name = type_match.group(2) if type_match else filename

        # Count spec pairs in the generated output
        spec_count = 0
        if spec_path.is_file():
            content = spec_path.read_text()
            spec_count = len(re.findall(r"/\*\nDescription:", content))

        entries.append({
            "file": filename,
            "module": mod_path,
            "kind": item_kind,
            "name": item_name,
            "size_bytes": size,
            "spec_count": spec_count,
            "status": "ok" if ok else "failed",
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
        _run_checks(str(project), str(output))
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
        _run_checks(str(project), str(output))


def _run_checks(project_path: str, output_dir: str) -> None:
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

    # 2. Compilation check
    print("\n[2] Compilation check ...")
    _check_specs(project_path, output_dir)


if __name__ == "__main__":
    main()
