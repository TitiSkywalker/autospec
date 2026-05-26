"""Cargo tools: read Cargo.toml metadata, discover tests, run cargo test."""

import os
import re
import subprocess
import tomllib
from pathlib import Path

from tool_registry import register, get_project_root


def read_cargo_toml() -> str:
    """Parse Cargo.toml and return structured project metadata."""
    root = Path(get_project_root())
    toml_path = root / "Cargo.toml"
    if not toml_path.is_file():
        return "Error: Cargo.toml not found in project root"
    try:
        with open(toml_path, "rb") as fh:
            data = tomllib.load(fh)
    except Exception as exc:
        return f"Error parsing Cargo.toml: {exc}"

    package = data.get("package", {})
    lib = data.get("lib", {})
    bin_entries = data.get("bin", [])
    features = data.get("features", {})

    lines = [f"# Cargo.toml: {package.get('name', 'unknown')} v{package.get('version', '0.0.0')}"]
    if package.get("description"):
        lines.append(f"Description: {package['description']}")
    if package.get("edition"):
        lines.append(f"Edition: {package['edition']}")

    # Dependencies
    deps = data.get("dependencies", {})
    if deps:
        lines.append("\n## Dependencies")
        for name, spec in sorted(deps.items()):
            if isinstance(spec, str):
                lines.append(f"  {name}: {spec}")
            elif isinstance(spec, dict):
                ver = spec.get("version", spec.get("path", spec.get("git", "?")))
                if spec.get("optional"):
                    ver += " (optional)"
                lines.append(f"  {name}: {ver}")
            else:
                lines.append(f"  {name}")

    dev_deps = data.get("dev-dependencies", {})
    if dev_deps:
        lines.append("\n## Dev-Dependencies")
        for name in sorted(dev_deps):
            lines.append(f"  {name}")

    build_deps = data.get("build-dependencies", {})
    if build_deps:
        lines.append("\n## Build-Dependencies")
        for name in sorted(build_deps):
            lines.append(f"  {name}")

    # Lib target
    if lib:
        lines.append(f"\n## Library target: {lib.get('name', package.get('name', '?'))}  (path: {lib.get('path', 'src/lib.rs')})")

    # Bin targets
    if bin_entries:
        if isinstance(bin_entries, dict):
            bin_entries = [bin_entries]
        for b in bin_entries:
            lines.append(f"\n## Binary target: {b.get('name', '?')}  (path: {b.get('path', 'src/main.rs')})")
    elif (root / "src" / "main.rs").exists():
        lines.append("\n## Binary target (inferred): src/main.rs")

    # Features
    if features:
        lines.append("\n## Features")
        for feat, deps_list in sorted(features.items()):
            dep_str = ", ".join(deps_list) if deps_list else "(none)"
            lines.append(f"  {feat}: [{dep_str}]")

    return "\n".join(lines)


def list_tests() -> str:
    """Discover all #[test] functions and test modules in the project."""
    root = Path(get_project_root())
    test_functions: list[tuple[str, str, int]] = []  # (file, name, line)

    rs_files = list(root.rglob("*.rs"))
    for f in sorted(rs_files):
        # Skip target/ directory
        if "target/" in str(f):
            continue
        try:
            content = f.read_text(encoding="utf-8")
        except Exception:
            continue
        rel = f.relative_to(root)
        # Find #[test] and the associated function name
        for m in re.finditer(r"#\[test\]\s*(?://[^\n]*\n\s*)*(?:async\s+)?fn\s+(\w+)", content):
            name = m.group(1)
            line_no = content[: m.start()].count("\n") + 1
            test_functions.append((str(rel), name, line_no))

    if not test_functions:
        return "No #[test] functions found in the project."

    lines = [f"Found {len(test_functions)} test functions:\n"]
    for file, name, line in test_functions:
        lines.append(f"  {file}:{line}  #[test] fn {name}")
    return "\n".join(lines)


def run_cargo_test(filter_str: str = "", timeout: int = 120) -> str:
    """Run `cargo test` in the project root, optionally filtering by test name."""
    root = Path(get_project_root())
    cmd = ["cargo", "test", "--quiet"]
    if filter_str:
        cmd.append(filter_str)
    try:
        proc = subprocess.run(
            cmd,
            cwd=str(root),
            capture_output=True,
            text=True,
            timeout=timeout,
            env={**os.environ, "CARGO_TERM_COLOR": "never"},
        )
    except subprocess.TimeoutExpired:
        return f"Error: cargo test timed out after {timeout}s"
    stdout = proc.stdout.strip()
    stderr = proc.stderr.strip()
    result = []
    if stdout:
        result.append("## stdout\n" + _truncate(stdout, 5000))
    if stderr:
        result.append("## stderr\n" + _truncate(stderr, 3000))
    result.append(f"\nExit code: {proc.returncode}")
    return "\n".join(result)


def _truncate(text: str, max_chars: int) -> str:
    if len(text) <= max_chars:
        return text
    return text[:max_chars] + f"\n... (truncated, {len(text)} total chars)"


# ————————————————————————————— Tool registration —————————————————————————————

register(
    name="read_cargo_toml",
    description="Parse Cargo.toml and return project metadata: name, version, dependencies, features, and target layout.",
    input_schema={
        "type": "object",
        "properties": {},
        "required": [],
    },
    handler=read_cargo_toml,
)

register(
    name="list_tests",
    description="Discover all #[test] functions in the project. Use ONLY to find examples of how functions are called (argument types, typical patterns). Test assertions are NOT ground truth for specifications.",
    input_schema={
        "type": "object",
        "properties": {},
        "required": [],
    },
    handler=list_tests,
)

register(
    name="run_cargo_test",
    description="Run cargo test in the project root. Optionally filter by test name substring. Returns stdout, stderr, and exit code.",
    input_schema={
        "type": "object",
        "properties": {
            "filter_str": {"type": "string", "description": "Optional test name filter (passed to cargo test)"},
            "timeout": {"type": "integer", "description": "Timeout in seconds (default 120)"},
        },
        "required": [],
    },
    handler=run_cargo_test,
)
