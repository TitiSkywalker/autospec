"""Agent orchestration — run spec-generation and fix agents per function."""

import re
import sys
import threading
import tomllib
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

from agent import SpecAgent
from pipeline.files import _merge_specs, _write_spec_json
from pipeline.generation import (
    _get_spec_targets,
    _make_filter_key,
    _parse_functions_from_section,
)
from prompt import FIX_SYSTEM_PROMPT, FUNC_SYSTEM_PROMPT


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
        f'Then call write_spec(filename="{filename}", content=...) '
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
        f'Then call write_spec(filename="{indiv_filename}", content=...) '
        "with the FIXED harness (include all header comments).",
    ]
    task = "\n".join(task_parts)

    try:
        indiv_output = output_dir
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
        print(
            f"  [FIX-FAIL] {indiv_filename}: {type(exc).__name__}: {exc}",
            file=sys.stderr,
        )
        return False


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
    only_functions: list[str] | None = None,
) -> None:
    """Divide and conquer: one slim agent per documented function.

    Agents run in parallel (*jobs* concurrent workers) since each function
    is independent.
    """

    project_path = str(project)
    output_dir = str(output)

    print("\nPhase 1: Generating rustdoc JSON and scanning for documented types...")
    targets = _get_spec_targets(project_path, output_dir)

    # Parse individual functions from all targets
    only_set: set[str] | None = set(only_functions) if only_functions else None
    all_funcs: list[tuple[str, str, list[dict]]] = []  # (filename, header, [funcs])
    total_skipped = 0
    for filename, doc_section in targets:
        functions = _parse_functions_from_section(doc_section)
        if only_set is not None:
            functions = [f for f in functions if _make_filter_key(f) in only_set]
        if not functions:
            continue
        # Extract the header (everything before the first ## section)
        header_end = doc_section.find("\n## ")
        header = doc_section[:header_end].strip() + "\n\n" if header_end > 0 else ""
        all_funcs.append((filename, header, functions))

    total_funcs = sum(len(funcs) for _, _, funcs in all_funcs)
    if only_set:
        print(
            f"\nParsed {total_funcs} matching functions (from {len(only_set)} in specs.toml)"
            f" across {len(all_funcs)} files."
        )
    else:
        print(
            f"\nParsed {total_funcs} documented functions across {len(all_funcs)} files."
        )

    if not all_funcs:
        print("No documented functions found. Nothing to do.")
        return

    # Spawn agents in parallel
    func_iterations = max(3, max_iterations)
    print(
        f"\nPhase 2: Generating harnesses — {jobs} parallel agents "
        f"({func_iterations} iterations max each, {total_funcs} functions)..."
    )

    # Flatten into (indiv_filename, merged_filename, func_info) tasks
    tasks: list[tuple[str, str, dict]] = []
    for merged_filename, _header, functions in all_funcs:
        for func_info in functions:
            module = func_info.get("module", "").replace("::", "_") or "crate_root"
            type_part = func_info.get("type_name") or "functions"
            indiv_filename = f"{module}__{type_part}__{func_info['name']}.rs"
            tasks.append((indiv_filename, merged_filename, func_info))

    func_results: list[tuple[str, str, str, bool]] = []  # (indiv, merged, name, ok)
    print_lock = threading.Lock()
    completed = 0

    def run_one(
        indiv_fn: str, merged_fn: str, func_info: dict
    ) -> tuple[str, str, str, bool]:
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
        futures = {
            pool.submit(run_one, ifn, mfn, fi): (ifn, mfn, fi) for ifn, mfn, fi in tasks
        }

        for future in as_completed(futures):
            indiv_fn, merged_fn, fn_name, ok = future.result()
            func_results.append((indiv_fn, merged_fn, fn_name, ok))
            with print_lock:
                completed += 1
                status = "OK" if ok else "FAIL"
                print(
                    f"  [{completed}/{total_funcs}] {indiv_fn} :: {fn_name}  {status}"
                )

    succeeded = sum(1 for _, _, _, ok in func_results if ok)
    failed = sum(1 for _, _, _, ok in func_results if not ok)

    # Phase 3: Merge individual files by type
    print(f"\nPhase 3: Merging individual files by type...")
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
        merged_path = output / "merged" / merged_fn
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
    print(
        f"Done: {succeeded} succeeded, {failed} failed "
        f"({total_funcs} harnesses total)"
    )

    # Write structured JSON manifest
    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)
    _write_spec_json(output_dir, crate_name, func_results)


def _run_auto_select(
    project: Path,
    output: Path,
    api_key: str,
    base_url: str,
    model: str,
    max_iterations: int,
    max_tokens: int,
    verbose: bool,
) -> None:
    """Run an agent that explores the codebase and selects algorithmically
    relevant functions for verification, writing results to specs.toml."""
    from agent import SpecAgent
    from prompt import AUTO_SELECT_PROMPT

    project_path = str(project)
    output_dir = str(output)

    task = (
        "Explore this Rust codebase and identify the most algorithmically "
        "interesting functions for formal verification. Focus on functions "
        "with non-trivial mathematical properties, algorithmic complexity, "
        "or safety-critical logic.\n\n"
        "1. Call read_rustdoc() to get the module summary\n"
        "2. Drill into promising modules with read_rustdoc(module=\"...\")\n"
        "3. Read source files of top candidates with read_file\n"
        "4. Call write_specs_toml ONCE with your selections\n\n"
        "Aim for 5-20 high-quality targets. Quality matters more than quantity."
    )

    print("Auto-select agent exploring the codebase...")
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
            system_prompt=AUTO_SELECT_PROMPT,
            tool_filter=["read_rustdoc", "read_file", "write_specs_toml", "search_code"],
        )
        agent.run(task=task)
    except Exception as exc:
        print(f"  Auto-select failed: {type(exc).__name__}: {exc}",
              file=sys.stderr)
        return

    toml_path = output / "specs.toml"
    if toml_path.is_file():
        n = toml_path.read_text().count('"') // 2
        print(f"Auto-select: wrote {n} function(s) to {toml_path}")
    else:
        print("Auto-select: agent completed but no specs.toml was written.")
