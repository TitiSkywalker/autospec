"""
Per-workspace lean-lsp-mcp HTTP server management.

Each workspace is an independent Lean project, so it needs its own Lean LSP
session. ``lean-lsp-mcp`` holds a single LSP session per process (rooted at
``--lean-project-path``), so the pipeline runs one HTTP server per workspace,
each on its own port, and points that workspace's ``.vibe/config.toml`` at it.
This module owns the launch / readiness / shutdown lifecycle behind one class.
"""

import socket
import subprocess
import time
from pathlib import Path


class MCPServer:
    """Manage one lean-lsp-mcp HTTP server per workspace.

    Workspace ``i`` is served at ``http://127.0.0.1:{base_port + i}/mcp`` with
    its Lean LSP rooted at that workspace. ``start`` launches a
    ``lean-lsp-mcp`` process per workspace and waits for each to bind its port;
    ``stop`` terminates them all. Per-workspace output is appended to
    ``log_dir/ws-{i}.log`` when ``log_dir`` is given, otherwise discarded.
    """

    DEFAULT_BASE_PORT = 3172
    DEFAULT_READY_TIMEOUT = 120.0
    _HOST = "127.0.0.1"

    def __init__(
        self,
        workspaces: list[Path],
        *,
        base_port: int = DEFAULT_BASE_PORT,
        ready_timeout: float = DEFAULT_READY_TIMEOUT,
        log_dir: Path | None = None,
    ) -> None:
        self.workspaces = [Path(w) for w in workspaces]
        self.base_port = base_port
        self.ready_timeout = ready_timeout
        self.log_dir = Path(log_dir) if log_dir is not None else None
        self._servers: list[tuple[subprocess.Popen, object]] = []

    def port_for(self, index: int) -> int:
        """The HTTP port assigned to workspace ``index``."""
        return self.base_port + index

    def start(self) -> None:
        """Launch one ``lean-lsp-mcp`` HTTP server per workspace and wait for readiness."""
        if self.log_dir is not None:
            self.log_dir.mkdir(parents=True, exist_ok=True)
        for i, ws in enumerate(self.workspaces):
            log = self._open_log(i)
            proc = subprocess.Popen(
                [
                    "uvx", "lean-lsp-mcp",
                    "--transport", "streamable-http",
                    "--host", self._HOST,
                    "--port", str(self.port_for(i)),
                    "--lean-project-path", str(ws),
                ],
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            self._servers.append((proc, log))
        for i, (proc, log) in enumerate(self._servers):
            port = self.port_for(i)
            if proc.poll() is not None:
                print(
                    f"ERROR: lean-lsp-mcp for workspace {i} exited early "
                    f"(code {proc.returncode}); see {self._log_name(log)}",
                    flush=True,
                )
            elif not self._wait_for_port(port, self.ready_timeout):
                print(
                    f"WARNING: lean-lsp-mcp for workspace {i} (port {port}) "
                    f"not ready after {self.ready_timeout:g}s; see {self._log_name(log)}",
                    flush=True,
                )

    def stop(self) -> None:
        """Terminate every lean-lsp-mcp server started by ``start``."""
        for proc, log in self._servers:
            if proc.poll() is None:
                proc.terminate()
        for proc, log in self._servers:
            try:
                proc.wait(timeout=10)
            except subprocess.TimeoutExpired:
                proc.kill()
            if log is not subprocess.DEVNULL:
                log.close()
        self._servers.clear()

    def _open_log(self, index: int):
        if self.log_dir is None:
            return subprocess.DEVNULL
        return open(self.log_dir / f"ws-{index}.log", "ab")

    @staticmethod
    def _log_name(log) -> str:
        return getattr(log, "name", None) or "<devnull>"

    def _wait_for_port(self, port: int, timeout: float) -> bool:
        """Poll until something accepts connections on ``port``."""
        deadline = time.time() + timeout
        while time.time() < deadline:
            try:
                with socket.create_connection((self._HOST, port), timeout=1.0):
                    return True
            except OSError:
                time.sleep(1.0)
        return False
