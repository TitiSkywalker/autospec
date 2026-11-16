"""Full-project sunburst: all touched files, one chart per agent, shared tree.

The tree spans the union of every file any agent ever read or cited across all
proved cases, structured as:

    root → library-group → directory-path → filename

Each leaf is coloured by how the agent for that panel accessed the file:

  green  — directly read AND cited by the proof
  blue   — directly read, not cited by the proof
  teal   — seen only indirectly (elaboration), cited
  purple — seen only indirectly, not cited
  red    — cited but never accessed (missed entirely)
  light grey — exists in the union tree; this agent never touched it

Usage::

    python3 analysis/agent_trace/read_used_project.py
    python3 analysis/agent_trace/read_used_project.py --out analysis/figs/my_output.html
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

# ── palette ───────────────────────────────────────────────────────────────────
STATUS_COLOR = {
    "cited_read":   "#2ca02c",  # green   — directly read + cited
    "read_uncited": "#1f77b4",  # blue    — directly read, not cited
    "cited_seen":   "#17becf",  # teal    — only seen indirectly + cited
    "seen_uncited": "#9467bd",  # purple  — only seen indirectly, not cited
    "cited_missed": "#d62728",  # red     — cited but never accessed
    "untouched":    "#e8e8e8",  # light grey — this agent never touched it
}
STATUS_LABEL = {
    "cited_read":   "directly read & cited",
    "read_uncited": "directly read, not cited",
    "cited_seen":   "seen indirectly & cited",
    "seen_uncited": "seen only indirectly",
    "cited_missed": "cited but never accessed",
    "untouched":    "not accessed by this agent",
}
_DIR_COLOR  = "#d0d0d0"   # inner directory rings
_ROOT_COLOR = "#b8b8b8"   # lib-group ring
_BG         = "#f5f5f5"


# ── data loading ──────────────────────────────────────────────────────────────

def _keys(v) -> list[str]:
    """Normalise a field that may be a list or a dict."""
    if isinstance(v, dict):
        return list(v.keys())
    return list(v) if v else []


def _lib(k: str) -> str:
    return k.split(":", 1)[0]


def _rel(k: str) -> str:
    return k.split(":", 1)[1]


def load_all(lib: str | None = None) -> tuple[set[str], dict[str, dict[str, str]]]:
    """Return (union_files, agent_status).

    union_files   — every key (lib:rel) touched by any agent in any proved case
    agent_status  — {agent: {key: status_string}}

    status values: cited_read | read_uncited | cited_seen | seen_uncited |
                   cited_missed | untouched

    lib — if given, restrict to files whose prefix matches (e.g. "aeneas",
          "mathlib", "program", "core").  None = all libraries.
    """
    # Per agent, accumulate direct reads, indirect seen, and cited sets.
    agent_direct:   dict[str, set[str]] = defaultdict(set)
    agent_indirect: dict[str, set[str]] = defaultdict(set)
    agent_cited:    dict[str, set[str]] = defaultdict(set)

    pattern = str(ROOT / "analysis" / "agent_trace" / "local" / "indirect_reads_data" / "*.json")
    for fpath in glob.glob(pattern):
        stem = Path(fpath).stem          # e.g. "claude-opus-5.5__array_two_sum"
        agent = stem.split("__", 1)[0]
        d = json.load(open(fpath))
        if "error" in d:
            continue

        direct_this: set[str] = set()
        for k in _keys(d.get("read")):
            agent_direct[agent].add(k)
            direct_this.add(k)

        seen_raw = set(_keys(d.get("seen_paths"))) | set(_keys(d.get("name_seen")))
        for k in seen_raw:
            if k not in direct_this:
                agent_indirect[agent].add(k)

        for k in _keys(d.get("used_explicit")):
            agent_cited[agent].add(k)

    # Remove from indirect anything also in direct (union-across-cases can create overlaps).
    for ag in agent_indirect:
        agent_indirect[ag] -= agent_direct[ag]

    # Apply library filter if requested.
    def _keep(k: str) -> bool:
        return lib is None or _lib(k) == lib

    for ag in list(agent_direct):
        agent_direct[ag]   = {k for k in agent_direct[ag]   if _keep(k)}
        agent_indirect[ag] = {k for k in agent_indirect[ag] if _keep(k)}
        agent_cited[ag]    = {k for k in agent_cited[ag]    if _keep(k)}

    union_files: set[str] = set()
    for s in (agent_direct, agent_indirect, agent_cited):
        for files in s.values():
            union_files |= files

    # Build per-agent status map.
    agent_status: dict[str, dict[str, str]] = {}
    for ag in set(agent_direct) | set(agent_indirect) | set(agent_cited):
        direct   = agent_direct.get(ag, set())
        indirect = agent_indirect.get(ag, set())
        cited    = agent_cited.get(ag, set())
        status: dict[str, str] = {}
        for k in union_files:
            r = k in direct
            s = k in indirect
            c = k in cited
            if r and c:
                status[k] = "cited_read"
            elif r:
                status[k] = "read_uncited"
            elif s and c:
                status[k] = "cited_seen"
            elif s:
                status[k] = "seen_uncited"
            elif c:
                status[k] = "cited_missed"
            else:
                status[k] = "untouched"
        agent_status[ag] = status

    return union_files, agent_status


# ── shared tree builder ───────────────────────────────────────────────────────

def build_tree(union_files: set[str]) -> tuple[list[str], list[str], list[str], list[int]]:
    """Build (ids, labels, parents, leaf_counts) for the shared union tree.

    Node-id scheme:
      "aeneas:Aeneas"                    — top-level directory (parent = "")
      "aeneas:Aeneas/Std/File.lean"      — leaf file

    No root node is emitted so there is no center hole.  Each top-level
    directory becomes a root of its own sub-tree; Plotly handles multiple
    roots in one sunburst.
    """
    dir_nodes: set[str] = set()
    dir_count: dict[str, int] = defaultdict(int)
    for k in union_files:
        lib = _lib(k)
        parts = _rel(k).split("/")
        for depth in range(1, len(parts)):
            dir_id = lib + ":" + "/".join(parts[:depth])
            dir_nodes.add(dir_id)
            dir_count[dir_id] += 1

    ids:     list[str] = []
    labels:  list[str] = []
    parents: list[str] = []
    values:  list[int] = []

    # Directory nodes — top-level dirs get parent="" (multi-root, no centre).
    for dir_id in sorted(dir_nodes):
        lib = _lib(dir_id)
        rel_dir = _rel(dir_id)
        ids.append(dir_id)
        labels.append(rel_dir.rsplit("/", 1)[-1])
        if "/" not in rel_dir:
            parents.append("")              # top-level: no parent → no hole
        else:
            parent_rel = rel_dir.rsplit("/", 1)[0]
            parents.append(lib + ":" + parent_rel)
        values.append(dir_count[dir_id])

    # Leaf file nodes.
    for k in sorted(union_files):
        lib = _lib(k)
        rel  = _rel(k)
        ids.append(k)
        labels.append(rel.rsplit("/", 1)[-1])
        parts = rel.split("/")
        if len(parts) == 1:
            parents.append("")              # flat file at top level
        else:
            parents.append(lib + ":" + "/".join(parts[:-1]))
        values.append(1)

    return ids, labels, parents, values


def color_list(ids: list[str], agent_status: dict[str, str]) -> list[str]:
    """Map each node in the shared tree to a colour for one agent's panel."""
    colors = []
    for nid in ids:
        if nid in agent_status:
            colors.append(STATUS_COLOR[agent_status[nid]])
        else:
            colors.append(_DIR_COLOR)        # directory node
    return colors


# ── per-agent stats ───────────────────────────────────────────────────────────

def _agent_stats(agent_status: dict[str, str]) -> dict[str, int]:
    counts: dict[str, int] = defaultdict(int)
    for s in agent_status.values():
        counts[s] += 1
    return dict(counts)


def _stats_bar_html(counts: dict[str, int]) -> str:
    total = sum(counts.values())
    touched = total - counts.get("untouched", 0)
    pct = round(100 * touched / total) if total else 0
    parts = [
        f"<b>{touched}</b>/{total} files touched ({pct}%)"
    ]
    for key in ("cited_read", "read_uncited", "cited_seen", "seen_uncited", "cited_missed"):
        n = counts.get(key, 0)
        if n:
            color = STATUS_COLOR[key]
            parts.append(
                f'<span style="color:{color}">&#9632;</span> '
                f'{STATUS_LABEL[key]}: <b>{n}</b>'
            )
    return " &nbsp;&middot;&nbsp; ".join(parts)


# ── HTML renderer ─────────────────────────────────────────────────────────────

_PANEL_JS = """
(function() {{
  var trace = {{
    type: "sunburst",
    ids:     {ids_json},
    labels:  {labels_json},
    parents: {parents_json},
    values:  {values_json},
    marker:  {{
      colors: {colors_json},
      line:   {{ color: "#fff", width: 0.8 }}
    }},
    hovertemplate: "%{{id}}<br>%{{value}} file(s)<extra></extra>",
    branchvalues: "total",
    maxdepth: 4,
    textfont: {{ size: 10 }},
    insidetextorientation: "radial",
  }};
  var layout = {{
    margin: {{t:8,b:8,l:8,r:8}},
    paper_bgcolor: "rgba(0,0,0,0)",
    plot_bgcolor:  "rgba(0,0,0,0)",
  }};
  Plotly.newPlot("{div_id}", [trace], layout, {{responsive:true, displayModeBar:false}});
}})();
"""

_HTML_PAGE = """\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Full project — read vs cited — all agents</title>
<script src="https://cdn.plot.ly/plotly-2.32.0.min.js"></script>
<style>
*, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  background: {bg};
  color: #1a1a1a;
  padding: 20px;
}}
h1 {{ font-size: 1.2rem; font-weight: 700; margin-bottom: 2px; }}
.subtitle {{ font-size: 0.8rem; color: #555; margin-bottom: 16px; }}
.grid {{
  display: grid;
  grid-template-columns: 1fr 1fr;
  grid-template-rows: auto auto;
  gap: 14px;
}}
.panel {{
  background: #fff;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 12px 14px 8px;
}}
.panel h2 {{ font-size: 0.95rem; font-weight: 600; margin-bottom: 2px; }}
.panel .stats {{
  font-size: 0.72rem; color: #444; margin-bottom: 8px; line-height: 1.6;
}}
.legend {{
  margin-top: 16px;
  padding-top: 10px;
  border-top: 1px solid #e0e0e0;
  display: flex; flex-wrap: wrap; gap: 6px 18px;
}}
.li {{ display: flex; align-items: center; gap: 6px; font-size: 0.75rem; color: #333; }}
.sw {{ width: 12px; height: 12px; border-radius: 2px; flex-shrink: 0; }}
</style>
</head>
<body>
<h1>Full project — file access by agent</h1>
<p class="subtitle">
  {n_files} unique files in the union tree &middot;
  each panel shows one agent &middot;
  all four share the same tree structure
</p>
<div class="grid">
{panels}
</div>
<div class="legend">
{legend}
</div>
<script src="https://cdn.plot.ly/plotly-2.32.0.min.js"></script>
{scripts}
</body>
</html>
"""


def render(out_path: Path, lib: str | None = None) -> None:
    import json as _json

    union_files, all_agent_status = load_all(lib=lib)
    ids, labels, parents, values = build_tree(union_files)

    ids_j     = _json.dumps(ids)
    labels_j  = _json.dumps(labels)
    parents_j = _json.dumps(parents)
    values_j  = _json.dumps(values)

    panels_html = []
    scripts_html = []

    agents = sorted(all_agent_status.keys())
    for agent in agents:
        div_id = f"chart-{agent.replace('.', '-')}"
        status = all_agent_status[agent]
        colors = color_list(ids, status)
        counts = _agent_stats(status)

        panels_html.append(
            f'<div class="panel">\n'
            f'  <h2>{agent}</h2>\n'
            f'  <div class="stats">{_stats_bar_html(counts)}</div>\n'
            f'  <div id="{div_id}" style="height:460px"></div>\n'
            f'</div>'
        )
        scripts_html.append(
            _PANEL_JS.format(
                div_id=div_id,
                ids_json=ids_j,
                labels_json=labels_j,
                parents_json=parents_j,
                values_json=values_j,
                colors_json=_json.dumps(colors),
            )
        )

    legend_items = "\n".join(
        f'<span class="li">'
        f'<span class="sw" style="background:{STATUS_COLOR[k]}"></span>'
        f'{STATUS_LABEL[k]}</span>'
        for k in STATUS_LABEL
    )

    page = _HTML_PAGE.format(
        bg=_BG,
        n_files=len(union_files),
        panels="\n".join(panels_html),
        legend=legend_items,
        scripts="\n".join(scripts_html),
    )

    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(page, encoding="utf-8")
    print(f"-> {out_path}")


# ── PNG rendering ─────────────────────────────────────────────────────────────
# 2×2 domain grid; each agent's sunburst gets one quadrant. A combined PNG is
# the primary output; per-agent PNGs are also written for slide reuse.

_DOMAINS = [
    {"x": [0.00, 0.50], "y": [0.50, 1.00]},   # top-left
    {"x": [0.50, 1.00], "y": [0.50, 1.00]},   # top-right
    {"x": [0.00, 0.50], "y": [0.00, 0.50]},   # bottom-left
    {"x": [0.50, 1.00], "y": [0.00, 0.50]},   # bottom-right
]


def render_png(out_dir: Path, lib: str | None = None, width: int = 2000, scale: int = 2) -> None:
    import plotly.graph_objects as go
    import plotly.io as pio

    union_files, all_agent_status = load_all(lib=lib)
    ids, labels, parents, values = build_tree(union_files)
    agents = sorted(all_agent_status.keys())
    lib_tag = f"_{lib}" if lib else ""

    out_dir.mkdir(parents=True, exist_ok=True)

    # ── combined 2×2 grid ──────────────────────────────────────────────────
    traces = []
    annotations = []
    for i, agent in enumerate(agents):
        status = all_agent_status[agent]
        colors = color_list(ids, status)
        counts = _agent_stats(status)
        dom = _DOMAINS[i]

        traces.append(go.Sunburst(
            ids=ids, labels=labels, parents=parents, values=values,
            marker=dict(colors=colors, line=dict(color="#fff", width=0.8)),
            hovertemplate="%{id}<br>%{value} file(s)<extra></extra>",
            branchvalues="total",
            maxdepth=4,
            textfont=dict(size=9),
            insidetextorientation="radial",
            domain=dom,
            sort=False,
        ))

        # quadrant title + stats line
        cx = (dom["x"][0] + dom["x"][1]) / 2
        y_top = dom["y"][1]
        y_bot = dom["y"][0]
        annotations.append(dict(
            x=cx, y=y_top - 0.005, xref="paper", yref="paper",
            text=f"<b>{agent}</b>", showarrow=False,
            font=dict(size=15, color="#1a1a1a"), yanchor="top",
        ))
        touched = sum(v for k, v in counts.items() if k != "untouched")
        annotations.append(dict(
            x=cx, y=y_top - 0.028, xref="paper", yref="paper",
            text=f"{touched} files touched", showarrow=False,
            font=dict(size=10, color="#555"), yanchor="top",
        ))

    # shared legend below the grid
    legend_pairs = [(STATUS_COLOR[k], STATUS_LABEL[k]) for k in STATUS_LABEL]
    legend_text = "    ".join(f"■ {lab}" for _, lab in legend_pairs)
    annotations.append(dict(
        x=0.5, y=-0.02, xref="paper", yref="paper",
        text=legend_text, showarrow=False,
        font=dict(size=10, color="#333"), yanchor="top",
    ))

    fig = go.Figure(data=traces)
    fig.update_layout(
        margin=dict(t=24, b=40, l=8, r=8),
        paper_bgcolor=_BG,
        plot_bgcolor=_BG,
        annotations=annotations,
        showlegend=False,
        font=dict(family="sans-serif"),
    )
    fig.update_xaxes(visible=False)
    fig.update_yaxes(visible=False)

    combined = out_dir / f"read_used_project{lib_tag}_sunburst.png"
    pio.write_image(fig, str(combined), width=width, height=width, scale=scale)
    print(f"-> {combined}")

    # ── per-agent high-res PNGs ─────────────────────────────────────────────
    for agent in agents:
        status = all_agent_status[agent]
        colors = color_list(ids, status)
        counts = _agent_stats(status)
        touched = sum(v for k, v in counts.items() if k != "untouched")

        single = go.Figure(go.Sunburst(
            ids=ids, labels=labels, parents=parents, values=values,
            marker=dict(colors=colors, line=dict(color="#fff", width=0.8)),
            hovertemplate="%{id}<br>%{value} file(s)<extra></extra>",
            branchvalues="total",
            maxdepth=5,
            textfont=dict(size=10),
            insidetextorientation="radial",
            sort=False,
        ))
        single.update_layout(
            margin=dict(t=44, b=8, l=8, r=8),
            paper_bgcolor=_BG,
            plot_bgcolor=_BG,
            title=dict(
                text=f"<b>{agent}</b>  &middot;  {touched} files touched",
                font=dict(size=14, color="#1a1a1a"), x=0.5, xanchor="center",
            ),
            showlegend=False,
            font=dict(family="sans-serif"),
        )
        path = out_dir / f"read_used_project{lib_tag}_{agent.replace('.', '-')}.png"
        pio.write_image(single, str(path), width=1200, height=1200, scale=2)
        print(f"-> {path}")


def main() -> None:
    ap = argparse.ArgumentParser(
        description="Full-project sunburst: all files, one panel per agent"
    )
    ap.add_argument(
        "--out",
        default=str(OUT_DIR / "read_used_project_sunburst.html"),
        help="output HTML path (when not --png, single lib only)",
    )
    ap.add_argument(
        "--lib",
        choices=("aeneas", "mathlib", "program", "core"),
        help="restrict to one library (default: aeneas and mathlib, separately)",
    )
    ap.add_argument(
        "--png",
        action="store_true",
        help="write PNGs instead of HTML",
    )
    ap.add_argument(
        "--png-dir",
        default=str(OUT_DIR),
        help="output directory for PNGs (default: analysis/figs)",
    )
    args = ap.parse_args()

    libs = [args.lib] if args.lib else ["aeneas", "mathlib"]

    if args.png:
        for lib in libs:
            render_png(Path(args.png_dir), lib=lib)
    else:
        if len(libs) == 1:
            render(Path(args.out), lib=libs[0])
        else:
            for lib in libs:
                stem = Path(args.out).stem.replace("_sunburst", "")
                out = Path(args.out).parent / f"{stem}_{lib}_sunburst.html"
                render(out, lib=lib)


if __name__ == "__main__":
    main()
