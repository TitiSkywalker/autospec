"""
Update toolchain.
"""

from utils import CmdRunner

if __name__ == "__main__":
    runner = CmdRunner()
    runner.cd("aeneas")
    runner.run("git checkout main")
    runner.run("git pull origin")
    runner.run("make setup-charon")
    runner.run("make")

    runner.cd("..")
    runner.run("cp ./aeneas/backends/lean/lean-toolchain ./lean_mode/lean-toolchain")

    runner.cd("lean_mode")
    runner.run("lake update -v")
    runner.run("lake build")
