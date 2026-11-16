"""Check or install the pinned Aeneas/Charon toolchain recorded in ``toolchain.toml``.

Usage::

    python3 utils/setup_toolchain.py            # verify the local toolchain (fast)
    python3 utils/setup_toolchain.py --install  # check out + build the pinned commits

The pipeline is tied to exact aeneas and charon commits, so this never pulls
upstream or runs ``lake update``; Lean dependencies stay at the versions in
``lean_mode/lake-manifest.json``.
"""

from __future__ import annotations

import argparse
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
AENEAS = ROOT / "aeneas"
CHARON = AENEAS / "charon"
LEAN_MODE = ROOT / "lean_mode"
PINS_FILE = ROOT / "toolchain.toml"


def load_pins() -> tuple[str, str]:
    pins = tomllib.loads(PINS_FILE.read_text())
    return pins["aeneas"]["commit"], pins["charon"]["commit"]


def git(repo: Path, *args: str) -> str:
    out = subprocess.run(["git", "-C", str(repo), *args], capture_output=True, text=True)
    return out.stdout.strip() if out.returncode == 0 else ""


def submodule_gitlink() -> str:
    # `git ls-tree HEAD aeneas` -> "160000 commit <hash>\taeneas"
    fields = git(ROOT, "ls-tree", "HEAD", "aeneas").split()
    return fields[2] if len(fields) >= 3 else ""


def charon_pin_file() -> str:
    path = AENEAS / "charon-pin"
    return path.read_text().strip().splitlines()[-1].strip() if path.exists() else ""


def charon_binary_version() -> str:
    exe = CHARON / "bin" / "charon"
    if not exe.exists():
        return ""
    out = subprocess.run([str(exe), "version"], capture_output=True, text=True)
    return out.stdout.strip()


def lean_toolchains() -> tuple[str, str]:
    def read(p: Path) -> str:
        return p.read_text().strip() if p.exists() else ""

    return (read(AENEAS / "backends" / "lean" / "lean-toolchain"),
            read(LEAN_MODE / "lean-toolchain"))


def check(aeneas_pin: str, charon_pin: str) -> list[str]:
    """Return a list of problems; empty means the toolchain matches the pins."""
    problems = []
    if submodule_gitlink() != aeneas_pin:
        problems.append(f"aeneas submodule gitlink is {submodule_gitlink() or '?'}, "
                        f"toolchain.toml pins {aeneas_pin}")
    head = git(AENEAS, "rev-parse", "HEAD")
    if head != aeneas_pin:
        problems.append(f"aeneas is checked out at {head or '?'}, expected {aeneas_pin}")
    head = git(CHARON, "rev-parse", "HEAD")
    if head != charon_pin:
        problems.append(f"aeneas/charon is checked out at {head or '?'}, expected {charon_pin}")
    if charon_pin_file() != charon_pin:
        problems.append(f"aeneas/charon-pin says {charon_pin_file() or '?'}, expected {charon_pin}")
    version = charon_binary_version()
    if charon_pin not in version:
        problems.append(f"charon binary reports '{version or 'missing'}', "
                        f"expected a build of {charon_pin[:9]}")
    if not (AENEAS / "bin" / "aeneas").exists():
        problems.append("aeneas binary aeneas/bin/aeneas is missing")
    backend, lean_mode = lean_toolchains()
    if backend != lean_mode:
        problems.append(f"lean_mode/lean-toolchain is '{lean_mode}', "
                        f"aeneas backend uses '{backend}'")
    return problems


def run(cmd: list[str], cwd: Path) -> None:
    print(f"$ (cd {cwd.relative_to(ROOT) if cwd != ROOT else '.'}) {' '.join(cmd)}", flush=True)
    subprocess.run(cmd, cwd=cwd, check=True)


def install(aeneas_pin: str, charon_pin: str) -> None:
    if submodule_gitlink() != aeneas_pin:
        sys.exit("toolchain.toml and the aeneas submodule gitlink disagree; fix one of them first")
    run(["git", "submodule", "update", "--init", "aeneas"], ROOT)
    if git(AENEAS, "rev-parse", "HEAD") != aeneas_pin:
        run(["git", "fetch", "origin", aeneas_pin], AENEAS)
        run(["git", "checkout", "--detach", aeneas_pin], AENEAS)
    # aeneas's Makefile refuses to build unless ./charon is at the commit named in
    # charon-pin, so record our pin there; `make setup-charon` then checks out
    # and builds exactly that charon commit.
    pin_file = AENEAS / "charon-pin"
    header = pin_file.read_text().splitlines()[0] if pin_file.exists() else "# charon commit"
    pin_file.write_text(f"{header}\n{charon_pin}\n")
    run(["make", "setup-charon"], AENEAS)
    run(["make"], AENEAS)
    backend, _ = lean_toolchains()
    (LEAN_MODE / "lean-toolchain").write_text(backend + "\n")
    run(["lake", "build"], LEAN_MODE)


def main() -> None:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--install", action="store_true",
                    help="check out and build the pinned commits, then verify")
    args = ap.parse_args()

    aeneas_pin, charon_pin = load_pins()
    if args.install:
        install(aeneas_pin, charon_pin)

    problems = check(aeneas_pin, charon_pin)
    if problems:
        print("Toolchain does not match toolchain.toml:", file=sys.stderr)
        for p in problems:
            print(f"  - {p}", file=sys.stderr)
        print("Run `python3 utils/setup_toolchain.py --install` to fix.", file=sys.stderr)
        sys.exit(1)
    print(f"Toolchain OK: aeneas {aeneas_pin[:9]}, charon {charon_pin[:9]}")


if __name__ == "__main__":
    main()
