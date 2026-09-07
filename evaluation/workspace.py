"""
Aeneas workspace for Rust verification.
"""

import os
import re
import shutil
import subprocess
from pathlib import Path

from utils.runner import CmdRunner

# The Lean library name. `create_workspace` produces a package named after it and `setup_testcase` copies each benchmark case into `<LIB>/Function.lean` and `<LIB>/Spec.lean`.
LIB = "RustVerif"


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


def create_workspace(workspace_path: str, vibe_dir: str | Path | None = None):
    """Create an independent Aeneas workspace with a clean git checkpoint.

    The workspace compiles the whole import chain ``Main -> RustVerif -> Basic -> Spec -> Function`` so a later ``lake build`` checks the specification.

    ``.lake`` (Mathlib's build tree) and ``aeneas`` are git-ignored, so the
    checkpoint is cheap to restore between test cases. If ``vibe_dir`` names an
    existing directory its whole tree is copied to the workspace's ``.vibe/``.
    """
    workspace = Path(workspace_path)
    # Already initialized (e.g. from a previous run): reuse it as-is.
    if (workspace / ".git").exists():
        return

    os.makedirs(workspace, exist_ok=True)

    # Initialize command line runner
    runner = CmdRunner(cwd=workspace_path)
    runner.run(f"lake init {LIB}")

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

    runner.run("cp ./aeneas/backends/lean/lean-toolchain ./lean-toolchain")
    runner.run("lake update -v")
    runner.run("lake build")

    # Deploy the complete project-level vibe config (config.toml, agents/, ...)
    # so the proving agent is self-contained inside the workspace.
    if vibe_dir is not None:
        src = Path(vibe_dir)
        if src.is_dir():
            shutil.copytree(src, workspace / ".vibe", dirs_exist_ok=True)

    # Save a checkpoint so the workspace can be reset between test cases. Ignore `aeneas` (a nested git repo) in addition to the `.lake` that `lake init` already ignored.
    with open(workspace / ".gitignore", "a") as file:
        file.write("/aeneas\n")
    runner.run("git init")
    runner.run("git add -A")
    runner.run(
        'git -c user.name="autospec" -c user.email="autospec@localhost" '
        'commit -m "checkpoint"'
    )


def setup_testcase(workspace_path: str, func_src: str, spec_src: str):
    """Install a test case's translated function and specification.

    The translated function is written to ``RustVerif/Function.lean`` (it only imports ``Aeneas``, so no import fixing is needed). The specification's ``import LeanMode.<dir>.<Crate>`` line is rewritten to ``import RustVerif.Function``.
    """
    lib = Path(workspace_path) / LIB
    (lib / "Function.lean").write_text(func_src)
    spec_src = re.sub(r"import LeanMode\.[^\n]+", f"import {LIB}.Function", spec_src)
    (lib / "Spec.lean").write_text(spec_src)


def clear_workspace(workspace_path: str):
    """Restore the workspace to its checkpoint, keeping ``.lake`` and ``aeneas``."""
    runner = CmdRunner(cwd=workspace_path)
    runner.run("git reset --hard HEAD", verbose=False, panic=False)
    runner.run("git clean -fd", verbose=False, panic=False)


if __name__ == "__main__":
    create_workspace("./local/temp/")
