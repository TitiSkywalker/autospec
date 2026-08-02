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

# ── display ──────────────────────────────────────────────────────────────────

_progress: dict[str, tuple[int, int]] = {}   # name -> (turn, max_turns)
_slots: list[str] = []                        # worker names in display order
_lines_drawn: int = 0                         # how many lines to overwrite


def _progress_cb(name: str) -> callable:
    def cb(turn: int, max_turns: int) -> None:
        _progress[name] = (turn, max_turns)
        if name not in _slots:
            _slots.append(name)
        _redraw()
    return cb


def _bar(current: int, total: int, width: int = 12) -> str:
    if total <= 0:
        return "[" + "-" * width + "]"
    n = int(width * current / total)
    return "[" + "#" * n + "-" * (width - n) + "]"


def _redraw() -> None:
    """Redraw progress bars — one per line, overwriting previous."""
    global _lines_drawn
    if not _slots:
        return
    lines = []
    for name in _slots:
        t, m = _progress.get(name, (0, 0))
        lines.append(f"  {_bar(t, m)} {t:3d}/{m:<3d} {name}")
    # Move up to first progress line, then overwrite
    if _lines_drawn > 0:
        sys.stderr.write(f"\033[{_lines_drawn}A")
    for line in lines:
        sys.stderr.write(f"\r\033[K{line}\n")
    _lines_drawn = len(lines)
    sys.stderr.flush()


def _print_step(label: str, name: str, status: str, detail: str = "") -> None:
    """Print a result line below the progress bars."""
    # Clear progress bars first
    global _lines_drawn
    if _lines_drawn > 0:
        sys.stderr.write(f"\033[{_lines_drawn}A")
    _lines_drawn = 0
    # Print the result
    sys.stderr.write(f"\r\033[K  [{label}] {name:40s} {status:5s} {detail}\n")
    # Remove from active
    _progress.pop(name, None)
    if name in _slots:
        _slots.remove(name)
    # Redraw remaining
    _redraw()


# ── planner ──────────────────────────────────────────────────────────────────

def run_planner(repo_path: Path, extract_dir: Path, config: dict) -> list[dict]:
    """Run a planner agent to discover extraction targets."""
    plan_file = extract_dir / "plan.json"
    if plan_file.exists():
        plan = json.loads(plan_file.read_text())
        _print_step("plan", repo_path.name, "CACHE", f"{len(plan)} targets")
        return plan

    # Use a log file to suppress verbose output
    log_dir = extract_dir / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / "planner.log"

    prompt = f"Write the plan to: {plan_file}"
    agent = VibeAgent(
        repo_path,
        instruction_file="instruction_plan.md",
        extract_dir=str(extract_dir),
        max_turns=config.get("plan_turns", 30),
        timeout=config.get("timeout", 7200),
        verbose=False,
        log_file=str(log_file),
        progress_callback=_progress_cb(f"[plan] {repo_path.name}"),
    )
    result = agent.run(prompt=prompt)

    if plan_file.exists():
        plan = json.loads(plan_file.read_text())
        elapsed = result.get("elapsed", 0)
        _print_step("plan", repo_path.name, "OK", f"{len(plan)} targets, {elapsed:.0f}s")
        return plan
    else:
        _print_step("plan", repo_path.name, "FAIL", result.get("error", "no plan.json"))
        return []


# ── extract worker ───────────────────────────────────────────────────────────

def _worker_target(args: tuple) -> dict:
    """Extract a single method → results/{repo}/{fn}/."""
    repo_path, target, extract_dir, config = args
    fn_name = f"{target['struct_name'].lower()}_{target['method']}".replace("::", "_")
    fn_output_dir = extract_dir / fn_name

    if (fn_output_dir / "src" / "main.rs").exists():
        _print_step("extract", fn_name, "CACHE")
        return {"name": fn_name, "ok": True, "cached": True, "elapsed": 0, "turns": 0}

    prompt = (
        f"Extract this method:\n"
        f"  File: {target['file']}\n"
        f"  Method: {target['method']}\n"
        f"  Struct: {target['struct_name']}\n"
        f"  Complexity: {target['complexity']}\n"
        f"  Output directory: {fn_output_dir}\n"
    )

    log_dir = extract_dir / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    log_file = log_dir / f"{fn_name}.log"

    max_turns = config.get("work_turns", 30)
    agent = VibeAgent(
        repo_path,
        instruction_file="instruction.md",
        extract_dir=str(fn_output_dir),
        max_turns=max_turns,
        timeout=config.get("timeout", 7200),
        verbose=False,
        cwd=repo_path,
        log_file=str(log_file),
        progress_callback=_progress_cb(f"[extract] {fn_name}"),
    )
    t0 = time.time()
    result = agent.run(prompt=prompt)
    elapsed = time.time() - t0
    ok = (fn_output_dir / "src" / "main.rs").exists()
    turns = result.get("turn_summary", {}).get("total_turns", max_turns)
    status = "OK" if ok else "FAIL"
    _print_step("extract", fn_name, status, f"{turns}T, {elapsed:.0f}s")
    return {"name": fn_name, "ok": ok, "elapsed": elapsed, "turns": turns}


# ── refine worker ────────────────────────────────────────────────────────────

def _refiner_target(args: tuple) -> dict:
    """Fix aeneas compatibility → refined/{repo}/{fn}/."""
    extract_dir, refined_dir, fn_name, config = args
    src_dir = extract_dir / fn_name
    dst_dir = refined_dir / fn_name

    if (dst_dir / "src" / "main.rs").exists():
        _print_step("refine", fn_name, "CACHE")
        return {"name": fn_name, "ok": True, "cached": True, "elapsed": 0, "turns": 0}

    dst_dir.mkdir(parents=True, exist_ok=True)

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

    max_turns = config.get("refine_turns", 20)
    agent = VibeAgent(
        dst_dir,
        instruction_file="instruction_refine.md",
        extract_dir=str(dst_dir),
        max_turns=max_turns,
        timeout=config.get("timeout", 7200),
        verbose=False,
        log_file=str(log_file),
        progress_callback=_progress_cb(f"[refine] {fn_name}"),
    )
    t0 = time.time()
    result = agent.run(prompt=prompt)
    elapsed = time.time() - t0
    ok = (dst_dir / "src" / "main.rs").exists()
    turns = result.get("turn_summary", {}).get("total_turns", max_turns)
    status = "OK" if ok else "FAIL"
    _print_step("refine", fn_name, status, f"{turns}T, {elapsed:.0f}s")
    return {"name": fn_name, "ok": ok, "elapsed": elapsed, "turns": turns}


# ── repo processor ───────────────────────────────────────────────────────────

def process_repo(repo_path: Path, config: dict) -> dict:
    """Plan + extract + refine for one repo."""
    name = repo_path.name
    extract_dir = Path(config["extract_dir"]).resolve() / name
    refined_dir = Path(config["refined_dir"]).resolve() / name

    print(f"\n{'=' * 50}")
    print(f"  {name}")
    print(f"{'=' * 50}")

    # ── Step 1: Plan ──
    print(f"  >>> plan")
    targets = run_planner(repo_path, extract_dir, config)
    if not targets:
        return {"extracted": [], "refined": []}

    # ── Step 2: Extract ──
    print(f"  >>> extract ({len(targets)} targets)")
    cached, pending = [], []
    for t in targets:
        fn_name = f"{t['struct_name'].lower()}_{t['method']}".replace("::", "_")
        if (extract_dir / fn_name / "src" / "main.rs").exists():
            cached.append({"name": fn_name, "ok": True, "cached": True})
        else:
            pending.append(t)

    extracted = list(cached)

    max_workers = config.get("max_workers", 4)
    if pending:
        t0 = time.time()
        args = [(repo_path, t, extract_dir, config) for t in pending]
        with ThreadPoolExecutor(max_workers=max_workers) as pool:
            futures = {pool.submit(_worker_target, a): a for a in args}
            for future in as_completed(futures):
                r = future.result()
                extracted.append(r)
        elapsed = time.time() - t0
        n = len(pending)
        ok = sum(1 for r in extracted if r["ok"] and not r.get("cached"))
        print(f"  {'extract':10s} {ok}/{n} done ({elapsed:.0f}s)")

    manifest = extract_dir / "manifest.txt"
    manifest.write_text("\n".join(
        r["name"] for r in extracted if r["ok"]) + "\n")

    # ── Step 3: Refine ──
    n_ok = sum(1 for r in extracted if r["ok"])
    print(f"  >>> refine ({n_ok} targets)")
    ok_extracted = [r for r in extracted if r["ok"]]
    ref_cached, ref_pending = [], []
    for r in ok_extracted:
        if (refined_dir / r["name"] / "src" / "main.rs").exists():
            ref_cached.append(r)
        else:
            ref_pending.append(r)

    refined = [{"name": r["name"], "ok": True, "cached": True} for r in ref_cached]

    if ref_pending:
        t0 = time.time()
        args = [(extract_dir, refined_dir, r["name"], config) for r in ref_pending]
        with ThreadPoolExecutor(max_workers=max_workers) as pool:
            futures = {pool.submit(_refiner_target, a): a for a in args}
            for future in as_completed(futures):
                r = future.result()
                refined.append(r)
        elapsed = time.time() - t0
        n = len(ref_pending)
        ok = sum(1 for r in refined if r["ok"] and not r.get("cached"))
        print(f"  {'refine':10s} {ok}/{n} done ({elapsed:.0f}s)")

    ref_manifest = refined_dir / "manifest.txt"
    ref_manifest.parent.mkdir(parents=True, exist_ok=True)
    ref_manifest.write_text("\n".join(
        r["name"] for r in refined if r["ok"]) + "\n")

    return {"extracted": extracted, "refined": refined}


# ── main ─────────────────────────────────────────────────────────────────────

def main():
    config_path = sys.argv[1] if len(sys.argv) > 1 else "config.toml"
    config = toml.loads(Path(config_path).read_text())

    print("Cloning repos...")
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
    print(f"\nDone. Extracted: {ext_ok}/{len(all_extracted)}, "
          f"Refined: {ref_ok}/{len(all_refined)} ({elapsed / 60:.0f}m)")


if __name__ == "__main__":
    main()
