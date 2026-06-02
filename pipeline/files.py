"""File management — merge individual harness files and write manifests."""

import json
import re
from pathlib import Path

def _merge_specs(output_dir: str) -> list[str]:
    """Merge per-function harness files into per-type bundled files.

    Reads ``output_dir/*.rs``, groups by ``{module}__{type}`` prefix
    (everything before the last ``__`` in the stem), and writes
    combined files to ``output_dir/merged/``.

    Idempotent — safe to call on already-merged output.
    """

    output = Path(output_dir)
    harness_files = sorted(output.glob("*.rs"))
    if not harness_files:
        return []

    # Group by merge key: stem before the last "__"
    groups: dict[str, list[Path]] = {}
    for fp in harness_files:
        stem = fp.stem
        if "__" not in stem:
            continue
        last_dd = stem.rfind("__")
        merge_key = stem[:last_dd]
        groups.setdefault(merge_key, []).append(fp)

    if not groups:
        return []

    merged_dir = output / "merged"
    merged_dir.mkdir(parents=True, exist_ok=True)
    merged_files: list[str] = []

    for merge_key, files in sorted(groups.items()):
        contents: list[str] = []
        module = ""
        type_kind = ""
        type_name = ""
        is_standalone = False
        func_names: list[str] = []

        for i, fp in enumerate(files):
            text = fp.read_text(encoding="utf-8")
            contents.append(text.strip())

            # Extract metadata from first file
            if i == 0:
                mod_match = re.search(r"// Module: (.+)", text)
                if mod_match:
                    module = mod_match.group(1).strip()
                type_match = re.search(r"// Type: (\w+) (.+)", text)
                if type_match:
                    type_kind = type_match.group(1)
                    type_name = type_match.group(2)
                if "// Standalone functions" in text:
                    is_standalone = True

            spec_match = re.search(r"// Spec-for: (.+)", text)
            if spec_match:
                func_names.append(spec_match.group(1).strip())

        # Reconstruct header
        header_lines = []
        if module:
            header_lines.append(f"// Module: {module}")
        if is_standalone:
            header_lines.append(f"// Standalone functions")
        elif type_kind and type_name:
            header_lines.append(f"// Type: {type_kind} {type_name}")
        header_lines.append(
            f"// Expected-fns: {', '.join(func_names) if func_names else '(none)'}"
        )
        header_lines.append("// Kani proof harnesses generated from doc comments.")
        header_lines.append("")

        combined = "\n".join(header_lines) + "\n" + "\n\n".join(contents) + "\n"

        out_path = merged_dir / f"{merge_key}.rs"
        out_path.write_text(combined, encoding="utf-8")
        merged_files.append(merge_key)

    print(f"  Merged {len(harness_files)} harness files → {len(merged_files)} type files")
    return sorted(merged_files)

def _write_spec_json(
    output_dir: str,
    crate_name: str,
    func_results: list[tuple[str, str, str, bool]],
) -> None:
    """Write a structured specs.json manifest with per-function entries."""

    entries: list[dict] = []
    for indiv_filename, merged_filename, func_name, ok in func_results:
        indiv_path = Path(output_dir) / indiv_filename
        size = indiv_path.stat().st_size if indiv_path.is_file() else 0

        desc = ""
        mod_path = ""
        if indiv_path.is_file():
            content = indiv_path.read_text()
            desc_match = re.search(r"Description: (.+)", content)
            if desc_match:
                desc = desc_match.group(1).strip()
            mod_match = re.search(r"// Module: (.+)", content)
            if mod_match:
                mod_path = mod_match.group(1).strip()

        entries.append({
            "file": indiv_filename,
            "merged_file": merged_filename,
            "source_function": func_name,
            "module": mod_path,
            "description": desc,
            "size_bytes": size,
            "status": "ok" if ok else "failed",
        })

    manifest = {
        "crate": crate_name,
        "total_harnesses": len(func_results),
        "succeeded": sum(1 for _, _, _, ok in func_results if ok),
        "failed": sum(1 for _, _, _, ok in func_results if not ok),
        "entries": entries,
    }

    json_path = Path(output_dir) / "specs.json"
    json_path.write_text(json.dumps(manifest, indent=2) + "\n")
    print(f"Wrote specs.json → {json_path}")
