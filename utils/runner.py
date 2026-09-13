"""
Running CMD commands with Python.
"""

import fcntl
import os
import pty
import shutil
import signal
import struct
import subprocess
import sys
import termios
import threading
from pathlib import Path


class CmdRunner:
    def __init__(self, cwd: str | None = None, verbose: bool = True):
        self.cwd = Path(cwd or os.getcwd()).resolve()
        self.verbose = verbose

    def cd(self, path: str):
        new_cwd = (self.cwd / path).resolve()
        if not new_cwd.is_dir():
            raise FileNotFoundError(new_cwd)
        self.cwd = new_cwd

    def run(
        self,
        command: str,
        verbose: bool | None = None,
        panic: bool = True,
        timeout: float | None = None,
    ) -> int:
        verbose = self.verbose if verbose is None else verbose
        if verbose:
            print(f"Executing: {command}")

        master, slave = pty.openpty()

        size = shutil.get_terminal_size(fallback=(80, 24))
        winsize = struct.pack("HHHH", size.lines, size.columns, 0, 0)
        fcntl.ioctl(slave, termios.TIOCSWINSZ, winsize)

        process = subprocess.Popen(
            command,
            shell=True,
            cwd=self.cwd,
            stdout=slave,
            stderr=slave,
            text=True,
            start_new_session=True,
        )

        os.close(slave)

        def reader():
            try:
                while True:
                    data = os.read(master, 1024)
                    if not data:
                        break
                    if verbose:
                        sys.stdout.write(data.decode())
                        sys.stdout.flush()
            except OSError:
                pass

        t = threading.Thread(target=reader, daemon=True)
        t.start()

        try:
            process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            pgid = os.getpgid(process.pid)
            os.killpg(pgid, signal.SIGTERM)
            try:
                process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                os.killpg(pgid, signal.SIGKILL)
                process.wait()
        finally:
            os.close(master)
            t.join(timeout=5)

        if verbose:
            print(f"Return code: {process.returncode}\n")

        if panic and process.returncode != 0:
            exit(0)

        return process.returncode

    def ls(self) -> list[str]:
        return sorted(self.cwd.iterdir(), key=lambda p: (p.is_file(), p.name))

    def ls_files(self) -> list[str]:
        return sorted(p.name for p in self.cwd.iterdir() if p.is_file())

    def ls_dirs(self) -> list[str]:
        return sorted(p.name for p in self.cwd.iterdir() if p.is_dir())
