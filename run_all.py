#!/usr/bin/env python3
"""Batch runner — plan extraction targets, then execute workers concurrently."""

import json
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import toml
from clone_repos import clone_repos
from agent import VibeAgent


def run_planner(repo_path: Path, results_dir: Path, config: dict) -> list[dict]:
    """Run a planner agent to discover extraction targets."""
    plan_file = results_dir / "plan.json"
    if plan_file.exists():
        plan = json.loads(plan_file.read_text())
        print(f"  Plan cached: {len(plan)} targets")
        return plan

    print("  Planning...")
    prompt = f"Write the plan to: {plan_file}"
    agent = VibeAgent(
        repo_path,
        instruction_file="instruction_plan.md",
        results_dir=str(results_dir),
        max_turns=config.get("plan_turns", 30),
        timeout=config.get("timeout", 7200),
        verbose=config.get("verbose", True),
    )
    result = agent.run(prompt=prompt)

    if plan_file.exists():
        plan = json.loads(plan_file.read_text())
        print(f"  Plan: {len(plan)} targets")
        return plan
    else:
        print(f"  Plan failed: {result.get('error', 'no plan.json produced')}")
        return []


def _worker_target(args: tuple) -> dict:
    """Module-level callable for ThreadPoolExecutor."""
    repo_path, target, results_dir, config = args
    fn_name = f"{target['struct_name'].lower()}_{target['method']}".replace("::", "_")
    fn_output_dir = results_dir / fn_name

    if (fn_output_dir / "src" / "main.rs").exists():
        return {"name": fn_name, "ok": True, "cached": True, "elapsed": 0}

    prompt = (
        f"Extract this method:\n"
        f"  File: {target['file']}\n"
        f"  Method: {target['method']}\n"
        f"  Struct: {target['struct_name']}\n"
        f"  Complexity: {target['complexity']}\n"
        f"  Output directory: {fn_output_dir}\n"
    )

    log_dir = results_dir / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / f"{fn_name}.log"

    agent = VibeAgent(
        repo_path,
        instruction_file="instruction.md",
        results_dir=str(results_dir),
        max_turns=config.get("work_turns", 30),
        timeout=config.get("timeout", 7200),
        verbose=config.get("verbose", True),
        log_file=str(log_file),
    )
    result = agent.run(prompt=prompt)
    ok = (fn_output_dir / "src" / "main.rs").exists()
    return {"name": fn_name, "ok": ok, "elapsed": result["elapsed"]}


def process_repo(repo_path: Path, config: dict) -> list[dict]:
    """Plan + execute workers concurrently for one repo."""
    name = repo_path.name
    results_dir = Path(config["results_dir"]).resolve() / name

    print(f"\n{'=' * 50}")
    print(f"Planning: {name}")
    print(f"{'=' * 50}")
    targets = run_planner(repo_path, results_dir, config)

    if not targets:
        return []

    # Split: cached vs pending
    cached, pending = [], []
    for t in targets:
        fn_name = f"{t['struct_name'].lower()}_{t['method']}".replace("::", "_")
        if (results_dir / fn_name / "src" / "main.rs").exists():
            cached.append({"repo": name, "function": fn_name, "ok": True, "cached": True})
        else:
            pending.append(t)

    if cached:
        print(f"  Cached: {len(cached)} targets")

    results = list(cached)

    if pending:
        max_workers = config.get("max_workers", 4)
        print(f"  Executing: {len(pending)} targets ({max_workers} workers)")
        t0 = time.time()

        args = [(repo_path, t, results_dir, config) for t in pending]
        with ThreadPoolExecutor(max_workers=max_workers) as pool:
            futures = {pool.submit(_worker_target, a): a for a in args}
            for i, future in enumerate(as_completed(futures)):
                r = future.result()
                results.append({
                    "repo": name,
                    "function": r["name"],
                    "ok": r["ok"],
                    "cached": False,
                })
                status = "OK" if r["ok"] else "FAIL"
                print(f"  [{i+1}/{len(pending)}] {status} {r['name']}")

        elapsed = time.time() - t0
        print(f"  Done in {elapsed:.0f}s")

    # Manifest
    manifest = results_dir / "manifest.txt"
    manifest.write_text("\n".join(
        r["function"] for r in results if r["ok"]
    ) + "\n")

    return results


def main():
    config_path = sys.argv[1] if len(sys.argv) > 1 else "config.toml"
    config = toml.loads(Path(config_path).read_text())

    print("=" * 50)
    print("Cloning repos...")
    print("=" * 50)
    paths = clone_repos(config_path)

    all_results = []
    t0 = time.time()

    for repo_path in paths:
        all_results.extend(process_repo(repo_path, config))

    total = len(all_results)
    ok = sum(1 for r in all_results if r["ok"])
    cached = sum(1 for r in all_results if r.get("cached"))
    elapsed = time.time() - t0
    print(f"\n{'=' * 50}")
    print(f"Done. {ok}/{total} OK, {cached} cached ({elapsed / 60:.0f}m)")


if __name__ == "__main__":
    main()
