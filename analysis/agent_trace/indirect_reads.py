"""Indirect file reads: what the agent *sees* through tool calls, beyond `cat`.

``read_used_overlap.py`` counts only *direct* reads (Bash read-verbs + the
``Read`` tool) and compares them to the files the proof actually uses. This
script widens the "read" side to match the rule: **a file counts as read if the
agent can see it in the text flowing through any tool call** — the agent's own
tool inputs (commands, code, ``#check`` queries) *and* the tool results
(stdout/stderr, LSP diagnostics). Concretely it adds two indirect channels:

* **path-seen** — a ``.lean``/``.rs``/``.py``/... path that literally appears
  in a tool call's text. This is how ``lake build`` / ``lean`` failures are
  counted: the compiler error ``RustVerif/Spec.lean:31:13: unsolved goals``
  puts ``Spec.lean`` (and any library file named in the error) in front of the
  agent without a ``cat``.
* **name-seen** — a declaration/module the agent *named* (``#check`` /
  ``#print`` / ``#find`` / ``#eval`` / ``#reduce`` queries, ``import``
  statements, and explicit lemma names like ``rw [Nat.gcd_comm]``), resolved
  back to the file that defines it via Treant's premise ``def_path``.

On the *used* side it also removes the automation: a premise is "explicit" only
if the agent wrote its name in the owning tactic (so ``rw [Nat.gcd_comm]``
counts, but the premises pulled in internally by ``omega``/``ring``/``simp``/
``decide``/``linarith``/Aeneas ``step`` do not). This is the "training memory"
channel, separated from "tactic automation" — the two things that were conflated
in the original "used → read" gap.

The report answers: of the files the *explicit* proof cites, how many did the
agent see directly vs. only indirectly (through a tool call's text) vs. not at
all.

Usage::

    python3 analysis/agent_trace/indirect_reads.py                 # full report (cached)
    python3 analysis/agent_trace/indirect_reads.py --rebuild        # ignore the cache
    python3 analysis/agent_trace/indirect_reads.py --only crypto_rsa
    python3 analysis/agent_trace/indirect_reads.py --report-only    # print from cache, no builds
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

from analysis.agent_trace.hmm import RUNS, discover_cases, _read_zstd  # noqa: E402
from analysis.agent_trace.agent_trace import _extract_paths  # noqa: E402
from analysis.proof_tree.workspace import ensure_workspace, ensure_workspace_at, _extract_theorems, _lake_build  # noqa: E402
from analysis.agent_trace.read_used_overlap import _norm_abs, _norm_read, _library_of, read_files, LIBS  # noqa: E402

# ======================= Configuration (edit here) =======================

CACHE_DIR = "analysis/agent_trace/local/indirect_reads_data"  # per-(agent,case) cache (gitignored)

# Query forms that name a declaration the agent is inspecting via the LSP.
_QUERY_RE = re.compile(r"^\s*(?:#check|#print|#find|#eval|#reduce)\s+@?([\w.]+)", re.MULTILINE)
_IMPORT_RE = re.compile(r"^\s*import\s+([\w.]+)", re.MULTILINE)

# ========================= (no need to edit below) =========================


def _name_in_target(full_name: str, target: str) -> bool:
    """Did the agent write this premise's name in the tactic text?

    Matches the full dotted name as a whole identifier (so ``Nat`` does NOT
    match ``Nat.add_comm``), then the short name (tail after the last ``.``),
    allowing a leading ``.`` (so ``add_comm`` matches ``Nat.add_comm``).
    """
    short = full_name.rsplit(".", 1)[-1]
    for cand in (full_name, short):
        left = r"(?<![A-Za-z0-9_])" if cand is short else r"(?<![A-Za-z0-9_.])"
        if re.search(left + re.escape(cand) + r"(?![A-Za-z0-9_.])", target):
            return True
    return False


def _explicit_and_all_used(ws: Path) -> tuple[set[str], set[str], dict, dict, Counter]:
    """Walk the proof tree once: (explicit keys, all keys, name2path, mod2path, explicit ref counts)."""
    explicit: set[str] = set()
    explicit_counts: Counter = Counter()
    all_keys: set[str] = set()
    name2path: dict[str, str] = {}
    mod2path: dict[str, str] = {}
    for th in _extract_theorems(ws):
        stack = list(th.roots)
        while stack:
            g = stack.pop()
            t = g.tactic
            if t is None:
                continue
            target = t.tactic.target or ""
            for pr in t.tactic.premises:
                key = _norm_abs(pr.def_path)
                all_keys.add(key)
                name2path.setdefault(pr.full_name, pr.def_path)
                mod2path.setdefault(pr.mod_name, pr.def_path)
                if _name_in_target(pr.full_name, target):
                    explicit.add(key)
                    explicit_counts[key] += 1
            stack.extend(t.goals)
    return explicit, all_keys, name2path, mod2path, explicit_counts


def _serialize_input(name: str, inp: dict) -> str:
    """The text of a tool call's input, in a form the agent sees."""
    if name in ("Bash", "bash"):
        return inp.get("command", "") or ""
    if name.startswith("mcp__") or "lean" in name.lower():
        return inp.get("code", "") or ""
    if name in ("Read", "read"):
        return inp.get("file_path", "") or ""
    if name in ("Edit", "Write", "edit", "write"):
        return (inp.get("file_path", "") or "") + "\n" + (inp.get("content", "") or "")
    return json.dumps(inp)


def _iter_surface_claude(trace: Path):
    """Yield (tool_name, input_text, result_text) for each tool call."""
    uses: dict[str, tuple[str, str]] = {}
    results: dict[str, str] = {}
    with trace.open(errors="replace") as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if e.get("type") == "assistant":
                for b in (e.get("message") or {}).get("content") or []:
                    if isinstance(b, dict) and b.get("type") == "tool_use":
                        name = b.get("name", "")
                        uses[b.get("id", "")] = (name, _serialize_input(name, b.get("input") or {}))
            elif e.get("type") == "user":
                for b in (e.get("message") or {}).get("content") or []:
                    if isinstance(b, dict) and b.get("type") == "tool_result":
                        c = b.get("content")
                        if isinstance(c, str):
                            txt = c
                        else:
                            txt = "".join(
                                (x.get("text", "") if isinstance(x, dict) else str(x))
                                for x in (c or [])
                            )
                        results[b.get("tool_use_id", "")] = txt
    for tid, (name, itxt) in uses.items():
        yield name, itxt, results.get(tid, "")


def _iter_surface_dsh(path: Path):
    """Yield (tool_name, input_text, result_text) for each dsh tool call."""
    calls: dict[str, tuple[str, str]] = {}
    results: dict[str, str] = {}
    for line in _read_zstd(path).splitlines():
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("type") == "tool/call":
            data = e.get("data") or {}
            name = data.get("name", "")
            try:
                args = json.loads(data.get("arguments") or "{}")
            except json.JSONDecodeError:
                args = {}
            calls[data.get("callId", "")] = (name, _serialize_input(name, args))
        elif e.get("type") == "tool/result":
            data = e.get("data") or {}
            msg = data.get("message") or {}
            tid = msg.get("toolCallId") or msg.get("callId") or ""
            content = msg.get("content") or []
            txt = "".join(
                (x.get("text", "") if isinstance(x, dict) else str(x)) for x in content
            )
            results[tid] = txt
    for cid, (name, itxt) in calls.items():
        yield name, itxt, results.get(cid, "")


def _iter_surface(case_dir: Path, kind: str):
    if kind == "claude":
        for f in (case_dir / "trace").glob("*.jsonl"):
            yield from _iter_surface_claude(f)
    elif kind == "dsh":
        yield from _iter_surface_dsh(case_dir / "trace" / "session.v4.jsonl.zstd")
    else:  # codex: rollout JSONL, tools nested under the ``exec`` orchestrator
        from analysis.agent_trace.hmm.agent_behavior import _iter_codex_tool_calls

        for f in (case_dir / "trace").glob("*.jsonl"):
            # The codex iterator yields (tool_name, command, file_path); the
            # command is the text the agent typed (its "surface" input), and
            # there is no separately-captured result text, so pass "".
            for name, cmd, fp in _iter_codex_tool_calls(f):
                yield name, cmd or fp or "", ""


def _resolve_name(q: str, name2path: dict, mod2path: dict) -> str | None:
    """Map a queried declaration/module name to a logical file key, else None."""
    if q in name2path:
        return _norm_abs(name2path[q])
    # program lemma queried without its module prefix (gcd_loop.eq_1 vs crypto_rsa.gcd_loop.eq_1)
    suffix = "." + q
    cands = [n for n in name2path if n.endswith(suffix)]
    if len(cands) == 1:
        return _norm_abs(name2path[cands[0]])
    # namespace tail of the query may equal the tail of a mod_name (Nat.ModEq -> Mathlib.Data.Nat.ModEq)
    parts = q.split(".")
    for i in range(len(parts), 0, -1):
        ns = ".".join(parts[:i])
        hits = [m for m in mod2path if m == ns or m.endswith("." + ns)]
        if len(hits) == 1:
            return _norm_abs(mod2path[hits[0]])
    return None


def _surface(case_dir: Path, kind: str) -> dict:
    """Path-seen and name-seen files from the trace's tool-call text (no build)."""
    seen_paths: set[str] = set()
    queries: list[str] = []
    imports: list[str] = []
    for name, itxt, rtxt in _iter_surface(case_dir, kind):
        surface = (itxt or "") + "\n" + (rtxt or "")
        for p in _extract_paths(surface):
            seen_paths.add(_norm_read(p))
        queries.extend(_QUERY_RE.findall(itxt))
        imports.extend(_IMPORT_RE.findall(itxt))
    return {"seen_paths": sorted(seen_paths), "queries": queries, "imports": imports}


def process_case(name: str, case_dir: Path, kind: str, ws: Path) -> dict:
    surf = _surface(case_dir, kind)  # needs no build, so always available

    (ws / "RustVerif" / "Function.lean").write_text((case_dir / "Function.lean").read_text())
    (ws / "RustVerif" / "Spec.lean").write_text((case_dir / "Spec.lean").read_text())
    ok, _log = _lake_build(ws)
    if not ok:
        return {"name": name, "error": "build_fail", **surf}

    try:
        explicit, all_used, name2path, mod2path, explicit_counts = _explicit_and_all_used(ws)
    except (Exception, SystemExit) as exc:
        return {"name": name, "error": "extract_fail", "detail": repr(exc), **surf}

    # resolve the queried/imported names to files
    name_seen: set[str] = set()
    for q in dict.fromkeys(surf["queries"]):
        r = _resolve_name(q, name2path, mod2path)
        if r:
            name_seen.add(r)
    for m in dict.fromkeys(surf["imports"]):
        r = _resolve_name(m, name2path, mod2path)
        if r:
            name_seen.add(r)

    read = dict(read_files(case_dir, kind))
    return {
        "name": name,
        "verdict": _verdict(case_dir),
        "read": read,
        "used_all": sorted(all_used),
        "used_explicit": sorted(explicit),
        "used_counts": dict(explicit_counts),
        "seen_paths": surf["seen_paths"],
        "name_seen": sorted(name_seen),
        "queries": surf["queries"],
    }


def _verdict(case_dir: Path) -> str:
    from analysis.accuracy.verdicts import read_verdict as _v

    return _v(case_dir)


def main() -> None:
    ap = argparse.ArgumentParser(description="Indirect file reads through tool calls")
    ap.add_argument("--rebuild", action="store_true", help="ignore the cache")
    ap.add_argument("--only", help="analyze a single case (verbose)")
    ap.add_argument("--limit", type=int, help="process at most N cases per agent")
    ap.add_argument("--report-only", action="store_true", help="print from cache, no builds")
    ap.add_argument("--jobs", type=int, default=1, help="parallel workers, each with its own workspace copy")
    args = ap.parse_args()

    cache_dir = ROOT / CACHE_DIR
    cache_dir.mkdir(parents=True, exist_ok=True)

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

    # Flatten to (name, case) tasks, skipping anything already cached (unless
    # --rebuild), so a resumed run only builds what's missing.
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

        # One workspace per worker THREAD (not per task index), via thread-local.
        _tls = threading.local()
        _tls_lock = threading.Lock()
        _next_ws = [0]

        def _worker_ws() -> Path:
            if getattr(_tls, "ws", None) is None:
                with _tls_lock:
                    i = _next_ws[0]
                    _next_ws[0] += 1
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

    _report(results)


def _sets_for(recs: list[dict]) -> tuple[set[str], set[str], set[str]]:
    """(direct, indirect, used) as *distinct* file sets over the built cases.

    ``direct`` = files the agent opened with cat/grep/Read; ``indirect`` = files
    it only saw in tool-call text (#check queries, lake errors, ls/find output);
    the two are disjoint (a file that was both directly read and indirectly seen
    counts as direct). ``used`` = files the final proof explicitly cites.
    """
    direct: set[str] = set()
    indirect: set[str] = set()
    used: set[str] = set()
    for r in recs:
        used |= set(r["used_explicit"])
        rd = set(r["read"])
        seen = rd | set(r["seen_paths"]) | set(r["name_seen"])
        direct |= rd
        indirect |= (seen - rd)
    return direct, indirect, used


def _pct(num: int, den: int) -> str:
    return f"{100.0 * num / den:.1f}%" if den else "  –"


def _report(results: dict[str, dict[str, dict]]) -> None:
    ok = {n: {c: r for c, r in d.items() if "error" not in r} for n, d in results.items()}
    n_cases = {n: len(d) for n, d in ok.items()}
    print(f"\n=== Indirect file reads (built-proof cases: {n_cases}) ===\n")

    print("口径: 直接读(D) = 用 cat/grep/Read 主动打开; 间接读(I) = 只出现在工具文本里")
    print("(#check 查询 / lake 报错 / ls、find 输出), 二者互斥(既直接又间接记作直接读)。")
    print("引用(U) = 证明确实引用的文件(已剔除自动化, 只算显式命名)。")

    # ============ 双向总表 ============
    print("\n=== 双向统计 (distinct 文件, union over built cases) ===\n")
    hdr = (f"{'模型':<14} | {'直接读D':>7} | {'D被引用':>9} | {'间接读I':>7} | "
           f"{'I被引用':>9} | {'引用U':>5} | {'U直接读':>9} | {'U间接读':>9} | {'U没读过':>9}")
    print(hdr)
    print("-" * len(hdr))
    for name in RUNS:
        D, I, U = _sets_for(list(ok[name].values()))
        d_used = len(D & U)
        i_used = len(I & U)
        u_direct = len(U & D)
        u_indirect = len(U & I)   # I 与 D 互斥, 故 U∩I = U 中仅间接
        u_unseen = len(U - D - I)
        print(f"{name:<14} | {len(D):>7} | {len(D & U):>4} ({_pct(d_used, len(D)):>5}) | "
              f"{len(I):>7} | {len(I & U):>4} ({_pct(i_used, len(I)):>5}) | "
              f"{len(U):>5} | {len(U & D):>4} ({_pct(u_direct, len(U)):>5}) | "
              f"{len(U & I):>4} ({_pct(u_indirect, len(U)):>5}) | "
              f"{len(U - D - I):>4} ({_pct(u_unseen, len(U)):>5})")
    print("\n  (D被引用 = 直接读过的文件里有多少被证明引用; I被引用 同理)")
    print("   (U直接读/U间接读/U没读过 = 引用的文件里, 各有多少是通过该方式接触到的)")

    # ============ 分库表 ============
    print("\n\n=== 分库 (每行 = 某库在某模型上的 D / I / U 及交叉) ===\n")
    hdr2 = (f"{'库':<9} {'模型':<14} | {'直接读D':>6} | {'D被引用':>9} | {'间接读I':>6} | "
            f"{'I被引用':>9} | {'引用U':>5} | {'U直接读':>9} | {'U间接读':>9} | {'U没读过':>9}")
    print(hdr2)
    print("-" * len(hdr2))
    for lib in LIBS:
        for name in RUNS:
            D, I, U = _sets_for(list(ok[name].values()))
            D = {k for k in D if _library_of(k) == lib}
            I = {k for k in I if _library_of(k) == lib}
            U = {k for k in U if _library_of(k) == lib}
            print(f"{lib:<9} {name:<14} | {len(D):>6} | {len(D & U):>4} ({_pct(len(D & U), len(D)):>5}) | "
                  f"{len(I):>6} | {len(I & U):>4} ({_pct(len(I & U), len(I)):>5}) | "
                  f"{len(U):>5} | {len(U & D):>4} ({_pct(len(U & D), len(U)):>5}) | "
                  f"{len(U & I):>4} ({_pct(len(U & I), len(U)):>5}) | "
                  f"{len(U - D - I):>4} ({_pct(len(U - D - I), len(U)):>5})")
        print()

    # query-level detail
    print("\n--- #check/#print/#find queries per agent (name → file resolution) ---")
    for name in RUNS:
        recs = list(ok[name].values())
        qs = [q for r in recs for q in r["queries"]]
        if not qs:
            print(f"  {name}: (none)")
            continue
        from collections import Counter as C
        print(f"  {name}: {len(qs)} queries, {len(set(qs))} distinct")
        for q, n in C(qs).most_common(8):
            print(f"    {n:>3}  {q}")


if __name__ == "__main__":
    main()
