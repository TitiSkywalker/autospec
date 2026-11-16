#!/usr/bin/env python3
"""End-to-end agent-trace analysis: build the caches, then run every report and
render every figure.

This is the one-command entry point for the ``analysis/agent_trace`` tooling.
It drives the full pipeline in dependency order, so a single invocation
reproduces the complete read-behaviour analysis for every agent:

1. **build**   — the two Lean-dependent scripts (``read_used_overlap`` and
                 ``indirect_reads``) run a ``lake build`` per (agent, case) and
                 cache the result. These are the slow steps; everything
                 downstream reads their caches.
2. **parse**   — the trace-parsing reports that need no Lean (``read_provenance``,
                 ``check_leak``, ``agent_trace``).
3. **plot**    — the figures: the read-vs-used sunburst/radial HTML charts
                 (``read_used_project``, ``read_used_radial``), the HMM
                 behaviour report + PNG heatmaps (``hmm``).

Everything is cached, so re-running skips the build when a case already has a
result (``--rebuild`` forces it). The build phase parallelises across cases with
``--jobs`` (each worker gets its own full copy of the proof-tree workspace).

Usage::

    python3 analysis/run_agent_trace.py                 # full pipeline (cached)
    python3 analysis/run_agent_trace.py --rebuild       # force-rebuild caches
    python3 analysis/run_agent_trace.py --jobs 4        # 4-way parallel build
    python3 analysis/run_agent_trace.py --only overlap indirect   # a subset
    python3 analysis/run_agent_trace.py --skip build    # no Lean, just reports+plots
    python3 analysis/run_agent_trace.py --list          # list the stages

Stages are a fixed sequence of named steps. ``--only`` / ``--skip`` select a
subset; steps still run in pipeline order, and the dependency (build before
plot) is preserved unless you ``--skip build``.

Outputs land in:

* ``analysis/agent_trace/local/``   — per-(agent, case) caches (gitignored).
* ``analysis/figs/``                — PNG/HTML figures.
* stdout                            — the printed reports.
"""

from __future__ import annotations

import argparse
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))  # make `analysis.*` importable regardless of cwd


# ── stage registry ────────────────────────────────────────────────────────────
# name -> (description, kind). Kinds:
#   build  — slow (runs `lake build` per case), needs the proof-tree workspace.
#   parse  — fast, only reads the archived traces.
#   plot   — fast, reads the build caches and writes figures under analysis/figs.
STAGES: dict[str, tuple[str, str]] = {
    "overlap":   ("read-vs-used file overlap (build + cache)", "build"),
    "indirect":  ("indirect file reads through tool calls (build + cache)", "build"),
    "provenance": ("per-agent read-volume/provenance report (parse only)", "parse"),
    "leak":      ("cross-run answer-leakage scan", "parse"),
    "trace":     ("raw per-case tool-call timeline (parse only)", "parse"),
    "project":   ("full-project sunburst HTML (read vs cited)", "plot"),
    "radial":    ("read-vs-cited radial sunburst HTML per agent×lib", "plot"),
    "casemap":   ("per-testcase read vs referenced file map HTML", "plot"),
    "hmm":       ("HMM behavioural comparison + occupancy PNG", "plot"),
    "heatmaps":  ("HMM emission/transition heatmap PNGs", "plot"),
}


def _run_report(name: str, argv: list[str], jobs: int, rebuild: bool) -> None:
    """Invoke one report's ``main()`` via its module, forwarding build flags.

    Each report parses ``sys.argv`` directly, so stash the orchestrator's argv
    and restore it around the call (and pass nothing down that the report's
    own parser would reject). Build-stage ``--jobs``/``--rebuild`` are handed
    through by setting ``sys.argv`` to just those flags.
    """
    saved = sys.argv
    sys.argv = [f"analysis/agent_trace/{name}.py", *argv]
    try:
        if name == "overlap":
            from analysis.agent_trace import read_used_overlap

            read_used_overlap.main()
        elif name == "indirect":
            from analysis.agent_trace import indirect_reads

            indirect_reads.main()
        elif name == "provenance":
            from analysis.agent_trace import read_provenance

            read_provenance.main()
        elif name == "leak":
            from analysis.agent_trace import check_leak

            check_leak.main()
        elif name == "trace":
            from analysis.agent_trace import agent_trace

            agent_trace.main()
        elif name == "project":
            from analysis.agent_trace import read_used_project

            read_used_project.main()
        elif name == "radial":
            from analysis.agent_trace import read_used_radial

            read_used_radial.main()
        elif name == "casemap":
            from analysis.agent_trace import read_used_case_map

            read_used_case_map.main()
        elif name == "hmm":
            from analysis.agent_trace.hmm import agent_behavior

            agent_behavior.main()
        elif name == "heatmaps":
            from analysis.agent_trace.hmm import hmm_heatmaps

            hmm_heatmaps.main()
        else:  # pragma: no cover — guarded by argparse choices
            raise ValueError(f"unknown stage: {name}")
    finally:
        sys.argv = saved


def _forward_args(name: str, jobs: int, rebuild: bool) -> list[str]:
    """The per-script argv prefix; ``--jobs``/``--rebuild`` only apply to builders."""
    if name in ("overlap", "indirect"):
        return ["--jobs", str(jobs)] + (["--rebuild"] if rebuild else [])
    return []


def main() -> None:
    ap = argparse.ArgumentParser(
        description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter
    )
    ap.add_argument("--only", nargs="*", choices=sorted(STAGES), metavar="STAGE",
                    help="run only these stages (default: all)")
    ap.add_argument("--skip", nargs="*", choices=sorted(STAGES), metavar="STAGE",
                    help="skip these stages")
    ap.add_argument("--jobs", type=int, default=1,
                    help="parallel workers for the build stages (each gets its own workspace copy)")
    ap.add_argument("--rebuild", action="store_true",
                    help="ignore per-case caches and rebuild (build stages only)")
    ap.add_argument("--list", action="store_true", help="list the stages and exit")
    args = ap.parse_args()

    if args.list:
        for name, (desc, kind) in STAGES.items():
            print(f"{name:<11} [{kind:<5}] {desc}")
        return

    selected = args.only or [n for n in STAGES if n not in (args.skip or [])]

    print(f"agent-trace pipeline: {', '.join(selected)}\n")
    for name in selected:
        desc, kind = STAGES[name]
        print(f"\n{'#' * 72}\n# {name} — {desc}\n{'#' * 72}", flush=True)
        _run_report(name, _forward_args(name, args.jobs, args.rebuild), args.jobs, args.rebuild)

    print(f"\n{'=' * 72}\nDone. Figures are under analysis/figs/; "
          f"caches under analysis/agent_trace/local/.")


if __name__ == "__main__":
    main()
