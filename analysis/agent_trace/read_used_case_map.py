"""Per-testcase file map: files the agent *read* vs files the proof *references*.

One interactive HTML page.  Pick an agent and a testcase; two sunbursts share
the same file tree (only files this case touched), so a file sits in the same
spot in both:

  blue   — read by the agent, directly (cat/grep/Read) or indirectly (it showed
           up in tool-call text: compiler errors, ``#check`` queries, ...)
  orange — referenced by the final proof

Shade encodes how many times: darker = more.  Light grey = not touched on that
side.  A file that is dark in both panels was read and used; blue-only is wasted
reading; orange-only is used without having been read.

Counts.  Read side: direct reads are counted; a file seen only indirectly counts
once.  Used side: ``used_counts`` (premise references per file) when the cache
has it, else the file counts once.  The cache only gains ``used_counts`` for
cases (re)built after ``indirect_reads`` started recording it.

Needs the ``indirect_reads`` cache.  Usage::

    python3 analysis/agent_trace/read_used_case_map.py
    python3 analysis/agent_trace/read_used_case_map.py --png [--agent A] [--case C]   # PNG per (agent, case)
    python3 analysis/agent_trace/read_used_case_map.py --used all     # incl. automation-pulled premises
    python3 analysis/agent_trace/read_used_case_map.py --out analysis/agent_trace/local/figs/my_map.html
"""

from __future__ import annotations

import argparse
import glob
import json
import math
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
sys.path.insert(0, str(ROOT))

from analysis.agent_trace.read_used_project import build_tree  # noqa: E402

CACHE_GLOB = str(ROOT / "analysis" / "agent_trace" / "local" / "indirect_reads_data" / "*.json")
OUT_PATH = ROOT / "analysis" / "agent_trace" / "local" / "figs" / "read_used_case_map.html"

# (lightest, darkest) shade per side
READ_SHADES = ("#cfe3f5", "#08306b")
USED_SHADES = ("#fde0c5", "#7f2704")
PNG_DIR = OUT_PATH.parent / "case_map"
# Include files merely named in tool output (ls/find listings, error paths)? In the
# aggregate maps those ~2400 extra mathlib files drown out the real reads.
PATH_SEEN = True
UNTOUCHED = "#ececec"
DIR_COLOR = "#d0d0d0"


def _mix(lo: str, hi: str, t: float) -> str:
    a = [int(lo[i:i + 2], 16) for i in (1, 3, 5)]
    b = [int(hi[i:i + 2], 16) for i in (1, 3, 5)]
    return "#" + "".join(f"{round(x + (y - x) * t):02x}" for x, y in zip(a, b))


def _shade(count: int, cmax: int, shades: tuple[str, str]) -> str:
    """Log-scaled so a few heavily-read files don't wash out the rest."""
    if count <= 0:
        return UNTOUCHED
    t = math.log1p(count) / math.log1p(cmax) if cmax > 0 else 1.0
    return _mix(*shades, 0.2 + 0.8 * t)  # floor: a single touch must stay visible


def _case_counts(d: dict, used_mode: str) -> tuple[dict[str, int], dict[str, int]]:
    """(read counts, used counts) for one cached (agent, case), `other:` dropped."""
    read = {k: n for k, n in d["read"].items()}
    seen = list(d.get("name_seen", [])) + (list(d.get("seen_paths", [])) if PATH_SEEN else [])
    for k in seen:
        read.setdefault(k, 1)

    if used_mode == "all":
        used = {k: 1 for k in d["used_all"]}
    else:
        counts = d.get("used_counts") or {}
        used = {k: max(1, counts.get(k, 1)) for k in d["used_explicit"]}

    def keep(k: str) -> bool:
        # real library sources only: no scaffolding, build output, or lake metadata
        rel = k.split(":", 1)[1]
        return (not k.startswith("other:") and rel.endswith(".lean")
                and not {"build", ".lake", "lake-packages"} & set(rel.split("/")))
    return ({k: n for k, n in read.items() if keep(k)},
            {k: n for k, n in used.items() if keep(k)})


def _under_lib(counts: dict[str, int]) -> dict[str, int]:
    """`lib:rel` -> `lib:lib/rel`, so each library is one root and loose top-level
    files (lakefile.lean, Spec.lean, ...) sit inside it instead of each becoming
    its own slice of the centre ring."""
    return {f"{k.split(':', 1)[0]}:{k.split(':', 1)[0]}/{k.split(':', 1)[1]}": n for k, n in counts.items()}


def _case_payload(read: dict[str, int], used: dict[str, int]) -> dict:
    read, used = _under_lib(read), _under_lib(used)
    files = set(read) | set(used)
    ids, labels, parents, values = build_tree(files)
    cmax = max([*read.values(), *used.values(), 1])
    is_file = files.__contains__

    def colors(counts: dict[str, int], shades: tuple[str, str]) -> list[str]:
        return [_shade(counts.get(i, 0), cmax, shades) if is_file(i) else DIR_COLOR for i in ids]

    both = set(read) & set(used)
    return {
        "ids": ids, "labels": labels, "parents": parents, "values": values,
        "read_colors": colors(read, READ_SHADES),
        "used_colors": colors(used, USED_SHADES),
        "read_counts": [read.get(i, 0) for i in ids],
        "used_counts": [used.get(i, 0) for i in ids],
        "n_read": len(read), "n_used": len(used), "n_both": len(both),
    }


def load(used_mode: str) -> dict[str, dict[str, dict]]:
    """{agent: {case: payload}} for every cached case that built successfully."""
    out: dict[str, dict[str, dict]] = {}
    for fpath in sorted(glob.glob(CACHE_GLOB)):
        agent, case = Path(fpath).stem.split("__", 1)
        d = json.loads(Path(fpath).read_text())
        if "error" in d:
            continue
        read, used = _case_counts(d, used_mode)
        payload = _case_payload(read, used)
        payload["verdict"] = d.get("verdict", "?")
        out.setdefault(agent, {})[case] = payload
    return out


_PAGE = """\
<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Read vs referenced files, per testcase</title>
<script src="https://cdn.plot.ly/plotly-2.32.0.min.js"></script>
<style>
body {{ font-family: -apple-system, "Segoe UI", sans-serif; background: #f5f5f5; margin: 0; padding: 20px; color: #1a1a1a; }}
h1 {{ font-size: 1.2rem; margin: 0 0 10px; }}
.controls {{ display: flex; gap: 12px; align-items: center; margin-bottom: 8px; }}
select {{ font-size: 0.9rem; padding: 3px 6px; }}
#stats {{ font-size: 0.85rem; margin-bottom: 10px; color: #333; }}
.grid {{ display: grid; grid-template-columns: 1fr 1fr; gap: 14px; }}
.panel {{ background: #fff; border: 1px solid #ddd; border-radius: 8px; padding: 10px 12px; }}
.panel h2 {{ font-size: 0.95rem; margin: 0 0 4px; }}
.sw {{ display: inline-block; width: 12px; height: 12px; border-radius: 2px; vertical-align: -1px; }}
</style>
</head>
<body>
<h1>Read vs referenced files</h1>
<div class="controls">
  <label>agent <select id="agent"></select></label>
  <label>testcase <select id="case"></select></label>
</div>
<div id="stats"></div>
<div class="grid">
  <div class="panel"><h2><span class="sw" style="background:{read_swatch}"></span> read (direct + indirect) &middot; darker = more reads</h2><div id="read" style="height:560px"></div></div>
  <div class="panel"><h2><span class="sw" style="background:{used_swatch}"></span> referenced by the proof &middot; darker = more references</h2><div id="used" style="height:560px"></div></div>
</div>
<script>
const DATA = {data};
const agentSel = document.getElementById("agent"), caseSel = document.getElementById("case");
Object.keys(DATA).forEach(a => agentSel.add(new Option(a, a)));

function fillCases() {{
  caseSel.innerHTML = "";
  Object.keys(DATA[agentSel.value]).forEach(c => {{
    const v = DATA[agentSel.value][c].verdict;
    caseSel.add(new Option(c + "  [" + v + "]", c));
  }});
}}

function draw(div, p, colors, counts, noun) {{
  Plotly.react(div, [{{
    type: "sunburst", ids: p.ids, labels: p.labels, parents: p.parents, values: p.values,
    branchvalues: "total", sort: false, insidetextorientation: "radial",
    marker: {{ colors: colors, line: {{ color: "#fff", width: 0.8 }} }},
    customdata: counts,
    hovertemplate: "%{{id}}<br>" + noun + ": %{{customdata}}<extra></extra>",
  }}], {{ margin: {{t: 4, b: 4, l: 4, r: 4}}, paper_bgcolor: "rgba(0,0,0,0)" }},
  {{ responsive: true, displayModeBar: false }});
}}

function render() {{
  const p = DATA[agentSel.value][caseSel.value];
  const pct = (a, b) => b ? Math.round(100 * a / b) + "%" : "–";
  document.getElementById("stats").innerHTML =
    "read <b>" + p.n_read + "</b> files &middot; referenced <b>" + p.n_used + "</b> &middot; both <b>" + p.n_both +
    "</b> &nbsp;|&nbsp; read &rarr; used: <b>" + pct(p.n_both, p.n_read) +
    "</b> &nbsp;|&nbsp; used &rarr; read: <b>" + pct(p.n_both, p.n_used) + "</b>";
  draw("read", p, p.read_colors, p.read_counts, "reads");
  draw("used", p, p.used_colors, p.used_counts, "references");
}}

agentSel.onchange = () => {{ fillCases(); render(); }};
caseSel.onchange = render;
fillCases(); render();
</script>
</body>
</html>
"""


def render(out_path: Path, used_mode: str) -> None:
    data = load(used_mode)
    if not data:
        sys.exit("no cached indirect_reads data; run: python3 analysis/run_agent_trace.py --only indirect")
    page = _PAGE.format(
        data=json.dumps(data, separators=(",", ":")),
        read_swatch=READ_SHADES[1],
        used_swatch=USED_SHADES[1],
    )
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_text(page, encoding="utf-8")
    n = sum(len(v) for v in data.values())
    print(f"-> {out_path} ({len(data)} agents, {n} cases)")


def _write_png(p: dict, name: str, path: Path, depth: int | None = None) -> None:
    """Draw one payload as read panel (left) + referenced panel (right)."""
    import plotly.graph_objects as go
    import plotly.io as pio

    tag = f" [{p['verdict']}]" if p.get("verdict") else ""
    traces = []
    for x0, colors, counts in (
        (0.0, p["read_colors"], p["read_counts"]),
        (0.5, p["used_colors"], p["used_counts"]),
    ):
        traces.append(go.Sunburst(
            ids=p["ids"], labels=p["labels"], parents=p["parents"], values=p["values"],
            branchvalues="total", sort=False, insidetextorientation="radial",
            marker=dict(colors=colors, line=dict(color="#fff", width=0.8)),
            textfont=dict(size=9), maxdepth=depth, domain=dict(x=[x0, x0 + 0.5], y=[0, 0.9]),
        ))
    fig = go.Figure(traces)
    pct = lambda a, b: f"{round(100 * a / b)}%" if b else "–"
    fig.update_layout(
        margin=dict(t=80, b=10, l=10, r=10), paper_bgcolor="#ffffff",
        font=dict(family="sans-serif"), showlegend=False,
        title=dict(
            text=(f"<b>{name}</b>{tag}<br>"
                  f"<sup>read {p['n_read']} · referenced {p['n_used']} · both {p['n_both']} · "
                  f"read→used {pct(p['n_both'], p['n_read'])} · used→read {pct(p['n_both'], p['n_used'])}</sup>"),
            x=0.5, xanchor="center", font=dict(size=15)),
        annotations=[
            dict(x=0.25, y=0.91, xref="paper", yref="paper", showarrow=False, yanchor="bottom",
                 text="<b>read</b> (direct + indirect) — darker = more", font=dict(size=12, color=READ_SHADES[1])),
            dict(x=0.75, y=0.91, xref="paper", yref="paper", showarrow=False, yanchor="bottom",
                 text="<b>referenced</b> by the proof — darker = more", font=dict(size=12, color=USED_SHADES[1])),
        ],
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    pio.write_image(fig, str(path), width=1800, height=950, scale=1)


GROUPS = {"aeneas": ("aeneas",), "mathlib_core": ("mathlib", "core")}


def aggregate(used_mode: str) -> dict[str, tuple[dict[str, int], dict[str, int]]]:
    """{agent: (read counts, used counts)} summed over all that agent's built cases."""
    sums: dict[str, tuple[dict, dict]] = {}
    for fpath in sorted(glob.glob(CACHE_GLOB)):
        agent, _case = Path(fpath).stem.split("__", 1)
        d = json.loads(Path(fpath).read_text())
        if "error" in d:
            continue
        read, used = _case_counts(d, used_mode)
        R, U = sums.setdefault(agent, ({}, {}))
        for tgt, src in ((R, read), (U, used)):
            for k, n in src.items():
                tgt[k] = tgt.get(k, 0) + n
    return sums


def _write_single(tree: tuple, colors: list[str], title: str, path: Path, depth: int | None) -> None:
    import plotly.graph_objects as go
    import plotly.io as pio

    ids, labels, parents, values = tree
    fig = go.Figure(go.Sunburst(
        ids=ids, labels=labels, parents=parents, values=values, branchvalues="total",
        sort=False, insidetextorientation="radial", maxdepth=depth, textfont=dict(size=9),
        marker=dict(colors=colors, line=dict(color="#fff", width=0.3)),
    ))
    fig.update_layout(
        margin=dict(t=60, b=10, l=10, r=10), paper_bgcolor="#ffffff", font=dict(family="sans-serif"),
        title=dict(text=title, x=0.5, xanchor="center", font=dict(size=15)),
    )
    path.parent.mkdir(parents=True, exist_ok=True)
    pio.write_image(fig, str(path), width=1400, height=1400, scale=1)


def render_aggregate_png(out_dir: Path, used_mode: str, agent: str | None, depth: int | None) -> None:
    """Per library group: one shared tree (union over all agents), so every agent's
    map is aligned; separate read / referenced PNGs, shades on one shared scale."""
    per_agent = aggregate(used_mode)
    for group, libs in GROUPS.items():
        keep = lambda k: k.split(":", 1)[0] in libs
        counts = {
            ag: (_under_lib({k: n for k, n in R.items() if keep(k)}),
                 _under_lib({k: n for k, n in U.items() if keep(k)}))
            for ag, (R, U) in per_agent.items()
        }
        files = {k for R, U in counts.values() for k in (*R, *U)}
        tree = build_tree(files)
        cmax = max((n for R, U in counts.values() for n in (*R.values(), *U.values())), default=1)
        for ag, (R, U) in counts.items():
            if agent and ag != agent:
                continue
            both = len(set(R) & set(U))
            for kind, c, shades in (("read", R, READ_SHADES), ("used", U, USED_SHADES)):
                colors = [(_shade(c.get(i, 0), cmax, shades) if i in files else DIR_COLOR) for i in tree[0]]
                title = (f"<b>{ag}</b> — {group} — {'read' if kind == 'read' else 'referenced'} files "
                         f"({len(c)} of {len(files)}; {both} in both) · darker = more")
                path = out_dir / group / f"{ag}_{kind}.png"
                _write_single(tree, colors, title, path, depth)
                print(f"-> {path}")


def render_png(out_dir: Path, used_mode: str, agent: str | None, case: str | None) -> None:
    """One PNG per (agent, case): read panel left, referenced panel right."""
    import plotly.graph_objects as go
    import plotly.io as pio

    n = 0
    for ag, cases in load(used_mode).items():
        if agent and ag != agent:
            continue
        for cs, p in cases.items():
            if case and cs != case:
                continue
            path = out_dir / ag / f"{cs}.png"
            _write_png(p, f"{ag} / {cs}", path)
            n += 1
    print(f"-> {out_dir} ({n} PNGs)")


def main() -> None:
    ap = argparse.ArgumentParser(description="Per-testcase read-vs-referenced file map")
    ap.add_argument("--out", default=str(OUT_PATH), help="output HTML path")
    ap.add_argument("--used", choices=("explicit", "all"), default="explicit",
                    help="explicit: premises the agent named; all: also automation-pulled premises")
    ap.add_argument("--png", action="store_true", help="write one PNG per (agent, case) instead of the HTML")
    ap.add_argument("--png-dir", default=str(PNG_DIR), help="PNG output directory")
    ap.add_argument("--aggregate", action="store_true", help="one PNG per agent, all testcases summed")
    ap.add_argument("--depth", type=int, help="aggregate PNG: max ring depth (default all)")
    ap.add_argument("--agent", help="PNG mode: only this agent")
    ap.add_argument("--case", help="PNG mode: only this testcase")
    ap.add_argument("--no-path-seen", action="store_true",
                    help="read = direct reads + #check/import-resolved names only (drop paths merely seen in tool output)")
    args = ap.parse_args()
    global PATH_SEEN
    PATH_SEEN = not (args.no_path_seen or args.aggregate)
    if args.aggregate:
        render_aggregate_png(Path(args.png_dir).parent / "aggregate", args.used, args.agent, args.depth)
    elif args.png:
        render_png(Path(args.png_dir), args.used, args.agent, args.case)
    else:
        render(Path(args.out), args.used)


if __name__ == "__main__":
    main()
