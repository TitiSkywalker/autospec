"""What the proving agent actually does across its (up to 100) tool calls."""

from __future__ import annotations

import argparse
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent

# ======================= Configuration (edit here) =======================

RESULTS_DIR = "evaluation/local/claude_opus_full_results"  # claude transcript dir
MAX_TOOL_CALLS = 100  # the run's tool-call cap
N_PHASES = 5  # temporal bins for phase analysis

# ========================= (no need to edit below) =========================

# --- Bash command classification -------------------------------------------------

# A segment is labelled by its leading verb after stripping `cd`/`timeout N`/
# `env VAR=x` prefixes. Verbs here are "read-only" (they print, never edit).
_READ_VERBS = {
    "cat",
    "sed",
    "grep",
    "egrep",
    "rg",
    "awk",
    "head",
    "tail",
    "wc",
    "ls",
    "find",
    "diff",
    "less",
    "more",
    "bat",
    "file",
    "du",
    "stat",
    "readlink",
    "strings",
    "nm",
    "objdump",
    "readelf",
    "zcat",
    "bzcat",
}
# Verbs that compile / type-check / run the proof assistant.
_BUILD_VERBS = {
    "lake",
    "lean",
    "cargo",
    "rustc",
    "gcc",
    "cc",
    "g++",
    "make",
    "cmake",
    "ninja",
    "elan",
}
# Verbs that (overwhelmingly) modify the filesystem.
_WRITE_VERBS = {
    "tee",
    "cp",
    "mv",
    "rm",
    "rmdir",
    "mkdir",
    "touch",
    "patch",
    "apply_patch",
    "install",
    "ln",
    "chmod",
}
# `git <sub>` is classified by sub-command.
_GIT_READ = {"diff", "show", "log", "status", "blame", "grep", "ls-files", "rev-parse"}
_GIT_WRITE = {
    "apply",
    "checkout",
    "restore",
    "reset",
    "add",
    "rm",
    "mv",
    "stash",
    "clean",
}

_HEREDOC_RE = re.compile(r"<<\s*'?([A-Za-z_][A-Za-z0-9_]*)'?")
_PATH_RE = re.compile(r"[\w./@-]+\.(lean|md|json|toml|py|rs|sh|txt|yaml|yml|olean|log)")


def _mask_heredocs(cmd: str) -> str:
    """Blank out heredoc bodies so their ``&&``/``;``/newlines don't split segments.

    ``<<'EOF' ... EOF`` bodies are replaced with spaces (the ``<<'EOF'`` marker
    and the closing ``EOF`` line are kept), so only real command separators
    remain for :func:`_split_segments`.
    """
    out = list(cmd)
    for m in _HEREDOC_RE.finditer(cmd):
        delim = m.group(1)
        j = m.end()
        end = None
        while j < len(cmd):
            nl = cmd.find("\n", j)
            if nl == -1:
                # Closing delimiter may sit on the final line with no trailing
                # newline; ``cmd[j:]`` is then that unterminated line.
                if cmd[j:].strip() == delim:
                    end = len(cmd)
                break
            if cmd[j:nl].strip() == delim:
                end = nl
                break
            j = nl + 1
        if end is not None:
            for k in range(m.end(), end):
                out[k] = " "
    return "".join(out)


def _split_segments(cmd: str) -> list[str]:
    r"""Split a Bash command into sub-commands on ``&&``/``||``/``;``/newlines.

    Only *unquoted, unescaped* separators count: ``2>&1``, ``|`` pipes and
    ``\|`` (escaped grep alternation) stay inside one segment, and a ``;`` or
    ``&&`` inside a quoted string does not split.
    """
    masked = _mask_heredocs(cmd)
    segs = []
    cur = []
    i = 0
    n = len(masked)
    quote = None  # current single/double-quote state
    while i < n:
        ch = masked[i]
        if quote is not None:
            cur.append(ch)
            if ch == "\\" and quote == '"' and i + 1 < n:
                cur.append(masked[i + 1])
                i += 2
                continue
            if ch == quote:
                quote = None
            i += 1
            continue
        if ch in "'\"":
            quote = ch
            cur.append(ch)
            i += 1
            continue
        if ch == "\\":  # escaped char: keep it with the next one (e.g. \|)
            cur.append(ch)
            if i + 1 < n:
                cur.append(masked[i + 1])
                i += 2
            else:
                i += 1
            continue
        if ch in "&|" and i + 1 < n and masked[i + 1] == ch:  # && / ||
            s = "".join(cur).strip()
            if s:
                segs.append(s)
            cur = []
            i += 2
            continue
        if ch in ";\n":
            s = "".join(cur).strip()
            if s:
                segs.append(s)
            cur = []
            i += 1
            continue
        cur.append(ch)
        i += 1
    s = "".join(cur).strip()
    if s:
        segs.append(s)
    return segs


def _verb(seg: str) -> str:
    """The leading command verb after stripping ``timeout N``/``nice``/``env``/``VAR=x``."""
    tokens = seg.split()
    i = 0
    while i < len(tokens):
        t = tokens[i]
        if (
            t in ("time", "timeout")
            and i + 1 < len(tokens)
            and re.match(r"^[\d.]+", tokens[i + 1])
        ):
            i += 2
        elif t == "nice":
            i += 1
            if i < len(tokens) and tokens[i] == "-n":
                i += 2
        elif t == "env":
            i += 1
            while i < len(tokens) and "=" in tokens[i]:
                i += 1
        elif re.match(r"^[A-Za-z_][A-Za-z0-9_]*=", t):
            i += 1
        else:
            break
    return tokens[i] if i < len(tokens) else ""


def classify_segment(seg: str) -> str:
    """Label one sub-command: ``read`` / ``write`` / ``build`` / ``other``."""
    verb = _verb(seg)
    if verb in _BUILD_VERBS:
        return "build"
    if verb == "sed":
        # `sed -i` edits in place; `sed -n A,Bp` / plain `sed` print.
        return "write" if re.search(r"(^|\s)-i", seg) else "read"
    if verb == "cat":
        # `cat > file` / `cat >> file` write; `cat file` (incl. `2>&1`) reads.
        if re.search(r"\bcat\s+>>", seg) or re.search(r"\bcat\s+>\s*[^&]", seg):
            return "write"
        return "read"
    if verb in ("python", "python3"):
        # A `python3 - <<'EOF'` script is (almost always) an edit of Spec/Scratch.
        return "write" if "<<" in seg else "other"
    if verb in _WRITE_VERBS:
        return "write"
    if verb == "perl":
        return "write" if re.search(r"\s-i", seg) else "read"
    if verb == "git":
        sub = seg.split()[1] if len(seg.split()) > 1 else ""
        if sub in _GIT_READ:
            return "read"
        if sub in _GIT_WRITE:
            return "write"
        return "other"
    if verb in _READ_VERBS:
        return "read"
    return "other"


def _tool_bucket(label: str) -> str:
    """Collapse a tool-call label to the coarse read/write/build/mcp/other bucket."""
    if label.startswith("Bash:"):
        return label.split(":", 1)[1]
    if label in ("Edit", "Write"):
        return "write"
    if label == "Read":
        return "read"
    # mcp__lean-lsp__lean_* and bare lean_* tools inspect/verify proof state
    return "mcp"


def classify_bash(cmd: str) -> tuple[str, list[tuple[str, str, str]]]:
    """Classify a whole Bash command: (primary label, [(segment, verb, label), ...])."""
    segs = _split_segments(cmd)
    labelled = [(s, _verb(s), classify_segment(s)) for s in segs]
    # Primary label: the most frequent non-"other" label, tie-broken write>build>read.
    counts = Counter(lbl for _, _, lbl in labelled)
    counts.pop("other", None)
    if not counts:
        return "other", labelled
    primary = max(("write", "build", "read"), key=lambda k: counts[k])
    return primary, labelled


# --- file provenance ------------------------------------------------------------

# Agent draft files: ``Scratch.lean``/``T5.lean``/``G.lean``/``M.lean``/``W2.lean``/...
# Single capital letter (+digits) is the agent's draft convention; Aeneas's own
# modules are real words (``Vec``/``Slice``/``Add``/...), so the two don't clash.
_SCRATCH_NAME_RE = re.compile(r"(Scratch\d*|Test|[A-Z]\d*|part\d+|ax)\.lean$")
# Project scaffolding the agent reads: challenge/solution/README/build config.
_SCAFFOLD = (
    "README.md",
    "Challenge.lean",
    "Solution.lean",
    "lakefile",
    "lake-manifest.json",
    "lean-toolchain",
    "Main.lean",
    "RustVerif.lean",
    "comparator",
    "CLAUDE.md",
    "instructions.md",
)
# Aeneas Std/Tactic modules the agent reads via a bare filename after ``cd``.
_AENEAS_BARE = {
    "Vec.lean",
    "Slice.lean",
    "SliceDef.lean",
    "WP.lean",
    "Primitives.lean",
    "RangeIter.lean",
    "Alloc.lean",
    "Scalar.lean",
    "Result.lean",
    "ControlFlow.lean",
    "Int.lean",
    "Usize.lean",
    "Isize.lean",
    "Bool.lean",
    "Char.lean",
    "String.lean",
    "List.lean",
    "Array.lean",
    "BitVec.lean",
    "F32.lean",
    "F64.lean",
    "Add.lean",
    "Sub.lean",
    "Mul.lean",
    "Div.lean",
    "Rem.lean",
    "Neg.lean",
    "Notation.lean",
    "Step.lean",
    "DspecInduction.lean",
    "Progress.lean",
    "Tactic.lean",
    "Wp.lean",
    "Aeneas.lean",
}


def _clean_path(p: str) -> str:
    p = p.strip().strip("\"'`(),;")
    while p.startswith("./"):
        p = p[2:]
    return p


def bucket_path(p: str) -> str:
    """Bucket a file path by provenance (problem / scratch / aeneas / ...)."""
    p = _clean_path(p)
    low = p.lower()
    base = p.rsplit("/", 1)[-1]
    if "/tmp/" in p or p.startswith("tmp/"):
        return "scratch drafts"
    if (
        "RustVerif/Spec.lean" in p
        or "RustVerif/Function.lean" in p
        or "RustVerif/Basic.lean" in p
    ):
        return "problem (Spec/Function)"
    if base in (
        "Spec.lean",
        "Function.lean",
        "Basic.lean",
    ):  # read via relative path (no RustVerif/ prefix)
        return "problem (Spec/Function)"
    if "RustVerif/" in p:
        return (
            "scratch drafts" if _SCRATCH_NAME_RE.search(base) else "RustVerif (other)"
        )
    if "tests/" in low or "Tutorial" in p or "Solutions" in p:
        return "aeneas tests/tutorial"
    if any(s in p for s in _SCAFFOLD):
        return "project scaffolding"
    # Aeneas library files are reached either by full path (``aeneas/...``) or by
    # a relative path after ``cd aeneas/backends/lean[/Aeneas[/Std...]]``.
    if (
        "aeneas" in low
        or low.startswith("std/")
        or "/std/" in low
        or low.startswith("tactic/")
        or "/tactic/" in low
        or low.startswith("scalar/")
        or "/scalar/" in low
        or base in _AENEAS_BARE
    ):
        return "aeneas library"
    if ".lake/" in p or p.endswith(".olean"):
        return "lake artifacts"
    if (
        "mathlib" in low
        or "/init/" in low
        or "/src/lean/" in low
        or "toolchains" in low
        or low.startswith("data/")
        or "linearalgebra" in low
    ):
        return "Mathlib / Lean core"
    if (
        "/scratch/" in low
        or low.startswith("scratch/")
        or _SCRATCH_NAME_RE.search(base)
    ):
        return "scratch drafts"
    if p.endswith(".rs"):
        return "original Rust source"
    if p.endswith(".py"):
        return "harness (Python)"
    if p.endswith((".toml", ".yml", ".yaml", ".log")) or base in (
        "verdict.txt",
        "status.txt",
    ):
        return "config / logs / results"
    if p.endswith(".md") or "documentation" in low:
        return "documentation"
    if "lean_mode/" in low or "translation/" in low:
        return "benchmark input (LeanMode)"
    return "other"


# --- transcript parsing ---------------------------------------------------------


def parse_trace(jsonl: Path) -> dict:
    """Parse one Claude Code transcript into the aggregates the report needs."""
    seg_labels: Counter = Counter()  # Bash sub-command label counts
    file_reads: Counter = Counter()  # normalized path -> times read
    file_buckets: Counter = Counter()  # provenance bucket -> times read
    tokens = {
        "input": 0,
        "output": 0,
        "cache_read": 0,
        "cache_creation": 0,
        "thinking": 0,
    }
    chars = {"thinking": 0, "tool_input": 0, "text": 0, "tool_result": 0}
    # char length of tool input by category, for the output-token attribution
    tool_input_chars: Counter = Counter()
    tool_calls: list[dict] = []  # ordered timeline (temporal analysis)
    cost = None

    with jsonl.open(errors="replace") as fh:
        for line in fh:
            try:
                e = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(e, dict):
                continue
            etype = e.get("type")

            if etype == "assistant":
                msg = e.get("message") or {}
                u = msg.get("usage") or {}
                if isinstance(u, dict):
                    tokens["input"] += u.get("input_tokens", 0) or 0
                    tokens["output"] += u.get("output_tokens", 0) or 0
                    tokens["cache_read"] += u.get("cache_read_input_tokens", 0) or 0
                    tokens["cache_creation"] += (
                        u.get("cache_creation_input_tokens", 0) or 0
                    )
                    det = u.get("output_tokens_details") or {}
                    tokens["thinking"] += det.get("thinking_tokens", 0) or 0

                for block in msg.get("content") or []:
                    if not isinstance(block, dict):
                        continue
                    btype = block.get("type")
                    if btype == "thinking":
                        chars["thinking"] += len(block.get("thinking", "") or "")
                    elif btype == "text":
                        chars["text"] += len(block.get("text", "") or "")
                    elif btype == "tool_use":
                        name = block.get("name", "")
                        inp = block.get("input") or {}
                        inp_str = json.dumps(inp) if not isinstance(inp, str) else inp
                        chars["tool_input"] += len(inp_str)
                        label = name
                        if name == "Bash":
                            primary, labelled = classify_bash(inp.get("command", ""))
                            label = f"Bash:{primary}"
                            for _seg, _v, lbl in labelled:
                                seg_labels[lbl] += 1
                                if lbl == "read":
                                    for fp in _extract_paths(_seg):
                                        file_reads[_clean_path(fp)] += 1
                                        file_buckets[bucket_path(fp)] += 1
                            tool_input_chars[f"tool:{label}"] += len(inp_str)
                        else:
                            seg_labels["mcp"] += 1
                            tool_input_chars[f"tool:{name}"] += len(inp_str)
                        tool_calls.append(
                            {
                                "name": name,
                                "label": label,
                                "ts": e.get("timestamp", ""),
                            }
                        )

            elif etype == "user":
                r = e.get("toolUseResult")
                if isinstance(r, dict):
                    chars["tool_result"] += len(r.get("stdout", "") or "")
                    chars["tool_result"] += len(r.get("stderr", "") or "")

            elif etype == "cost-state":
                cost = e

    return {
        "seg_labels": seg_labels,
        "file_reads": file_reads,
        "file_buckets": file_buckets,
        "tokens": tokens,
        "chars": chars,
        "tool_input_chars": tool_input_chars,
        "tool_calls": tool_calls,
        "cost": cost,
    }


def _extract_paths(cmd: str) -> list[str]:
    """Best-effort file paths mentioned in a Bash command (mask heredoc bodies first)."""
    masked = _mask_heredocs(cmd)
    out = []
    for m in _PATH_RE.finditer(masked):
        p = m.group(0)
        # drop obvious non-files: URL scheme, binary `2>` fragments
        if "://" in p:
            continue
        out.append(p)
    return out


# --- discovery ------------------------------------------------------------------


def discover_traces(results_dir: Path) -> list[tuple[str, Path]]:
    """[(case_name, trace_jsonl)] for every case with an archived transcript."""
    out = []
    for case in sorted(results_dir.iterdir()):
        if not case.is_dir():
            continue
        tdir = case / "trace"
        if not tdir.is_dir():
            continue
        for f in sorted(tdir.glob("*.jsonl")):
            out.append((case.name, f))
    return out


def verdict(case_dir: Path) -> str:
    v = case_dir / "verdict.txt"
    return v.read_text().strip() if v.is_file() else "UNKNOWN"


# --- reporting helpers ----------------------------------------------------------


def _pct(counter: Counter, keys) -> dict:
    total = sum(counter.get(k, 0) for k in keys) or 1
    return {k: 100.0 * counter.get(k, 0) / total for k in keys}


def _fmt_pct(pct: dict, keys) -> str:
    return "  ".join(f"{k}: {pct[k]:.1f}%" for k in keys if pct.get(k))


def _agg(results: list[dict]) -> dict:
    """Merge per-case parse results into one aggregate dict."""
    agg = defaultdict(lambda: Counter())
    agg_tokens = Counter()
    agg_chars = Counter()
    agg_tool_input = Counter()
    total_tool_calls = 0
    for r in results:
        for k, v in r["seg_labels"].items():
            agg["seg_labels"][k] += v
        for k, v in r["file_reads"].items():
            agg["file_reads"][k] += v
        for k, v in r["file_buckets"].items():
            agg["file_buckets"][k] += v
        for k, v in r["tokens"].items():
            agg_tokens[k] += v
        for k, v in r["chars"].items():
            agg_chars[k] += v
        for k, v in r["tool_input_chars"].items():
            agg_tool_input[k] += v
        total_tool_calls += len(r["tool_calls"])
    return {
        "seg_labels": agg["seg_labels"],
        "file_reads": agg["file_reads"],
        "file_buckets": agg["file_buckets"],
        "tokens": agg_tokens,
        "chars": agg_chars,
        "tool_input_chars": agg_tool_input,
        "total_tool_calls": total_tool_calls,
    }


def _report_aggregate(results: list[dict], top_files: int) -> None:
    agg = _agg(results)
    tokens = agg["tokens"]
    total_in = tokens["input"] + tokens["cache_read"] + tokens["cache_creation"]
    total_out = tokens["output"]
    total_tok = total_in + total_out

    print(
        f"\n=== Agent trace analysis ({len(results)} cases, "
        f"{agg['total_tool_calls']} tool calls, cap {MAX_TOOL_CALLS}/case) ==="
    )

    # ---- tool-call type mix ----
    type_counts = Counter()
    for r in results:
        for tc in r["tool_calls"]:
            type_counts[tc["label"]] += 1
    print("\n--- tool-call types (% of tool calls) ---")
    for label, n in type_counts.most_common():
        print(
            f"  {label:<24} {n:>6}  {100.0 * n / max(agg['total_tool_calls'], 1):5.1f}%"
        )

    # ---- read / write / build (Bash sub-command level) ----
    seg = agg["seg_labels"]
    pct = _pct(seg, ("read", "write", "build", "other", "mcp"))
    print("\n--- what the agent does: Bash sub-commands (% of actions) ---")
    print(f"  {_fmt_pct(pct, ('read', 'write', 'build', 'other', 'mcp'))}")
    print(f"  (raw counts: {dict(seg.most_common())})")
    print(
        "  note: 'read' = cat/sed -n/grep/head/... (reading code); 'write' = heredoc/"
        "sed -i/python edit;"
    )
    print(
        "        'build' = lake/lean; 'mcp' = lean-lsp proof-state queries; "
        "'other' is mostly `cd` navigation."
    )

    # ---- which code was read ----
    buckets = agg["file_buckets"]
    bpct = _pct(buckets, sorted(buckets))
    print("\n--- code read: provenance buckets (% of file-read events) ---")
    for b, p in sorted(bpct.items(), key=lambda kv: -kv[1]):
        print(f"  {b:<26} {p:5.1f}%   (n={buckets[b]})")
    print(f"\n  top-{top_files} files read:")
    for fp, n in agg["file_reads"].most_common(top_files):
        print(f"    {n:>5}  {fp}")

    # ---- token allocation ----
    print("\n--- token allocation ---")
    print(
        f"  input (context) : {total_in:>12,}  ({100.0 * total_in / max(total_tok, 1):.1f}%)"
    )
    print(
        f"  output (generate): {total_out:>12,}  ({100.0 * total_out / max(total_tok, 1):.1f}%)"
    )
    print("    input split:")
    for k, label in (
        ("cache_read", "cache_read (reused context)"),
        ("cache_creation", "cache_creation (fresh context)"),
        ("input", "uncached input"),
    ):
        print(
            f"      {label:<30} {tokens[k]:>12,}  {100.0 * tokens[k] / max(total_in, 1):.1f}%"
        )
    thinking = tokens["thinking"]
    non_thinking = max(total_out - thinking, 0)
    print(f"    output split:")
    print(
        f"      thinking (reasoning)         {thinking:>12,}  {100.0 * thinking / max(total_out, 1):.1f}%"
    )
    print(
        f"      non-thinking (text+tools)    {non_thinking:>12,}  {100.0 * non_thinking / max(total_out, 1):.1f}%"
    )

    # split non-thinking output between prose and tool inputs by char length
    chars = agg["chars"]
    prose_chars = chars["text"]
    tool_chars = chars["tool_input"]
    both = prose_chars + tool_chars
    prose_tok = non_thinking * prose_chars / both if both else 0
    tool_tok = non_thinking - prose_tok
    print("    non-thinking split (by generated chars):")
    print(
        f"      prose (text blocks)          {prose_tok:>12,.0f}  ({100.0 * prose_tok / max(total_out, 1):.1f}%)"
    )
    print(
        f"      tool-call inputs (code/cmds) {tool_tok:>12,.0f}  ({100.0 * tool_tok / max(total_out, 1):.1f}%)"
    )
    print("    tool-call output further split by category (char-based):")
    for label, n in agg["tool_input_chars"].most_common():
        frac = n / max(tool_chars, 1)
        print(
            f"      {label:<28} {tool_tok * frac:>10,.0f} tok  ({100.0 * frac:4.1f}% of tool input)"
        )

    # ---- temporal / duration ----
    _report_temporal(results)


def _report_temporal(results: list[dict]) -> None:
    """Phase mix (read/write/build over the run) and API-vs-tool time split."""
    # read/write/build mix per phase, averaged over cases with >= N_PHASES calls
    phase_counts = [Counter() for _ in range(N_PHASES)]
    phase_cases = 0
    for r in results:
        calls = r["tool_calls"]
        if len(calls) < N_PHASES:
            continue
        phase_cases += 1
        for i, tc in enumerate(calls):
            bin_i = min(int(i * N_PHASES / len(calls)), N_PHASES - 1)
            phase_counts[bin_i][_tool_bucket(tc["label"])] += 1

    print(
        f"\n--- temporal behaviour ({phase_cases} cases with >= {N_PHASES} tool calls) ---"
    )
    if phase_cases:
        keys = ("read", "write", "build", "mcp", "other")
        print("  mix of tool calls per phase of the run:")
        header = "  " + "".join(
            f"  {'phase ' + str(i + 1):>12}" for i in range(N_PHASES)
        )
        print(header)
        for k in keys:
            row = f"  {k:<8}"
            for i in range(N_PHASES):
                p = 100.0 * phase_counts[i][k] / max(sum(phase_counts[i].values()), 1)
                row += f"  {p:>10.1f}%"
            print(row)

    # duration split (from cost-state): model "thinking" (API) vs tool execution
    api = tool = total = 0
    n = 0
    for r in results:
        c = r["cost"]
        if isinstance(c, dict):
            api += c.get("totalAPIDuration", 0) or 0
            tool += c.get("totalToolDuration", 0) or 0
            total += c.get("totalDuration", 0) or 0
            n += 1
    if n:
        print(f"\n  wall-clock split ({n} cases, seconds):")
        print(f"    total             : {total / 1000:>10,.1f}")
        print(
            f"    model/API (think) : {api / 1000:>10,.1f}  ({100.0 * api / max(total, 1):.1f}%)"
        )
        print(
            f"    tool execution    : {tool / 1000:>10,.1f}  ({100.0 * tool / max(total, 1):.1f}%)"
        )


def _report_case(case_name: str, trace: Path, top_files: int) -> None:
    """Detailed per-case timeline."""
    r = parse_trace(trace)
    v = verdict(trace.parent.parent)  # trace/<uuid>.jsonl -> case dir
    print(f"\n=== {case_name} ===  ({len(r['tool_calls'])} tool calls, verdict {v})")
    calls = r["tool_calls"]
    for i, tc in enumerate(calls, 1):
        ts = tc["ts"][11:19] if tc["ts"] else ""
        print(f"  {i:>3}  {ts}  {tc['label']:<22} {tc['name']}")
    print("\n  sub-command mix:", dict(r["seg_labels"].most_common()))
    print("  tokens:", dict(r["tokens"]))
    print("  files read (top):")
    for fp, n in r["file_reads"].most_common(top_files):
        print(f"    {n:>4}  {fp}")


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Analyze the proving agent's session traces"
    )
    ap.add_argument(
        "--results-dir",
        default=RESULTS_DIR,
        help="results dir with <case>/trace/*.jsonl",
    )
    ap.add_argument("--case", help="dump the ordered timeline for one case")
    ap.add_argument(
        "--top-files", type=int, default=25, help="how many top files to list"
    )
    args = ap.parse_args()

    results_dir = (ROOT / args.results_dir).resolve()
    traces = discover_traces(results_dir)
    if not traces:
        sys.exit(f"no traces found under {results_dir}")

    if args.case:
        match = [t for name, t in traces if name == args.case]
        if not match:
            sys.exit(f"no such case: {args.case}")
        _report_case(args.case, match[0], args.top_files)
        return

    results = [parse_trace(t) for _name, t in traces]
    _report_aggregate(results, args.top_files)


if __name__ == "__main__":
    main()
