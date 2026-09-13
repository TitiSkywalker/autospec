"""
Loguru-backed logging for the evaluation pipeline.

``get_logger(obj)`` returns a logger bound to the object's class name so every
line is prefixed with a colorized ``[LEVEL ClassName]`` tag.
"""

import sys

from loguru import logger

logger.remove()
logger.add(
    sys.stdout,
    format="[<level>{level} {extra[classname]}</level>] {message}",
    colorize=True,
    level="INFO",
)


def get_logger(obj, name: str | None = None):
    """Return a logger bound to ``obj``'s class name (or an explicit ``name``)."""
    return logger.bind(classname=name or obj.__class__.__name__)
