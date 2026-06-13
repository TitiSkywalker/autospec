"""Static HTML report generator — cargo-doc-styled multi-page spec browser."""

import html
import json
import re
import shutil
import subprocess
from pathlib import Path

from pipeline.checks import _load_verify_results
from pipeline.report_assets import SELECTION_SCRIPT, SELECTION_TOOLBAR, STYLES

# ── Helpers ──────────────────────────────────────────────────────────────────


def _build_sig_and_source_lookups(
    project_path: str, crate_name: str
) -> tuple[dict[str, str], dict[str, str], dict[str, tuple[str, int]]]:
    """Build ``{key → signature}``, ``{key → source_code}``, and ``{key → (file, line)}``."""
    from rustdoc import RustDoc
    from tools.doc_json import ensure_rustdoc_json

    json_path = ensure_rustdoc_json(Path(project_path), crate_name)
    if not json_path:
        print("  Note: rustdoc JSON not found — signatures and sources omitted.")
        return {}, {}, {}

    try:
        doc_obj = RustDoc(str(json_path))
    except Exception as exc:
        print(
            f"  Note: cannot parse rustdoc JSON ({exc}) — signatures and sources omitted."
        )
        return {}, {}, {}

    sig_lookup: dict[str, str] = {}
    item_refs: list[tuple[str, "RustItem"]] = []

    for wi in doc_obj.walk_public():
        mp = wi.module_path
        if wi.item.kind == "function":
            name = wi.reexport_name or wi.item.name or ""
            key = f"{mp}::{name}" if mp else name
            if key:
                sig_lookup[key] = doc_obj.format_sig(wi.item)
                item_refs.append((key, wi.item))
        elif wi.item.kind in ("struct", "enum", "trait"):
            tname = wi.reexport_name or wi.item.name or ""
            for _trait_name, methods, is_external in doc_obj.get_impls(wi.item.id):
                if is_external:
                    continue
                for mname, _mdocs, mitem in methods:
                    if not mname or not mitem:
                        continue
                    key = f"{mp}::{tname}::{mname}" if mp else f"{tname}::{mname}"
                    sig_lookup[key] = doc_obj.format_sig(mitem)
                    item_refs.append((key, mitem))

    # Build source + file lookups using span data
    project_root = Path(project_path)
    file_cache: dict[str, list[str]] = {}  # filename → lines
    source_lookup: dict[str, str] = {}
    file_lookup: dict[str, tuple[str, int]] = {}

    for key, item in item_refs:
        if not item.file or not item.begin or not item.end:
            continue
        src_path = project_root / item.file
        if not src_path.is_file():
            continue
        # Record relative path + line for the source link
        file_lookup[key] = (item.file, item.begin[0])
        if item.file not in file_cache:
            file_cache[item.file] = src_path.read_text().split("\n")
        lines = file_cache[item.file]
        begin_line = item.begin[0] - 1  # 0-based
        end_line = item.end[0]  # exclusive
        if 0 <= begin_line < end_line <= len(lines):
            source_lookup[key] = "\n".join(lines[begin_line:end_line])

    return sig_lookup, source_lookup, file_lookup


def _find_rustdoc_assets_dir() -> Path | None:
    """Locate the rustdoc static.files directory from the active toolchain, or None."""
    try:
        result = subprocess.run(
            ["rustup", "which", "rustdoc"],
            capture_output=True,
            text=True,
            timeout=5,
        )
        if result.returncode != 0:
            return None
        toolchain_dir = Path(result.stdout.strip()).parent.parent
        assets_dir = toolchain_dir / "share" / "doc" / "rust" / "html" / "static.files"
        return assets_dir if assets_dir.is_dir() else None
    except Exception:
        return None


def _copy_rustdoc_assets(report_dir: Path) -> str | None:
    """Copy rustdoc CSS and font files to *report_dir*. Returns the CSS filename."""
    assets_dir = _find_rustdoc_assets_dir()
    if not assets_dir:
        return None
    for f in assets_dir.iterdir():
        if f.is_file():
            shutil.copy2(f, report_dir / f.name)
    matches = sorted(assets_dir.glob("rustdoc-*.css"))
    return matches[0].name if matches else None


def _badge(status: str) -> str:
    cls = {
        "PASS": "badge-pass",
        "FAIL": "badge-fail",
        "TIMEOUT": "badge-timeout",
    }.get(status, "badge-not-run")
    label = status if status in ("PASS", "FAIL", "TIMEOUT") else "not run"
    return f'<span class="verification-badge {cls}">{label}</span>'


def _make_fn_key(mod_name: str, merged_file: str, fn_name: str) -> str:
    """Build selection key: ``module::Type::function`` or ``module::function``."""
    clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
    if clean.endswith("__functions") or clean == "functions":
        if mod_name and mod_name != "(root)":
            return f"{mod_name}::{fn_name}"
        return fn_name
    type_name = clean.split("__")[-1]
    if mod_name and mod_name != "(root)":
        return f"{mod_name}::{type_name}::{fn_name}"
    return f"{type_name}::{fn_name}"


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
            node = node["_children"].setdefault(
                segment, {"_types": {}, "_children": {}}
            )
        # Last segment: merge types into it
        last = parts[-1]
        node["_children"].setdefault(last, {"_types": {}, "_children": {}})
        node["_children"][last]["_types"].update(types)
    return tree


def _extract_harness_code(source: str) -> str:
    """Extract the Rust code portion (after the header comment block)."""
    m = re.search(r"(#\[kani::proof\].*)", source, re.DOTALL)
    return m.group(1).strip() if m else source


def _make_page(
    crate_name: str, sidebar_html: str, body: str, css_link: str, title: str
) -> str:
    """Wrap content in the full HTML page chrome."""
    head_extra = f'<link rel="stylesheet" href="{css_link}">' if css_link else ""
    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>{title} — {html.escape(crate_name)}</title>
{head_extra}
<style>{STYLES}</style>
</head>
<body>
<nav class="sidebar">
{sidebar_html}
</nav>
<main class="main-content">
{body}
</main>
<div id="modal-overlay" class="modal-overlay" onclick="if(event.target===this)closeModal()">
  <div class="modal-box" onclick="event.stopPropagation()">
    <div class="modal-header">
      <h3>specs.toml</h3>
      <button class="modal-close" onclick="closeModal()">&times;</button>
    </div>
    <div class="modal-body">
      <p class="path-hint" id="modal-path-hint"></p>
      <textarea id="modal-textarea" readonly></textarea>
    </div>
    <div class="modal-footer">
      <button onclick="downloadToml()">Download file</button>
      <button class="primary" onclick="copyToml()">Copy to clipboard</button>
    </div>
  </div>
</div>
<div id="copy-toast" class="copy-toast">Copied to clipboard</div>
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
    is_sub = current_page != "index"
    rel = "../" if is_sub else ""  # prefix for index.html links
    page_dir = "" if is_sub else "pages/"  # prefix for type-page links
    parts.append(
        f'<h2 class="sidebar-crate">'
        f'<a href="{rel}index.html">{html.escape(crate_name)}</a></h2>'
    )

    def render_node(name: str, node: dict, depth: int, full_path: str) -> None:
        types: dict = node.get("_types", {})
        children: dict = node.get("_children", {})
        if not types and not children:
            return

        has_children = bool(children)
        if has_children or depth > 0:
            parts.append("<details open>")
            mod_anchor = html.escape(full_path) if full_path else "root"
            parts.append(
                f'<summary class="sidebar-mod">'
                f'<a href="{rel}index.html#mod-{mod_anchor}">'
                f"{html.escape(name)}</a>"
                f"</summary>"
            )
            parts.append("<ul>")
        elif depth == 0:
            parts.append("<ul>")

        # Types in this module
        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"{page_dir}{clean}.html"

            is_current = clean == current_page
            parts.append("<li>")
            if is_current:
                parts.append(
                    f'<span class="sidebar-type">{html.escape(type_label)}</span>'
                )
            else:
                parts.append(
                    f'<a class="sidebar-type" href="{page_file}">'
                    f"{html.escape(type_label)}</a>"
                )
            parts.append("<ul>")

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
                    f"fn {html.escape(fn_name)}</a> {badge}</li>"
                )

            parts.append("</ul>")
            parts.append("</li>")

        # Sub-modules (recurse)
        for child_name in sorted(children):
            child_full = f"{full_path}::{child_name}" if full_path else child_name
            render_node(child_name, children[child_name], depth + 1, child_full)

        if has_children or depth > 0:
            parts.append("</ul>")
            parts.append("</details>")
        elif depth == 0:
            parts.append("</ul>")

    render_node(crate_name, module_tree, 0, "")
    return "\n".join(parts)


# ── Function cards ───────────────────────────────────────────────────────────


def _build_fn_card(
    entry: dict,
    merged_file: str,
    mod_name: str,
    sig_lookup: dict[str, str],
    source_lookup: dict[str, str],
    file_lookup: dict[str, tuple[str, int]],
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

    # Signature + source lookup — use same key logic as _make_fn_key
    clean_mf = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
    sig_key = _make_fn_key(mod_name, clean_mf, fn_name)
    sig = sig_lookup.get(sig_key, "")
    orig_source = source_lookup.get(sig_key, "")
    src_file, src_line = file_lookup.get(sig_key, ("", 0))

    # Fallback: for re-exported types, try the first segment of merged_file
    # (e.g. "base" from "base__Matrix") as the module path.
    if not sig and not orig_source:
        mf_parts = clean_mf.split("__")
        if len(mf_parts) >= 2:
            alt_key = _make_fn_key(mf_parts[0], clean_mf, fn_name)
            sig = sig_lookup.get(alt_key, "")
            orig_source = source_lookup.get(alt_key, "")
            if not src_file:
                src_file, src_line = file_lookup.get(alt_key, ("", 0))
    # Fallback 2: bare function name (root-level re-exports)
    if not sig:
        sig = sig_lookup.get(fn_name, "")
    if not orig_source:
        orig_source = source_lookup.get(fn_name, "")
    if not src_file:
        src_file, src_line = file_lookup.get(fn_name, ("", 0))

    # Build source-file reference (click to copy)
    source_link = ""
    if src_file and src_line:
        display = f"{src_file}:{src_line}"
        source_link = (
            f' <span class="src-link" title="Click to copy path"'
            f' onclick="navigator.clipboard.writeText(\'{html.escape(display)}\')'
            f'.then(function(){{'
            f'var t=document.getElementById(\"copy-toast\");'
            f't.textContent=\'Copied: {html.escape(display)}\';'
            f't.classList.add(\"show\");'
            f'setTimeout(function(){{t.classList.remove(\"show\")}},2000)'
            f'}})">{html.escape(display)}</span>'
        )

    parts: list[str] = []
    parts.append(f'<section id="{anchor}" class="spec-card">')
    parts.append('<div class="spec-heading">')
    if is_api_only:
        fn_key = _make_fn_key(mod_name, clean_mf, fn_name)
        parts.append(
            f'<input type="checkbox" class="fn-checkbox" data-key="{html.escape(fn_key)}"'
            f" onchange=\"toggleFn(this.getAttribute('data-key'), this.checked)\">"
        )
    parts.append(f'<a class="fn" href="#{anchor}">fn {html.escape(fn_name)}</a>')
    parts.append(badge_html)
    parts.append(source_link)
    parts.append("</div>")
    if sig:
        parts.append(f'<div class="sig"><code>{html.escape(sig)}</code></div>')
    if desc:
        parts.append(f'<div class="docblock"><p>{html.escape(desc)}</p></div>')
    if orig_source and orig_source.strip():
        parts.append('<details class="source-details orig-source">')
        parts.append("<summary>Original source</summary>")
        parts.append(
            f'<pre><code class="language-rust">'
            f"{html.escape(orig_source)}</code></pre>"
        )
        parts.append("</details>")
    if harness_code:
        parts.append('<details class="source-details">')
        parts.append(f"<summary>Harness source ({html.escape(fn)})</summary>")
        parts.append(
            f'<pre><code class="language-rust">'
            f"{html.escape(harness_code)}</code></pre>"
        )
        parts.append("</details>")
    if error_text:
        parts.append('<details class="error-details">')
        parts.append("<summary>Kani verification output</summary>")
        parts.append(f'<pre class="error-output">{html.escape(error_text)}</pre>')
        parts.append("</details>")
    parts.append("</section>")
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
    file_lookup: dict[str, tuple[str, int]],
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

    sidebar = _build_sidebar(
        crate_name,
        module_tree,
        verify_lookup,
        current_page=clean,
        is_api_only=is_api_only,
    )

    # Breadcrumb: crate > module > type
    breadcrumb = (
        f'<div class="breadcrumbs">'
        f'<a href="../index.html">{html.escape(crate_name)}</a>'
        f' :: <a href="../index.html#mod-{html.escape(mod_name)}">{html.escape(mod_name)}</a>'
        f" :: {html.escape(type_label)}"
        f"</div>"
    )

    cards = []
    for entry in funcs:
        cards.append(
            _build_fn_card(
                entry,
                clean,
                mod_name,
                sig_lookup,
                source_lookup,
                file_lookup,
                spec_sources,
                verify_lookup,
                is_api_only=is_api_only,
            )
        )

    selection_ui = (SELECTION_TOOLBAR + "\n") if is_api_only else ""
    selection_js = SELECTION_SCRIPT if is_api_only else ""
    body = (
        f"<h1>{html.escape(type_label)}</h1>\n"
        f"{breadcrumb}\n"
        f"{selection_ui}"
        f'{"".join(cards)}\n'
        f"{selection_js}"
    )

    title = f"{type_label} — {crate_name}"
    return page_stem, _make_page(crate_name, sidebar, body, css_link, title)


def _build_index_page(
    crate_name: str,
    module_tree: dict,
    verify_lookup: dict[str, dict],
    css_link: str,
    passed: int,
    failed: int,
    timed_out: int,
    not_run: int,
    is_api_only: bool = False,
) -> str:
    """Generate the landing page (index.html) with summary and module overview."""

    sidebar = _build_sidebar(
        crate_name,
        module_tree,
        verify_lookup,
        current_page="index",
        is_api_only=is_api_only,
    )

    # Module/type overview list (render tree)
    overview: list[str] = []
    overview.append('<div class="index-module-list">')

    def _render_overview_node(
        name: str, node: dict, depth: int, full_path: str
    ) -> None:
        types: dict = node.get("_types", {})
        children: dict = node.get("_children", {})
        if not types and not children:
            return

        if depth > 0:
            mod_id = html.escape(full_path)
            overview.append(f'<details open id="mod-{mod_id}">')
            overview.append(
                f'<summary class="sidebar-mod">{html.escape(name)}</summary>'
            )
        overview.append("<ul>")

        for merged_file in sorted(types):
            funcs = types[merged_file]
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean
            page_file = f"pages/{clean}.html"
            if not is_api_only:
                p = sum(
                    1
                    for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status")
                    == "PASS"
                )
                f = sum(
                    1
                    for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status")
                    == "FAIL"
                )
                t = sum(
                    1
                    for e in funcs
                    if verify_lookup.get(e.get("file", "")[:-3], {}).get("status")
                    == "TIMEOUT"
                )
                badges = []
                if p:
                    badges.append(f"{p} passed")
                if f:
                    badges.append(f"{f} failed")
                if t:
                    badges.append(f"{t} timed out")
                status_str = f'<span class="fn-count">({", ".join(badges) if badges else "not run"})</span>'
            else:
                status_str = f'<span class="fn-count">({len(funcs)} fn{"s" if len(funcs) != 1 else ""})</span>'
            overview.append(
                f"<li>"
                f'<a href="{page_file}">{html.escape(type_label)}</a>'
                f"{status_str}"
                f"</li>"
            )

        # Sub-modules
        for child_name in sorted(children):
            child_full = f"{full_path}::{child_name}" if full_path else child_name
            _render_overview_node(
                child_name, children[child_name], depth + 1, child_full
            )

        overview.append("</ul>")
        if depth > 0:
            overview.append("</details>")

    _render_overview_node(crate_name, module_tree, 0, "")
    overview.append("</div>")

    if not is_api_only:
        summary = (
            f'<div class="summary-row">\n'
            f'  <div class="summary-card pass"><span class="count">{passed}</span>Passed</div>\n'
            f'  <div class="summary-card fail"><span class="count">{failed}</span>Failed</div>\n'
            f'  <div class="summary-card timeout"><span class="count">{timed_out}</span>Timed Out</div>\n'
            f'  <div class="summary-card notrun"><span class="count">{not_run}</span>Not Run</div>\n'
            f"</div>\n"
        )
        heading = f"<h1>Autospec Report — {html.escape(crate_name)}</h1>\n"
        title = f"Autospec Report — {crate_name}"
    else:
        summary = ""
        heading = f"<h1>API Overview — {html.escape(crate_name)}</h1>\n"
        title = f"API Overview — {crate_name}"

    selection_ui = SELECTION_TOOLBAR if is_api_only else ""
    selection_js = SELECTION_SCRIPT if is_api_only else ""
    body = heading + summary + selection_ui + "".join(overview) + selection_js
    return _make_page(crate_name, sidebar, body, css_link, title)


# ── Main entry point ─────────────────────────────────────────────────────────


def _collect_api_entries(
    project_path: str, crate_name: str | None = None
) -> tuple[str, list[dict]]:
    """Build spec-style entries from rustdoc JSON alone (no harnesses).

    Follows glob and named re-exports so the report matches docs.rs coverage.

    Returns (crate_name, entries) where each entry has:
    module, source_function, merged_file, description, file (empty string).
    """
    from rustdoc import RustDoc
    from tools.doc_json import _find_doc_json, ensure_rustdoc_json

    root = Path(project_path)
    import tomllib

    with open(root / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    effective_crate = crate_name or cargo.get("package", {}).get("name", root.name)

    json_path = _find_doc_json(root, effective_crate)
    if not json_path:
        is_workspace = "workspace" in cargo
        has_package = "package" in cargo and "name" in cargo.get("package", {})
        if is_workspace and not has_package and not crate_name:
            members = cargo.get("workspace", {}).get("members", [])
            print(
                f"  Workspace detected with members: {', '.join(members[:8])}"
                f"{'...' if len(members) > 8 else ''}"
            )
            print(f"  Use --crate to specify which member to analyze.")
            return effective_crate, []

        json_path = ensure_rustdoc_json(root, effective_crate)
        if not json_path:
            return effective_crate, []

    doc_obj = RustDoc(str(json_path))
    entries: list[dict] = []

    for wi in doc_obj.walk_public():
        mp = wi.module_path
        if wi.item.kind == "function":
            fn_name = wi.reexport_name or wi.item.name or ""
            mf = f"{mp}__functions" if mp else "functions"
            entries.append(
                {
                    "module": mp or "(root)",
                    "source_function": fn_name,
                    "merged_file": mf,
                    "file": "",
                    "description": (wi.item.docs or "").split("\n")[0].strip(),
                }
            )
        elif wi.item.kind in ("struct", "enum", "trait"):
            tname = wi.reexport_name or wi.item.name or ""
            mf = f"{mp}__{tname}" if mp else tname
            for _trait_name, methods, is_external in doc_obj.get_impls(wi.item.id):
                if is_external:
                    continue
                for mname, mdocs, mitem in methods:
                    if not mname or not mitem:
                        continue
                    entries.append(
                        {
                            "module": mp or "(root)",
                            "source_function": mname,
                            "merged_file": mf,
                            "file": "",
                            "description": (mdocs or "").split("\n")[0].strip(),
                        }
                    )

    return effective_crate, entries


def _generate_html_report(
    project_path: str, output_dir: str, crate_name: str | None = None
) -> None:
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
        crate_name, entries = _collect_api_entries(project_path, crate_name)
        if not entries:
            print(
                "Error: no public API found (and no specs.json). "
                "Generate rustdoc JSON first: cargo +nightly rustdoc -- -Zunstable-options --output-format json"
            )
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
    sig_lookup, source_lookup, file_lookup = _build_sig_and_source_lookups(project_path, crate_name)

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

    css_name = _copy_rustdoc_assets(report_dir)
    if css_name:
        index_css = css_name
        page_css = f"../{css_name}"
    else:
        print("  Note: rustdoc assets not found, using inline styles.")
        index_css = page_css = ""

    # 7. Compute summary stats (verification run only)
    if not api_only:
        passed = sum(1 for v in verify_lookup.values() if v.get("status") == "PASS")
        failed = sum(1 for v in verify_lookup.values() if v.get("status") == "FAIL")
        timed_out = sum(
            1 for v in verify_lookup.values() if v.get("status") == "TIMEOUT"
        )
        not_run = len(entries) - passed - failed - timed_out
    else:
        passed = failed = timed_out = 0
        not_run = 0

    # 7.5 Build module tree for sidebar + index navigation
    module_tree = _build_module_tree(grouped)

    # 8. Generate index page
    index_html = _build_index_page(
        crate_name,
        module_tree,
        verify_lookup,
        index_css,
        passed,
        failed,
        timed_out,
        not_run,
        is_api_only=api_only,
    )
    (report_dir / "index.html").write_text(index_html)
    print(f"Wrote {report_dir / 'index.html'}")

    # 9. Generate per-type pages under pages/
    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        for merged_file in sorted(types):
            funcs = types[merged_file]
            page_stem, page_html = _build_type_page(
                crate_name,
                mod_name,
                merged_file,
                funcs,
                module_tree,
                sig_lookup,
                source_lookup,
                file_lookup,
                spec_sources,
                verify_lookup,
                page_css,
                is_api_only=api_only,
            )
            (pages_dir / f"{page_stem}.html").write_text(page_html)
            print(f"Wrote {pages_dir / f'{page_stem}.html'}")
