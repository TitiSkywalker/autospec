"""Flatten nested for-loops via an external tool (best-effort).

If flatten-loops produces code that doesn't compile the original files
are restored and the pipeline continues without loop flattening.
"""

import shutil
import subprocess
from pathlib import Path

from .log import phase, warn

# Path to the flatten-loops binary, resolved relative to this file.
_FLATTEN_TOOL = (
    Path(__file__).resolve().parent.parent
    / "tools" / "flatten-loops" / "target" / "release" / "flatten-loops"
)


def try_flatten_nested_loops(project_path: Path, source_path: Path) -> None:
    """Run flatten-loops on every ``.rs`` file in *project_path*.

    If compilation breaks the original files are restored from
    *source_path*.  Never raises — flattening is always best-effort.
    """
    if not _FLATTEN_TOOL.exists():
        return

    modified: list[Path] = []
    for rs in project_path.rglob("*.rs"):
        if "f64_helpers" in rs.parts or "target" in rs.parts:
            continue
        try:
            result = subprocess.run(
                [str(_FLATTEN_TOOL), str(rs), "--in-place"],
                capture_output=True,
                text=True,
                timeout=30,
            )
            if result.returncode != 0:
                warn(f"flatten-loops failed on {rs.name}: {result.stderr.strip()}")
            elif result.stderr.strip():
                modified.append(rs)
        except subprocess.TimeoutExpired:
            warn(f"flatten-loops timed out on {rs.name}, skipping")

    if not modified:
        return

    check = subprocess.run(
        ["cargo", "check"],
        cwd=str(project_path),
        capture_output=True,
        text=True,
    )
    if check.returncode == 0:
        phase("flatten", ", ".join(p.name for p in modified))
    else:
        for rs in modified:
            rel = rs.relative_to(project_path)
            src = source_path / rel
            if src.exists():
                shutil.copy2(src, rs)
        warn("flatten-loops broke compilation, restored originals")
