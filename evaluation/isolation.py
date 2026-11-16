"""Filesystem boundary for untrusted proving agents (not for the verifier).

Only the current workspace, standard runtime files, and explicitly approved
read-only paths are visible. Bubblewrap's mount/PID/IPC namespaces prevent a
CLI's shell tools from reading sibling workspaces or saved benchmark results.
"""

import shutil
from dataclasses import dataclass, field
from pathlib import Path


# Runner-home state the MCP server needs back inside its private HOME. Without
# it `uvx` cannot see the solved package cache and re-resolves (then downloads)
# every launch; when the runner reaches PyPI only through a mirror configured
# in ~/.config/uv/uv.toml it hangs until the readiness timeout instead.
DEFAULT_MCP_CACHE_DIRS = (".cache/uv",)
DEFAULT_MCP_HOME_FILES = (".config/uv/uv.toml",)


@dataclass
class SandboxConfig:
    enabled: bool = True
    fresh_workspace: bool = True
    # Runtime directories outside /usr, /bin, /lib*, e.g. ~/.elan or ~/.local/share.
    ro_paths: list[Path] = field(default_factory=list)
    # Individual credential/config files from the runner's home, copied as
    # read-only mounts into the agent's otherwise private HOME.
    home_files: list[str] = field(default_factory=list)
    # Package-manager state for the MCP server, relative to the runner's home.
    # Directories are mounted read-write (caches take locks and add entries),
    # files read-only. Missing entries are skipped: they are caches and
    # per-runner config, never something the run's correctness depends on.
    mcp_cache_dirs: list[str] = field(
        default_factory=lambda: list(DEFAULT_MCP_CACHE_DIRS)
    )
    mcp_home_files: list[str] = field(
        default_factory=lambda: list(DEFAULT_MCP_HOME_FILES)
    )


def _inside(path: Path, parent: Path) -> bool:
    return path == parent or parent in path.parents


def sandbox_command(
    command: list[str],
    workspace: Path,
    config: SandboxConfig,
    forbidden: list[Path],
    *,
    home_name: str = ".agent-home",
) -> tuple[list[str], dict[str, str]]:
    """Return (argv, env overrides). Never silently run without isolation."""
    if not config.enabled:
        return command, {}
    bwrap = shutil.which("bwrap")
    if not bwrap:
        raise RuntimeError("agent sandbox requires bubblewrap (bwrap)")

    ws = workspace.resolve(strict=True)
    if home_name not in (".agent-home", ".mcp-home"):
        raise ValueError("unsupported sandbox home directory")
    home = ws / home_name
    home.mkdir(mode=0o700, exist_ok=True)
    for directory in (".codex", ".dsh", ".claude"):
        (home / directory).mkdir(mode=0o700, exist_ok=True)
    forbidden = [p.resolve() for p in forbidden]

    # The worker workspace may not be nested within the input, output, or base
    # mirror. An enclosing approved mount would disclose their contents too.
    if any(_inside(ws, p) or _inside(p, ws) for p in forbidden):
        raise ValueError("agent workspace overlaps a protected benchmark/result path")

    mounts: list[tuple[Path, Path]] = []
    for name in ("/usr", "/bin", "/lib", "/lib64"):
        path = Path(name)
        if path.exists():
            mounts.append((path, path))
    for name in (
        "/etc/ssl", "/etc/ca-certificates", "/etc/resolv.conf",
        "/etc/hosts", "/etc/nsswitch.conf", "/etc/passwd", "/etc/group",
        "/etc/ld.so.cache", "/etc/localtime",
    ):
        path = Path(name)
        if path.exists():
            mounts.append((path, path))

    for path in config.ro_paths:
        path = path.resolve(strict=True)
        if path == Path("/") or any(
            _inside(path, p) or _inside(p, path) for p in [ws, *forbidden]
        ):
            raise ValueError(f"unsafe sandbox read-only mount: {path}")
        mounts.append((path, path))

    home_mounts: list[tuple[Path, Path, bool]] = []
    host_home = Path.home().resolve()

    def check_relative(relative: str, field_name: str) -> Path:
        rel = Path(relative)
        if rel.is_absolute() or ".." in rel.parts or not rel.parts:
            raise ValueError(f"{field_name} must be relative paths: {relative}")
        return rel

    def check_inside_home(src: Path, relative: str, field_name: str) -> None:
        if not _inside(src, host_home):
            raise ValueError(f"{field_name} must name paths inside the runner home: {relative}")
        if any(_inside(src, p) for p in forbidden):
            raise ValueError(f"{field_name} entry overlaps protected data: {src}")

    for relative in config.home_files if home_name == ".agent-home" else ():
        rel = check_relative(relative, "home_files")
        src = (host_home / rel).resolve(strict=True)
        check_inside_home(src, relative, "home_files")
        if not src.is_file():
            raise ValueError(f"home_files must name files inside the runner home: {relative}")
        home_mounts.append((src, home / rel, True))

    # MCP package state. Distinct from ``home_files``: it is runner tooling
    # state, not agent credentials, so the agent's home never receives it.
    if home_name == ".mcp-home":
        for relative in config.mcp_cache_dirs:
            rel = check_relative(relative, "mcp_cache_dirs")
            src = (host_home / rel).resolve()
            if not src.is_dir():
                continue  # cold runner: let uv fetch (or fail) on its own
            check_inside_home(src, relative, "mcp_cache_dirs")
            if any(_inside(p, src) or _inside(src, p) for p in [ws, *forbidden]):
                raise ValueError(f"unsafe sandbox cache mount: {src}")
            home_mounts.append((src, home / rel, False))
        for relative in config.mcp_home_files:
            rel = check_relative(relative, "mcp_home_files")
            src = (host_home / rel).resolve()
            if not src.is_file():
                continue
            check_inside_home(src, relative, "mcp_home_files")
            home_mounts.append((src, home / rel, True))

    # The executable must live in a visible mount; do not automatically grant
    # access to its parent directory (which might contain benchmark results).
    exe = shutil.which(command[0])
    if exe is None:
        raise FileNotFoundError(f"agent command not found: {command[0]}")
    binary = Path(exe).resolve()
    if not _inside(binary, ws) and not any(_inside(binary, src) for src, _ in mounts):
        raise ValueError(
            f"agent executable {binary} is not visible in the sandbox; "
            "add its runtime directory to [prover.sandbox].ro_paths"
        )

    argv = [
        bwrap, "--unshare-all", "--share-net", "--die-with-parent",
        "--new-session", "--proc", "/proc", "--dev", "/dev", "--tmpfs", "/tmp",
    ]
    created = {Path("/"), Path("/tmp"), Path("/proc"), Path("/dev")}

    def add_parents(destination: Path) -> None:
        for parent in reversed(destination.parents):
            if parent not in created:
                argv.extend(("--dir", str(parent)))
                created.add(parent)

    for src, dst in mounts:
        add_parents(dst)
        argv.extend(("--ro-bind", str(src), str(dst)))
        if src.is_dir():
            created.add(dst)
    add_parents(ws)
    argv.extend(("--bind", str(ws), str(ws), "--chdir", str(ws)))
    created.add(ws)
    created.add(home)
    # Bind these *after* the workspace; otherwise that bind would shadow the
    # read-only credential files with the underlying (empty) private home.
    for src, dst, read_only in home_mounts:
        add_parents(dst)
        argv.extend(("--ro-bind" if read_only else "--bind", str(src), str(dst)))
    # A sandboxed agent cannot see the host home, including previous sessions.
    # Keep caches and temporary data inside its disposable workspace.
    env = {
        "HOME": str(home),
        "XDG_CONFIG_HOME": str(home / ".config"),
        "XDG_CACHE_HOME": str(home / ".cache"),
        "XDG_DATA_HOME": str(home / ".local" / "share"),
        "TMPDIR": "/tmp",
        "DSH_HOME": str(home / ".dsh"),
        "CODEX_HOME": str(home / ".codex"),
        "CLAUDE_CONFIG_DIR": str(home / ".claude"),
    }
    for key, value in env.items():
        argv.extend(("--setenv", key, value))
    argv.extend(("--", *command))
    return argv, env
