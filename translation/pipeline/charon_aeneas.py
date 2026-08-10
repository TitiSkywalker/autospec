"""Charon → LLBC adapter → Aeneas pipeline stage."""

import os
import re
import subprocess
import sys
from pathlib import Path

from .log import fail, phase, dbg
from .utils import parse_adapter_stderr

# Paths resolved relative to this file.
_ROOT = Path(__file__).resolve().parent.parent
_F64_ADAPTER = _ROOT / "tools" / "f64_adapter.py"
_I64_ADAPTER = _ROOT / "tools" / "i64_adapter.py"


def run_charon(charon_bin: str, workdir: Path) -> bool:
    """Run charon to produce ``.llbc`` files.  Returns True on success."""
    result = subprocess.run(
        [charon_bin, "cargo", "--preset=aeneas", "--", "--lib"],
        cwd=workdir,
        capture_output=True,
        text=True,
    )
    if result.stdout.strip():
        dbg(result.stdout.strip())
    if result.returncode != 0:
        fail("charon", "failed")
        for line in result.stderr.split("\n"):
            if line.strip():
                print(f"           {line.strip()}", file=sys.stderr)
        return False
    phase("charon", "ok")
    return True


def run_adapter(workdir: Path) -> dict[str, str] | None:
    """Run the LLBC adapter on every ``.llbc`` file in *workdir*.

    Returns aggregated adapter stats, or *None* on failure.
    """
    llbc_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".llbc"))
    if not llbc_files:
        fail("adapter", "no .llbc file found")
        return None

    adapter_stats: dict[str, str] = {}
    for f in llbc_files:
        llbc_path = str(workdir / f)
        result = subprocess.run(
            [sys.executable, str(_F64_ADAPTER), llbc_path, "-o", llbc_path],
            capture_output=True,
            text=True,
        )
        if result.stdout.strip():
            dbg(result.stdout.strip())
        if result.returncode != 0:
            fail("adapter", result.stderr.strip()[:200])
            return None
        adapter_stats = parse_adapter_stderr(result.stderr)

    parts = [f"f64={adapter_stats.get('f64', '?')}"]
    if adapter_stats.get("binop") and adapter_stats["binop"] != "0":
        parts.append(f"{adapter_stats['binop']} binop/neg")
    if adapter_stats.get("consts") and adapter_stats["consts"] != "0":
        parts.append(f"{adapter_stats['consts']} consts")
    phase("adapter", ", ".join(parts))
    return adapter_stats


def run_i64_adapter(workdir: Path) -> dict[str, str] | None:
    """Run the I64 LLBC adapter on every ``.llbc`` file in *workdir*.

    Returns aggregated adapter stats, or *None* on failure.
    """
    llbc_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".llbc"))
    if not llbc_files:
        fail("i64-adapter", "no .llbc file found")
        return None

    adapter_stats: dict[str, str] = {}
    for f in llbc_files:
        llbc_path = str(workdir / f)
        result = subprocess.run(
            [sys.executable, str(_I64_ADAPTER), llbc_path, "-o", llbc_path],
            capture_output=True,
            text=True,
        )
        if result.stdout.strip():
            dbg(result.stdout.strip())
        if result.returncode != 0:
            fail("i64-adapter", result.stderr.strip()[:200])
            return None
        adapter_stats = parse_adapter_stderr(result.stderr)

    parts = [f"i64={adapter_stats.get('i64', '?')}"]
    if adapter_stats.get("binop") and adapter_stats["binop"] != "0":
        parts.append(f"{adapter_stats['binop']} binop/unop")
    if adapter_stats.get("consts") and adapter_stats["consts"] != "0":
        parts.append(f"{adapter_stats['consts']} consts")
    phase("i64-adapter", ", ".join(parts))
    return adapter_stats


def run_aeneas(aeneas_bin: str, workdir: Path, *, loops_to_rec: bool = False) -> list[str] | None:
    """Run aeneas on every ``.llbc`` file in *workdir*.

    Returns a list of aeneas warning strings, or *None* on error.
    """
    llbc_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".llbc"))
    aeneas_warnings: list[str] = []

    for f in llbc_files:
        cmd = [aeneas_bin, "-all-computable", "-sequential", "-backend", "lean"]
        if loops_to_rec:
            cmd.append("-loops-to-rec")
        cmd.append(str(workdir / f))
        result = subprocess.run(
            cmd,
            cwd=str(workdir),
            capture_output=True,
            text=True,
        )
        for line in (result.stdout + result.stderr).split("\n"):
            line = re.sub(r"\x1b\[[0-9;]*m", "", line).strip()
            if not line:
                continue
            dbg(f"[aeneas] {line}")
            if "[Error" in line or "error:" in line.lower():
                fail("aeneas", line)
                return None
            m = re.match(r"\[(?:Warn|Warning)\s*\]\s*(.*)", line)
            if m:
                w = m.group(1)
                if "external, unknown definitions" not in w:
                    aeneas_warnings.append(w)

    lean_files = sorted(f for f in os.listdir(str(workdir)) if f.endswith(".lean"))
    if not lean_files:
        fail("aeneas", "no .lean file generated")
        return None

    phase("aeneas", "ok")
    return aeneas_warnings
