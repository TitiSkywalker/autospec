#!/usr/bin/env python3
"""Build every .lean file under LeanMode/ individually with ``lake build``.

For each test-case directory three files are checked when present:

* ``<Crate>.lean``   – translated implementation
* ``Test_test.lean`` – translated concrete tests (``#eval`` results parsed;
  ``#[should_panic]`` tests are expected to return ``Result.fail``)
* ``Spec.lean``      – specification / theorem statements

Usage::

    python scripts/check_all.py            # check everything
    python scripts/check_all.py -w 8       # 8 concurrent workers
"""

import argparse
import re
import subprocess
import sys
import threading
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
LEAN_MODE = ROOT / "lean_mode"

# Matches info lines like:
#   info: .../Test.lean:564:0: Aeneas.Std.Result.ok ()
#   info: .../Test.lean:570:0: Aeneas.Std.Result.fail (...)
INFO_RE = re.compile(
    r"info:.*\.lean:(\d+):\d+:\s+Aeneas\.Std\.Result\.(ok|fail|error)",
    re.IGNORECASE,
)


# ── helpers ──────────────────────────────────────────────────────────────────


def _lean_module(dir_name: str, file_stem: str) -> str:
    """Turn a directory + file-stem into a Lake module name.

    ``LeanMode/array_binary_search/ArrayBinarySearch.lean``
        → ``LeanMode.array_binary_search.ArrayBinarySearch``
    """
    return f"LeanMode.{dir_name}.{file_stem}"


def _parse_should_panic(rust_test_path: Path) -> set[str]:
    """Extract function names annotated with ``#[should_panic]`` from Rust test source."""
    if not rust_test_path.exists():
        return set()
    text = rust_test_path.read_text()
    return set(
        re.findall(
            r"#\[should_panic[^\]]*\]\s*\n\s*fn\s+(\w+)",
            text,
        )
    )


def _parse_eval_map(lean_path: Path) -> dict[int, str]:
    """Map line numbers to function names for ``#eval`` statements in a .lean file."""
    if not lean_path.exists():
        return {}
    mapping: dict[int, str] = {}
    for i, line in enumerate(lean_path.read_text().splitlines(), 1):
        m = re.match(r"#eval\s+(\w+)", line.strip())
        if m:
            mapping[i] = m.group(1)
    return mapping


# ── main ─────────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Build every .lean file under LeanMode/"
    )
    parser.add_argument(
        "-w",
        "--workers",
        type=int,
        default=16,
        help="Number of concurrent workers (default: 16)",
    )
    args = parser.parse_args()

    WorkItem = tuple[
        str, str, Path | None, Path | None
    ]  # (label, module, lean_path, rust_test_path)

    work: list[WorkItem] = []

    dirs = sorted(
        d
        for d in (LEAN_MODE / "LeanMode").iterdir()
        if d.is_dir() and d.name != "failure"
    )

    for d in dirs:
        for lean_file in sorted(d.glob("*.lean")):
            module = _lean_module(d.name, lean_file.stem)
            label = f"{d.name}/{lean_file.name}"
            rust_test = d / "tests" / "test.rs"
            work.append(
                (label, module, lean_file, rust_test if rust_test.exists() else None)
            )

    passed: list[str] = []
    failed: list[tuple[str, list[str]]] = []  # (label, error_lines)
    lock = threading.Lock()
    done_event = threading.Event()
    remaining = len(work)

    def build_one(item: WorkItem) -> None:
        """Run ``lake build <module>`` and record the outcome."""
        nonlocal remaining
        label, module, lean_path, rust_test_path = item
        result = subprocess.run(
            ["lake", "build", module],
            capture_output=True,
            text=True,
            cwd=str(LEAN_MODE),
        )
        combined = result.stdout + result.stderr
        ok = result.returncode == 0
        lines: list[str] = []

        if ok and lean_path and rust_test_path:
            # Check #eval results against expectations.
            #   should_panic test → expect Result.fail / Result.error
            #   normal test      → expect Result.ok
            should_panic = _parse_should_panic(rust_test_path)
            eval_map = _parse_eval_map(lean_path)
            bad: list[str] = []
            for m in INFO_RE.finditer(combined):
                line_no = int(m.group(1))
                status = m.group(2).lower()
                fn_name = eval_map.get(line_no, f"<line {line_no}>")
                is_should_panic = fn_name in should_panic
                if is_should_panic:
                    if status == "ok":
                        bad.append(f"{fn_name}: expected panic but got ok")
                else:
                    if status in ("fail", "error"):
                        bad.append(f"{fn_name}: unexpected {status}")
            if bad:
                ok = False
                lines = bad
        elif not ok:
            lines = [l.strip() for l in combined.splitlines() if "error:" in l.lower()]
            if not lines:
                lines = [f"exit code {result.returncode}"]

        with lock:
            if ok:
                status = "OK"
                tail = ""
            elif result.returncode != 0:
                status = "FAIL"
                tail = f"  ← {lines[0]}" if lines else ""
            else:
                status = "FAIL"
                tail = f"  ← {len(lines)} test(s) failed"

            print(f"{label:<50} {status}{tail}", flush=True)
            if ok:
                passed.append(label)
            else:
                failed.append((label, lines))
            remaining -= 1
            if remaining == 0:
                done_event.set()

    print(
        f"Checking {len(work)} modules across {len(dirs)} projects "
        f"({args.workers} concurrent)...\n"
    )

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        for item in work:
            executor.submit(build_one, item)

    done_event.wait()

    print()
    print(f"Passed: {len(passed)}, Failed: {len(failed)}")

    if failed:
        print("\nFailures:")
        for label, lines in failed:
            print(f"  {label}")
            for line in lines[:8]:
                print(f"    {line}")
            if len(lines) > 8:
                print(f"    ... and {len(lines) - 8} more")
        sys.exit(1)


if __name__ == "__main__":
    main()
