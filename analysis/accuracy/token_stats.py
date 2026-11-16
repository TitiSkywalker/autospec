"""Token usage statistics and cost-accuracy curves.

``python3 analysis/run_accuracy.py tokens`` prints the numbers to the terminal
and saves the curve plots to ``OUT_DIR``. The runs to compare and the model
prices are configured in the section below.
"""

import argparse
import json
import subprocess
from pathlib import Path

from analysis.accuracy.runs import RUNS
from analysis.accuracy.verdicts import read_verdict

ROOT = Path(__file__).resolve().parents[2]

# ======================= Configuration (edit here) =======================

# Official price table (USD per million tokens), keyed by the RUNS label.
# cache_read / cache_write fall back to the input price when omitted.
# Leave empty ({}) to only plot the token curve.
PRICES = {
    "GPT-6-sol": {
        "input": 2.0,
        "output": 10.0,
        "cache_read": 0.2,
        "cache_write": 5.0,
    },
    "GPT-5.6-sol": {
        "input": 4.0,
        "output": 20.0,
        "cache_read": 0.4,
        "cache_write": 10.0,
    },
    "Opus-5.5": {
        "input": 4.0,
        "output": 20.0,
        "cache_read": 0.2,
        "cache_write": 8.0,
    },
    "Opus-4.5": {
        "input": 5.0,
        "output": 25.0,
        "cache_read": 0.5,
        "cache_write": 10.0,
    },
    "DeepSeek-V4.1-Flash": {
        "input": 0.27,
        "output": 1.10,
        "cache_read": 0.027,
        "cache_write": 0.27,
    },
    "GLM-5.3": {
        "input": 1.4,
        "output": 4.4,
        "cache_read": 0.26,
        "cache_write": 0.0,
    },
    "Leanstral-1.5": {  # locally deployed: no API cost
        "input": 0.0,
        "output": 0.0,
        "cache_read": 0.0,
        "cache_write": 0.0,
    },
    "Kimi-K3": {  # USD (converted from RMB at 7.2)
        "input": 2.78,
        "output": 13.89,
        "cache_read": 0.28,
        "cache_write": 2.78,
    },
}

OUT_DIR = "analysis/accuracy/local"  # plot output dir

# ========================= (no need to edit below) =========================

_TOKEN_KEYS = ("input", "output", "cache_read", "cache_write")


def _read_zstd(path: Path) -> str:
    """Decompress a (possibly streaming-frame) zstd file to text.

    Prefers the ``zstandard`` package; falls back to the ``zstd`` CLI.
    """
    try:
        import zstandard

        with open(path, "rb") as fh:
            return zstandard.ZstdDecompressor().stream_reader(fh).read().decode()
    except Exception:
        return subprocess.run(
            ["zstd", "-dc", str(path)], capture_output=True
        ).stdout.decode()


def _parse_claude(path: Path) -> dict:
    """Sum token usage across assistant messages in a Claude transcript."""
    t = dict.fromkeys(_TOKEN_KEYS, 0)
    for line in Path(path).open(errors="replace"):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if not isinstance(e, dict) or e.get("type") != "assistant":
            continue
        u = (e.get("message") or {}).get("usage") or {}
        iters = u.get("iterations") or []
        if iters:
            for it in iters:
                t["input"] += it.get("input_tokens", 0) or 0
                t["output"] += it.get("output_tokens", 0) or 0
                t["cache_read"] += it.get("cache_read_input_tokens", 0) or 0
                t["cache_write"] += it.get("cache_creation_input_tokens", 0) or 0
        else:
            t["input"] += u.get("input_tokens", 0) or 0
            t["output"] += u.get("output_tokens", 0) or 0
            t["cache_read"] += u.get("cache_read_input_tokens", 0) or 0
            t["cache_write"] += u.get("cache_creation_input_tokens", 0) or 0
    return t


def _parse_dsh(path: Path) -> dict:
    """Sum token usage across assistant/message events in a dsh session log."""
    t = dict.fromkeys(_TOKEN_KEYS, 0)
    for line in _read_zstd(path).splitlines():
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("type") != "assistant/message":
            continue
        u = (e.get("data") or {}).get("usage") or {}
        t["input"] += u.get("inputTokens", 0) or 0
        t["output"] += u.get("outputTokens", 0) or 0
        t["cache_read"] += u.get("cacheReadTokens", 0) or 0
        t["cache_write"] += u.get("cacheWriteTokens", 0) or 0
    return t


def _parse_codex(path: Path) -> dict:
    """Sum token usage from a codex rollout JSONL.

    Codex emits ``event_msg``/``payload.type = "token_count"`` lines with a
    cumulative ``info.total_token_usage`` dict; the final such line holds the
    case total. Fields mirror the Claude naming (input_tokens,
    cached_input_tokens, cache_write_input_tokens, output_tokens).
    """
    t = dict.fromkeys(_TOKEN_KEYS, 0)
    last = None
    for line in Path(path).open(errors="replace"):
        try:
            e = json.loads(line)
        except json.JSONDecodeError:
            continue
        if e.get("type") != "event_msg":
            continue
        info = (e.get("payload") or {}).get("info") or {}
        if not isinstance(info, dict):
            continue
        tot = info.get("total_token_usage")
        if isinstance(tot, dict):
            last = tot
    if last is not None:
        t["input"] = last.get("input_tokens", 0) or 0
        t["output"] = last.get("output_tokens", 0) or 0
        t["cache_read"] = last.get("cached_input_tokens", 0) or 0
        t["cache_write"] = last.get("cache_write_input_tokens", 0) or 0
    return t


def _parse_vibe_meta(path: Path) -> dict:
    """Token usage from a vibe session ``meta.json``.

    vibe records session totals in ``stats``: ``session_prompt_tokens`` (every
    turn's prompt, so the cached prefix is counted repeatedly — the same
    convention as the Claude transcripts above), ``session_cached_tokens`` and
    ``session_completion_tokens``.
    """
    t = dict.fromkeys(_TOKEN_KEYS, 0)
    try:
        stats = (json.loads(Path(path).read_text()) or {}).get("stats") or {}
    except (json.JSONDecodeError, OSError):
        return t
    t["input"] = stats.get("session_prompt_tokens", 0) or 0
    t["output"] = stats.get("session_completion_tokens", 0) or 0
    t["cache_read"] = stats.get("session_cached_tokens", 0) or 0
    return t


def _collect(results_dir: Path) -> dict:
    """{case_name: {tokens..., solved}}"""
    cases = {}
    for d in sorted(Path(results_dir).iterdir()):
        if not d.is_dir():
            continue
        trace = d / "trace"
        if (trace / "session.v4.jsonl.zstd").is_file():
            tokens = _parse_dsh(trace / "session.v4.jsonl.zstd")
        elif (trace / "meta.json").is_file():
            # vibe: a per-case session log next to sub-agent sessions in agents/
            tokens = _parse_vibe_meta(trace / "meta.json")
        elif list(trace.glob("rollout-*.jsonl")):
            # codex rollout JSONL (one per session)
            tokens = dict.fromkeys(_TOKEN_KEYS, 0)
            for f in trace.glob("rollout-*.jsonl"):
                for k, v in _parse_codex(f).items():
                    tokens[k] += v
        else:
            jsonls = sorted(trace.glob("*.jsonl")) if trace.is_dir() else []
            tokens = dict.fromkeys(_TOKEN_KEYS, 0)
            if jsonls:
                for f in jsonls:
                    for k, v in _parse_claude(f).items():
                        tokens[k] += v
            else:
                continue
        tokens["solved"] = read_verdict(d) == "PROVED"
        cases[d.name] = tokens
    return cases


def _total(t: dict) -> int:
    return sum(t.get(k, 0) for k in _TOKEN_KEYS)


def _dollars(t: dict, p: dict) -> float:
    p_in, p_out = p["input"], p["output"]
    return (
        t["input"] * p_in
        + t["output"] * p_out
        + t["cache_read"] * p.get("cache_read", p_in)
        + t["cache_write"] * p.get("cache_write", p_in)
    ) / 1e6


def _curve(cases: dict, cost) -> tuple[list, list]:
    """Sort by cost ascending; return (average cost per case, cumulative solved)."""
    xs, ys, cum, solved = [], [], 0.0, 0
    for i, (_, c) in enumerate(
        sorted(cases.items(), key=lambda kv: cost(kv[1])), start=1
    ):
        cum += cost(c)
        if c["solved"]:
            solved += 1
        xs.append(cum / i)  # average cost per case over the i cheapest cases
        ys.append(solved)
    return xs, ys


def _plot(runs, xlabel, path, title):
    import matplotlib

    matplotlib.use("Agg")
    import matplotlib.pyplot as plt

    fig, ax = plt.subplots(figsize=(8, 5))
    for label, cases, cost in runs:
        xs, ys = _curve(cases, cost)
        ax.plot(
            xs, ys, drawstyle="steps-post", label=f"{label} ({ys[-1]}/{len(cases)})"
        )
    # symlog: log-like for real prices, but can also show zero-cost (local) runs
    ax.set_xscale("symlog", linthresh=0.01)
    ax.set_xlabel(xlabel)
    ax.set_ylabel("cumulative solved")
    ax.set_title(title)
    ax.grid(alpha=0.3)
    ax.legend()
    fig.tight_layout()
    fig.savefig(path, dpi=150)
    plt.close(fig)


def main(argv: list[str] | None = None) -> None:
    ap = argparse.ArgumentParser(description="Token usage and cost-accuracy curves")
    ap.parse_args(argv)

    runs = [(label, _collect(ROOT / p)) for label, p in RUNS.items()]

    print("\n=== Token usage ===")
    print(
        f"{'run':<22} {'cases':>5} {'input':>10} {'output':>10} {'cache_rd':>10} {'cache_wr':>10} {'total':>12} {'tok/proof':>12}"
    )
    for label, cases in runs:
        agg = dict.fromkeys(_TOKEN_KEYS, 0)
        for c in cases.values():
            for k in _TOKEN_KEYS:
                agg[k] += c[k]
        total = _total(agg)
        proved = sum(1 for c in cases.values() if c["solved"])
        per = total / proved if proved else float("inf")
        print(
            f"{label:<22} {len(cases):>5} {agg['input']:>10,} {agg['output']:>10,} "
            f"{agg['cache_read']:>10,} {agg['cache_write']:>10,} {total:>12,} {per:>12,.0f}"
        )

    if PRICES:
        print("\n=== Dollar cost ===")
        priced = []
        for label, cases in runs:
            p = PRICES.get(label)
            if p is None:
                print(f"{label:<22} (no price, skipped)")
                continue
            cost = sum(_dollars(c, p) for c in cases.values())
            solved = [_dollars(c, p) for c in cases.values() if c["solved"]]
            print(
                f"{label:<22} total $ {cost:,.2f}   per testcase $ {cost / len(cases):,.2f}"
                f"   per solved testcase (solved only) $ {sum(solved) / len(solved) if solved else 0:,.2f}"
            )
            priced.append((label, cases, lambda t, p=p: _dollars(t, p)))


if __name__ == "__main__":
    main()
