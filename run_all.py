#!/usr/bin/env python3
"""Batch runner — clone repos then run VibeAgent sequentially."""

import sys
import time
from pathlib import Path

import toml
from clone_repos import clone_repos
from agent import VibeAgent


def main():
    config_path = sys.argv[1] if len(sys.argv) > 1 else "config.toml"
    config = toml.loads(Path(config_path).read_text())

    # Clone
    print("=" * 50)
    print("Cloning repos...")
    print("=" * 50)
    paths = clone_repos(config_path)

    # Config
    results_dir = config.get("results_dir", "results")
    max_turns = config.get("max_turns", 50)
    timeout = config.get("timeout", 7200)
    verbose = config.get("verbose", True)

    all_results = []
    t0 = time.time()

    for i, repo_path in enumerate(paths):
        name = repo_path.name
        print(f"\n{'=' * 50}")
        print(f"[{i + 1}/{len(paths)}] {name}")
        print(f"{'=' * 50}")

        agent = VibeAgent(
            repo_path,
            results_dir=results_dir,
            max_turns=max_turns,
            timeout=timeout,
            verbose=verbose,
        )
        result = agent.run()
        r = result.get("results", {})
        print(f"  Functions: {r.get('count', 0)}")
        print(f"  Elapsed:   {result['elapsed']:.0f}s")

        all_results.append({
            "repo": name,
            "ok": result["ok"],
            "elapsed": result["elapsed"],
            "functions": r.get("count", 0),
        })

    total = sum(r["functions"] for r in all_results)
    elapsed = time.time() - t0
    print(f"\n{'=' * 50}")
    print(f"Done. {total} functions ({elapsed / 60:.0f}m)")
    print(f"{'=' * 50}")
    for r in all_results:
        status = "✓" if r["ok"] else "✗"
        print(f"  {status} {r['repo']}: {r['functions']} fns ({r['elapsed']:.0f}s)")


if __name__ == "__main__":
    main()
