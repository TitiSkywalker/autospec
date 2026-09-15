"""
Loguru-backed logging for the evaluation pipeline.

``get_logger(obj)`` returns a logger bound to the object's class name so every
line is prefixed with a colorized ``[LEVEL ClassName]`` tag. ``configure``
re-points the single global sink — worker processes send it to a queue, the
manager leaves it on stdout — while bound loggers keep following whichever sink
is installed.
"""

import sys

from loguru import logger

_FORMAT = "[<level>{level} {extra[classname]}</level>] {message}"


def configure(sink, colorize=None, level="INFO"):
    """Install ``sink`` as the sole loguru sink.

    ``colorize=None`` auto-detects the terminal; pass ``True`` for non-tty sinks
    (a queue, the status board) that must still bake in color codes.
    """
    logger.remove()
    logger.add(sink, format=_FORMAT, colorize=colorize, level=level)


def get_logger(obj, name: str | None = None):
    """Return a logger bound to ``obj``'s class name (or an explicit ``name``)."""
    return logger.bind(classname=name or obj.__class__.__name__)


configure(sys.stdout)  # default: plain terminal output
