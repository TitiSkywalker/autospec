"""
In-place terminal status board.

Only the manager (main process) writes to the terminal. Workers report progress
and log lines over a queue, and the board keeps one status line per worker —
redrawn in place with ANSI cursor control — while log lines scroll above it.
"""

import re
import sys

_ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
_CLEAR = "\x1b[2K"  # erase the entire line
# Status-board lines are rendered faint (dim) so they read as lighter than the
# scrolling log lines above them. Swap to "\x1b[90m" for an explicit gray.
_DIM = "\x1b[2m"
_RESET = "\x1b[0m"


class StatusBoard:
    """Redraw ``labels`` status lines in place; ``log`` emits a line above them."""

    def __init__(self, labels, stream=None):
        self.labels = list(labels)
        self.texts = ["idle"] * len(self.labels)
        self.rows = len(self.labels)
        self.stream = stream or sys.stdout
        self.tty = self.stream.isatty()
        self._drawn = False

    def update(self, index, text):
        """Replace slot ``index``'s text and redraw the board in place."""
        if not (0 <= index < self.rows):
            return
        self.texts[index] = text
        if self.tty:
            self._render()

    def log(self, line):
        """Write a (possibly multi-line) log line above the board, then redraw."""
        line = line.rstrip("\n")
        if not self.tty:
            print(self._plain(line), file=self.stream, flush=True)
            return
        self._render(line + "\n")

    def _render(self, prefix=""):
        """(Re)draw every slot; ``prefix`` is written above the board first."""
        if self._drawn:
            head = f"\x1b[{self.rows}A" + prefix
        else:
            self._drawn = True
            head = prefix
        body = "".join(_CLEAR + self._slot(i) + "\n" for i in range(self.rows))
        self.stream.write(head + body)
        self.stream.flush()

    def _slot(self, i):
        text = self.texts[i]
        line = f"[{self.labels[i]}] {text}" if text else f"[{self.labels[i]}]"
        return f"{_DIM}{line}{_RESET}"

    @staticmethod
    def _plain(text):
        return _ANSI_RE.sub("", text)
