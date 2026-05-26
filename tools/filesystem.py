"""File system tools: directory listing, file reading, code search."""

import os
import re
import subprocess
from pathlib import Path

from tool_registry import register, get_project_root


def _resolve(path: str = "") -> Path:
    """Resolve a project-relative path to an absolute path, rejecting escapes."""
    root = Path(get_project_root())
    resolved = (root / path).resolve()
    # Prevent directory traversal outside the project
    if not str(resolved).startswith(str(root.resolve())):
        raise ValueError(f"Path {path!r} escapes project root")
    return resolved


def list_directory(path: str = "") -> str:
    """List files and directories under a project-relative path, with sizes."""
    full = _resolve(path)
    if not full.is_dir():
        return f"Error: {path!r} is not a directory"
    lines = []
    try:
        items = sorted(full.iterdir(), key=lambda p: (p.is_file(), p.name))
    except PermissionError:
        return f"Error: permission denied reading {path!r}"
    for item in items:
        tag = "f" if item.is_file() else "d"
        try:
            size = item.stat().st_size if item.is_file() else 0
        except OSError:
            size = 0
        indent = ""
        if item.is_dir():
            indent = "/"
        lines.append(f"  [{tag}] {item.name}{indent}  ({_fmt_size(size)})")
    if not lines:
        return f"(empty)  {full}"
    return f"{full}/\n" + "\n".join(lines)


def _fmt_size(n: int) -> str:
    for unit in ("B", "KB"):
        if n < 1024:
            return f"{n} {unit}"
        n //= 1024
    return f"{n} MB"


def read_file(path: str, offset: int = 1, limit: int = 200) -> str:
    """Read a file. offset is the starting line number (1-indexed), limit is max lines."""
    full = _resolve(path)
    if not full.is_file():
        return f"Error: {path!r} is not a file"
    try:
        lines_data = full.read_text(encoding="utf-8").splitlines()
    except UnicodeDecodeError:
        return f"Error: {path!r} is not a UTF-8 text file"
    start = max(0, offset - 1)
    end = min(start + limit, len(lines_data))
    selected = lines_data[start:end]
    result_lines = [f"{full}  lines {start+1}-{end} of {len(lines_data)}"]
    for i, line in enumerate(selected, start=start + 1):
        result_lines.append(f"{i:>5} | {line}")
    return "\n".join(result_lines)


def search_code(pattern: str, file_glob: str = "*.rs", path: str = "") -> str:
    """Search for a regex pattern in source files. file_glob filters by filename."""
    full = _resolve(path)
    try:
        proc = subprocess.run(
            ["grep", "-n", "--include", file_glob, "-r", pattern, str(full)],
            capture_output=True, text=True, timeout=30,
        )
    except subprocess.TimeoutExpired:
        return "Error: search timed out"
    output = proc.stdout.strip()
    if not output:
        lines_found = proc.stderr.strip()
        if lines_found:
            return f"grep error: {lines_found}"
        return f"No matches for {pattern!r} in {file_glob} under {path or '.'}"
    # Truncate very long results
    lines_found = output.splitlines()
    if len(lines_found) > 100:
        return "\n".join(lines_found[:100]) + f"\n... (truncated, {len(lines_found)} total matches)"
    return output


# ————————————————————————————— Tool registration —————————————————————————————

register(
    name="list_directory",
    description="List files and subdirectories in a project-relative path. Shows file sizes and types (f=file, d=directory).",
    input_schema={
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Project-relative directory path, empty string for root"},
        },
        "required": [],
    },
    handler=list_directory,
)

register(
    name="read_file",
    description="Read a file from the project. Returns content with line numbers. Use offset and limit for large files.",
    input_schema={
        "type": "object",
        "properties": {
            "path": {"type": "string", "description": "Project-relative file path"},
            "offset": {"type": "integer", "description": "Line number to start from (1-indexed, default 1)"},
            "limit": {"type": "integer", "description": "Maximum lines to return (default 200)"},
        },
        "required": ["path"],
    },
    handler=read_file,
)

register(
    name="search_code",
    description="Search for a regex pattern in Rust source files using grep. Useful for finding function/struct/trait definitions.",
    input_schema={
        "type": "object",
        "properties": {
            "pattern": {"type": "string", "description": "Regex pattern to search for (e.g. 'pub fn ', '#\\[test\\]')"},
            "file_glob": {"type": "string", "description": "Filename pattern (default: *.rs)"},
            "path": {"type": "string", "description": "Subdirectory to search in (default: project root)"},
        },
        "required": ["pattern"],
    },
    handler=search_code,
)
