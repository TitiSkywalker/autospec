"""Pipeline logging helpers."""

import sys

_VERBOSE = False


def set_verbose(v: bool) -> None:
    global _VERBOSE
    _VERBOSE = v


def dbg(msg: str) -> None:
    """Print debug info when verbose mode is on."""
    if _VERBOSE:
        sys.stdout.flush()
        print(f"  {'debug':<10} \033[2m{msg}\033[0m", file=sys.stderr)


def phase(tag: str, msg: str = "") -> None:
    """Print a pipeline phase status line with consistent alignment."""
    print(f"  {tag:<10} {msg}")


def fail(tag: str, reason: str) -> None:
    """Print a phase failure message to stderr."""
    sys.stdout.flush()
    print(f"  {tag:<10} \033[31mFAIL\033[0m  {reason}", file=sys.stderr)


def warn(msg: str) -> None:
    """Print a warning message."""
    print(f"  {'warn':<10} \033[33m{msg}\033[0m")
