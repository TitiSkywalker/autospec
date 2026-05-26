"""Central tool registry — inspired by OpenGauss's clean registration pattern.

Each tool module calls register() at import time. The registry then provides
Anthropic-format tool definitions and dispatch to the agent loop.
"""

from typing import Any, Callable

_registry: dict[str, dict] = {}

# Project context — set by agent before dispatching tools.
_project_root: str = ""
_output_dir: str = ""


def set_context(project_root: str, output_dir: str) -> None:
    global _project_root, _output_dir
    _project_root = project_root
    _output_dir = output_dir


def get_project_root() -> str:
    return _project_root


def get_output_dir() -> str:
    return _output_dir


def register(name: str, description: str, input_schema: dict, handler: Callable) -> None:
    """Register a tool with its Anthropic-format schema and handler function."""
    _registry[name] = {
        "name": name,
        "description": description,
        "input_schema": input_schema,
        "handler": handler,
    }


def get_definitions() -> list[dict]:
    """Return Anthropic-compatible tool definitions."""
    return [
        {"name": t["name"], "description": t["description"], "input_schema": t["input_schema"]}
        for t in sorted(_registry.values(), key=lambda t: t["name"])
    ]


def dispatch(name: str, args: dict[str, Any]) -> str:
    """Execute a tool by name and return its result as a string."""
    tool = _registry.get(name)
    if tool is None:
        return f"Error: unknown tool '{name}'"
    try:
        return str(tool["handler"](**args))
    except Exception as exc:
        return f"Tool '{name}' error: {type(exc).__name__}: {exc}"
