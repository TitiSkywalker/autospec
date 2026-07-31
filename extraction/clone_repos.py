#!/usr/bin/env python3
"""Clone repos listed in a config.toml file."""

import subprocess
import sys
from pathlib import Path

import toml


def clone_repos(config_path: str = "config.toml") -> list[Path]:
    """Read config.toml, clone every repo in [repos], return local paths."""

    config = toml.loads(Path(config_path).read_text())
    workdir = Path(config.get("workdir", "/tmp/deoop-repos"))
    workdir.mkdir(parents=True, exist_ok=True)
    shallow = config.get("shallow", True)

    paths = []
    for item in config.get("repo", []):
        url = item["url"]
        name = item.get("name") or url.rstrip("/").removesuffix(".git").split("/")[-1]
        target = workdir / name

        if (target / ".git").exists():
            print(f"  [skip] {name} (already cloned)")
            paths.append(target)
            continue

        cmd = ["git", "clone"]
        if shallow:
            cmd += ["--depth", "1"]
        cmd += [url, str(target)]

        print(f"  [clone] {name} ...", end=" ", flush=True)
        r = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        if r.returncode == 0:
            print("OK")
            paths.append(target)
        else:
            print("FAIL")
            if target.exists():
                subprocess.run(["rm", "-rf", str(target)])

    return paths


if __name__ == "__main__":
    cfg = sys.argv[1] if len(sys.argv) > 1 else "config.toml"
    paths = clone_repos(cfg)
    print(f"Cloned {len(paths)} repo(s).")
