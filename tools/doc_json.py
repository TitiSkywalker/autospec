"""rustdoc JSON tools — generate and read structured crate documentation."""

import os
import subprocess
from pathlib import Path

from tool_registry import register, get_project_root

from rustdoc import RustDoc, RustItem


def _find_doc_json(root: Path, crate_name: str) -> Path | None:
    """Find the rustdoc JSON, searching up for workspace target directories."""
    # First check under the project root
    candidate = root / "target" / "doc" / f"{crate_name}.json"
    if candidate.is_file():
        return candidate
    # Walk upward looking for a workspace target directory
    for parent in [root] + list(root.parents):
        ws_target = parent / "target" / "doc"
        if ws_target.is_dir():
            matches = list(ws_target.glob(f"{crate_name}.json"))
            if matches:
                return matches[0]
    # Broader search: any JSON in any target/doc up the tree
    for parent in [root] + list(root.parents):
        doc_dir = parent / "target" / "doc"
        if doc_dir.is_dir():
            matches = list(doc_dir.glob("*.json"))
            if matches:
                return matches[0]
    return None


def generate_doc_json(
    nightly: str = "nightly",
    features: str = "",
    regenerate: bool = False,
) -> str:
    """Run `cargo rustdoc` to generate the rustdoc JSON file.

    Requires a nightly toolchain. The JSON is written to
    target/doc/<crate_name>.json (rustdoc's default location).

    Args:
        nightly: Nightly toolchain specifier (default: 'nightly').
        features: Comma-separated feature flags, or 'all' for --all-features.
                  Default '' uses the crate's default features.
        regenerate: If true, delete existing JSON and rebuild.
    """
    root = Path(get_project_root())

    import tomllib
    with open(root / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", root.name)

    expected_json = _find_doc_json(root, crate_name)
    if expected_json and not regenerate:
        return (
            f"rustdoc JSON already exists: {expected_json}\n"
            f"Size: {expected_json.stat().st_size:,} bytes\n"
            f"Run read_rustdoc() to see the full public API.\n"
            f"(Use regenerate=True to rebuild with different features.)"
        )

    cmd = [
        "cargo", f"+{nightly}", "rustdoc",
    ]
    if features == "all":
        cmd.append("--all-features")
    elif features:
        cmd.extend(["--features", features])

    cmd.extend(["--", "-Zunstable-options", "--output-format", "json"])

    try:
        proc = subprocess.run(
            cmd,
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=300,
            env={**os.environ, "CARGO_TERM_COLOR": "never"},
        )
    except subprocess.TimeoutExpired:
        return "Error: cargo rustdoc timed out after 300s"

    if proc.returncode != 0:
        stderr = proc.stderr.strip()[-2000:] if proc.stderr else "(no output)"
        return f"Error: cargo rustdoc failed (exit {proc.returncode})\n\n{stderr}"

    expected_json = _find_doc_json(root, crate_name)
    if not expected_json:
        return (
            f"Error: rustdoc succeeded but JSON not found for crate '{crate_name}'. "
            f"Check target/doc/ for .json files."
        )

    return (
        f"Generated rustdoc JSON: {expected_json}\n"
        f"Size: {expected_json.stat().st_size:,} bytes\n"
        f"Run read_rustdoc() to see the full public API."
    )


def read_rustdoc(module: str = "") -> str:
    """Read the generated rustdoc JSON.

    With module="" (default): shows a summary of all public modules with item
    counts. Use this first to get the lay of the land.

    With module="base::allocator": shows all public items in that specific
    module with their full doc comments and impl blocks.

    Call this after generate_doc_json().
    """
    root = Path(get_project_root())

    import tomllib
    with open(root / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", root.name)

    json_path = _find_doc_json(root, crate_name)
    if not json_path:
        return "Error: No rustdoc JSON found. Run generate_doc_json first."

    try:
        doc = RustDoc(str(json_path))
    except Exception as exc:
        return f"Error parsing rustdoc JSON: {exc}"

    # ——— Summary mode: show module tree with counts ———
    if not module:
        output: list[str] = [
            f"# Crate: {doc.root().name} v{doc.crate_version}",
            f"Total items: {len(doc.items)}",
            "",
            "## Module tree (public items with doc counts)",
            "Use `read_rustdoc(module=\"path\")` to drill into a module.",
            "",
        ]

        def _summarize(module_id: int, depth: int, path: str) -> None:
            mod = doc.items.get(module_id)
            if not mod:
                return
            name = mod.name or "(root)"

            has_docs = 0
            total = 0
            child_modules: list[RustItem] = []
            for child in doc.children(module_id):
                if child.visibility != "public":
                    continue
                total += 1
                if child.docs:
                    has_docs += 1
                if child.kind == "module" and child.visibility == "public":
                    child_modules.append(child)

            indent = "  " * depth
            label = path if depth > 0 else name
            output.append(f"{indent}📁 {label}/  ({has_docs}/{total} documented)")

            for cm in sorted(child_modules, key=lambda c: c.name or ""):
                cm_name = cm.name or "?"
                _summarize(cm.id, depth + 1, f"{path}::{cm_name}" if depth > 0 else cm_name)

        _summarize(doc.root_id, 0, doc.root().name or "crate")
        return "\n".join(output)

    # ——— Detail mode: show items in a specific module ———
    # Resolve the module path to an item ID
    crate_name = doc.root().name or ""
    # Allow the crate name itself as a module path alias
    if module == crate_name:
        module = ""

    parts = [p for p in module.replace("::", "/").split("/") if p]
    target_id = doc.root_id

    for part in parts:
        found = False
        for child in doc.children(target_id):
            if child.name == part or (
                child.kind == "use"
                and child.inner.get("name") == part
                and child.inner.get("id") in doc.items
            ):
                if child.kind == "use":
                    resolved_id = child.inner.get("id")
                    if resolved_id in doc.items:
                        child = doc.items[resolved_id]
                target_id = child.id
                found = True
                break
        if not found:
            return (
                f"Error: module '{part}' not found in path '{module}'. "
                f"Available children of current module: "
                + ", ".join(
                    c.name or c.inner.get("name", "?")
                    for c in doc.children(target_id)
                    if c.visibility == "public"
                )
            )

    target = doc.items.get(target_id)
    if not target:
        return f"Error: item {target_id} not found for module '{module}'"

    output: list[str] = [
        f"# Module: {target.name or '(root)'}",
        f"Path: {module}",
        "",
    ]

    def _append_impls(item_id: int, name: str, lines: list[str]) -> None:
        impls = doc.get_impls(item_id)
        for trait_name, methods in impls:
            label = f"impl {trait_name} for {name}" if trait_name else f"impl {name}"
            lines.append(f"  [{label}]")
            for mname, mdocs in methods:
                mdocs_text = (mdocs or "").strip()
                if mdocs_text:
                    lines.append(f"    fn {mname}() — {mdocs_text}")
                else:
                    lines.append(f"    fn {mname}()")
            lines.append("")

    for child in doc.children(target_id):
        if child.visibility != "public":
            continue

        kind = child.kind
        name = child.name or ""

        # Re-exports
        if kind == "use":
            inner_name = child.inner.get("name", "")
            resolved_id = child.inner.get("id")
            if resolved_id is not None and resolved_id in doc.items:
                resolved = doc.items[resolved_id]
                if child.inner.get("is_glob") and resolved.kind == "module":
                    # Glob re-export: `pub use some_module::*`
                    # Show the module's public items inline.
                    for rc in doc.children(resolved.id):
                        if rc.visibility != "public":
                            continue
                        rkind = rc.kind
                        rname = rc.name or ""
                        if rkind in ("module",) or not rname:
                            continue
                        rd = rc.docs or ""
                        if len(rd) > 1000:
                            rd = rd[:1000] + "\n... (truncated)"
                        output.append(f"\n## {rkind}: {rname} (via {inner_name}::{rname})")
                        if rd:
                            output.append(f"\n{rd}\n")
                        if rkind in ("struct", "enum", "trait"):
                            _append_impls(rc.id, rname, output)
                elif not child.inner.get("is_glob"):
                    # Named re-export: `pub use some_module::Item`
                    output.append(f"\n## {resolved.kind}: {inner_name} (re-export)")
                    if resolved.docs:
                        output.append(f"\n{resolved.docs}\n")
                    _append_impls(resolved.id, inner_name, output)
            continue

        # Sub-modules — list but don't descend
        if kind == "module":
            continue

        docs = child.docs or ""
        if len(docs) > 2000:
            docs = docs[:2000] + "\n... (truncated)"

        output.append(f"\n## {kind}: {name}")
        if docs:
            output.append(f"\n{docs}\n")

        if kind in ("struct", "enum", "trait"):
            _append_impls(child.id, name, output)

    output.append(f"\n---\n{len(output)} lines")
    return "\n".join(output)


# ————————————————————————————— Tool registration —————————————————————————————

register(
    name="generate_doc_json",
    description=(
        "Run `cargo +nightly rustdoc` to generate a rustdoc JSON file. "
        "Requires nightly Rust. Call this first with features='all' to capture "
        "the complete public API, then use read_rustdoc to inspect it."
    ),
    input_schema={
        "type": "object",
        "properties": {
            "nightly": {
                "type": "string",
                "description": "Nightly toolchain specifier (default: 'nightly')",
            },
            "features": {
                "type": "string",
                "description": "Feature flags: '' (default), 'all' for --all-features, or comma-separated like 'ecdsa,ecdh'",
            },
            "regenerate": {
                "type": "boolean",
                "description": "If true, delete existing JSON and rebuild (default: false)",
            },
        },
        "required": [],
    },
    handler=generate_doc_json,
)

register(
    name="read_rustdoc",
    description=(
        "Read the generated rustdoc JSON. With no arguments, shows a summary "
        "of all public modules with documented item counts. "
        "Pass module=\"path::to::module\" to drill into a specific module "
        "and see all public items with their full doc comments and impl blocks. "
        "Doc comments are the PRIMARY source for extracting pre/post conditions."
    ),
    input_schema={
        "type": "object",
        "properties": {
            "module": {
                "type": "string",
                "description": "Module path to drill into (e.g. 'base::allocator'). Empty = summary view.",
            },
        },
        "required": [],
    },
    handler=read_rustdoc,
)
