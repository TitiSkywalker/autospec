"""Spec target discovery — parse rustdoc output to find documented functions."""

import re
import sys
from pathlib import Path

import tools  # noqa: F401 — triggers tool registration
from tool_registry import set_context, dispatch

def _split_detail(detail: str) -> list[dict]:
    """Split read_rustdoc detail output into per-item sections.

    Each section starts with ``## kind: name`` and extends until the
    next ``## `` header or end of text. Only sections that actually
    contain doc comments (beyond the header line) are returned.
    """
    sections: list[dict] = []
    lines = detail.split("\n")

    current: dict | None = None  # {kind, name, start}

    for i, line in enumerate(lines):
        # Match any ## kind: name header as a section boundary
        m = re.match(r"^## (\w+): (.+)", line)
        if m:
            kind = m.group(1)
            if current is not None:
                text = "\n".join(lines[current["start"] : i])
                if _section_has_docs(text):
                    sections.append(
                        {"kind": current["kind"], "name": current["name"], "text": text}
                    )
            raw_name = m.group(2).strip()
            # Strip re-export suffixes: "(via path)", "(re-export)"
            clean_name = re.sub(r"\s*\(via .+\)", "", raw_name).strip()
            clean_name = re.sub(r"\s*\(re-export\)", "", clean_name).strip()
            current = {"kind": kind, "name": clean_name, "start": i}

    if current is not None:
        text = "\n".join(lines[current["start"] :])
        if _section_has_docs(text):
            sections.append(
                {"kind": current["kind"], "name": current["name"], "text": text}
            )

    return sections

def _section_has_docs(text: str) -> bool:
    """Check whether a detail section has real doc comments beyond the header."""
    for line in text.split("\n")[1:]:  # skip the ## header
        stripped = line.strip()
        # Real doc content is a non-empty line that isn't a sub-header or impl stub
        if stripped and not stripped.startswith("##") and not stripped.startswith("[impl") and not stripped.startswith("  fn "):
            return True
    return False

def _parse_functions_from_section(doc_section: str) -> list[dict]:
    """Parse individual documented functions from a doc section.

    Handles two formats:
    1. Type sections (struct/enum/trait): extracts ``fn name(...) -> T — doc``
       lines from ``[impl ...]`` blocks.
    2. Standalone function sections: ``## function: name`` headers followed by
       ``fn sig`` and doc text.

    Returns a list of dicts with keys: name, sig, doc, impl, module,
    type_kind, type_name, is_standalone.
    """
    mod_match = re.search(r"// Module: (.+)", doc_section)
    type_match = re.search(r"// Type: (\w+) (.+)", doc_section)
    is_standalone = "// Standalone functions" in doc_section

    module = mod_match.group(1).strip() if mod_match else ""
    type_kind = type_match.group(1) if type_match else ""
    type_name = type_match.group(2) if type_match else ""

    # Extract struct fields / enum variants so agents don't need read_file
    fields_text = ""
    fm = re.search(r"\nFields:\n((?:  .+\n)+)", doc_section)
    if fm:
        fields_text = fm.group(1).strip()

    functions: list[dict] = []

    if is_standalone:
        # Standalone functions: each has "## function: name" header,
        # followed by "fn sig" line and doc text.
        parts = re.split(r"\n## function: ", doc_section)
        for part in parts[1:]:  # skip header portion before first function
            lines = part.strip().split("\n")
            name = lines[0].strip()
            sig = ""
            doc_lines: list[str] = []
            for line in lines[1:]:
                if line.startswith("fn ") and not sig:
                    sig = line.strip()
                elif sig and line.strip() and not line.startswith("["):
                    doc_lines.append(line.strip())
                elif line.startswith("["):
                    break
            doc = " ".join(doc_lines).strip()
            if doc and sig and name:
                functions.append({
                    "name": name,
                    "sig": sig,
                    "doc": doc,
                    "impl": "standalone",
                    "module": module,
                    "type_kind": "",
                    "type_name": "",
                    "is_standalone": True,
                    "fields": "",
                })
    else:
        # Struct/enum/trait methods from impl blocks
        current_impl = "inherent"
        for line in doc_section.split("\n"):
            impl_m = re.match(r"^  \[impl (.+)\]$", line)
            if impl_m:
                current_impl = impl_m.group(1)
                continue

            # Return type may contain spaces: Result<T, E>, & Type, (A, B)
            fn_m = re.match(r"^    fn (\w+)\(([^)]*)\) -> (.+?) — (.+)$", line)
            if fn_m:
                name = fn_m.group(1)
                params = fn_m.group(2)
                ret = fn_m.group(3)
                doc = fn_m.group(4).strip()
                if not doc:
                    continue
                functions.append({
                    "name": name,
                    "sig": f"fn {name}({params}) -> {ret}",
                    "doc": doc,
                    "impl": current_impl,
                    "module": module,
                    "type_kind": type_kind,
                    "type_name": type_name,
                    "is_standalone": False,
                    "fields": fields_text,
                })

    return functions

# ——— Standard trait names whose properties are well-known ———
_STANDARD_TRAITS: set[str] = {
    "Clone", "Copy", "Debug", "Display", "Default",
    "PartialEq", "Eq", "PartialOrd", "Ord", "Hash",
    "Borrow", "BorrowMut", "AsRef", "AsMut", "Deref", "DerefMut",
    "From", "Into", "TryFrom", "TryInto",
    "Serialize", "Deserialize", "Zeroize", "ToOwned", "FromStr",
    "Iterator", "IntoIterator", "FromIterator", "DoubleEndedIterator",
    "ExactSizeIterator", "Index", "IndexMut",
    "Add", "Sub", "Mul", "Div", "Rem", "Neg", "Not",
    "AddAssign", "SubAssign", "MulAssign", "DivAssign",
    "BitAnd", "BitOr", "BitXor", "Shl", "Shr",
    "Sum", "Product", "FusedIterator",
    "Send", "Sync", "Unpin", "Sized",
    "ConditionallySelectable", "CtOption",
    "Any", "Freeze", "RefUnwindSafe", "UnwindSafe", "UnsafeUnpin",
    "CloneToUninit",
}


def _strip_standard_trait_impls(detail: str) -> str:
    """Remove impl blocks for standard traits from a detail section.

    Standard trait impls (Clone, Debug, Default, From, etc.) have
    well-known semantics and don't need specifications.
    """
    lines = detail.split("\n")
    result: list[str] = []
    i = 0

    while i < len(lines):
        line = lines[i]
        # Check if this is an impl block header
        m = re.match(r"^  \[impl (.+)\]$", line)
        if m:
            label = m.group(1)
            # Determine if this is a standard trait impl
            # Format: "impl TraitName for TypeName" or "impl TypeName"
            skip = False
            if " for " in label:
                trait_part = label.split(" for ")[0]
                if trait_part in _STANDARD_TRAITS:
                    skip = True

            if skip:
                # Skip lines until the next blank line or non-impl content
                i += 1
                while i < len(lines):
                    if lines[i].strip() == "":
                        break
                    if re.match(r"^  \[impl ", lines[i]):
                        break
                    if re.match(r"^## \w+:", lines[i]):
                        break
                    i += 1
                # Skip trailing blank lines
                while i < len(lines) and lines[i].strip() == "":
                    i += 1
                continue

        result.append(line)
        i += 1

    return "\n".join(result)

def _load_specs_toml(output_dir: str) -> list[str] | None:
    """Load the specs.toml selection file. Returns list of function paths or None."""
    toml_path = Path(output_dir) / "specs.toml"
    if not toml_path.is_file():
        return None
    try:
        import tomllib
        with open(toml_path, "rb") as f:
            data = tomllib.load(f)
        return data.get("verify", {}).get("functions", [])
    except Exception as e:
        print(f"  Warning: failed to parse specs.toml: {e}")
        return None


def _make_filter_key(func_info: dict) -> str:
    """Build a selection key from function info: ``module::Type::name`` or ``module::name``."""
    mod = func_info.get("module", "")
    type_name = func_info.get("type_name", "")
    name = func_info.get("name", "")
    if type_name:
        return f"{mod}::{type_name}::{name}" if mod else f"{type_name}::{name}"
    return f"{mod}::{name}" if mod else name


def _get_spec_targets(project_path: str, output_dir: str,
                      only_functions: list[str] | None = None) -> list[tuple[str, str]]:
    """Generate per-type spec targets from documented modules.

    Returns list of (safe_filename, doc_section) where doc_section is
    the full pre-parsed docs for one type or a group of standalone functions.
    """

    abs_project = str(Path(project_path).resolve())
    abs_output = str(Path(output_dir).resolve())
    set_context(abs_project, abs_output)

    # Generate JSON if needed
    result = dispatch("generate_doc_json", {"features": "all"})
    if result.startswith("Error:") and "already exists" not in result:
        print(f"Warning: generate_doc_json: {result}", file=sys.stderr)

    # Get module summary
    summary = dispatch("read_rustdoc", {})
    if summary.startswith("Error:"):
        print(f"Error: {summary}", file=sys.stderr)
        sys.exit(1)

    # Parse candidate modules from summary
    import re
    candidates: list[str] = []
    for line in summary.split("\n"):
        stripped = line.strip()
        if not stripped.startswith("📁"):
            continue
        m = re.match(r"📁\s+(\S+?)/?\s+\((\d+)/(\d+)\s+documented\)", stripped)
        if not m:
            continue
        mod_path = m.group(1)
        numerator = int(m.group(2))
        if numerator > 0:
            candidates.append(mod_path)

    # Build per-type targets from each candidate module
    targets: list[tuple[str, str]] = []

    for mod_path in candidates:
        if "test_vector" in mod_path.lower():
            print(f"  (skip) {mod_path} — test data")
            continue

        detail = dispatch("read_rustdoc", {"module": mod_path})
        if detail.startswith("Error:"):
            print(f"  (skip) {mod_path} — {detail[:80]}")
            continue

        # Strip standard trait impl blocks (Clone, Debug, From, etc.)
        detail = _strip_standard_trait_impls(detail)

        sections = _split_detail(detail)
        safe_mod = mod_path.replace("::", "_") or "crate_root"

        type_sections = [s for s in sections if s["kind"] in ("struct", "enum", "trait")]
        func_sections = [s for s in sections if s["kind"] == "function"]

        def _doc_functions(text: str) -> list[str]:
            """Extract names of documented functions from doc-section text."""
            # Format: "fn name(...) -> Type — doc" or "fn name(...) -> Type\n"
            return sorted(set(re.findall(r"fn (\w+)\([^)]*\) -> \S+", text)))

        for s in type_sections:
            name = s["name"]
            safe_filename = f"{safe_mod}__{name}.rs"
            expected = _doc_functions(s["text"])
            header = (
                f"// Module: {mod_path}\n"
                f"// Type: {s['kind']} {name}\n"
                f"// Expected-fns: {', '.join(expected) if expected else '(none)'}\n"
                f"// Kani proof harnesses generated from doc comments.\n\n"
            )
            targets.append((safe_filename, header + s["text"]))
            print(f"  (target) {safe_filename}")

        if func_sections:
            safe_filename = f"{safe_mod}__functions.rs"
            expected = _doc_functions("\n".join(s["text"] for s in func_sections))
            header = (
                f"// Module: {mod_path}\n"
                f"// Standalone functions\n"
                f"// Expected-fns: {', '.join(expected) if expected else '(none)'}\n"
                f"// Kani proof harnesses generated from doc comments.\n\n"
            )
            merged = header + "\n\n".join(s["text"] for s in func_sections)
            targets.append((safe_filename, merged))
            print(f"  (target) {safe_filename}  ({len(func_sections)} functions)")

    return targets
