"""Read-file behaviour differences across proving agents (fast, reproducible).

This is the *fast* counterpart to ``read_used_overlap.py``: it does no Lean
build, only parses the archived traces, so it reruns in seconds over any number
of cases. It reproduces the read-file comparison report: for each agent, how
much it reads (vs. writes / LSP queries), how widely it reads, and *what* it
reads (by file provenance).

Every metric is computed over the set of cases shared by all agents in
``RUNS`` (imported from ``agent_behavior`` so the scripts stay in sync), using
the same tool-call classification (``classify_bash``), the same read-event
counting and provenance bucketing (``read_buckets``) as ``agent_trace`` /
``agent_behavior``, and the same verdict logic as ``accuracy``. So this report
is exactly the "read-file" slice of the four scripts it reuses.

Output:

* **accuracy**     — PROVED / total per agent on the shared cases.
* **read volume**  — read events, read share of tool calls, distinct files
                     read, read concentration (events-per-distinct-file and the
                     single most-read file's share).
* **provenance**   — where the read events point (aeneas library / problem /
                     scaffolding / mathlib / ...), plus the top files per agent.

Usage::

    python3 analysis/agent_trace/read_provenance.py            # full report over shared cases
    python3 analysis/agent_trace/read_provenance.py --top 20   # more top-files per agent
"""

from __future__ import annotations

import argparse
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm import RUNS, discover_cases, _category_of, _iter_case_tool_calls  # noqa: E402
from analysis.agent_trace.agent_trace import _clean_path, _extract_paths, bucket_path, classify_bash  # noqa: E402
from analysis.accuracy.verdicts import read_verdict as _verdict  # noqa: E402


def _file_reads(case_dir: Path, kind: str) -> Counter:
    """Normalized path -> times read, for one (agent, case).

    Same counting as ``agent_behavior.read_buckets``: every Bash read-verb
    segment contributes each path it mentions, and every native ``Read`` tool
    contributes its ``file_path`` — but here the key is the cleaned path (not
    the provenance bucket), so we can list the actual top files.
    """
    c: Counter = Counter()
    for name, cmd, fp in _iter_case_tool_calls(case_dir, kind):
        if name.lower() == "bash":
            for seg, _v, lbl in classify_bash(cmd)[1]:
                if lbl == "read":
                    for p in _extract_paths(seg):
                        c[_clean_path(p)] += 1
        elif name in ("Read", "read") and fp:
            c[_clean_path(fp)] += 1
    return c


def main() -> None:
    ap = argparse.ArgumentParser(description="Read-file behaviour comparison across agents")
    ap.add_argument("--top", type=int, default=15, help="top files per agent to list")
    args = ap.parse_args()

    cases_by_agent = {name: discover_cases(ROOT / path) for name, (path, _k) in RUNS.items()}
    shared = sorted(set.intersection(*(set(c) for c in cases_by_agent.values())))

    print(f"shared cases: {len(shared)}\n")

    # --- accuracy ---
    print(f"{'agent':<14} {'PROVED':>7} {'UNPROVED':>9} {'BUILD_FAIL':>10} {'total':>6} {'rate':>7}")
    for name, (path, _k) in RUNS.items():
        counts = Counter(_verdict(cases_by_agent[name][c]) for c in shared)
        p, t = counts["PROVED"], len(shared)
        print(f"{name:<14} {p:>7} {counts['UNPROVED']:>9} {counts['BUILD_FAIL']:>10} {t:>6} {p/t:>7.1%}")
    print("  (verdict.txt where present; else sorry-heuristic on Spec.lean)")

    # --- read volume & breadth ---
    print(f"\n{'agent':<14} {'tool_calls':>11} {'read_events':>11} {'read/call':>9} "
          f"{'distinct':>9} {'ev/distinct':>12} {'top1_share':>12}")
    files_by_agent: dict[str, Counter] = {}
    for name, (path, kind) in RUNS.items():
        files: Counter = Counter()
        cats: Counter = Counter()
        total = 0
        for c in shared:
            for n, cmd, fp in _iter_case_tool_calls(cases_by_agent[name][c], kind):
                total += 1
                cats[_category_of(n, cmd)] += 1
            files.update(_file_reads(cases_by_agent[name][c], kind))
        files_by_agent[name] = files
        reads = sum(files.values())
        distinct = len(files)
        top1 = files.most_common(1)[0]
        print(f"{name:<14} {total:>11} {reads:>11} {100*reads/max(total,1):>8.1f}% "
              f"{distinct:>9} {reads/max(distinct,1):>12.1f}  {top1[0][-24:]:>12} {100*top1[1]/max(reads,1):.0f}%")

    # --- provenance (reuse the exact bucketing from agent_behavior) ---
    print(f"\n{'bucket':<26}" + "".join(f"{a:>16}" for a in RUNS))
    provs: dict[str, Counter] = {}
    for name, (path, kind) in RUNS.items():
        buckets: Counter = Counter()
        for c in shared:
            buckets.update(_bucket_reads(cases_by_agent[name][c], kind))
        provs[name] = buckets
    all_buckets = {b for pc in provs.values() for b in pc}
    totals = {b: sum(pc.get(b, 0) for pc in provs.values()) for b in all_buckets}
    for b in sorted(all_buckets, key=lambda b: -totals[b]):
        print(f"{b:<26}" + "".join(f"{100*provs[a].get(b,0)/max(sum(provs[a].values()),1):>15.1f}%" for a in RUNS))

    # --- action mix (the full picture: read vs write vs LSP) ---
    print(f"\n{'agent':<14}" + "".join(f"{c:>9}" for c in ("read", "write", "build", "mcp", "other")))
    for name, (path, kind) in RUNS.items():
        cats: Counter = Counter()
        for c in shared:
            for n, cmd, fp in _iter_case_tool_calls(cases_by_agent[name][c], kind):
                cats[_category_of(n, cmd)] += 1
        total = sum(cats.values())
        print(f"{name:<14}" + "".join(f"{100*cats[k]/total:>8.1f}%" for k in ("read", "write", "build", "mcp", "other")))

    # --- top files per agent ---
    for name in RUNS:
        print(f"\n--- {name}: top-{args.top} files read ---")
        for k, n in files_by_agent[name].most_common(args.top):
            print(f"  {n:>5}  {k}")


def _bucket_reads(case_dir: Path, kind: str) -> Counter:
    """Provenance buckets for every file the agent read in a case.

    Exactly ``agent_behavior.read_buckets`` (which this re-derives rather than
    imports, so the file-path counting stays in one obvious place).
    """
    buckets: Counter = Counter()
    for name, cmd, fp in _iter_case_tool_calls(case_dir, kind):
        if name.lower() == "bash":
            for seg, _v, lbl in classify_bash(cmd)[1]:
                if lbl == "read":
                    for p in _extract_paths(seg):
                        buckets[bucket_path(p)] += 1
        elif name in ("Read", "read") and fp:
            buckets[bucket_path(fp)] += 1
    return buckets


if __name__ == "__main__":
    main()
