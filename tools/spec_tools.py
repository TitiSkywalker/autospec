"""Specification output tool."""

from pathlib import Path

from tool_registry import register, get_output_dir


def write_spec(filename: str, content: str) -> str:
    """Write a specification file to the output directory."""
    out_dir = Path(get_output_dir())
    out_dir.mkdir(parents=True, exist_ok=True)
    out_path = out_dir / filename
    # Prevent writes outside output dir
    if not str(out_path.resolve()).startswith(str(out_dir.resolve())):
        return f"Error: filename {filename!r} escapes output directory"
    try:
        out_path.write_text(content, encoding="utf-8")
    except Exception as exc:
        return f"Error writing {filename}: {exc}"
    return f"Wrote {out_path.stat().st_size} bytes to {out_path}"


register(
    name="write_spec",
    description="Write a specification file to the output directory. Use markdown format for readable specs, or JSON for structured data.",
    input_schema={
        "type": "object",
        "properties": {
            "filename": {"type": "string", "description": "Output filename (e.g. 'spec.md', 'spec.json')"},
            "content": {"type": "string", "description": "The complete specification content"},
        },
        "required": ["filename", "content"],
    },
    handler=write_spec,
)
