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
    begin: tuple[int, int] | None  # (line, col) 1-based
    end: tuple[int, int] | None
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
                begin=tuple(item["span"]["begin"]) if "begin" in (item.get("span") or {}) else None,
                end=tuple(item["span"]["end"]) if "end" in (item.get("span") or {}) else None,
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
    ) -> list[tuple[str | None, list[tuple[str | None, str | None, Any]], bool]]:
        """Return all impl blocks (trait name, [(method_name, doc, item)], is_external).

        *is_external* is True when the trait belongs to a crate other than
        the root crate (e.g. Clone, Debug, From from std/core).  Inherent
        impls (trait_name is None) always have is_external=False.
        """
        root_crate = self.items[self.root_id].crate_id
        results: list[tuple[str | None, list[tuple[str | None, str | None, Any]], bool]] = []

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
            is_external = False
            if trait:
                tid = trait.get("resolved_path", {}).get("id") or trait.get("id")
                t_item = self.items.get(int(tid)) if tid is not None else None
                if t_item is not None:
                    trait_name = t_item.name
                    is_external = t_item.crate_id != root_crate
                else:
                    # Trait item not in index → definitely external
                    trait_name = trait.get("path")
                    is_external = True

            methods = []
            for mid in impl.get("items", []):
                m_item = self.items.get(int(mid))
                if m_item:
                    methods.append((m_item.name, m_item.docs, m_item))
            results.append((trait_name, methods, is_external))

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
        """Return the fields of a struct or enum variant.

        Struct fields are stored as separate items in the index, referenced
        by ID from the struct's ``inner.kind.plain.fields`` (or tuple/newtype
        variants). This method resolves those references into dicts with
        ``name`` and ``type`` keys suitable for ``format_type``.
        """
        item = self.items.get(item_id)
        if not item:
            return []
        inner = item.inner

        # Struct with named fields: inner.kind.plain.fields = [id, id, ...]
        if item.kind == "struct":
            kind_data = inner.get("kind", {})
            plain = kind_data.get("plain", {})
            field_ids = plain.get("fields", [])
            if field_ids:
                return self._resolve_fields(field_ids)
            # Tuple struct: inner.kind.tuple = [id, id, ...]
            tuple_ids = kind_data.get("tuple", [])
            if tuple_ids:
                return self._resolve_fields(tuple_ids)
            return []

        # Enum: inner.kind.variants = [{variant, fields: [id, ...]}, ...]
        if item.kind == "enum":
            kind_data = inner.get("kind", {})
            variants = kind_data.get("variants", [])
            result = []
            for v in variants:
                vresult = {"name": v.get("name", "?"), "inner": {}}
                vkind = list(v.get("inner", {}).keys())[0] if v.get("inner") else None
                if vkind == "tuple":
                    vresult["inner"] = {"tuple": self._resolve_fields(
                        [t.get("id", t) for t in v["inner"]["tuple"]]
                    )}
                elif vkind == "plain":
                    vresult["inner"] = {"struct": self._resolve_fields(
                        v["inner"]["plain"].get("fields", [])
                    )}
                result.append(vresult)
            return result

        return []

    def _resolve_fields(self, field_ids: list[int]) -> list[dict[str, Any]]:
        """Resolve struct_field item IDs to {name, type} dicts."""
        result = []
        for fid in field_ids:
            fitem = self.items.get(int(fid))
            if not fitem:
                continue
            entry: dict[str, Any] = {"name": fitem.name}
            # The type is the single key in fitem.inner (e.g. "primitive", "resolved_path", ...)
            for type_kind, type_data in fitem.inner.items():
                if type_kind in ("primitive", "resolved_path", "borrowed_ref",
                                 "generic", "tuple", "array", "slice",
                                 "raw_ptr", "qualified_path", "fn_ptr"):
                    entry["type"] = {type_kind: type_data}
                    break
            result.append(entry)
        return result

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

    # ------------------------------------------------------------------
    # Type formatting
    # ------------------------------------------------------------------

    def format_type(self, t: Any) -> str:
        """Render a rustdoc JSON type node as a Rust type string."""
        if isinstance(t, str):
            return t
        if "primitive" in t:
            return t["primitive"]
        if "generic" in t:
            return t["generic"]
        if "never" in t:
            return "!"
        if "unit" in t:
            return "()"
        if "borrowed_ref" in t:
            br = t["borrowed_ref"]
            inner = self.format_type(br["type"])
            lt = br.get("lifetime") or ""
            if br.get("is_mutable"):
                return f"&{lt}mut {inner}".replace("  ", " ") if lt else f"&mut {inner}"
            return f"&{lt} {inner}" if lt else f"&{inner}"
        if "resolved_path" in t:
            rp = t["resolved_path"]
            name = rp.get("path", "?")
            # Strip $crate:: prefix (internal rustdoc representation)
            if name.startswith("$crate::"):
                name = name[8:]
            args = rp.get("args")
            if args and "angle_bracketed" in args:
                gen_args = args["angle_bracketed"]["args"]
                parts = []
                for a in gen_args:
                    if "type" in a:
                        parts.append(self.format_type(a["type"]))
                    elif "lifetime" in a:
                        parts.append(a["lifetime"])
                    elif "const" in a:
                        parts.append(str(a["const"]))
                    elif "binding" in a:
                        b = a["binding"]
                        parts.append(f"{b['name']} = {self.format_type(b['type'])}")
                    else:
                        parts.append("?")
                return f"{name}<{', '.join(parts)}>"
            return name
        if "tuple" in t:
            elems = ", ".join(self.format_type(e) for e in t["tuple"])
            return f"({elems})"
        if "array" in t:
            return f"[{self.format_type(t['array']['type'])}; {t['array']['len']}]"
        if "slice" in t:
            return f"[{self.format_type(t['slice']['type'])}]"
        if "qualified_path" in t:
            qp = t["qualified_path"]
            ty = self.format_type(qp["self_type"])
            return f"<{ty} as {qp.get('trait_', {}).get('path', '?')}>::{qp['name']}"
        if "impl_trait" in t:
            bounds = ", ".join(
                b.get("path", b.get("name", "?"))
                for b in t["impl_trait"]
            )
            return f"impl {bounds}"
        if "dyn_trait" in t:
            bounds = ", ".join(
                b.get("path", b.get("name", "?"))
                for b in t["dyn_trait"]
            )
            return f"dyn {bounds}"
        if "raw_ptr" in t:
            inner = self.format_type(t["raw_ptr"]["type"])
            if t["raw_ptr"].get("is_mutable"):
                return f"*mut {inner}"
            return f"*const {inner}"
        if "fn_ptr" in t:
            inp = ", ".join(
                self.format_type(i["type"]) for i in t["fn_ptr"].get("inputs", [])
            )
            out = self.format_type(t["fn_ptr"]["output"])
            return f"fn({inp}) -> {out}"
        return "?"

    def format_sig(self, item: RustItem) -> str:
        """Format a function/method item's signature as a Rust string."""
        inner = item.inner
        # Method items have sig directly; standalone functions too
        sig = inner.get("sig", {})
        if not sig:
            return "fn ?()"

        inputs = sig.get("inputs", [])
        output = sig.get("output")

        params: list[str] = []
        for name, ty in inputs:
            # Skip self params — they're implicit
            if name == "self" and isinstance(ty, str) and ty == "&self":
                continue
            if name == "self" and isinstance(ty, dict):
                if "borrowed_ref" in ty:
                    if ty["borrowed_ref"].get("is_mutable"):
                        params.append("&mut self")
                    else:
                        params.append("&self")
                elif "raw_ptr" in ty:
                    if ty["raw_ptr"].get("is_mutable"):
                        params.append("self: *mut Self")
                    else:
                        params.append("self: *const Self")
                elif "generic" in ty:
                    params.append("self")
                else:
                    params.append(f"{name}: {self.format_type(ty)}")
            else:
                params.append(f"{name}: {self.format_type(ty)}")

        out_str = self.format_type(output) if output else "()"
        fn_name = item.name or ""
        return f"fn {fn_name}({', '.join(params)}) -> {out_str}"


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
