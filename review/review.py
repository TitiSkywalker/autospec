#!/usr/bin/env python3
"""Automated review of every testcase's ``Spec.lean`` by launching one agent per testcase.

For each testcase under ``lean_mode/LeanMode/`` (skipping ``failure/`` and any
directory without ``Spec.lean``), this spawns a coding agent — ``codex`` by
default, ``claude`` alternatively — that can compile and run code, asks it to
audit the specification (is it provable? missing conditions? counterexamples?),
and saves each agent's report to ``review/reports/<name>.md``.

Usage::

    python3 review/review.py                       # review everything (4 concurrent)
    python3 review/review.py --only matrix_qr      # one testcase
    python3 review/review.py --agent claude -w 8   # use claude code, 8 workers
    python3 review/review.py --resume              # skip testcases already reviewed
    python3 review/review.py --list                # just list testcases
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
CASES_DIR = LEAN_MODE / "LeanMode"
PROMPT_PATH = Path(__file__).resolve().parent / "REVIEW_PROMPT.md"

# Matches the "Verdict: PASS|FAIL|CONCERNS" line the agent is asked to emit.
VERDICT_RE = re.compile(r"Verdict\s*[:：]\s*(PASS|FAIL|CONCERNS|UNKNOWN)", re.IGNORECASE)


# ── discovery ────────────────────────────────────────────────────────────────


def discover_testcases() -> list[dict]:
    """Return one dict per reviewable testcase (has ``Spec.lean``, skips ``failure/``)."""
    cases: list[dict] = []
    for d in sorted(CASES_DIR.iterdir()):
        if not d.is_dir() or d.name == "failure":
            continue
        if not (d / "Spec.lean").exists():
            continue
        lean_files = sorted(p.name for p in d.glob("*.lean"))
        # The impl translation is the .lean file that is neither Spec nor Test_test.
        impl_files = [p for p in lean_files if p not in ("Spec.lean", "Test_test.lean")]
        crate_lean = impl_files[0] if len(impl_files) == 1 else None
        crate_module = f"LeanMode.{d.name}.{Path(crate_lean).stem}" if crate_lean else None
        cases.append(
            {
                "name": d.name,
                "dir": str(d),
                "lean_files": lean_files,
                "crate_lean": crate_lean,
                "crate_module": crate_module,
                "spec_module": f"LeanMode.{d.name}.Spec",
            }
        )
    return cases


def render_prompt(template: str, case: dict, report_path: Path) -> str:
    return (
        template.replace("{{NAME}}", case["name"])
        .replace("{{DIR}}", case["dir"])
        .replace("{{FILES}}", ", ".join(case["lean_files"]))
        .replace("{{CRATE_LEAN}}", case["crate_lean"] or "(none)")
        .replace("{{CRATE_MODULE}}", case["crate_module"] or "(none)")
        .replace("{{SPEC_MODULE}}", case["spec_module"])
        .replace("{{ROOT}}", str(ROOT))
        .replace("{{REPORT_PATH}}", str(report_path))
    )


# ── agent backends ───────────────────────────────────────────────────────────


def _run_codex(prompt: str, model: str | None, timeout: int, report_path: Path):
    """Run ``codex exec`` headless. The agent writes its report to ``report_path``
    itself; codex's ``-o`` last message is captured to a fallback file only."""
    last_msg = report_path.with_suffix(".msg")
    cmd = [
        "codex", "exec",
        "--ephemeral",
        "-s", "danger-full-access",
        "-C", str(ROOT),
        "-o", str(last_msg),
    ]
    if model:
        cmd += ["-m", model]
    proc = subprocess.run(
        cmd, input=prompt, text=True, capture_output=True,
        cwd=str(ROOT), timeout=timeout,
    )
    report = report_path.read_text() if report_path.exists() else ""
    if not report.strip() and last_msg.exists():
        report = last_msg.read_text()
    if not report.strip():
        report = proc.stdout
    if last_msg.exists():
        last_msg.unlink()
    return report, proc.stdout + proc.stderr, proc.returncode


def _run_claude(prompt: str, model: str | None, timeout: int, report_path: Path):
    """Run ``claude -p`` headless. The agent writes its report to ``report_path``
    itself; stdout is the fallback."""
    cmd = ["claude", "-p", "--output-format", "text", "--dangerously-skip-permissions"]
    if model:
        cmd += ["--model", model]
    proc = subprocess.run(
        cmd, input=prompt, text=True, capture_output=True,
        cwd=str(ROOT), timeout=timeout,
    )
    report = report_path.read_text() if report_path.exists() else ""
    if not report.strip():
        report = proc.stdout
    return report, proc.stdout + proc.stderr, proc.returncode


# ── main ─────────────────────────────────────────────────────────────────────


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Review each testcase's Spec.lean with a coding agent."
    )
    parser.add_argument(
        "--only",
        action="append",
        metavar="NAME",
        help="Review only the given testcase(s). Repeatable.",
    )
    parser.add_argument(
        "--agent",
        choices=("codex", "claude"),
        default="codex",
        help="Agent backend (default: codex)",
    )
    parser.add_argument(
        "-w", "--workers", type=int, default=4,
        help="Number of concurrent agents (default: 4)",
    )
    parser.add_argument(
        "--timeout", type=int, default=1200,
        help="Per-testcase agent timeout in seconds (default: 1200)",
    )
    parser.add_argument(
        "--model", metavar="MODEL",
        help="Model override for the agent (codex: -m, claude: --model)",
    )
    parser.add_argument(
        "--output", metavar="DIR", default=str(ROOT / "review" / "reports"),
        help="Report output directory (default: review/reports)",
    )
    parser.add_argument(
        "--resume", action="store_true",
        help="Skip testcases whose report already exists and is non-empty",
    )
    parser.add_argument(
        "--list", action="store_true",
        help="List reviewable testcases and exit",
    )
    args = parser.parse_args()

    cases = discover_testcases()

    if args.only:
        wanted = set(args.only)
        cases = [c for c in cases if c["name"] in wanted]
        missing = wanted - {c["name"] for c in cases}
        if missing:
            print(f"Warning: not found (or no Spec.lean): {', '.join(sorted(missing))}")

    if args.list:
        for c in cases:
            print(c["name"])
        print(f"\n{len(cases)} testcases")
        return

    if not cases:
        print("Nothing to review.")
        return

    template = PROMPT_PATH.read_text()
    out_dir = Path(args.output)
    out_dir.mkdir(parents=True, exist_ok=True)

    results: list[tuple[str, str, str, str]] = []  # (name, agent_status, verdict, tail)
    lock = threading.Lock()

    def review_one(case: dict) -> None:
        nonlocal results
        name = case["name"]
        report_path = out_dir / f"{name}.md"
        log_path = out_dir / f"{name}.log"

        if args.resume and report_path.exists() and report_path.stat().st_size > 0:
            with lock:
                results.append((name, "SKIPPED", "?", ""))
                print(f"{name:<32} SKIPPED", flush=True)
            return

        prompt = render_prompt(template, case, report_path)

        # Clear a stale report so a failed re-run isn't mistaken for a result.
        if report_path.exists():
            report_path.unlink()

        try:
            if args.agent == "codex":
                report, log, rc = _run_codex(prompt, args.model, args.timeout, report_path)
            else:
                report, log, rc = _run_claude(prompt, args.model, args.timeout, report_path)
        except subprocess.TimeoutExpired:
            status, verdict, tail = "TIMEOUT", "?", f"agent exceeded {args.timeout}s"
            report = f"# Review: {name}\n\nTimeout after {args.timeout}s.\n"
            log = tail
        except Exception as e:  # noqa: BLE001
            status, verdict, tail = "ERROR", "?", str(e)
            report = f"# Review: {name}\n\nError: {e}\n"
            log = tail
        else:
            status = "OK" if rc == 0 else "FAIL"
            m = VERDICT_RE.search(report)
            verdict = m.group(1).upper() if m else "?"
            tail = report.strip().splitlines()[0] if report.strip() else ""

        report_path.write_text(report.strip() + "\n" if report.strip() else "")
        log_path.write_text(log)

        with lock:
            results.append((name, status, verdict, tail))
            print(f"{name:<32} {status:<8} verdict={verdict:<8} {tail[:60]}", flush=True)

    print(
        f"Reviewing {len(cases)} testcases with {args.agent} "
        f"({args.workers} concurrent)...\n"
    )

    with ThreadPoolExecutor(max_workers=args.workers) as executor:
        for case in cases:
            executor.submit(review_one, case)

    # ── summary ───────────────────────────────────────────────────────────────
    ok = sum(1 for _, s, _, _ in results if s == "OK")
    fail = sum(1 for _, s, _, _ in results if s == "FAIL")
    timeout = sum(1 for _, s, _, _ in results if s == "TIMEOUT")
    error = sum(1 for _, s, _, _ in results if s == "ERROR")
    skipped = sum(1 for _, s, _, _ in results if s == "SKIPPED")

    print()
    print(
        f"OK: {ok}, FAIL: {fail}, TIMEOUT: {timeout}, "
        f"ERROR: {error}, SKIPPED: {skipped}"
    )

    # Index file for quick browsing.
    lines = [
        "# Review summary",
        "",
        f"- agent: `{args.agent}`",
        f"- testcases: {len(cases)}",
        "",
        "| testcase | agent | verdict | report |",
        "|---|---|---|---|",
    ]
    for name, status, verdict, _ in results:
        lines.append(f"| {name} | {status} | {verdict} | [{name}.md]({name}.md) |")
    (out_dir / "SUMMARY.md").write_text("\n".join(lines) + "\n")

    if fail or error:
        sys.exit(1)


if __name__ == "__main__":
    main()
