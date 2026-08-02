"""Pipeline logging helpers."""


def phase(tag: str, msg: str = "") -> None:
    """Print a pipeline phase status line with consistent alignment."""
    print(f"  {tag:<10} {msg}")


def fail(tag: str, reason: str) -> None:
    """Print a phase failure message to stderr."""
    print(f"  {tag:<10} \033[31mFAIL\033[0m  {reason}", file=__import__("sys").stderr)


def warn(msg: str) -> None:
    """Print a warning message."""
    print(f"           \033[33m⚠\033[0m  {msg}")
