"""Sunburst charts of file access patterns for coding agents.

For each agent × library pair we produce an interactive HTML page with two
Plotly sunburst charts:

  Left  — direct reads (files the agent explicitly opened)
  Right — indirect / seen (files elaborated-through but never opened,
           plus cited files the agent missed entirely)

Each leaf is coloured by whether it also appears in the proof's ``used_explicit``
set, so the overlap between "what the agent read" and "what the proof needs"
is immediately visible.

Usage::

    python3 analysis/agent_trace/read_used_radial.py                    # all agents × both libs
    python3 analysis/agent_trace/read_used_radial.py --agent claude-opus-5.5 --lib mathlib
    python3 analysis/agent_trace/read_used_radial.py --agent claude-opus-5.5 --lib aeneas
"""

from __future__ import annotations

import argparse
import glob
import json
import sys
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.hmm import RUNS  # noqa: E402

OUT_DIR = ROOT / "analysis" / "figs"

# ── palette ──────────────────────────────────────────────────────────────────
# Colorblind-safe categorical palette; hues validated for CVD separation.
# Statuses that appear on leaf nodes only; inner directory nodes are grey.
STATUS_COLOR = {
    "cited_read":   "#2ca02c",  # green   — cited AND directly read
    "read_uncited": "#1f77b4",  # blue    — directly read, proof didn't cite it
    "cited_seen":   "#17becf",  # teal    — cited AND seen indirectly
    "seen_uncited": "#9467bd",  # purple  — seen indirectly, proof didn't cite it
    "cited_missed": "#d62728",  # red     — cited but never seen or read at all
}
STATUS_LABEL = {
    "cited_read":   "cited & directly read",
    "read_uncited": "directly read, not cited",
    "cited_seen":   "cited & indirectly seen",
    "seen_uncited": "only seen indirectly",
    "cited_missed": "cited but never accessed",
}
_INNER = "#e0e0e0"   # directory interior nodes
_BG    = "#f7f7f7"   # page background


# ── data loading ─────────────────────────────────────────────────────────────

def _lib_of(k: str) -> str:
    return k.split(":", 1)[0]


def _rel_of(k: str) -> str:
    return k.split(":", 1)[1]


def _load(agent: str, lib: str) -> tuple[dict[str, str], dict[str, str]]:
    """Return (direct_status, nondirect_status).

    direct_status    — rel-path → status for files in agent's ``read`` set
    nondirect_status — rel-path → status for files seen only indirectly, plus
                       cited files the agent never accessed at all
    """
    # Accumulate across all proved cases for this agent+lib.
    directly_read: set[str] = set()
    indirectly_seen: set[str] = set()
    cited: set[str] = set()

    pattern = str(ROOT / "analysis" / "agent_trace" / "local" / "indirect_reads_data"
                  / f"{agent}__*.json")
    for fpath in glob.glob(pattern):
        d = json.load(open(fpath))
        if "error" in d or d.get("verdict") != "PROVED":
            continue

        case_direct: set[str] = set()
        for k in list(d.get("read") or []):
            if _lib_of(k) == lib:
                rel = _rel_of(k)
                case_direct.add(rel)
                directly_read.add(rel)

        seen_raw = set(list(d.get("seen_paths") or []))
        seen_raw |= set(list(d.get("name_seen") or []))
        for k in seen_raw:
            if _lib_of(k) == lib:
                rel = _rel_of(k)
                if rel not in case_direct:
                    indirectly_seen.add(rel)

        for k in list(d.get("used_explicit") or []):
            if _lib_of(k) == lib:
                cited.add(_rel_of(k))

    # Remove from indirectly_seen any file that was also directly read.
    indirectly_seen -= directly_read

    def _direct_status(rel: str) -> str:
        return "cited_read" if rel in cited else "read_uncited"

    def _nondirect_status(rel: str) -> str:
        if rel in indirectly_seen:
            return "cited_seen" if rel in cited else "seen_uncited"
        # cited but never accessed
        return "cited_missed"

    direct_status = {r: _direct_status(r) for r in directly_read}

    missed = cited - directly_read - indirectly_seen
    nondirect_files = indirectly_seen | missed
    nondirect_status = {r: _nondirect_status(r) for r in nondirect_files}

    return direct_status, nondirect_status, cited


# ── sunburst data builder ─────────────────────────────────────────────────────

def _build_sunburst(file_status: dict[str, str]) -> tuple[list, list, list, list, list]:
    """Convert {rel_path: status} into (ids, labels, parents, values, colors).

    Interior nodes (directories) are summed from leaf counts.  The root node
    has parent="" and id="root".
    """
    # Count leaves per file (each file = 1).
    leaf_weight: dict[str, int] = {rel: 1 for rel in file_status}

    # Collect all directory nodes and accumulate counts bottom-up.
    dir_count: dict[str, int] = defaultdict(int)
    for rel in file_status:
        parts = rel.split("/")
        for depth in range(1, len(parts)):
            anc = "/".join(parts[:depth])
            dir_count[anc] += 1

    ids: list[str] = []
    labels: list[str] = []
    parents: list[str] = []
    values: list[int] = []
    colors: list[str] = []

    root_id = "(root)"

    # Add directories (inner rings).
    for d, cnt in dir_count.items():
        ids.append(d)
        labels.append(d.rsplit("/", 1)[-1])
        parts = d.split("/")
        parents.append("/".join(parts[:-1]) if len(parts) > 1 else root_id)
        values.append(cnt)
        colors.append(_INNER)

    # Add file leaves (outer ring).
    for rel, status in file_status.items():
        ids.append(rel)
        labels.append(rel.rsplit("/", 1)[-1])
        parts = rel.split("/")
        parents.append("/".join(parts[:-1]) if len(parts) > 1 else root_id)
        values.append(1)
        colors.append(STATUS_COLOR[status])

    # Add root.
    total = sum(leaf_weight.values())
    ids.append(root_id)
    labels.append(root_id)
    parents.append("")
    values.append(total)
    colors.append(_INNER)

    return ids, labels, parents, values, colors


# ── HTML renderer ─────────────────────────────────────────────────────────────

_HTML_TMPL = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title}</title>
<script src="https://cdn.plot.ly/plotly-2.32.0.min.js"></script>
<style>
  *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
  body {{
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
    background: {bg};
    color: #1a1a1a;
    padding: 24px;
  }}
  h1 {{
    font-size: 1.25rem;
    font-weight: 600;
    margin-bottom: 4px;
    letter-spacing: -0.01em;
  }}
  .subtitle {{
    font-size: 0.85rem;
    color: #555;
    margin-bottom: 20px;
  }}
  .charts {{
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }}
  .panel {{
    background: #fff;
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 16px 16px 8px;
  }}
  .panel h2 {{
    font-size: 0.95rem;
    font-weight: 600;
    margin-bottom: 2px;
    color: #222;
  }}
  .panel .desc {{
    font-size: 0.75rem;
    color: #666;
    margin-bottom: 10px;
  }}
  .legend {{
    display: flex;
    flex-wrap: wrap;
    gap: 6px 16px;
    margin-top: 16px;
    padding-top: 12px;
    border-top: 1px solid #eee;
  }}
  .legend-item {{
    display: flex;
    align-items: center;
    gap: 6px;
    font-size: 0.78rem;
    color: #333;
  }}
  .swatch {{
    width: 12px;
    height: 12px;
    border-radius: 2px;
    flex-shrink: 0;
  }}
  .stats {{
    font-size: 0.78rem;
    color: #555;
    margin-top: 10px;
    display: flex;
    gap: 16px;
    flex-wrap: wrap;
  }}
  .stat {{ font-weight: 600; color: #222; }}
</style>
</head>
<body>
<h1>{title}</h1>
<p class="subtitle">{subtitle}</p>
<div class="charts">
  <div class="panel">
    <h2>Direct reads</h2>
    <p class="desc">Files the agent explicitly opened. Colour = cited by proof or not.</p>
    <div id="chart-direct" style="height:480px"></div>
    <div class="stats">{stats_direct}</div>
  </div>
  <div class="panel">
    <h2>Indirect access &amp; missed citations</h2>
    <p class="desc">Files reached only through elaboration, plus cited files the agent never opened.</p>
    <div id="chart-indirect" style="height:480px"></div>
    <div class="stats">{stats_indirect}</div>
  </div>
</div>
<div class="legend">
  {legend_items}
</div>
<script>
var direct = {direct_json};
var indirect = {indirect_json};
var layout = {{
  margin: {{t:16,b:8,l:8,r:8}},
  paper_bgcolor: "rgba(0,0,0,0)",
  plot_bgcolor:  "rgba(0,0,0,0)",
  sunburstcolorway: [],
}};
var config = {{responsive: true, displayModeBar: false}};
Plotly.newPlot("chart-direct",   [direct],   layout, config);
Plotly.newPlot("chart-indirect", [indirect], layout, config);
</script>
</body>
</html>"""


def _sunburst_trace_json(file_status: dict[str, str], chart_title: str) -> str:
    import json as _json
    if not file_status:
        # Empty placeholder trace.
        return _json.dumps({
            "type": "sunburst",
            "ids": ["(none)"], "labels": ["no data"],
            "parents": [""], "values": [1],
            "marker": {"colors": [_INNER]},
            "hovertemplate": "<b>%{label}</b><extra></extra>",
            "branchvalues": "total",
            "maxdepth": 4,
        })
    ids, labels, parents, values, colors = _build_sunburst(file_status)
    # Build custom hover text: for leaves show status label.
    id_to_status = dict(file_status)
    hover = []
    for nid in ids:
        if nid in id_to_status:
            hover.append(f"<b>{nid}</b><br>{STATUS_LABEL[id_to_status[nid]]}<extra></extra>")
        else:
            hover.append(f"<b>{nid}</b><br>{values[ids.index(nid)]} files<extra></extra>")

    trace = {
        "type": "sunburst",
        "ids": ids,
        "labels": labels,
        "parents": parents,
        "values": values,
        "marker": {"colors": colors, "line": {"color": "#fff", "width": 1}},
        "hovertemplate": hover,
        "branchvalues": "total",
        "maxdepth": 4,
        "textfont": {"size": 11},
    }
    return _json.dumps(trace)


def _stats_html(file_status: dict[str, str]) -> str:
    counts: dict[str, int] = defaultdict(int)
    for s in file_status.values():
        counts[s] += 1
    total = len(file_status)
    parts = [f"<span>Total: <span class='stat'>{total}</span></span>"]
    for key, label in STATUS_LABEL.items():
        if key in counts:
            parts.append(
                f"<span>{label.split(',')[0]}: "
                f"<span class='stat'>{counts[key]}</span></span>"
            )
    return " ".join(parts)


def _legend_html() -> str:
    items = []
    for key, label in STATUS_LABEL.items():
        color = STATUS_COLOR[key]
        items.append(
            f'<span class="legend-item">'
            f'<span class="swatch" style="background:{color}"></span>'
            f'{label}</span>'
        )
    return "\n  ".join(items)


def _render(agent: str, lib: str) -> None:
    direct_status, nondirect_status, cited = _load(agent, lib)

    title = f"{agent} — {lib}: read vs cited"
    n_cited = len(cited)
    n_direct = len(direct_status)
    n_indirect = len(nondirect_status)
    overlap = sum(1 for s in direct_status.values() if s == "cited_read")
    overlap_pct = round(100 * overlap / n_cited) if n_cited else 0

    subtitle = (
        f"{n_direct} directly read · "
        f"{n_indirect} indirectly seen / missed · "
        f"{n_cited} cited by proofs · "
        f"{overlap} ({overlap_pct}%) cited files were directly read"
    )

    direct_json   = _sunburst_trace_json(direct_status,   "direct reads")
    indirect_json = _sunburst_trace_json(nondirect_status, "indirect + missed")

    html = _HTML_TMPL.format(
        title=title,
        subtitle=subtitle,
        bg=_BG,
        direct_json=direct_json,
        indirect_json=indirect_json,
        stats_direct=_stats_html(direct_status),
        stats_indirect=_stats_html(nondirect_status),
        legend_items=_legend_html(),
    )

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    path = OUT_DIR / f"read_used_{agent}_{lib}_sunburst.html"
    path.write_text(html, encoding="utf-8")
    print(f"  -> {path}")


def main() -> None:
    ap = argparse.ArgumentParser(description="Sunburst charts of read vs cited file access")
    ap.add_argument("--agent", help="single agent (default: all)")
    ap.add_argument("--lib", choices=("aeneas", "mathlib"), help="single library (default: both)")
    args = ap.parse_args()

    agents = [args.agent] if args.agent else list(RUNS)
    libs = [args.lib] if args.lib else ("aeneas", "mathlib")

    for agent in agents:
        for lib in libs:
            _render(agent, lib)


if __name__ == "__main__":
    main()
