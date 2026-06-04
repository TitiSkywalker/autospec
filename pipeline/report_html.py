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
    item_refs: list[tuple[str, RustItem]] = []
    seen_modules: set[int] = set()

    def _record_type_methods(item: RustItem, mod_path: str) -> None:
        type_name = item.name or ""
        for _trait_name, methods, is_external in doc_obj.get_impls(item.id):
            if is_external:
                continue
            for mname, _mdocs, mitem in methods:
                if not mname or not mitem:
                    continue
                key = f"{mod_path}::{type_name}::{mname}"
                sig_lookup[key] = doc_obj.format_sig(mitem)
                item_refs.append((key, mitem))

    def _process_use(child: RustItem, mod_path: str) -> None:
        """Follow re-exports to capture sigs and sources."""
        resolved_id = child.inner.get("id")
        if resolved_id is None or resolved_id not in doc_obj.items:
            return
        resolved = doc_obj.items[resolved_id]

        if child.inner.get("is_glob"):
            if resolved.kind != "module" or resolved.id in seen_modules:
                return
            seen_modules.add(resolved.id)
            # Use the resolved module's path so keys match _collect_api_entries
            glob_mod_path = (f"{mod_path}::{resolved.name}" if mod_path
                            else (resolved.name or ""))
            for rc in doc_obj.children(resolved.id):
                if rc.visibility != "public":
                    continue
                if rc.kind == "function":
                    key = f"{glob_mod_path}::{rc.name}" if glob_mod_path else (rc.name or "")
                    if key:
                        sig_lookup[key] = doc_obj.format_sig(rc)
                        item_refs.append((key, rc))
                elif rc.kind in ("struct", "enum", "trait"):
                    _record_type_methods(rc, glob_mod_path)
                elif rc.kind == "use":
                    _process_use(rc, glob_mod_path)
        else:
            if resolved.kind == "function":
                key = f"{mod_path}::{child.inner.get('name', resolved.name)}" if mod_path else (child.inner.get("name") or resolved.name or "")
                if key:
                    sig_lookup[key] = doc_obj.format_sig(resolved)
                    item_refs.append((key, resolved))
            elif resolved.kind in ("struct", "enum", "trait"):
                tname = child.inner.get("name") or resolved.name
                for _trait_name, methods, is_external in doc_obj.get_impls(resolved.id):
                    if is_external:
                        continue
                    for mname, _mdocs, mitem in methods:
                        if not mname or not mitem:
                            continue
                        key = f"{mod_path}::{tname}::{mname}" if mod_path else f"{tname}::{mname}"
                        sig_lookup[key] = doc_obj.format_sig(mitem)
                        item_refs.append((key, mitem))

    def walk(module_id: int, mod_path: str) -> None:
        seen_modules.add(module_id)
        for child in doc_obj.children(module_id):
            if child.visibility != "public":
                continue
            if child.kind == "function":
                key = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                if key:
                    sig_lookup[key] = doc_obj.format_sig(child)
                    item_refs.append((key, child))
            elif child.kind in ("struct", "enum", "trait"):
                _record_type_methods(child, mod_path)
            elif child.kind == "module":
                next_path = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                walk(child.id, next_path)
            elif child.kind == "use":
                _process_use(child, mod_path)

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


def _build_module_tree(grouped: dict[str, dict[str, list[dict]]]) -> dict:
    """Convert flat module→types dict into a nested tree.

    Returns a dict with ``_types`` (direct children) and ``_children``
    (submodule name → subtree) so that ``linalg::svd`` nests under ``linalg``.
    """
    tree: dict = {"_types": {}, "_children": {}}

    for module_path, types in sorted(grouped.items()):
        if module_path == "(root)":
            # Root-level items go directly into tree
            tree["_types"].update(types)
            continue

        parts = module_path.split("::")
        # Walk down the tree for each segment except the last
        node = tree
        for segment in parts[:-1]:
            node = node["_children"].setdefault(segment, {"_types": {}, "_children": {}})
        # Last segment: merge types into it
        last = parts[-1]
        node["_children"].setdefault(last, {"_types": {}, "_children": {}})
        node["_children"][last]["_types"].update(types)
    return tree


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
    module_tree: dict,
    verify_lookup: dict[str, dict],
    current_page: str,
    is_api_only: bool = False,
) -> str:
    """Build sidebar <nav> with collapsible module tree, linking across pages."""
    parts: list[str] = []
    parts.append(f'<h2 class="sidebar-crate">'
                 f'<a href="index.html">{html.escape(crate_name)}</a></h2>')

    def render_node(name: str, node: dict, depth: int, full_path: str) -> None:
        types: dict = node.get("_types", {})
        children: dict = node.get("_children", {})
        if not types and not children:
            return

        has_children = bool(children)
        if has_children or depth > 0:
            parts.append('<details open>')
            # Heading: module name links to index anchor
            mod_anchor = html.escape(full_path) if full_path else "root"
            parts.append(
                f'<summary class="sidebar-mod">'
                f'<a href="index.html#mod-{mod_anchor}">'
                f'{html.escape(name)}</a>'
                f'</summary>'
            )
            parts.append('<ul>')
        elif depth == 0:
            # Root-level types/functions
            parts.append('<ul>')

        # Types in this module
        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"pages/{clean}.html"

            is_current = (clean == current_page)
            parts.append('<li>')
            if is_current:
                parts.append(f'<span class="sidebar-type">{html.escape(type_label)}</span>')
            else:
                parts.append(f'<a class="sidebar-type" href="{page_file}">'
                             f'{html.escape(type_label)}</a>')
            parts.append('<ul>')

            for entry in funcs:
                fn_name = entry["source_function"]
                anchor = f"fn-{clean}__{fn_name}"
                badge = ""
                if not is_api_only:
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

        # Sub-modules (recurse)
        for child_name in sorted(children):
            child_full = f"{full_path}::{child_name}" if full_path else child_name
            render_node(child_name, children[child_name], depth + 1, child_full)

        if has_children or depth > 0:
            parts.append('</ul>')
            parts.append('</details>')
        elif depth == 0:
            parts.append('</ul>')

    render_node(crate_name, module_tree, 0, "")
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
    is_api_only: bool = False,
) -> str:
    """Build a single function card <section>."""
    fn_name = entry["source_function"]
    anchor = f"fn-{merged_file}__{fn_name}"
    desc = entry.get("description", "")
    fn = entry.get("file", "")
    harness_code = ""
    badge_html = ""
    error_text = ""
    if not is_api_only:
        source = spec_sources.get(fn, "")
        harness_code = _extract_harness_code(source)
        vkey = fn[:-3] if fn.endswith(".rs") else fn
        vr = verify_lookup.get(vkey, {})
        badge_html = _badge(vr.get("status", ""))
        error_text = vr.get("error", "")

    # Signature + source lookup
    clean_mf = merged_file
    type_part = clean_mf.split("__")[-1] if "__" in clean_mf else ""
    if type_part and type_part != "functions":
        sig_key = f"{mod_name}::{type_part}::{fn_name}"
    else:
        sig_key = f"{mod_name}::{fn_name}"
    sig = sig_lookup.get(sig_key, "")
    orig_source = source_lookup.get(sig_key, "")

    # Fallback: for re-exported types, the merged_file's first segment
    # (e.g. "base" from "base__Matrix") may be the original module path.
    if not sig and not orig_source:
        mf_parts = merged_file.split("__")
        if len(mf_parts) >= 2:
            alt_mod = mf_parts[0]
            if type_part and type_part != "functions":
                alt_key = f"{alt_mod}::{type_part}::{fn_name}"
            else:
                alt_key = f"{alt_mod}::{fn_name}"
            sig = sig_lookup.get(alt_key, "")
            orig_source = source_lookup.get(alt_key, "")
    # Fallback 2: bare function name (root-level re-exports)
    if not sig:
        sig = sig_lookup.get(fn_name, "")
    if not orig_source:
        orig_source = source_lookup.get(fn_name, "")

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
    module_tree: dict,
    sig_lookup: dict[str, str],
    source_lookup: dict[str, str],
    spec_sources: dict[str, str],
    verify_lookup: dict[str, dict],
    css_link: str,
    is_api_only: bool = False,
) -> tuple[str, str]:
    """Generate a page for one type / free-function group.

    Returns (page_stem, html).
    """
    clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
    type_label = clean.split("__")[-1] if "__" in clean else clean
    page_stem = clean

    sidebar = _build_sidebar(crate_name, module_tree, verify_lookup,
                             current_page=clean, is_api_only=is_api_only)

    # Breadcrumb: crate > module > type
    breadcrumb = (
        f'<div class="breadcrumbs">'
        f'<a href="../index.html">{html.escape(crate_name)}</a>'
        f' :: <a href="../index.html#mod-{html.escape(mod_name)}">{html.escape(mod_name)}</a>'
        f' :: {html.escape(type_label)}'
        f'</div>'
    )

    cards = []
    for entry in funcs:
        cards.append(_build_fn_card(entry, clean, mod_name, sig_lookup,
                                    source_lookup, spec_sources, verify_lookup,
                                    is_api_only=is_api_only))

    body = (f'<h1>{html.escape(type_label)}</h1>\n'
            f'{breadcrumb}\n'
            f'{"".join(cards)}')

    title = f"{type_label} — {crate_name}"
    return page_stem, _make_page(crate_name, sidebar, body, css_link, title)


def _build_index_page(
    crate_name: str,
    module_tree: dict,
    verify_lookup: dict[str, dict],
    css_link: str,
    passed: int, failed: int, timed_out: int, not_run: int,
    is_api_only: bool = False,
) -> str:
    """Generate the landing page (index.html) with summary and module overview."""

    sidebar = _build_sidebar(crate_name, module_tree, verify_lookup,
                             current_page="index", is_api_only=is_api_only)

    # Module/type overview list (render tree)
    overview: list[str] = []
    overview.append('<div class="index-module-list">')

    def _render_overview_node(name: str, node: dict, depth: int, full_path: str) -> None:
        types: dict = node.get("_types", {})
        children: dict = node.get("_children", {})
        if not types and not children:
            return

        if depth > 0:
            mod_id = html.escape(full_path)
            overview.append(f'<details open id="mod-{mod_id}">')
            overview.append(f'<summary class="sidebar-mod">{html.escape(name)}</summary>')
        overview.append('<ul>')

        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"pages/{clean}.html"
            if not is_api_only:
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
            else:
                status_str = f'<span class="fn-count">({len(funcs)} fn{"s" if len(funcs) != 1 else ""})</span>'
            overview.append(
                f'<li>'
                f'<a href="{page_file}">{html.escape(type_label)}</a>'
                f'{status_str}'
                f'</li>'
            )

        # Sub-modules
        for child_name in sorted(children):
            child_full = f"{full_path}::{child_name}" if full_path else child_name
            _render_overview_node(child_name, children[child_name], depth + 1, child_full)

        overview.append('</ul>')
        if depth > 0:
            overview.append('</details>')

    _render_overview_node(crate_name, module_tree, 0, "")
    overview.append('</div>')

    if not is_api_only:
        summary = (f'<div class="summary-row">\n'
                   f'  <div class="summary-card pass"><span class="count">{passed}</span>Passed</div>\n'
                   f'  <div class="summary-card fail"><span class="count">{failed}</span>Failed</div>\n'
                   f'  <div class="summary-card timeout"><span class="count">{timed_out}</span>Timed Out</div>\n'
                   f'  <div class="summary-card notrun"><span class="count">{not_run}</span>Not Run</div>\n'
                   f'</div>\n')
        heading = f'<h1>Autospec Report — {html.escape(crate_name)}</h1>\n'
        title = f"Autospec Report — {crate_name}"
    else:
        summary = ""
        heading = f'<h1>API Overview — {html.escape(crate_name)}</h1>\n'
        title = f"API Overview — {crate_name}"

    body = heading + summary + "".join(overview)
    return _make_page(crate_name, sidebar, body, css_link, title)


# ── Main entry point ─────────────────────────────────────────────────────────

def _collect_api_entries(project_path: str) -> tuple[str, list[dict]]:
    """Build spec-style entries from rustdoc JSON alone (no harnesses).

    Follows glob and named re-exports so the report matches docs.rs coverage.

    Returns (crate_name, entries) where each entry has:
    module, source_function, merged_file, description, file (empty string).
    """
    from tools.doc_json import _find_doc_json
    from rustdoc import RustDoc, RustItem

    root = Path(project_path)
    import tomllib
    with open(root / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", root.name)

    json_path = _find_doc_json(root, crate_name)
    if not json_path:
        return crate_name, []

    doc_obj = RustDoc(str(json_path))
    entries: list[dict] = []
    seen_modules: set[int] = set()  # prevent infinite recursion via glob re-exports

    def _add_fn_entry(child: RustItem, mod_path: str) -> None:
        mf = f"{mod_path}__functions" if mod_path else "functions"
        entries.append({
            "module": mod_path or "(root)",
            "source_function": child.name,
            "merged_file": mf,
            "file": "",
            "description": (child.docs or "").split("\n")[0].strip(),
        })

    def _add_type_entries(child: RustItem, mod_path: str, type_name: str | None = None) -> None:
        """Collect methods for a type (struct/enum/trait) and add entries."""
        tname = type_name or child.name or ""
        mf = f"{mod_path}__{tname}" if mod_path else tname
        for _trait_name, methods, is_external in doc_obj.get_impls(child.id):
            if is_external:
                continue
            for mname, mdocs, mitem in methods:
                if not mname or not mitem:
                    continue
                entries.append({
                    "module": mod_path or "(root)",
                    "source_function": mname,
                    "merged_file": mf,
                    "file": "",
                    "description": (mdocs or "").split("\n")[0].strip(),
                })

    def _process_use(child: RustItem, mod_path: str) -> None:
        """Handle a `use` (re-export) item — glob or named."""
        resolved_id = child.inner.get("id")
        if resolved_id is None or resolved_id not in doc_obj.items:
            return
        resolved = doc_obj.items[resolved_id]

        if child.inner.get("is_glob"):
            # Glob re-export: follow the resolved module's items.
            # Use the resolved module's path so keys match the sig lookup.
            if resolved.kind != "module":
                return
            if resolved.id in seen_modules:
                return
            seen_modules.add(resolved.id)
            glob_path = (f"{mod_path}::{resolved.name}" if mod_path
                         else (resolved.name or ""))
            for rc in doc_obj.children(resolved.id):
                if rc.visibility != "public":
                    continue
                if rc.kind == "function":
                    _add_fn_entry(rc, glob_path)
                elif rc.kind in ("struct", "enum", "trait"):
                    _add_type_entries(rc, glob_path)
                elif rc.kind == "use":
                    _process_use(rc, glob_path)
        else:
            # Named re-export: resolve and process the single item
            if resolved.kind == "function":
                _add_fn_entry(resolved, mod_path)
            elif resolved.kind in ("struct", "enum", "trait"):
                _add_type_entries(resolved, mod_path, type_name=child.inner.get("name") or resolved.name)
            elif resolved.kind == "module":
                # Skip named module re-exports — they're aliases
                pass

    def walk(module_id: int, mod_path: str) -> None:
        seen_modules.add(module_id)
        for child in doc_obj.children(module_id):
            if child.visibility != "public":
                continue
            if child.kind == "function":
                _add_fn_entry(child, mod_path)
            elif child.kind in ("struct", "enum", "trait"):
                _add_type_entries(child, mod_path)
            elif child.kind == "module":
                next_path = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                walk(child.id, next_path)
            elif child.kind == "use":
                _process_use(child, mod_path)

    walk(doc_obj.root_id, "")
    return crate_name, entries


def _generate_html_report(project_path: str, output_dir: str) -> None:
    """Generate a multi-page static HTML report like cargo doc.

    If ``specs.json`` exists, shows per-function spec content and verification
    status.  Otherwise falls back to an API-only report from rustdoc JSON.

    Writes to ``output/report/``:
    - ``index.html`` — landing page with summary and module tree
    - ``{module}__{Type}.html`` — one page per type / free-function group
    - ``rustdoc.css`` — copied from the active toolchain
    """

    output = Path(output_dir)
    report_dir = output / "report"
    report_dir.mkdir(parents=True, exist_ok=True)

    # 1. Read specs.json (optional)
    specs_path = output / "specs.json"
    api_only = not specs_path.is_file()

    if api_only:
        crate_name, entries = _collect_api_entries(project_path)
        if not entries:
            print("Error: no public API found (and no specs.json). "
                  "Generate rustdoc JSON first: cargo +nightly rustdoc -- -Zunstable-options --output-format json")
            return
        print(f"  No specs.json — API-only report ({len(entries)} public functions)")
    else:
        manifest = json.loads(specs_path.read_text())
        crate_name = manifest.get("crate", "unknown")
        entries: list[dict] = manifest.get("entries", [])

    if not entries:
        print("No entries found.")
        return

    # 2. Read verify_results.json (optional; only meaningful with specs)
    verify_lookup: dict[str, dict] = {}
    if not api_only:
        verify_data = _load_verify_results(str(output))
        if verify_data:
            for r in verify_data.get("results", []):
                verify_lookup[r["source_file"]] = r

    # 3. Build function signature + source lookups from rustdoc JSON
    sig_lookup, source_lookup = _build_sig_and_source_lookups(project_path, crate_name)

    # 4. Group entries by module → merged_file
    grouped = _group_entries(entries)

    # 5. Read harness source code for each entry
    spec_sources: dict[str, str] = {}
    if not api_only:
        for entry in entries:
            fn = entry.get("file", "")
            path = output / fn
            if path.is_file():
                spec_sources[fn] = path.read_text()

    # 6. Copy rustdoc CSS to report directory
    pages_dir = report_dir / "pages"
    pages_dir.mkdir(parents=True, exist_ok=True)

    css_path = _find_rustdoc_css()
    if css_path:
        shutil.copy2(css_path, report_dir / "rustdoc.css")
        index_css = "rustdoc.css"
        page_css = "../rustdoc.css"
    else:
        print("  Note: rustdoc CSS not found, using inline styles.")
        index_css = page_css = ""

    # 7. Compute summary stats (verification run only)
    if not api_only:
        passed = sum(1 for v in verify_lookup.values() if v.get("status") == "PASS")
        failed = sum(1 for v in verify_lookup.values() if v.get("status") == "FAIL")
        timed_out = sum(1 for v in verify_lookup.values() if v.get("status") == "TIMEOUT")
        not_run = len(entries) - passed - failed - timed_out
    else:
        passed = failed = timed_out = 0
        not_run = 0

    # 7.5 Build module tree for sidebar + index navigation
    module_tree = _build_module_tree(grouped)

    # 8. Generate index page
    index_html = _build_index_page(crate_name, module_tree, verify_lookup,
                                   index_css, passed, failed, timed_out, not_run,
                                   is_api_only=api_only)
    (report_dir / "index.html").write_text(index_html)
    print(f"Wrote {report_dir / 'index.html'}")

    # 9. Generate per-type pages under pages/
    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        for merged_file in sorted(types):
            funcs = types[merged_file]
            page_stem, page_html = _build_type_page(
                crate_name, mod_name, merged_file, funcs,
                module_tree, sig_lookup, source_lookup, spec_sources, verify_lookup, page_css,
                is_api_only=api_only,
            )
            (pages_dir / f"{page_stem}.html").write_text(page_html)
            print(f"Wrote {pages_dir / f'{page_stem}.html'}")
