"""Specification output tools."""

import json
from pathlib import Path

from tool_registry import register, get_output_dir


def _write_one(out_dir: Path, filename: str, content: str) -> str:
    out_path = out_dir / filename
    # Prevent writes outside output dir
    if not str(out_path.resolve()).startswith(str(out_dir.resolve())):
        return f"Error: filename {filename!r} escapes output directory"
    try:
        out_path.write_text(content, encoding="utf-8")
    except Exception as exc:
        return f"Error writing {filename}: {exc}"
    return f"Wrote {out_path.stat().st_size} bytes to {out_path}"


def write_spec(filename: str, content: str) -> str:
    """Write a single specification file to the output directory."""
    out_dir = Path(get_output_dir())
    out_dir.mkdir(parents=True, exist_ok=True)
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
        "Write a single specification file to the output directory. "
        "Use for small projects. For multi-module projects, prefer write_specs "
        "to write one file per module in a single call."
    ),
    input_schema={
        "type": "object",
        "properties": {
            "filename": {
                "type": "string",
                "description": "Output filename, e.g. 'spec.rs' or 'field_element.rs'",
            },
            "content": {
                "type": "string",
                "description": "The complete specification content",
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
        "Example: {\"field_element.rs\": \"... spec content ...\", \"scalar.rs\": \"...\"}. "
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
