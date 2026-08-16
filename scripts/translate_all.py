#!/usr/bin/env python3
"""Re-translate all Rust test cases in LeanMode/ concurrently (skip failure/)."""

import argparse
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN_MODE = ROOT / "lean_mode" / "LeanMode"
TRANSLATE = ROOT / "translation" / "translate.py"


def _parse_model(stdout: str) -> str:
    """Extract the resolved integer model from translate.py's stdout."""
    m = re.search(r"int-model=(\w+)", stdout)
    return m.group(1) if m else "?"


def _parse_float_model(stdout: str) -> str:
    """Extract the resolved float-model (Rat/Real) from translate.py's stdout.

    The final value lives in the ``done`` line (after any auto-switch to Real),
    so take the last ``float-model=…`` occurrence.
    """
    m = re.findall(r"float-model=(\w+)", stdout)
    return m[-1] if m else "?"


def main() -> None:
    parser = argparse.ArgumentParser(description="Re-translate all LeanMode test cases")
    parser.add_argument(
        "-w",
        "--workers",
        type=int,
        default=16,
        help="Number of concurrent workers (default: 16)",
    )
    args = parser.parse_args()

    dirs = sorted(
        d
        for d in LEAN_MODE.iterdir()
        if d.is_dir() and d.name != "failure" and (d / "Cargo.toml").exists()
    )

    passed: list[str] = []
    failed: list[str] = []
    done = threading.Event()
    lock = threading.Lock()
    remaining = len(dirs)

    def translate_one(d: Path) -> None:
        """Translate a single project and print status immediately."""
        nonlocal remaining
        result = subprocess.run(
            [sys.executable, str(TRANSLATE), str(d)],
            capture_output=True,
            text=True,
        )
        ok = result.returncode == 0
        model = _parse_model(result.stdout)
        float_model = _parse_float_model(result.stdout)
        with lock:
            status = "OK" if ok else "FAIL"
            print(
                f"{d.name:<30} int-model={model:<6} float-model={float_model:<4} {status}",
                flush=True,
            )
            if not ok:
                for line in result.stderr.splitlines():
                    if "FAIL" in line or "Error" in line or "error" in line:
                        print(f"    {line.strip()}", flush=True)
            (passed if ok else failed).append(d.name)
            remaining -= 1
            if remaining == 0:
                done.set()

    print(f"Translating {len(dirs)} projects ({args.workers} concurrent)...\n")

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        for d in dirs:
            executor.submit(translate_one, d)

    done.wait()

    print()
    print(f"Passed: {len(passed)}, Failed: {len(failed)}")

    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
