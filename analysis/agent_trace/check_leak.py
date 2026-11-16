"""Detect cross-run answer leakage in the proving agents' traces.

A trace is *leaked* when the agent reads something outside its own isolated
workspace that could reveal a prior answer. Three answer-relevant signals are
checked, plus one informational one:

* **proof read**   — ``*_results/<case>/(Spec|Function).lean`` (another run's
                     actual proof content).
* **verdict read** — ``verdict.txt`` / ``verify.log`` / ``status.txt`` /
                     ``verification_meta.json`` (which cases were PROVED).
* **benchmark read** — ``lean_mode/LeanMode/<case>/`` (another benchmark case).
* (informational)  — the evaluation harness Python/config, and other agents'
                     session stores, are counted separately but do not, by
                     themselves, mark a case as leaked.

For every leaked ``(agent, case)`` the script copies the offending trace into
``OUTPUT_DIR/<agent>/<case>/`` together with a ``leak.txt`` holding the matched
evidence, and writes ``leak_manifest.json`` + ``summary.md``.

Usage::

    python3 analysis/agent_trace/check_leak.py                 # scan every agent in RUNS
    python3 analysis/agent_trace/check_leak.py --out local/agent_leak
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm import RUNS, discover_cases, _iter_case_tool_calls  # noqa: E402

OUTPUT_DIR = ROOT / "local" / "agent_leak"

_PROOF_RE = re.compile(r"[\w.-]+_results/([\w-]+)/(Spec|Function)\.lean")
_VERDICT_RE = re.compile(r"[\w.-]+_results/([\w-]+)/(verdict\.txt|verify\.log|status\.txt)")
_META_RE = re.compile(r"[\w.-]+_results/verification_meta\.json")
_BENCH_RE = re.compile(r"lean_mode/LeanMode/([\w-]+)/")
_HARNESS_RE = re.compile(r"/evaluation/[^\s'\"`;&|)]*")
_SESSION_RE = re.compile(r"\.claude/projects|\.dsh|\.vibe/sessions|\.codex/sessions")


def scan_agent(rel: str, kind: str) -> dict:
    """Return {case: leak-record} for every leaked case of one agent."""
    cases = discover_cases(ROOT / rel)
    out = {}
    for case in sorted(cases):
        proof = Counter()
        verdict = 0
        benchmark = Counter()
        harness = 0
        session = 0
        examples: list[str] = []
        for name, cmd, fp in _iter_case_tool_calls(cases[case], kind):
            text = (cmd or "") + "\n" + (fp or "")
            for m in _PROOF_RE.finditer(text):
                proof[m.group(1)] += 1
                _note(examples, text)
            for m in _VERDICT_RE.finditer(text):
                verdict += 1
                _note(examples, text)
            if _META_RE.search(text):
                verdict += 1
                _note(examples, text)
            for m in _BENCH_RE.finditer(text):
                benchmark[m.group(1)] += 1
                _note(examples, text)
            if _HARNESS_RE.search(text) and "ws-" not in text:
                harness += 1
            if _SESSION_RE.search(text):
                session += 1
        if proof or verdict or benchmark:
            out[case] = {
                "proof_reads": dict(proof),
                "proof_same_case": sum(n for c, n in proof.items() if c == case),
                "verdict_reads": verdict,
                "benchmark_reads": dict(benchmark),
                "harness_refs": harness,
                "session_refs": session,
                "examples": examples[:6],
            }
    return out


def _note(examples: list[str], text: str) -> None:
    if len(examples) < 6:
        s = " ".join(text.split())[:180]
        if s not in examples:
            examples.append(s)


def _copy_trace(rel: str, case: str, dst: Path) -> None:
    dst.mkdir(parents=True, exist_ok=True)
    trace = ROOT / rel / case / "trace"
    if not trace.is_dir():
        return
    for f in trace.iterdir():
        if f.is_file() and f.suffix != ".lock":
            shutil.copy2(f, dst / f.name)


def main() -> None:
    ap = argparse.ArgumentParser(description="Detect cross-run answer leakage in agent traces")
    ap.add_argument("--out", default=str(OUTPUT_DIR), help="output dir for leaked traces + manifest")
    args = ap.parse_args()

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)

    manifest: dict = {}
    summary_lines = ["# Agent-trace leakage report\n"]
    for agent, (rel, kind) in RUNS.items():
        leaks = scan_agent(rel, kind)
        manifest[agent] = leaks
        n_proof = sum(1 for r in leaks.values() if r["proof_reads"])
        n_verdict = sum(1 for r in leaks.values() if r["verdict_reads"])
        n_bench = sum(1 for r in leaks.values() if r["benchmark_reads"])
        summary_lines.append(
            f"\n## {agent} — {len(leaks)} leaked cases "
            f"(proof {n_proof}, verdict {n_verdict}, benchmark {n_bench})\n"
        )
        for case, rec in sorted(leaks.items()):
            dst = out / agent / case
            _copy_trace(rel, case, dst)
            (dst / "leak.txt").write_text(json.dumps(rec, indent=2) + "\n")
            kinds = []
            if rec["proof_reads"]:
                kinds.append(f"proof[{sum(rec['proof_reads'].values())}x, same-case {rec['proof_same_case']}x]")
            if rec["verdict_reads"]:
                kinds.append(f"verdict[{rec['verdict_reads']}x]")
            if rec["benchmark_reads"]:
                kinds.append(f"benchmark[{sum(rec['benchmark_reads'].values())}x]")
            summary_lines.append(f"- `{case}` — " + ", ".join(kinds))

    (out / "leak_manifest.json").write_text(json.dumps(manifest, indent=2) + "\n")
    (out / "summary.md").write_text("\n".join(summary_lines) + "\n")

    print(f"leaked cases per agent:")
    for agent, leaks in manifest.items():
        print(f"  {agent:<14} {len(leaks):>3} leaked cases")
    print(f"\nartifacts -> {out}/")
    print(f"  leak_manifest.json   machine-readable manifest")
    print(f"  summary.md           human-readable summary")
    print(f"  <agent>/<case>/      copied trace + leak.txt evidence")


if __name__ == "__main__":
    main()
