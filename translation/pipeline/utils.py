"""Shared utilities for the translation pipeline."""

import re
from pathlib import Path


def read_crate_name(project_path: Path) -> str:
    """Read package name from Cargo.toml, or fall back to directory name."""
    toml_path = project_path / "Cargo.toml"
    if toml_path.exists():
        try:
            text = toml_path.read_text()
            m = re.search(r'^name\s*=\s*"([^"]+)"', text, re.MULTILINE)
            if m:
                return m.group(1).replace("-", "_")
        except Exception:
            pass
    return project_path.name.replace("-", "_")


def capitalize_first(s: str) -> str:
    """Uppercase the first character, leave the rest unchanged."""
    if not s:
        return s
    return s[0].upper() + s[1:]


def find_lake_root(path: Path) -> Path | None:
    """Walk up from *path* to find a directory containing lakefile.toml or lakefile.lean."""
    for parent in [path] + list(path.parents):
        if (parent / "lakefile.toml").exists() or (parent / "lakefile.lean").exists():
            return parent
    return None


def parse_adapter_stderr(stderr: str) -> dict[str, str]:
    """Parse adapter stderr lines like ``[adapter] f64=306, usize=311, ...``."""
    stats: dict[str, str] = {}
    for line in stderr.split("\n"):
        m = re.match(r"\[adapter\]\s+(.+)", line.strip())
        if not m:
            continue
        body = m.group(1)
        if "=" in body:
            for part in body.split(","):
                part = part.strip()
                if "=" in part:
                    k, v = part.split("=", 1)
                    stats[k.strip()] = v.strip()
        elif "no f64" in body:
            stats["f64"] = "none"
        else:
            m2 = re.match(r"(\d+)\s+float consts?,\s*(\d+)\s+binop/neg", body)
            if m2:
                stats["consts"] = m2.group(1)
                stats["binop"] = m2.group(2)
    return stats
