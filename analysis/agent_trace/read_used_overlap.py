"""Overlap between the files an agent *reads* and the files its proof *uses*.

For each (agent, case) this script computes two sets of files:

* **read** — every file the agent's trace shows it reading (Bash read-verbs
  plus the native ``Read`` tool), normalized and canonicalized the same way as
  ``agent_behavior.read_buckets`` does.
* **used** — every file a premise in the final proof's proof-tree is *defined*
  in (``Premise.def_path``), extracted with Treant. This is the "ground truth"
  of which files actually contributed to the proof.

The overlap answers the question in one number per agent: of the files the
agent read, how many actually mattered, and of the files the proof needs, how
many the agent read before needing them.  A low "read → used" recall (many
read files never cited) means the agent's reading is exploratory rather than
goal-directed; a low "used → read" recall means the agent proved things
without having read the library it cites (blind reliance on the LSP).

Canonicalization.  ``def_path`` comes out of Treant as an absolute path into
the *shared proof-tree workspace* (``analysis/local/proof_tree_ws``), while
the agent's trace records relative paths inside *its own* per-run workspace.
Both are folded onto a common "logical path" key so they can be intersected:

* ``aeneas:``  ``<...>/aeneas/backends/lean/<rel>``  → ``Aeneas/<rel>``
* ``program:`` ``<...>/RustVerif/<rel>``              → ``RustVerif/<rel>``
* ``mathlib:`` ``<...>/.lake/packages/mathlib/<rel>`` → ``Mathlib/<rel>``
* ``core:``    ``<...>/src/lean/<rel>``               → ``Init/<rel>`` (Lean4 core)

Plus a **read-side heuristic**: a bare ``Vec.lean`` / ``Std/Vec.lean`` /
``Aeneas/Std/Vec.lean`` the agent read (a relative path after ``cd`` into the
aeneas tree) is mapped to the same ``Aeneas/Std/Vec.lean`` key even without a
``/aeneas/backends/lean/`` prefix, so we don't under-count the agents that read
via short relative paths.

The proof tree is identical across agents for a case (they all prove the same
``Function.lean`` translation and their ``Spec.lean`` files are extensionally
the same set of lemmas when the verdict matches), but since agents differ in
*verdict*, each case's proof tree is built from each agent's own ``Spec.lean``
(what that agent actually proved — or failed to prove — uses).

Usage::

    python3 analysis/agent_trace/read_used_overlap.py                 # full report (cached)
    python3 analysis/agent_trace/read_used_overlap.py --rebuild        # ignore the cache
    python3 analysis/agent_trace/read_used_overlap.py --only crypto_rsa
    python3 analysis/agent_trace/read_used_overlap.py --limit 5        # quick sample
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm import RUNS, discover_cases, _iter_case_tool_calls  # noqa: E402
from analysis.agent_trace.agent_trace import (  # noqa: E402
    _AENEAS_BARE,
    _clean_path,
    _extract_paths,
    bucket_path,
    classify_bash,
)
from analysis.proof_tree.workspace import ensure_workspace, ensure_workspace_at, _extract_theorems, _lake_build  # noqa: E402

_SORRY_RE = re.compile(r"\b(sorry|admit)\b")


def _verdict(case_dir: Path) -> str:
    """PROVED/UNPROVED/BUILD_FAIL/UNKNOWN (verdict.txt, else sorry-heuristic)."""
    v = case_dir / "verdict.txt"
    if v.is_file():
        return v.read_text().strip() or "UNKNOWN"
    spec = case_dir / "Spec.lean"
    status = case_dir / "status.txt"
    if spec.is_file():
        if status.is_file() and status.read_text().strip() != "ok":
            return "BUILD_FAIL"
        return "PROVED" if not _SORRY_RE.search(spec.read_text()) else "UNPROVED"
    return "UNKNOWN"

# ======================= Configuration (edit here) =======================

CACHE_DIR = "analysis/agent_trace/local/read_used_overlap_data"  # per-(agent,case) cache (gitignored)

# Case-level statistics fall back to this when the proof tree cannot be built
# (e.g. UNPROVED specs with a `sorry` inside a lemma that treant can't parse).
# ``used`` is then unknown, so the case is dropped from overlap stats but kept
# in read-volume stats.
# ========================= (no need to edit below) =========================


# The citable libraries a proof can draw premises from. "other" (scaffolding,
# README, lakefile, ...) is read for orientation and is never a premise's
# def_path, so it is excluded from the overlap breakdown.
LIBS = ("aeneas", "mathlib", "core", "program")


def _library_of(key: str) -> str:
    """The library bucket of a canonical read/used key (its ``prefix:`` part)."""
    return key.split(":", 1)[0]


def _norm_abs(p: str) -> str:
    """Absolute path (as def_path arrives) → logical key."""
    s = p.replace("\\", "/")
    for marker, prefix in (
        ("/aeneas/backends/lean/", "aeneas:"),
        ("/.lake/packages/mathlib/", "mathlib:"),
        ("/src/lean/", "core:"),
    ):
        if marker in s:
            return prefix + s.split(marker, 1)[1].strip("/")
    if "/RustVerif/" in s:
        return "program:" + s.split("/RustVerif/", 1)[1].strip("/")
    return "other:" + Path(s).name


def _norm_read(p: str) -> str:
    """A path the agent read (relative, possibly partial) → logical key."""
    s = p.replace("\\", "/").strip("/")
    # full canonical prefixes the trace might already contain
    if "/aeneas/backends/lean/" in s:
        return "aeneas:" + s.split("/aeneas/backends/lean/", 1)[1].strip("/")
    if ".lake/packages/mathlib/" in s:
        return "mathlib:" + s.split(".lake/packages/mathlib/", 1)[1].strip("/")
    if "/src/lean/" in s:
        return "core:" + s.split("/src/lean/", 1)[1].strip("/")
    if "RustVerif/" in s:
        return "program:" + s.split("RustVerif/", 1)[1].strip("/")

    base = s.rsplit("/", 1)[-1]
    # aeneas library read via a relative path after `cd` into the aeneas tree:
    #   Vec.lean | Std/Vec.lean | Aeneas/Std/Vec.lean | .../Aeneas/Std/Vec.lean
    for prefix in ("Aeneas/Std/", "Std/", "Aeneas/Tactic/", "Tactic/", "Aeneas/"):
        if s.startswith(prefix):
            return "aeneas:" + s
    # bare module names that are aeneas library files (Vec.lean, WP.lean, ...)
    if base in _AENEAS_BARE:
        return "aeneas:" + "Aeneas/Std/" + base
    # Scratch/Tmp/etc. in RustVerif are program-local drafts
    if base.startswith(("Scratch", "Tmp", "Test")) or base.endswith(".lean"):
        return "program:" + base
    return "other:" + base


def read_files(case_dir: Path, kind: str) -> Counter:
    """Logical file key -> times read, for one (agent, case)."""
    c: Counter = Counter()
    for name, cmd, fp in _iter_case_tool_calls(case_dir, kind):
        if name.lower() == "bash":
            for seg, _v, lbl in classify_bash(cmd)[1]:
                if lbl == "read":
                    for p in _extract_paths(seg):
                        c[_norm_read(p)] += 1
        elif name in ("Read", "read") and fp:
            c[_norm_read(fp)] += 1
    return c


def used_files(ws: Path) -> set[str]:
    """Logical file keys for every premise's def_path in the current Spec.lean."""
    used: set[str] = set()
    for th in _extract_theorems(ws):
        stack = list(th.roots)
        while stack:
            g = stack.pop()
            t = g.tactic
            if t is None:
                continue
            for pr in t.tactic.premises:
                used.add(_norm_abs(pr.def_path))
            stack.extend(t.goals)
    return used


def _overlap(read: Counter, used: set[str]) -> dict:
    """Per-case totals: read/used/both plus the two whole-proof ratios."""
    read_keys = set(read)
    used_keys = set(used)
    inter = read_keys & used_keys
    orient = {"other:", "scaffold:"}
    readable = {k for k in read_keys if not k.startswith(tuple(orient))}
    return {
        "n_read": len(read_keys),
        "n_used": len(used_keys),
        "n_both": len(inter),
        "recall": len(inter) / len(readable) if readable else 0.0,   # read -> cited
        "precision": len(inter) / len(used_keys) if used_keys else 0.0,  # cited -> read
    }


def _per_lib(read: dict, used: list[str]) -> dict[str, dict[str, int]]:
    """Per-library read/used/both counts, derived from the cached read/used keys.

    Computed here (not stored) so the per-library breakdown works on caches
    produced before it existed and never goes stale.
    """
    read_keys = set(read)
    used_keys = set(used)
    inter = read_keys & used_keys
    return {
        lib: {
            "read": sum(1 for k in read_keys if _library_of(k) == lib),
            "used": sum(1 for k in used_keys if _library_of(k) == lib),
            "both": sum(1 for k in inter if _library_of(k) == lib),
        }
        for lib in LIBS
    }


def process_case(name: str, case_dir: Path, kind: str, ws: Path) -> dict:
    (ws / "RustVerif" / "Function.lean").write_text(
        (case_dir / "Function.lean").read_text()
    )
    (ws / "RustVerif" / "Spec.lean").write_text((case_dir / "Spec.lean").read_text())

    ok, _log = _lake_build(ws)
    if not ok:
        return {"name": name, "error": "build_fail"}

    try:
        used = used_files(ws)
    except (Exception, SystemExit) as exc:
        return {"name": name, "error": "extract_fail", "detail": repr(exc)}

    read = read_files(case_dir, kind)
    return {
        "name": name,
        "verdict": _verdict(case_dir),
        "read": dict(read),
        "used": sorted(used),
        **_overlap(read, used),
    }


def main() -> None:
    ap = argparse.ArgumentParser(description="Read-vs-used file overlap per agent")
    ap.add_argument("--rebuild", action="store_true", help="ignore the cache")
    ap.add_argument("--only", help="analyze a single case (verbose)")
    ap.add_argument("--limit", type=int, help="process at most N cases per agent")
    ap.add_argument(
        "--report-only",
        action="store_true",
        help="only print the report from existing cache (no builds)",
    )
    ap.add_argument("--jobs", type=int, default=1, help="parallel workers, each with its own workspace copy")
    args = ap.parse_args()

    cases_by_agent = {
        name: discover_cases(ROOT / path) for name, (path, _k) in RUNS.items()
    }
    shared = sorted(
        set.intersection(*(set(c) for c in cases_by_agent.values()))
        & {
            c
            for c in cases_by_agent["claude-opus-5.5"]
            if (cases_by_agent["claude-opus-5.5"][c] / "Function.lean").is_file()
        }
    )
    if args.only:
        shared = [c for c in shared if c == args.only]
        if not shared:
            sys.exit(f"no such shared case: {args.only}")
    if args.limit:
        shared = shared[: args.limit]

    cache_dir = ROOT / CACHE_DIR
    cache_dir.mkdir(parents=True, exist_ok=True)

    # Flatten to (name, case) tasks, skipping cached entries (unless --rebuild).
    tasks = []
    for name, (path, kind) in RUNS.items():
        for case in shared:
            if (cache_dir / f"{name}__{case}.json").is_file() and not args.rebuild:
                continue
            if args.report_only:
                continue
            tasks.append((name, case, cases_by_agent[name][case], kind))

    if not tasks:
        print("all cases cached; nothing to build", file=sys.stderr)

    elif args.jobs <= 1:
        ws = ensure_workspace()
        for name, case, case_dir, kind in tasks:
            print(f"[{name}/{case}] building + extracting...", file=sys.stderr, flush=True)
            r = process_case(case, case_dir, kind, ws)
            (cache_dir / f"{name}__{case}.json").write_text(json.dumps(r))

    else:
        from concurrent.futures import ThreadPoolExecutor
        import threading

        # One workspace per worker THREAD (not per task index). ``ThreadPoolExecutor``
        # schedules tasks onto any free thread, so keying by task index lets two
        # in-flight tasks collide on the same workspace and corrupt each other's
        # Spec/Function.lean. A thread-local holds each thread's own hardlink clone.
        _tls = threading.local()
        _tls_lock = threading.Lock()
        _next_ws = [0]

        def _worker_ws() -> Path:
            if getattr(_tls, "ws", None) is None:
                with _tls_lock:
                    i = _next_ws[0]
                    _next_ws[0] += 1
                # Namespace by process so two scripts (overlap/indirect) running
                # concurrently don't share a workspace either.
                _tls.ws = ensure_workspace_at(
                    ROOT / "analysis" / "local" / f"proof_tree_ws_p{i}_{os.getpid()}"
                )
            return _tls.ws

        lock = threading.Lock()

        def _run(idx_task):
            _idx, (name, case, case_dir, kind) = idx_task
            ws = _worker_ws()
            print(f"[{name}/{case}] building + extracting...", file=sys.stderr, flush=True)
            r = process_case(case, case_dir, kind, ws)
            with lock:
                (cache_dir / f"{name}__{case}.json").write_text(json.dumps(r))
            return r

        with ThreadPoolExecutor(max_workers=args.jobs) as ex:
            list(ex.map(_run, enumerate(tasks)))

    # Load the full cache for the report.
    results: dict[str, dict[str, dict]] = defaultdict(dict)
    for name, (path, kind) in RUNS.items():
        for case in shared:
            cf = cache_dir / f"{name}__{case}.json"
            if cf.is_file():
                results[name][case] = json.loads(cf.read_text())

    _report(results, shared)


def _report(results: dict[str, dict[str, dict]], shared: list[str]) -> None:
    print(f"\n=== Read-vs-used file overlap ({len(shared)} shared cases) ===")

    # per-agent aggregate over cases where the proof tree built, split by verdict
    print(f"\n{'agent':<14} {'verdict':>9} {'cases':>5} {'read_files':>11} {'used_files':>11} "
          f"{'read∩used':>10} {'read→used':>10} {'used→read':>10}")
    print("-" * 88)
    for name in RUNS:
        recs = [r for r in results[name].values() if "error" not in r]
        by_v: dict[str, list] = defaultdict(list)
        for r in recs:
            by_v[r.get("verdict", "?")].append(r)
        for v in ("PROVED", "UNPROVED", "BUILD_FAIL"):
            if v not in by_v:
                continue
            vr = by_v[v]
            n_read = sum(r["n_read"] for r in vr)
            n_used = sum(r["n_used"] for r in vr)
            n_both = sum(r["n_both"] for r in vr)
            readable = sum(
                len({k for k in r["read"] if not k.startswith(("other:", "scaffold:"))})
                for r in vr
            )
            recall = n_both / readable if readable else 0.0
            precision = n_both / n_used if n_used else 0.0
            print(f"{name:<14} {v:>9} {len(vr):>5} {n_read:>11} {n_used:>11} "
                  f"{n_both:>10} {recall*100:>9.1f}% {precision*100:>9.1f}%")
    print("-" * 88)
    print("read→used: of the files read (excl. orientation), the fraction later cited by the proof.")
    print("used→read: of the files the proof cites, the fraction the agent had read.")

    # per-library read/used/both, so mathlib and aeneas can be read separately
    print("\n--- per-library read / used / both (distinct files, summed over cases) ---")
    for lib in LIBS:
        print(f"\n  {lib}:")
        print(f"    {'agent':<14} {'read':>6} {'used':>6} {'both':>6} {'read→used':>10} {'used→read':>10}")
        for name in RUNS:
            recs = [r for r in results[name].values() if "error" not in r]
            pl = [_per_lib(r["read"], r["used"]) for r in recs]
            rd = sum(p[lib]["read"] for p in pl)
            us = sum(p[lib]["used"] for p in pl)
            bo = sum(p[lib]["both"] for p in pl)
            print(f"    {name:<14} {rd:>6} {us:>6} {bo:>6} "
                  f"{100*bo/max(rd,1):>9.1f}% {100*bo/max(us,1):>9.1f}%")

    # per-agent: which cited files were NOT read (the LSP blind-spot)
    print("\n--- cited-but-not-read files (proof used a library the agent never read) ---")
    for name in RUNS:
        recs = [r for r in results[name].values() if "error" not in r]
        miss: Counter = Counter()
        for r in recs:
            used = set(r["used"])
            read = set(r["read"])
            for u in used - read:
                miss[u] += 1
        print(f"\n{name} (top 15 of {len(miss)} distinct):")
        for k, n in miss.most_common(15):
            print(f"    {n:>3}  {k}")


if __name__ == "__main__":
    main()
