"""
Prepare a full Aeneas workspace for Rust verification.
"""

import os
import subprocess
from pathlib import Path

from utils.runner import CmdRunner


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


def create_workspace(workspace_path: str, workspace_name: str):
    # Creat target directory
    os.makedirs(workspace_path, exist_ok=True)

    # Initialize command line runner
    runner = CmdRunner(cwd=workspace_path)
    runner.run(f"lake init {workspace_name}")

    # Pin aeneas to the same commit this repository's submodule uses
    aeneas_commit = aeneas_pinned_commit()
    runner.run("git clone https://github.com/AeneasVerif/aeneas.git")
    runner.cd("aeneas")
    runner.run(f"git checkout {aeneas_commit}")
    # Fail fast if the checkout did not land on the expected commit.
    runner.run(f'test "$(git rev-parse HEAD)" = "{aeneas_commit}"')
    runner.run("make setup-charon")
    runner.run("make")
    runner.cd("..")

    with open(Path(workspace_path) / "lakefile.toml", "a") as file:
        file.write("""\n[[require]]
name = "aeneas"
path = "./aeneas/backends/lean"\n
""")

    runner.run("cp ./aeneas/backends/lean/lean-toolchain ./lean-toolchain")
    runner.run("lake update -v")
    runner.run("lake build")


def clear_workspace(workspace_path: str):
    # TODO: rollback all changes after initialization using git
    # This is equivalent to recreating the workspace
    pass


if __name__ == "__main__":
    create_workspace("./local/temp/", "RustVerif")
