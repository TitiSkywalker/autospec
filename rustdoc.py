"""
Parser for rustdoc JSON output (format_version 57).

Usage:
    doc = RustDoc("target/doc/nalgebra.json")
    lu = doc.find_by_name("LU")[0]
    for trait, methods in doc.get_impls(lu.id):
        for name, doc_text in methods:
            print(f"[{trait or 'inherent'}] {name}: {doc_text}")
"""

import json
from dataclasses import dataclass, field
from typing import Any


# Data model
@dataclass
class RustItem:
    id: int
    crate_id: int
    name: str | None
    kind: str  # "struct", "enum", "trait", "function", "module", "assoc_type", etc.
    docs: str | None
    file: str | None
    visibility: str
    inner: dict[str, Any] = field(repr=False)

    def summary(self) -> str:
        label = self.name or "(anonymous)"
        return f"{label} [{self.kind}]"


class RustDoc:
    """Parsed rustdoc JSON for a single crate."""

    def __init__(self, path: str):
        with open(path) as f:
            raw: dict[str, Any] = json.load(f)

        self.format_version: int = raw["format_version"]
        self.crate_version: str = raw.get("crate_version", "?")
        self.includes_private: bool = raw.get("includes_private", False)

        self.root_id: int = raw["root"]
        self._index: dict[str, Any] = raw["index"]
        self._paths: dict[str, Any] = raw.get("paths", {})
        self._external_crates: dict[str, Any] = raw.get("external_crates", {})

        # Build typed items
        self.items: dict[int, RustItem] = {}
        for id_str, item in self._index.items():
            inner = item.get("inner", {})
            kind = list(inner.keys())[0] if inner else "unknown"
            self.items[int(id_str)] = RustItem(
                id=int(id_str),
                crate_id=item.get("crate_id", 0),
                name=item.get("name"),
                kind=kind,
                docs=item.get("docs"),
                file=item.get("span", {}).get("filename") if item.get("span") else None,
                visibility=item.get("visibility", "default"),
                inner=inner[kind] if kind in inner else {},
            )

    # ------------------------------------------------------------------
    # Lookup
    # ------------------------------------------------------------------

    def __getitem__(self, item_id: int) -> RustItem:
        return self.items[item_id]

    def root(self) -> RustItem:
        return self.items[self.root_id]

    def find_by_name(self, name: str) -> list[RustItem]:
        return [item for item in self.items.values() if item.name == name]

    def search(
        self, keyword: str, kinds: tuple[str, ...] | None = None
    ) -> list[RustItem]:
        results = []
        for item in self.items.values():
            if kinds and item.kind not in kinds:
                continue
            text = f"{item.name or ''} {item.docs or ''}".lower()
            if keyword.lower() in text:
                results.append(item)
        return results

    # ------------------------------------------------------------------
    # Traversal
    # ------------------------------------------------------------------

    def children(self, item_id: int) -> list[RustItem]:
        """Direct children of a module/struct/trait/enum."""
        item = self.items.get(item_id)
        if not item:
            return []
        child_ids = item.inner.get("items", [])
        return [
            self.items.get(int(cid)) for cid in child_ids if self.items.get(int(cid))
        ]

    def walk(self, start_id: int | None = None, depth: int = 0):
        """Recursively print the item tree from *start_id* (default: root)."""
        if start_id is None:
            start_id = self.root_id
        item = self.items.get(start_id)
        if not item:
            return
        print(f"{'  ' * depth}{item.summary()}")
        for child in self.children(start_id):
            self.walk(child.id, depth + 1)

    # ------------------------------------------------------------------
    # Impl blocks
    # ------------------------------------------------------------------

    def get_impls(
        self, for_item_id: int
    ) -> list[tuple[str | None, list[tuple[str | None, str | None]]]]:
        """Return all impl blocks (trait name, [(method_name, doc)]) for a given item id."""
        results: list[tuple[str | None, list[tuple[str | None, str | None]]]] = []

        for id_str, raw in self._index.items():
            inner = raw.get("inner", {})
            impl = inner.get("impl", {})
            if not impl:
                continue

            for_type = impl.get("for")
            if not self._type_matches(int(id_str), for_type, for_item_id):
                continue

            trait = impl.get("trait")
            trait_name = None
            if trait:
                tid = trait.get("resolved_path", {}).get("id") or trait.get("id")
                t_item = self.items.get(int(tid)) if tid is not None else None
                trait_name = t_item.name if t_item else None

            methods: list[tuple[str | None, str | None]] = []
            for mid in impl.get("items", []):
                m_item = self.items.get(int(mid))
                if m_item:
                    methods.append((m_item.name, m_item.docs))
            results.append((trait_name, methods))

        # Sort: inherent impls first, then trait impls
        results.sort(key=lambda x: (x[0] is not None, x[0] or ""))
        return results

    def _type_matches(self, impl_id: int, for_type: Any, target_id: int) -> bool:
        """Check if *for_type* (int, dict, list) resolves to *target_id*."""
        if isinstance(for_type, int):
            return for_type == target_id
        if isinstance(for_type, dict):
            for key in ("resolved_path", "generic", "dyn_trait", "impl_trait"):
                val = for_type.get(key)
                if val is not None:
                    if isinstance(val, dict) and val.get("id") == target_id:
                        return True
        return False

    # ------------------------------------------------------------------
    # Generics
    # ------------------------------------------------------------------

    def generics_of(self, item_id: int) -> list[dict[str, Any]]:
        """Return the generic parameters of an item."""
        item = self.items.get(item_id)
        if not item:
            return []
        return item.inner.get("generics", {}).get("params", [])

    def where_predicates_of(self, item_id: int) -> list[dict[str, Any]]:
        """Return the where-clause predicates of an item."""
        item = self.items.get(item_id)
        if not item:
            return []
        return item.inner.get("generics", {}).get("where_predicates", [])

    # ------------------------------------------------------------------
    # Fields (structs / enums)
    # ------------------------------------------------------------------

    def fields_of(self, item_id: int) -> list[dict[str, Any]]:
        """Return the fields of a struct or enum variant."""
        item = self.items.get(item_id)
        if not item:
            return []
        kind_data = item.inner
        # Struct fields
        if "fields" in kind_data:
            return kind_data["fields"]
        # Enum variants — return all variants with their fields
        if "variants" in kind_data:
            return kind_data["variants"]
        return []

    # ------------------------------------------------------------------
    # External crates
    # ------------------------------------------------------------------

    def external_crate_names(self) -> dict[int, str]:
        return {int(k): v["name"] for k, v in self._external_crates.items()}

    # ------------------------------------------------------------------
    # Statistics
    # ------------------------------------------------------------------

    def stats(self) -> dict[str, int]:
        counts: dict[str, int] = {}
        for item in self.items.values():
            counts[item.kind] = counts.get(item.kind, 0) + 1
        return dict(sorted(counts.items()))


# ---------------------------------------------------------------------------
# Quick CLI: python -m pipeline.doc <path-to-nalgebra.json>
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import sys

    path = sys.argv[1] if len(sys.argv) > 1 else "target/doc/nalgebra.json"
    doc = RustDoc(path)

    root = doc.root()
    print(f"Crate: {root.name} v{doc.crate_version}")
    print(f"Format version: {doc.format_version}")
    print(f"Items: {len(doc.items)}")
    print()

    print("--- Item kind counts ---")
    for kind, count in doc.stats().items():
        print(f"  {kind}: {count}")
    print()

    # Show some algorithm-related structs
    keywords = [
        "decomposition",
        "factorization",
        "qr",
        "svd",
        "lu",
        "cholesky",
        "bidiagonal",
        "hessenberg",
        "schur",
        "ldlt",
    ]
    print("--- Algorithm-related structs ---")
    seen = set()
    for item in doc.items.values():
        if item.kind != "struct" or not item.name:
            continue
        text = f"{item.name} {item.docs or ''}".lower()
        if any(kw in text for kw in keywords) and item.name not in seen:
            seen.add(item.name)
            print("SS")
            print(f"  {item.name}: {(item.docs or '')[:120]}")
            print("DD")
