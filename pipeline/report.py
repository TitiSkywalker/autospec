"""Generate structured reports: doc coverage → specs → Kani results."""

import json
import tomllib
from pathlib import Path

from rustdoc import RustDoc, RustItem


def _generate_report(
    project_path: str,
    output_dir: str,
    run_verify: bool = False,
    kani_timeout: int = 120,
    verify_counts: tuple[int, int, int] | None = None,
) -> None:
    """Generate a structured report: doc coverage → specs → Kani results.

    If *verify_counts* is provided as (passed, failed, timed_out), those
    values are used instead of re-running verification.
    """
    import json
    import tomllib

    from rustdoc import RustDoc, RustItem

    project = Path(project_path)
    output = Path(output_dir)

    with open(project / "Cargo.toml", "rb") as fh:
        cargo = tomllib.load(fh)
    crate_name = cargo.get("package", {}).get("name", project.name)

    # ── 1. Doc coverage from rustdoc JSON ──────────────────────────────
    from tools.doc_json import _find_doc_json

    json_path = _find_doc_json(project, crate_name)
    if not json_path:
        print("Error: No rustdoc JSON found. Run with --check or --check-only first.")
        return

    doc = RustDoc(str(json_path))

    # Walk modules and count documented public items
    total_documented = 0
    total_public = 0
    documented_by_kind: dict[str, int] = {}
    public_by_kind: dict[str, int] = {}
    documented_modules = 0
    total_modules = 0
    documented_methods = 0
    total_methods = 0

    def walk_module(item_id: int) -> None:
        nonlocal total_documented, total_public, documented_modules, total_modules
        nonlocal documented_methods, total_methods
        for child in doc.children(item_id):
            if child.visibility != "public":
                continue
            if child.kind == "module":
                total_modules += 1
                if child.docs:
                    documented_modules += 1
                walk_module(child.id)
            elif child.kind in (
                "struct",
                "enum",
                "trait",
                "function",
                "const",
                "type",
                "static",
                "union",
            ):
                total_public += 1
                kind = child.kind
                public_by_kind[kind] = public_by_kind.get(kind, 0) + 1
                if child.docs:
                    total_documented += 1
                    documented_by_kind[kind] = documented_by_kind.get(kind, 0) + 1
                # Count documented methods from impl blocks (skip external traits)
                for trait_name, methods, is_external in doc.get_impls(child.id):
                    if is_external:
                        continue
                    for mname, mdocs, _m_item in methods:
                        if mname is not None:
                            total_methods += 1
                            if mdocs:
                                documented_methods += 1

    walk_module(doc.root_id)

    # Source line counts
    src_dir = project / "src"
    source_lines = 0
    source_files = 0
    if src_dir.is_dir():
        for rs in sorted(src_dir.rglob("*.rs")):
            # Skip harness files injected by earlier runs
            if "autospec_harness" in rs.name:
                continue
            source_files += 1
            source_lines += len(rs.read_text().splitlines())

    # ── 2. Spec generation from specs.json ─────────────────────────────
    specs_json = output / "specs.json"
    spec_files_count = 0
    total_harnesses = 0
    spec_funcs: set[str] = set()
    spec_entries = []
    spec_status_ok = 0
    spec_status_failed = 0

    if specs_json.is_file():
        manifest = json.loads(specs_json.read_text())
        spec_entries = manifest.get("entries", [])
        spec_files_count = len(spec_entries)
        spec_status_ok = manifest.get("succeeded", 0)
        spec_status_failed = manifest.get("failed", 0)
        for entry in spec_entries:
            total_harnesses += 1
            func = entry.get("source_function", "")
            mod = entry.get("module", "")
            # Use merged_file to disambiguate same-named methods on different types
            merged = entry.get("merged_file", "")
            key = (
                f"{merged}::{func}"
                if merged and func
                else (f"{mod}::{func}" if func and mod else func)
            )
            if key:
                spec_funcs.add(key)

    # ── 3. Kani verification ──────────────────────────────────────────
    verify_passed = verify_failed = verify_timeout = -1  # -1 = not run
    if verify_counts is not None:
        verify_passed, verify_failed, verify_timeout = verify_counts
    elif run_verify and spec_files_count > 0:
        print("\n  Running Kani verification ...\n")
        _, verify_passed, verify_failed, verify_timeout = _verify_specs(
            project_path, output_dir, timeout=kani_timeout
        )

    # ── 4. Print report ────────────────────────────────────────────────
    type_docs = (
        documented_by_kind.get("struct", 0)
        + documented_by_kind.get("enum", 0)
        + documented_by_kind.get("trait", 0)
    )
    type_pub = (
        public_by_kind.get("struct", 0)
        + public_by_kind.get("enum", 0)
        + public_by_kind.get("trait", 0)
    )
    func_docs = documented_by_kind.get("function", 0)
    func_pub = public_by_kind.get("function", 0)

    total_documentable = total_documented + documented_methods
    total_pub = total_public + total_methods
    doc_pct = (total_documentable / total_pub * 100) if total_pub > 0 else 0.0

    # Coverage: only functions and methods produce harnesses, not types
    spec_targets = func_docs + documented_methods
    spec_cov_pct = (len(spec_funcs) / spec_targets * 100) if spec_targets > 0 else 0.0

    print()
    print("=" * 60)
    print(f"  Autospec Report — {crate_name}")
    print("=" * 60)

    print(f"\n── Documentation")
    print(f"  Modules with docs:  {documented_modules}/{total_modules}")
    print(f"  Documented items:   {total_documentable}/{total_pub}  ({doc_pct:.0f}%)")
    print(f"    Types:            {type_docs}/{type_pub}")
    print(f"    Functions:        {func_docs}/{func_pub}")
    print(f"    Methods:          {documented_methods}/{total_methods}")
    print(f"  Source files:       {source_files}  ({source_lines} lines)")

    print(f"\n── Specifications")
    print(f"  Spec files:         {spec_files_count}")
    print(f"  Total harnesses:    {total_harnesses}")
    print(
        f"  Coverage:           {len(spec_funcs)}/{spec_targets} functions & methods  ({spec_cov_pct:.0f}%)"
    )
    print(f"  Generation status:  {spec_status_ok} ok, {spec_status_failed} failed")

    if verify_passed >= 0:
        total_verified = verify_passed + verify_failed + verify_timeout
        verify_pct = (
            (verify_passed / total_verified * 100) if total_verified > 0 else 0.0
        )
        print(f"\n── Kani Verification")
        print(f"  Total harnesses:    {total_verified}")
        print(f"  Passed:             {verify_passed}")
        print(f"  Failed:             {verify_failed}")
        print(f"  Timed out:          {verify_timeout}")
        print(f"  Pass rate:          {verify_pct:.0f}%")
    else:
        print(f"\n── Kani Verification  (not run — use --verify to include)")

    print()
    print("=" * 60)
