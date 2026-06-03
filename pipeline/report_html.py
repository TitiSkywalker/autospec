"""Static HTML report generator — cargo-doc-styled multi-page spec browser."""

import html
import json
import re
import shutil
import subprocess
from pathlib import Path

from pipeline.checks import _load_verify_results


# ── Shared CSS ───────────────────────────────────────────────────────────────

_STYLES = """
:root {
  --badge-green: #2e7d32;
  --badge-red: #c62828;
  --badge-amber: #e65100;
  --badge-gray: #757575;
}
body {
  margin: 0;
  font-family: "Fira Sans", Arial, sans-serif;
  display: flex;
  min-height: 100vh;
}
.sidebar {
  width: 260px;
  flex-shrink: 0;
  background: #f5f5f5;
  padding: 16px;
  overflow-y: auto;
  border-right: 1px solid #e0e0e0;
  font-size: 14px;
}
.sidebar h2 {
  font-size: 16px;
  margin: 0 0 12px 0;
  color: #333;
}
.sidebar details {
  margin: 4px 0;
}
.sidebar summary {
  cursor: pointer;
  font-weight: 600;
  color: #555;
  padding: 2px 0;
}
.sidebar ul {
  list-style: none;
  padding-left: 16px;
  margin: 2px 0;
}
.sidebar li {
  padding: 1px 0;
}
.sidebar a {
  color: #c25c00;
  text-decoration: none;
}
.sidebar a:hover {
  text-decoration: underline;
}
.sidebar-type {
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}
.main-content {
  flex: 1;
  padding: 24px 40px;
  max-width: 960px;
}
h1 {
  font-size: 24px;
  border-bottom: 1px solid #ddd;
  padding-bottom: 8px;
  margin-bottom: 20px;
}
.breadcrumbs {
  font-size: 14px;
  margin-bottom: 16px;
  color: #555;
}
.breadcrumbs a {
  color: #c25c00;
  text-decoration: none;
}
.breadcrumbs a:hover {
  text-decoration: underline;
}
.summary-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}
.summary-card {
  padding: 12px 16px;
  border-radius: 6px;
  text-align: center;
  font-size: 14px;
  color: white;
}
.summary-card.pass { background: var(--badge-green); }
.summary-card.fail { background: var(--badge-red); }
.summary-card.timeout { background: var(--badge-amber); }
.summary-card.notrun { background: var(--badge-gray); }
.summary-card .count {
  display: block;
  font-size: 28px;
  font-weight: 700;
}
.index-module-list {
  margin: 16px 0;
}
.index-module-list details {
  margin: 8px 0;
}
.index-module-list summary {
  cursor: pointer;
  font-size: 16px;
  font-weight: 600;
  color: #333;
  padding: 4px 0;
}
.index-module-list ul {
  list-style: none;
  padding-left: 24px;
  margin: 4px 0;
}
.index-module-list li {
  padding: 4px 0;
  font-size: 14px;
}
.index-module-list a {
  color: #c25c00;
  text-decoration: none;
}
.index-module-list a:hover {
  text-decoration: underline;
}
.index-module-list .fn-count {
  color: #888;
  font-size: 12px;
  margin-left: 6px;
}
.section-header {
  font-size: 18px;
  margin: 24px 0 12px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid #eee;
  color: #333;
}
.spec-card {
  margin: 16px 0;
  padding: 12px 16px;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
}
.spec-heading {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}
a.fn {
  color: #c25c00;
  font-family: "Fira Mono", monospace;
}
.verification-badge {
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  color: white;
}
.badge-pass  { background: var(--badge-green); }
.badge-fail  { background: var(--badge-red); }
.badge-timeout { background: var(--badge-amber); }
.badge-not-run { background: var(--badge-gray); }
.docblock {
  margin: 8px 0 8px 24px;
  color: #555;
  font-style: italic;
}
.sig {
  margin: 4px 0 4px 24px;
  font-size: 13px;
  color: #666;
}
.sig code {
  font-family: "Fira Mono", monospace;
}
.source-details {
  margin-top: 8px;
}
.source-details summary {
  cursor: pointer;
  color: #1a73e8;
  font-size: 13px;
}
.source-details pre {
  background: #f8f8f8;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 14px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
}
.orig-source summary {
  color: #388e3c;
}
.error-details {
  margin-top: 10px;
}
.error-details summary {
  cursor: pointer;
  color: #c62828;
  font-size: 13px;
  font-weight: 600;
}
.error-output {
  background: #fff5f5;
  border: 1px solid #ffcdd2;
  border-radius: 4px;
  padding: 12px;
  margin-top: 8px;
  overflow-x: auto;
  font-size: 12px;
  line-height: 1.5;
  max-height: 400px;
  overflow-y: auto;
  white-space: pre-wrap;
  word-break: break-all;
}
"""


# ── Helpers ──────────────────────────────────────────────────────────────────

def _build_sig_and_source_lookups(project_path: str, crate_name: str) -> tuple[dict[str, str], dict[str, str]]:
    """Build ``{module::function_name → signature}`` and ``{key → source_code}`` from rustdoc JSON."""
    from tools.doc_json import _find_doc_json
    from rustdoc import RustDoc, RustItem

    json_path = _find_doc_json(Path(project_path), crate_name)
    if not json_path:
        print("  Note: rustdoc JSON not found — signatures and sources omitted. "
              "Run generate_doc_json first.")
        return {}, {}

    try:
        doc_obj = RustDoc(str(json_path))
    except Exception as exc:
        print(f"  Note: cannot parse rustdoc JSON ({exc}) — signatures and sources omitted.")
        return {}, {}

    sig_lookup: dict[str, str] = {}
    # Collect (key, item) pairs during walk, extract source later
    item_refs: list[tuple[str, RustItem]] = []

    def walk(module_id: int, mod_path: str) -> None:
        for child in doc_obj.children(module_id):
            if child.visibility != "public":
                continue
            if child.kind == "function":
                key = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                if key:
                    sig_lookup[key] = doc_obj.format_sig(child)
                    item_refs.append((key, child))
            elif child.kind in ("struct", "enum", "trait"):
                type_name = child.name or ""
                for _trait_name, methods, is_external in doc_obj.get_impls(child.id):
                    if is_external:
                        continue
                    for mname, _mdocs, mitem in methods:
                        if not mname or not mitem:
                            continue
                        key = f"{mod_path}::{type_name}::{mname}"
                        sig_lookup[key] = doc_obj.format_sig(mitem)
                        item_refs.append((key, mitem))
            elif child.kind == "module":
                next_path = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                walk(child.id, next_path)

    walk(doc_obj.root_id, "")

    # Build source lookup using span data
    project_root = Path(project_path)
    file_cache: dict[str, list[str]] = {}  # filename → lines
    source_lookup: dict[str, str] = {}

    for key, item in item_refs:
        if not item.file or not item.begin or not item.end:
            continue
        src_path = project_root / item.file
        if not src_path.is_file():
            continue
        if item.file not in file_cache:
            file_cache[item.file] = src_path.read_text().split("\n")
        lines = file_cache[item.file]
        begin_line = item.begin[0] - 1  # 0-based
        end_line = item.end[0]  # exclusive
        if 0 <= begin_line < end_line <= len(lines):
            source_lookup[key] = "\n".join(lines[begin_line:end_line])

    return sig_lookup, source_lookup


def _find_rustdoc_css() -> str | None:
    """Locate rustdoc CSS from the active Rust toolchain, or None."""
    try:
        result = subprocess.run(
            ["rustup", "which", "rustdoc"],
            capture_output=True, text=True, timeout=5,
        )
        if result.returncode != 0:
            return None
        toolchain_dir = Path(result.stdout.strip()).parent.parent
        css_dir = toolchain_dir / "share" / "doc" / "rust" / "html" / "static.files"
        if not css_dir.is_dir():
            return None
        matches = sorted(css_dir.glob("rustdoc-*.css"))
        return str(matches[0]) if matches else None
    except Exception:
        return None


def _badge(status: str) -> str:
    cls = {
        "PASS": "badge-pass",
        "FAIL": "badge-fail",
        "TIMEOUT": "badge-timeout",
    }.get(status, "badge-not-run")
    label = status if status in ("PASS", "FAIL", "TIMEOUT") else "not run"
    return f'<span class="verification-badge {cls}">{label}</span>'


def _group_entries(entries: list[dict]) -> dict[str, dict[str, list[dict]]]:
    """Group spec entries by module, then by merged_file."""
    grouped: dict[str, dict[str, list[dict]]] = {}
    for e in entries:
        mod = e.get("module", "(unknown)")
        mf = e.get("merged_file") or mod
        grouped.setdefault(mod, {}).setdefault(mf, []).append(e)
    return grouped


def _extract_harness_code(source: str) -> str:
    """Extract the Rust code portion (after the header comment block)."""
    m = re.search(r"(#\[kani::proof\].*)", source, re.DOTALL)
    return m.group(1).strip() if m else source


def _make_page(crate_name: str, sidebar_html: str, body: str,
               css_link: str, title: str) -> str:
    """Wrap content in the full HTML page chrome."""
    head_extra = f'<link rel="stylesheet" href="{css_link}">' if css_link else ""
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — {html.escape(crate_name)}</title>
{head_extra}
<style>{_STYLES}</style>
</head>
<body>
<nav class="sidebar">
{sidebar_html}
</nav>
<main class="main-content">
{body}
</main>
</body>
</html>"""


# ── Sidebar ──────────────────────────────────────────────────────────────────

def _build_sidebar(
    crate_name: str,
    grouped: dict[str, dict[str, list[dict]]],
    verify_lookup: dict[str, dict],
    current_page: str,  # filename stem (e.g. "geometry__Point") or "index"
) -> str:
    """Build sidebar <nav> with collapsible module tree, linking across pages."""
    parts: list[str] = []
    parts.append(f'<h2 class="sidebar-crate">'
                 f'<a href="index.html">{html.escape(crate_name)}</a></h2>')

    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        is_current_mod = any(
            f"{mf[:-3] if mf.endswith('.rs') else mf}" == current_page
            for mf in types
        )
        parts.append('<details open>')
        parts.append(f'<summary class="sidebar-mod">{html.escape(mod_name)}</summary>')
        parts.append('<ul>')

        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"{clean}.html"

            is_current = (clean == current_page)
            parts.append(f'<li>')
            if is_current:
                parts.append(f'<span class="sidebar-type">{html.escape(type_label)}</span>')
            else:
                parts.append(f'<a class="sidebar-type" href="{page_file}">'
                             f'{html.escape(type_label)}</a>')
            parts.append('<ul>')

            for entry in funcs:
                fn_name = entry["source_function"]
                anchor = f"fn-{clean}__{fn_name}"
                vkey = entry.get("file", "")
                if vkey.endswith(".rs"):
                    vkey = vkey[:-3]
                vr = verify_lookup.get(vkey, {})
                badge = _badge(vr.get("status", ""))
                parts.append(
                    f'<li><a href="{page_file}#{anchor}">'
                    f'fn {html.escape(fn_name)}</a> {badge}</li>'
                )

            parts.append('</ul>')
            parts.append('</li>')

        parts.append('</ul>')
        parts.append('</details>')

    return "\n".join(parts)


# ── Function cards ───────────────────────────────────────────────────────────

def _build_fn_card(
    entry: dict,
    merged_file: str,
    mod_name: str,
    sig_lookup: dict[str, str],
    source_lookup: dict[str, str],
    spec_sources: dict[str, str],
    verify_lookup: dict[str, dict],
) -> str:
    """Build a single function card <section>."""
    fn_name = entry["source_function"]
    anchor = f"fn-{merged_file}__{fn_name}"
    desc = entry.get("description", "")
    fn = entry.get("file", "")
    source = spec_sources.get(fn, "")
    harness_code = _extract_harness_code(source)
    vkey = fn[:-3] if fn.endswith(".rs") else fn
    vr = verify_lookup.get(vkey, {})
    badge_html = _badge(vr.get("status", ""))

    # Signature + source lookup
    clean_mf = merged_file
    type_part = clean_mf.split("__")[-1] if "__" in clean_mf else ""
    if type_part and type_part != "functions":
        sig_key = f"{mod_name}::{type_part}::{fn_name}"
    else:
        sig_key = f"{mod_name}::{fn_name}"
    sig = sig_lookup.get(sig_key, "")
    orig_source = source_lookup.get(sig_key, "")

    parts: list[str] = []
    parts.append(f'<section id="{anchor}" class="spec-card">')
    parts.append('<div class="spec-heading">')
    parts.append(f'<a class="fn" href="#{anchor}">fn {html.escape(fn_name)}</a>')
    parts.append(badge_html)
    parts.append('</div>')
    if sig:
        parts.append(f'<div class="sig"><code>{html.escape(sig)}</code></div>')
    if desc:
        parts.append(f'<div class="docblock"><p>{html.escape(desc)}</p></div>')
    if orig_source and orig_source.strip():
        parts.append('<details class="source-details orig-source">')
        parts.append('<summary>Original source</summary>')
        parts.append(f'<pre><code class="language-rust">'
                     f'{html.escape(orig_source)}</code></pre>')
        parts.append('</details>')
    if harness_code:
        parts.append('<details class="source-details">')
        parts.append(f'<summary>Harness source ({html.escape(fn)})</summary>')
        parts.append(f'<pre><code class="language-rust">'
                     f'{html.escape(harness_code)}</code></pre>')
        parts.append('</details>')
    error_text = vr.get("error", "")
    if error_text:
        parts.append('<details class="error-details">')
        parts.append('<summary>Kani verification output</summary>')
        parts.append(f'<pre class="error-output">{html.escape(error_text)}</pre>')
        parts.append('</details>')
    parts.append('</section>')
    return "\n".join(parts)


# ── Page generators ──────────────────────────────────────────────────────────

def _build_type_page(
    crate_name: str,
    mod_name: str,
    merged_file: str,
    funcs: list[dict],
    grouped: dict[str, dict[str, list[dict]]],
    sig_lookup: dict[str, str],
    source_lookup: dict[str, str],
    spec_sources: dict[str, str],
    verify_lookup: dict[str, dict],
    css_link: str,
) -> tuple[str, str]:
    """Generate a page for one type / free-function group.

    Returns (page_stem, html).
    """
    clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
    type_label = clean.split("__")[-1] if "__" in clean else clean
    page_stem = clean

    sidebar = _build_sidebar(crate_name, grouped, verify_lookup, current_page=clean)

    # Breadcrumb: crate > module > type
    breadcrumb = (
        f'<div class="breadcrumbs">'
        f'<a href="index.html">{html.escape(crate_name)}</a>'
        f' :: <a href="index.html#mod-{html.escape(mod_name)}">{html.escape(mod_name)}</a>'
        f' :: {html.escape(type_label)}'
        f'</div>'
    )

    cards = []
    for entry in funcs:
        cards.append(_build_fn_card(entry, clean, mod_name, sig_lookup,
                                    source_lookup, spec_sources, verify_lookup))

    body = (f'<h1>{html.escape(type_label)}</h1>\n'
            f'{breadcrumb}\n'
            f'{"".join(cards)}')

    title = f"{type_label} — {crate_name}"
    return page_stem, _make_page(crate_name, sidebar, body, css_link, title)


def _build_index_page(
    crate_name: str,
    grouped: dict[str, dict[str, list[dict]]],
    verify_lookup: dict[str, dict],
    css_link: str,
    passed: int, failed: int, timed_out: int, not_run: int,
) -> str:
    """Generate the landing page (index.html) with summary and module overview."""

    sidebar = _build_sidebar(crate_name, grouped, verify_lookup, current_page="index")

    # Module/type overview list
    overview: list[str] = []
    overview.append('<div class="index-module-list">')
    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        overview.append(f'<details open id="mod-{html.escape(mod_name)}">')
        overview.append(f'<summary>{html.escape(mod_name)}</summary>')
        overview.append('<ul>')
        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"{clean}.html"
            # Count verification statuses
            p = sum(1 for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status") == "PASS")
            f = sum(1 for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status") == "FAIL")
            t = sum(1 for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status") == "TIMEOUT")
            badges = []
            if p: badges.append(f'{p} passed')
            if f: badges.append(f'{f} failed')
            if t: badges.append(f'{t} timed out')
            status_str = f'<span class="fn-count">({", ".join(badges) if badges else "not run"})</span>'
            overview.append(
                f'<li>'
                f'<a href="{page_file}">{html.escape(type_label)}</a>'
                f'{status_str}'
                f'</li>'
            )
        overview.append('</ul>')
        overview.append('</details>')
    overview.append('</div>')

    body = (f'<h1>Autospec Report — {html.escape(crate_name)}</h1>\n'
            f'<div class="summary-row">\n'
            f'  <div class="summary-card pass"><span class="count">{passed}</span>Passed</div>\n'
            f'  <div class="summary-card fail"><span class="count">{failed}</span>Failed</div>\n'
            f'  <div class="summary-card timeout"><span class="count">{timed_out}</span>Timed Out</div>\n'
            f'  <div class="summary-card notrun"><span class="count">{not_run}</span>Not Run</div>\n'
            f'</div>\n'
            f'{"".join(overview)}')

    title = f"Autospec Report — {crate_name}"
    return _make_page(crate_name, sidebar, body, css_link, title)


# ── Main entry point ─────────────────────────────────────────────────────────

def _generate_html_report(project_path: str, output_dir: str) -> None:
    """Generate a multi-page static HTML report like cargo doc.

    Writes to ``output/report/``:
    - ``index.html`` — landing page with summary and module tree
    - ``{module}__{Type}.html`` — one page per type / free-function group
    - ``rustdoc.css`` — copied from the active toolchain
    """

    output = Path(output_dir)

    # 1. Read specs.json
    specs_path = output / "specs.json"
    if not specs_path.is_file():
        print("Error: specs.json not found. Run generation or --check first.")
        return

    manifest = json.loads(specs_path.read_text())
    crate_name = manifest.get("crate", "unknown")
    entries: list[dict] = manifest.get("entries", [])

    if not entries:
        print("No entries in specs.json.")
        return

    # 2. Read verify_results.json (optional)
    verify_data = _load_verify_results(str(output))
    verify_lookup: dict[str, dict] = {}
    if verify_data:
        for r in verify_data.get("results", []):
            verify_lookup[r["source_file"]] = r

    # 3. Build function signature + source lookups from rustdoc JSON
    sig_lookup, source_lookup = _build_sig_and_source_lookups(project_path, crate_name)

    # 4. Group entries by module → merged_file
    grouped = _group_entries(entries)

    # 5. Read harness source code for each entry
    spec_sources: dict[str, str] = {}
    for entry in entries:
        fn = entry.get("file", "")
        path = output / fn
        if path.is_file():
            spec_sources[fn] = path.read_text()

    # 6. Copy rustdoc CSS to report directory
    report_dir = output / "report"
    report_dir.mkdir(parents=True, exist_ok=True)

    css_path = _find_rustdoc_css()
    if css_path:
        shutil.copy2(css_path, report_dir / "rustdoc.css")
        css_link = "rustdoc.css"
    else:
        print("  Note: rustdoc CSS not found, using inline styles.")
        css_link = ""

    # 7. Compute summary stats
    passed = sum(1 for v in verify_lookup.values() if v.get("status") == "PASS")
    failed = sum(1 for v in verify_lookup.values() if v.get("status") == "FAIL")
    timed_out = sum(1 for v in verify_lookup.values() if v.get("status") == "TIMEOUT")
    not_run = len(entries) - passed - failed - timed_out

    # 8. Generate index page
    index_html = _build_index_page(crate_name, grouped, verify_lookup,
                                   css_link, passed, failed, timed_out, not_run)
    (report_dir / "index.html").write_text(index_html)
    print(f"Wrote {report_dir / 'index.html'}")

    # 9. Generate per-type pages
    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        for merged_file in sorted(types):
            funcs = types[merged_file]
            page_stem, page_html = _build_type_page(
                crate_name, mod_name, merged_file, funcs,
                grouped, sig_lookup, source_lookup, spec_sources, verify_lookup, css_link,
            )
            (report_dir / f"{page_stem}.html").write_text(page_html)
            print(f"Wrote {report_dir / f'{page_stem}.html'}")
