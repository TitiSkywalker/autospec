"""Static HTML report generator — cargo-doc-styled per-function spec browser."""

import html
import json
import re
import shutil
import subprocess
from pathlib import Path

from pipeline.checks import _load_verify_results


def _build_sig_lookup(project_path: str, crate_name: str) -> dict[str, str]:
    """Build a lookup ``{module::function_name → signature}`` from rustdoc JSON.

    Returns an empty dict if the JSON is not found or can't be parsed.
    """
    from tools.doc_json import _find_doc_json
    from rustdoc import RustDoc

    json_path = _find_doc_json(Path(project_path), crate_name)
    if not json_path:
        print("  Note: rustdoc JSON not found — signatures omitted. "
              "Run generate_doc_json first.")
        return {}

    try:
        doc = RustDoc(str(json_path))
    except Exception as exc:
        print(f"  Note: cannot parse rustdoc JSON ({exc}) — signatures omitted.")
        return {}

    lookup: dict[str, str] = {}

    def walk(module_id: int, mod_path: str) -> None:
        for child in doc.children(module_id):
            if child.visibility != "public":
                continue

            # Standalone function
            if child.kind == "function":
                key = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                if key:
                    lookup[key] = doc.format_sig(child)

            # Type with methods
            elif child.kind in ("struct", "enum", "trait"):
                type_name = child.name or ""
                for trait_name, methods, is_external in doc.get_impls(child.id):
                    if is_external:
                        continue
                    for mname, _mdocs, mitem in methods:
                        if not mname or not mitem:
                            continue
                        # Key includes type to disambiguate same-named methods
                        lookup[f"{mod_path}::{type_name}::{mname}"] = doc.format_sig(mitem)

            # Recurse into submodules
            elif child.kind == "module":
                next_path = f"{mod_path}::{child.name}" if mod_path else (child.name or "")
                walk(child.id, next_path)

    walk(doc.root_id, "")
    return lookup


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


def _build_sidebar(
    crate_name: str,
    grouped: dict[str, dict[str, list[dict]]],
    verify_lookup: dict[str, dict],
) -> str:
    """Build sidebar <nav> HTML with collapsible module tree."""
    parts: list[str] = []
    parts.append(f'<h2 class="sidebar-crate">{html.escape(crate_name)}</h2>')

    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        parts.append('<details open>')
        parts.append(f'<summary class="sidebar-mod">{html.escape(mod_name)}</summary>')
        parts.append('<ul>')

        for merged_file in sorted(types):
            funcs = types[merged_file]
            # Extract type label from merged_file: geometry__Point.rs → Point
            clean = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
            type_label = clean.split("__")[-1] if "__" in clean else clean

            parts.append(f'<li>')
            parts.append(f'<span class="sidebar-type">{html.escape(type_label)}</span>')
            parts.append('<ul>')

            for entry in funcs:
                fn_name = entry["source_function"]
                anchor = f"fn-{merged_file}__{fn_name}"
                vkey = entry.get("file", "")
                if vkey.endswith(".rs"):
                    vkey = vkey[:-3]
                vr = verify_lookup.get(vkey, {})
                badge = _badge(vr.get("status", ""))
                parts.append(
                    f'<li><a href="#{anchor}">'
                    f'fn {html.escape(fn_name)}</a> {badge}</li>'
                )

            parts.append('</ul>')
            parts.append('</li>')

        parts.append('</ul>')
        parts.append('</details>')

    return "\n".join(parts)


def _extract_description(source: str) -> str:
    """Extract Description from a harness file's header comment block."""
    m = re.search(r"Description:\s*(.+?)(?:\*/|\n)", source)
    return m.group(1).strip() if m else ""


def _extract_harness_code(source: str) -> str:
    """Extract the Rust code portion (after the header comment block)."""
    # Find first #[kani::proof] or pub fn
    m = re.search(r"(#\[kani::proof\].*)", source, re.DOTALL)
    return m.group(1).strip() if m else source


def _generate_html_report(project_path: str, output_dir: str) -> None:
    """Generate a static HTML report with per-function spec content and verification status."""

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
    verify_data = _load_verify_results(output_dir)
    verify_lookup: dict[str, dict] = {}
    if verify_data:
        for r in verify_data.get("results", []):
            verify_lookup[r["source_file"]] = r

    # 2.5. Build function signature lookup from rustdoc JSON
    sig_lookup = _build_sig_lookup(project_path, crate_name)

    # 3. Group entries by module → merged_file
    grouped = _group_entries(entries)

    # 4. Read harness source code for each entry
    spec_sources: dict[str, str] = {}
    for entry in entries:
        fn = entry.get("file", "")
        path = output / fn
        if path.is_file():
            spec_sources[fn] = path.read_text()

    # 5. Copy rustdoc CSS to report directory
    report_dir = output / "report"
    report_dir.mkdir(parents=True, exist_ok=True)

    css_path = _find_rustdoc_css()
    if css_path:
        shutil.copy2(css_path, report_dir / "rustdoc.css")
        css_link = "rustdoc.css"
    else:
        print("  Note: rustdoc CSS not found, using inline styles.")
        css_link = ""

    # 6. Compute summary stats
    passed = sum(1 for v in verify_lookup.values() if v.get("status") == "PASS")
    failed = sum(1 for v in verify_lookup.values() if v.get("status") == "FAIL")
    timed_out = sum(1 for v in verify_lookup.values() if v.get("status") == "TIMEOUT")
    not_run = len(entries) - passed - failed - timed_out

    # 7. Build sidebar
    sidebar = _build_sidebar(crate_name, grouped, verify_lookup)

    # 8. Build function cards
    cards: list[str] = []
    for mod_name in sorted(grouped):
        types = grouped[mod_name]
        cards.append(f'<h2 class="section-header" id="mod-{html.escape(mod_name)}">'
                     f'{html.escape(mod_name)}</h2>')

        for merged_file in sorted(types):
            funcs = types[merged_file]

            for entry in funcs:
                fn_name = entry["source_function"]
                anchor = f"fn-{merged_file}__{fn_name}"
                desc = entry.get("description", "")
                fn = entry.get("file", "")
                source = spec_sources.get(fn, "")
                harness_code = _extract_harness_code(source)
                vkey = fn[:-3] if fn.endswith(".rs") else fn
                vr = verify_lookup.get(vkey, {})
                badge_html = _badge(vr.get("status", ""))

                # Look up signature from rustdoc JSON
                # For methods: key = module::TypeName::method
                # For standalone fns: key = module::function
                clean_mf = merged_file[:-3] if merged_file.endswith(".rs") else merged_file
                type_part = clean_mf.split("__")[-1] if "__" in clean_mf else ""
                if type_part and type_part != "functions":
                    sig_key = f"{mod_name}::{type_part}::{fn_name}"
                else:
                    sig_key = f"{mod_name}::{fn_name}"
                sig = sig_lookup.get(sig_key, "")

                cards.append(f'<section id="{anchor}" class="spec-card">')
                cards.append('<div class="spec-heading">')
                cards.append(f'<a class="fn">fn {html.escape(fn_name)}</a>')
                cards.append(badge_html)
                cards.append('</div>')
                if sig:
                    cards.append(f'<div class="sig"><code>{html.escape(sig)}</code></div>')

                if desc:
                    cards.append(f'<div class="docblock"><p>{html.escape(desc)}</p></div>')

                if harness_code:
                    cards.append('<details class="source-details">')
                    cards.append(f'<summary>Harness source ({html.escape(fn)})</summary>')
                    cards.append(f'<pre><code class="language-rust">'
                                 f'{html.escape(harness_code)}</code></pre>')
                    cards.append('</details>')

                # Show Kani error output for failed harnesses
                error_text = vr.get("error", "")
                if error_text:
                    cards.append('<details class="error-details">')
                    cards.append('<summary>Kani verification output</summary>')
                    cards.append(f'<pre class="error-output">'
                                 f'{html.escape(error_text)}</pre>')
                    cards.append('</details>')

                cards.append('</section>')

    # 9. Assemble full HTML page
    head_extra = f'<link rel="stylesheet" href="{css_link}">' if css_link else ""
    html_page = f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Autospec Report — {html.escape(crate_name)}</title>
{head_extra}
<style>
:root {{
  --badge-green: #2e7d32;
  --badge-red: #c62828;
  --badge-amber: #e65100;
  --badge-gray: #757575;
}}
body {{
  margin: 0;
  font-family: "Fira Sans", Arial, sans-serif;
  display: flex;
  min-height: 100vh;
}}
.sidebar {{
  width: 260px;
  flex-shrink: 0;
  background: #f5f5f5;
  padding: 16px;
  overflow-y: auto;
  border-right: 1px solid #e0e0e0;
  font-size: 14px;
}}
.sidebar h2 {{
  font-size: 16px;
  margin: 0 0 12px 0;
  color: #333;
}}
.sidebar details {{
  margin: 4px 0;
}}
.sidebar summary {{
  cursor: pointer;
  font-weight: 600;
  color: #555;
  padding: 2px 0;
}}
.sidebar ul {{
  list-style: none;
  padding-left: 16px;
  margin: 2px 0;
}}
.sidebar li {{
  padding: 1px 0;
}}
.sidebar a {{
  color: #c25c00;
  text-decoration: none;
}}
.sidebar a:hover {{
  text-decoration: underline;
}}
.sidebar-type {{
  font-size: 12px;
  color: #888;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}}
.main-content {{
  flex: 1;
  padding: 24px 40px;
  max-width: 960px;
}}
h1 {{
  font-size: 24px;
  border-bottom: 1px solid #ddd;
  padding-bottom: 8px;
  margin-bottom: 20px;
}}
.summary-row {{
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 12px;
  margin-bottom: 24px;
}}
.summary-card {{
  padding: 12px 16px;
  border-radius: 6px;
  text-align: center;
  font-size: 14px;
  color: white;
}}
.summary-card.pass {{ background: var(--badge-green); }}
.summary-card.fail {{ background: var(--badge-red); }}
.summary-card.timeout {{ background: var(--badge-amber); }}
.summary-card.notrun {{ background: var(--badge-gray); }}
.summary-card .count {{
  display: block;
  font-size: 28px;
  font-weight: 700;
}}
.section-header {{
  font-size: 18px;
  margin: 24px 0 12px 0;
  padding-bottom: 4px;
  border-bottom: 1px solid #eee;
  color: #333;
}}
.spec-card {{
  margin: 16px 0;
  padding: 12px 16px;
  border: 1px solid #e8e8e8;
  border-radius: 6px;
}}
.spec-heading {{
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 16px;
  font-weight: 600;
}}
a.fn {{
  color: #c25c00;
  font-family: "Fira Mono", monospace;
}}
.verification-badge {{
  display: inline-block;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.3px;
  text-transform: uppercase;
  color: white;
}}
.badge-pass  {{ background: var(--badge-green); }}
.badge-fail  {{ background: var(--badge-red); }}
.badge-timeout {{ background: var(--badge-amber); }}
.badge-not-run {{ background: var(--badge-gray); }}
.docblock {{
  margin: 8px 0 8px 24px;
  color: #555;
  font-style: italic;
}}
.sig {{
  margin: 4px 0 4px 24px;
  font-size: 13px;
  color: #666;
}}
.sig code {{
  font-family: "Fira Mono", monospace;
}}
.source-details {{
  margin-top: 8px;
}}
.source-details summary {{
  cursor: pointer;
  color: #1a73e8;
  font-size: 13px;
}}
.source-details pre {{
  background: #f8f8f8;
  border: 1px solid #e0e0e0;
  border-radius: 4px;
  padding: 14px;
  overflow-x: auto;
  font-size: 13px;
  line-height: 1.5;
}}
.error-details {{
  margin-top: 10px;
}}
.error-details summary {{
  cursor: pointer;
  color: #c62828;
  font-size: 13px;
  font-weight: 600;
}}
.error-output {{
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
}}
</style>
</head>
<body>
<nav class="sidebar">
{sidebar}
</nav>
<main class="main-content">
<h1>Autospec Report — {html.escape(crate_name)}</h1>
<div class="summary-row">
  <div class="summary-card pass"><span class="count">{passed}</span>Passed</div>
  <div class="summary-card fail"><span class="count">{failed}</span>Failed</div>
  <div class="summary-card timeout"><span class="count">{timed_out}</span>Timed Out</div>
  <div class="summary-card notrun"><span class="count">{not_run}</span>Not Run</div>
</div>
{"".join(cards)}
</main>
</body>
</html>"""

    index_path = report_dir / "index.html"
    index_path.write_text(html_page)
    print(f"Wrote HTML report → {index_path}")
