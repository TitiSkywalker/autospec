#!/usr/bin/env python3
"""Batch runner — plan, extract, then refine for aeneas compatibility.

Workflow:
  1. Planner:   discover extraction targets → results/{repo}/plan.json
  2. Worker:    extract each method            → results/{repo}/{fn}/
  3. Refiner:   fix aeneas compatibility       → refined/{repo}/{fn}/
"""

import json
import shutil
import sys
import time
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path

import toml

from agent import VibeAgent
from clone_repos import clone_repos


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
    """Extract a single method → results/{repo}/{fn}/."""
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


def _refiner_target(args: tuple) -> dict:
    """Fix aeneas compatibility → refined/{repo}/{fn}/.
    Reads the original from results_dir, writes to refined_dir.
    """
    results_dir, refined_dir, fn_name, config = args
    src_dir = results_dir / fn_name
    dst_dir = refined_dir / fn_name

    if (dst_dir / "src" / "main.rs").exists():
        return {"name": fn_name, "ok": True, "cached": True, "elapsed": 0}

    dst_dir.mkdir(parents=True, exist_ok=True)

    # Copy the original to the refined directory so the agent can work in-place
    if not (dst_dir / "src").exists():
        shutil.copytree(src_dir, dst_dir, dirs_exist_ok=True,
                        ignore=shutil.ignore_patterns("target"))

    prompt = (
        f"Fix aeneas compatibility for this extracted function.\n"
        f"  Input (original, do not modify): {src_dir}\n"
        f"  Output (write fixed version here): {dst_dir}\n"
    )

    log_dir = refined_dir / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / f"{fn_name}.log"

    agent = VibeAgent(
        dst_dir,
        instruction_file="instruction_refine.md",
        results_dir=str(refined_dir),  # parent dir, agent appends fn_name
        max_turns=config.get("refine_turns", 20),
        timeout=config.get("timeout", 7200),
        verbose=config.get("verbose", True),
        log_file=str(log_file),
    )
    result = agent.run(prompt=prompt)
    ok = (dst_dir / "src" / "main.rs").exists()
    return {"name": fn_name, "ok": ok, "elapsed": result["elapsed"]}


def process_repo(repo_path: Path, config: dict) -> dict:
    """Plan + extract + refine for one repo."""
    name = repo_path.name
    results_dir = Path(config["results_dir"]).resolve() / name
    refined_dir = Path(config["refined_dir"]).resolve() / name

    print(f"\n{'=' * 50}")
    print(f"Planning: {name}")
    print(f"{'=' * 50}")

    # ── Step 1: Plan ──
    targets = run_planner(repo_path, results_dir, config)
    if not targets:
        return {"extracted": [], "refined": []}

    # ── Step 2: Extract ──
    cached, pending = [], []
    for t in targets:
        fn_name = f"{t['struct_name'].lower()}_{t['method']}".replace("::", "_")
        if (results_dir / fn_name / "src" / "main.rs").exists():
            cached.append({"name": fn_name, "ok": True, "cached": True})
        else:
            pending.append(t)

    if cached:
        print(f"  Extract cached: {len(cached)}")
    extracted = list(cached)

    if pending:
        max_workers = config.get("max_workers", 4)
        print(f"  Extracting: {len(pending)} targets ({max_workers} workers)")
        t0 = time.time()
        args = [(repo_path, t, results_dir, config) for t in pending]
        with ThreadPoolExecutor(max_workers=max_workers) as pool:
            futures = {pool.submit(_worker_target, a): a for a in args}
            for future in as_completed(futures):
                r = future.result()
                extracted.append(r)
                status = "OK" if r["ok"] else "FAIL"
                print(f"  [{len(extracted)-len(cached)}/{len(pending)}] {status} {r['name']}")
        print(f"  Extract done in {time.time() - t0:.0f}s")

    # Manifest
    manifest = results_dir / "manifest.txt"
    manifest.write_text("\n".join(
        r["name"] for r in extracted if r["ok"]) + "\n")

    # ── Step 3: Refine ──
    ok_extracted = [r for r in extracted if r["ok"]]
    ref_cached, ref_pending = [], []
    for r in ok_extracted:
        if (refined_dir / r["name"] / "src" / "main.rs").exists():
            ref_cached.append(r)
        else:
            ref_pending.append(r)

    if ref_cached:
        print(f"  Refine cached: {len(ref_cached)}")
    refined = [{"name": r["name"], "ok": True, "cached": True} for r in ref_cached]

    if ref_pending:
        print(f"  Refining: {len(ref_pending)} targets")
        t0 = time.time()
        args = [(results_dir, refined_dir, r["name"], config) for r in ref_pending]
        with ThreadPoolExecutor(max_workers=max_workers) as pool:
            futures = {pool.submit(_refiner_target, a): a for a in args}
            for future in as_completed(futures):
                r = future.result()
                refined.append(r)
                status = "OK" if r["ok"] else "FAIL"
                print(f"  [{len(refined)-len(ref_cached)}/{len(ref_pending)}] {status} {r['name']}")
        print(f"  Refine done in {time.time() - t0:.0f}s")

    # Refine manifest
    ref_manifest = refined_dir / "manifest.txt"
    ref_manifest.parent.mkdir(parents=True, exist_ok=True)
    ref_manifest.write_text("\n".join(
        r["name"] for r in refined if r["ok"]) + "\n")

    return {"extracted": extracted, "refined": refined}


def main():
    config_path = sys.argv[1] if len(sys.argv) > 1 else "config.toml"
    config = toml.loads(Path(config_path).read_text())

    print("=" * 50)
    print("Cloning repos...")
    print("=" * 50)
    paths = clone_repos(config_path)

    all_extracted = []
    all_refined = []
    t0 = time.time()

    for repo_path in paths:
        result = process_repo(repo_path, config)
        all_extracted.extend(result["extracted"])
        all_refined.extend(result["refined"])

    ext_ok = sum(1 for r in all_extracted if r["ok"])
    ref_ok = sum(1 for r in all_refined if r["ok"])
    elapsed = time.time() - t0
    print(f"\n{'=' * 50}")
    print(f"Done. Extracted: {ext_ok}/{len(all_extracted)}, "
          f"Refined: {ref_ok}/{len(all_refined)} ({elapsed / 60:.0f}m)")


if __name__ == "__main__":
    main()
