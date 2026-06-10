"""Specification output tools."""

import json
import threading
from pathlib import Path

from tool_registry import get_output_dir, register

# Per-file locks for thread-safe concurrent writes
_file_locks: dict[str, threading.Lock] = {}
_file_locks_lock = threading.Lock()


def _get_file_lock(filename: str) -> threading.Lock:
    with _file_locks_lock:
        if filename not in _file_locks:
            _file_locks[filename] = threading.Lock()
        return _file_locks[filename]


def _write_one(out_dir: Path, filename: str, content: str) -> str:
    out_path = out_dir / filename
    # Prevent writes outside output dir
    if not str(out_path.resolve()).startswith(str(out_dir.resolve())):
        return f"Error: filename {filename!r} escapes output directory"
    out_path.parent.mkdir(parents=True, exist_ok=True)
    try:
        out_path.write_text(content, encoding="utf-8")
    except Exception as exc:
        return f"Error writing {filename}: {exc}"
    return f"Wrote {out_path.stat().st_size} bytes to {out_path}"


def write_spec(filename: str, content: str, append: bool = False) -> str:
    """Write or append to a specification file in the output directory.

    Set append=True to add content to an existing file; default overwrites.
    Thread-safe: per-file locking prevents interleaved appends.
    """
    out_dir = Path(get_output_dir())
    out_dir.mkdir(parents=True, exist_ok=True)

    lock = _get_file_lock(filename)
    with lock:
        if append and (out_dir / filename).is_file():
            existing = (out_dir / filename).read_text(encoding="utf-8")
            # Remove trailing blank lines before appending
            existing = existing.rstrip("\n") + "\n\n"
            content = existing + content
        return _write_one(out_dir, filename, content)


def write_specs(files: str) -> str:
    """Write multiple specification files at once.

    Args:
        files: A JSON object mapping filenames to contents, e.g.
               '{"field_element.rs": "...", "scalar.rs": "..."}'.
    """
    out_dir = Path(get_output_dir())
    out_dir.mkdir(parents=True, exist_ok=True)

    try:
        mapping = json.loads(files)
    except json.JSONDecodeError as exc:
        return f"Error: files is not valid JSON: {exc}"

    if not isinstance(mapping, dict):
        return "Error: files must be a JSON object (dict)"

    results = []
    for filename, content in mapping.items():
        result = _write_one(out_dir, filename, content)
        results.append(f"  {result}")

    return "Wrote {} files:\n{}".format(len(mapping), "\n".join(results))


# ————————————————————————————— Tool registration —————————————————————————————

register(
    name="write_spec",
    description=(
        "Write a specification file. "
        "Use append=True to add content to an existing file. "
        "Default (append=False) overwrites with new content."
    ),
    input_schema={
        "type": "object",
        "properties": {
            "filename": {
                "type": "string",
                "description": "Output filename, e.g. 'geometry__Point.rs'",
            },
            "content": {
                "type": "string",
                "description": "The harness block (or full file content on first write)",
            },
            "append": {
                "type": "boolean",
                "description": "If true, append to existing file; if false, overwrite (default: false)",
            },
        },
        "required": ["filename", "content"],
    },
    handler=write_spec,
)

register(
    name="write_specs",
    description=(
        "Write MULTIPLE specification files at once — one per module. "
        "Pass a JSON object mapping filename → content. "
        'Example: {"field_element.rs": "... spec content ...", "scalar.rs": "..."}. '
        "Use this to split specs by module: each public module gets its own file. "
        "Always include a spec.rs or INDEX.md listing the generated files."
    ),
    input_schema={
        "type": "object",
        "properties": {
            "files": {
                "type": "string",
                "description": (
                    "JSON object mapping filenames to full spec contents. "
                    'Example: {"field_element.rs": "// spec for FieldElement...", '
                    '"scalar.rs": "// spec for Scalar..."}'
                ),
            },
        },
        "required": ["files"],
    },
    handler=write_specs,
)
