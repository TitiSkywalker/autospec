"""
Aeneas workspace for Rust verification.
"""

import os
import re
import shutil
import subprocess
from pathlib import Path

from evaluation.logger import get_logger
from utils.runner import CmdRunner

# The Lean library name. `create_workspace` produces a package named after it.
LIB = "RustVerif"
logger = get_logger(None, "Workspace")


def aeneas_pinned_commit() -> str:
    """The aeneas commit this repository's submodule is pinned to."""
    root = Path(__file__).resolve().parent.parent
    out = subprocess.run(
        ["git", "ls-tree", "HEAD", "aeneas"],
        cwd=root,
        capture_output=True,
        text=True,
        check=True,
    )
    # Output is "<mode> commit <sha>\taeneas"
    return out.stdout.split()[2]


def create_workspace(
    workspace_path: str,
    vibe_dir: str | Path | None = None,
    base_dir: str | Path | None = None,
    mcp_port: int | None = None,
):
    """Create an independent Aeneas workspace with a clean git checkpoint.

    The workspace compiles the whole import chain ``Main -> RustVerif -> Basic -> Spec -> Function`` so a later ``lake build`` checks the specification.

    ``.lake`` (Mathlib's build tree) and ``aeneas`` are git-ignored, so the
    checkpoint is cheap to restore between test cases. If ``base_dir`` names an
    existing workspace it is copied verbatim (prebuilt aeneas + Mathlib) instead
    of being rebuilt from scratch. ``vibe_dir`` (if given) is always copied to
    the workspace's ``.vibe/`` so the proving agent is self-contained. If
    ``mcp_port`` is given, the deployed ``.vibe/config.toml`` is pointed at a
    lean-lsp-mcp HTTP server on that port (127.0.0.1).
    """
    workspace = Path(workspace_path)
    # Already initialized (e.g. from a previous run): reuse it as-is.
    if (workspace / ".git").exists():
        logger.info(f"reusing workspace {workspace}")
        return

    # Toolchain: build aeneas + Mathlib from scratch, or copy a prebuilt base.
    if base_dir is not None and Path(base_dir).is_dir():
        logger.info(f"copying base mirror -> {workspace}")
        shutil.copytree(base_dir, workspace, symlinks=True, dirs_exist_ok=True)
    else:
        _build_workspace(workspace_path)

    # Deploy the project-local vibe config (config.toml, agents/, ...).
    if vibe_dir is not None and Path(vibe_dir).is_dir():
        logger.info(f"deploying vibe config -> {workspace / '.vibe'}")
        shutil.copytree(vibe_dir, workspace / ".vibe", dirs_exist_ok=True)
    if mcp_port is not None:
        logger.info(f"pointing mcp server at port {mcp_port}")
        point_mcp_server(workspace, mcp_port)

    # Save a checkpoint so the workspace can be reset between test cases.
    logger.info(f"saving git checkpoint in {workspace}")
    runner = CmdRunner(cwd=workspace_path, verbose=False)
    runner.run("git init")
    runner.run("git add -A")
    runner.run(
        'git -c user.name="autospec" -c user.email="autospec@localhost" '
        'commit --allow-empty -m "checkpoint"'
    )


def point_mcp_server(workspace_path: str | Path, port: int) -> None:
    """Point the workspace's ``.vibe/config.toml`` at its own lean-lsp-mcp HTTP server."""
    config = Path(workspace_path) / ".vibe" / "config.toml"
    if not config.is_file():
        return
    text = config.read_text()
    text, n = re.subn(
        r"http://127\.0\.0\.1:\d+/mcp",
        f"http://127.0.0.1:{port}/mcp",
        text,
    )
    if n:
        config.write_text(text)


def _build_workspace(workspace_path: str):
    """Build the aeneas toolchain and Mathlib from scratch in ``workspace_path``."""
    workspace = Path(workspace_path)
    os.makedirs(workspace, exist_ok=True)
    logger.info(f"building workspace {workspace} from scratch")
    runner = CmdRunner(cwd=workspace_path, verbose=False)
    runner.run(f"lake init {LIB}")

    # Pin aeneas to the same commit this repository's submodule uses
    aeneas_commit = aeneas_pinned_commit()
    logger.info(f"cloning aeneas @ {aeneas_commit[:8]}")
    runner.run("git clone https://github.com/AeneasVerif/aeneas.git")
    runner.cd("aeneas")
    runner.run(f"git checkout {aeneas_commit}")
    # Fail fast if the checkout did not land on the expected commit.
    runner.run(f'test "$(git rev-parse HEAD)" = "{aeneas_commit}"')
    logger.info("building aeneas (make setup-charon && make)")
    runner.run("make setup-charon")
    runner.run("make")
    runner.cd("..")

    # Ignore the nested aeneas repo on top of the `.lake` that `lake init` ignored.
    with open(workspace / ".gitignore", "a") as file:
        file.write("/aeneas\n")

    with open(workspace / "lakefile.toml", "a") as file:
        file.write("""\n[[require]]
name = "aeneas"
path = "./aeneas/backends/lean"\n
""")

    # Module skeleton: the library root imports Basic, which imports Spec, which (once a test case is installed) imports Function.
    lib = workspace / LIB
    lib.mkdir(parents=True, exist_ok=True)
    (lib / "Function.lean").write_text("-- Target function.\n")
    (lib / "Spec.lean").write_text("-- Specification.\n")
    (lib / "Basic.lean").write_text(f'import {LIB}.Spec\n\ndef hello := "world"\n')
    (workspace / f"{LIB}.lean").write_text(f"import {LIB}.Basic\n")
    (workspace / "Main.lean").write_text(
        f'import {LIB}\n\ndef main : IO Unit :=\n  IO.println s!"Hello, {{hello}}!"\n'
    )

    logger.info("lake update")
    runner.run("cp ./aeneas/backends/lean/lean-toolchain ./lean-toolchain")
    runner.run("lake update -v")
    logger.info("lake build (Mathlib)")
    runner.run("lake build")


if __name__ == "__main__":
    create_workspace("./local/base/")
