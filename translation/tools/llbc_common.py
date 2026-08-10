#!/usr/bin/env python3
"""
Shared utilities for LLBC JSON manipulation.

Used by ``f64_adapter.py`` and ``i64_adapter.py`` to avoid code duplication.
"""

from __future__ import annotations

import json
import re
from typing import Any

# ── Type helpers ─────────────────────────────────────────────────────────────


def _is_target_ty(ty: dict | None, type_ids: set) -> bool:
    """Check whether a type node references any of the given dedup IDs."""
    if isinstance(ty, dict):
        if "Deduplicated" in ty:
            return ty["Deduplicated"] in type_ids
        if "HashConsedValue" in ty:
            return ty["HashConsedValue"][0] in type_ids
    return False


# ── Type discovery ───────────────────────────────────────────────────────────


def _discover_bool_id(raw_json: str) -> int | None:
    """Find the dedup ID for the Bool literal type.

    Scans for::

        "HashConsedValue":[390,{"Literal":"Bool"}]

    This is more reliable than looking at function return locals, which
    may have type ``Result<Bool>`` rather than ``Bool`` itself.
    """
    m = re.search(
        r'"HashConsedValue"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Literal"\s*:\s*"Bool"\s*\}',
        raw_json,
    )
    if m:
        return int(m.group(1))
    return None


def _count_dedup_in_locals(translated: dict) -> dict[int, int]:
    """Count occurrences of each Dedup ID in function locals."""
    counts: dict[int, int] = {}
    for fd in translated.get("fun_decls") or []:
        if fd is None or fd.get("body") is None:
            continue
        body = fd["body"]
        if not isinstance(body, dict) or "Structured" not in body:
            continue
        for loc in body["Structured"].get("locals", {}).get("locals") or []:
            ty = loc.get("ty")
            if isinstance(ty, dict) and "Deduplicated" in ty:
                did = ty["Deduplicated"]
                counts[did] = counts.get(did, 0) + 1
    return counts


# ── Function declaration builder ─────────────────────────────────────────────


def _make_fun_decl(
    def_id: int,
    crate: str,
    name: str,
    inputs: list[dict],
    output: dict,
    template: dict | None = None,
) -> dict:
    """Build a Foreign function declaration.

    If template is provided (an existing Foreign function decl from the LLBC),
    copies its exact format for fields like span, attr_info, generics, etc.
    """
    if template is not None:
        fd = json.loads(json.dumps(template))  # deep copy
        fd["def_id"] = def_id
        fd["item_meta"]["name"] = [
            {"Ident": [crate, 0]},
            {"Ident": [name, 0]},
        ]
        fd["item_meta"]["span"]["data"]["file_id"] = 0
        fd["item_meta"]["lang_item"] = None
        fd["signature"]["inputs"] = inputs
        fd["signature"]["output"] = output
        fd["body"] = "Opaque"
        return fd

    return {
        "def_id": def_id,
        "item_meta": {
            "name": [
                {"Ident": [crate, 0]},
                {"Ident": [name, 0]},
            ],
            "span": {
                "data": {
                    "file_id": 0,
                    "beg": {"line": 0, "col": 0},
                    "end": {"line": 0, "col": 0},
                },
                "generated_from_span": None,
            },
            "source_text": None,
            "attr_info": {
                "attributes": [],
                "inline": None,
                "rename": None,
                "public": True,
            },
            "is_local": False,
            "opacity": "Foreign",
            "lang_item": None,
        },
        "generics": {
            "regions": [],
            "types": [],
            "const_generics": [],
            "trait_clauses": [],
            "regions_outlive": [],
            "types_outlive": [],
            "trait_type_constraints": [],
        },
        "signature": {
            "is_unsafe": False,
            "abi": "Rust",
            "is_variadic": False,
            "inputs": inputs,
            "output": output,
        },
        "src": "TopLevel",
        "is_global_initializer": None,
        "body": "Opaque",
    }


# ── Statement builders ────────────────────────────────────────────────────────


def _call_stmt(func_id: int, args: list, dest: dict, span: dict | None = None) -> dict:
    if span is None:
        span = _zero_span()
    return {
        "span": span,
        "id": 0,
        "kind": {
            "Call": {
                "call": {
                    "func": {
                        "Regular": {
                            "kind": {"Fun": {"Regular": func_id}},
                            "generics": {
                                "regions": [],
                                "types": [],
                                "const_generics": [],
                                "trait_refs": [],
                            },
                        }
                    },
                    "args": args,
                    "dest": dest,
                },
                "on_unwind": {"span": span, "statements": []},
            }
        },
        "comments_before": [],
    }


# ── Switch condition transformation ───────────────────────────────────────────


def _transform_switch_conditions(
    stmts: list,
    locals_info: dict,
    type_ids: set,
    helper_ids: dict[str, int],
    binop_map: dict[str, str],
    is_ty_fn=None,
) -> int:
    """Replace target-type BinaryOp in Switch(If) conditions with helper calls.

    Parameters
    ----------
    stmts: statement list to transform in-place.
    locals_info: the ``locals`` dict from the function body.
    type_ids: set of dedup IDs for the target type (e.g. f64 or i64).
    helper_ids: map from helper name → def_id.
    binop_map: map from LLBC op name → helper name.
    is_ty_fn: optional ``(ty) -> bool`` override (defaults to
        ``_is_target_ty(ty, type_ids)``).
    """
    if is_ty_fn is None:
        def is_ty_fn(ty): return _is_target_ty(ty, type_ids)

    bool_dedup = _find_bool_dedup(locals_info)
    count = 0

    def _next_local() -> int:
        existing = [loc["index"] for loc in locals_info.get("locals", [])]
        return max(existing, default=-1) + 1

    def _walk(stmts: list) -> None:
        nonlocal count
        i = 0
        while i < len(stmts):
            s = stmts[i]
            kind = s.get("kind", {})
            if not isinstance(kind, dict):
                i += 1
                continue
            if "Switch" in kind and "If" in kind["Switch"]:
                sw = kind["Switch"]["If"]
                if len(sw) >= 3:
                    cond = sw[0]
                    if isinstance(cond, dict) and "BinaryOp" in cond:
                        binop = cond["BinaryOp"]
                        if len(binop) == 3:
                            op = binop[0]
                            op_name = list(op.keys())[0] if isinstance(op, dict) else str(op)
                            if op_name in binop_map:
                                lt = _operand_ty(binop[1])
                                rt = _operand_ty(binop[2])
                                if (is_ty_fn(lt) or is_ty_fn(rt)) and bool_dedup is not None:
                                    hname = binop_map[op_name]
                                    if hname in helper_ids:
                                        tmp_id = _next_local()
                                        locals_info.setdefault("locals", []).append({
                                            "index": tmp_id, "name": None,
                                            "span": _zero_span(),
                                            "ty": {"Deduplicated": bool_dedup},
                                        })
                                        tmp_ty = {"Deduplicated": bool_dedup}
                                        sw[0] = {"Move": {"kind": {"Local": tmp_id}, "ty": tmp_ty}}
                                        call_s = _call_stmt(
                                            helper_ids[hname],
                                            [binop[1], binop[2]],
                                            {"kind": {"Local": tmp_id}, "ty": tmp_ty},
                                        )
                                        stmts.insert(i, call_s)
                                        i += 1
                                        count += 1
                    for bi in [1, 2]:
                        b = sw[bi]
                        if isinstance(b, dict) and "statements" in b:
                            _walk(b["statements"])
            elif "Loop" in kind:
                _walk(kind["Loop"]["statements"])
            i += 1

    _walk(stmts)
    return count


def _find_bool_dedup(locals_info: dict) -> int | None:
    for loc in locals_info.get("locals", []):
        ty = loc.get("ty", {})
        ty_str = json.dumps(ty)
        if '"Literal":"Bool"' in ty_str or '"Literal": "Bool"' in ty_str:
            if isinstance(ty, dict):
                if "Deduplicated" in ty:
                    return ty["Deduplicated"]
                if "HashConsedValue" in ty:
                    hv = ty["HashConsedValue"]
                    if isinstance(hv, list) and len(hv) >= 2:
                        return hv[0] if isinstance(hv[0], int) else None
    return None


# ── Body walkers ─────────────────────────────────────────────────────────────


def _walk_stmts(stmts: list, visitor, **kw):
    for stmt in stmts:
        visitor(stmt, **kw)
        _recurse(stmt, visitor, **kw)


def _recurse(node: Any, visitor, **kw):
    if isinstance(node, dict):
        if "statements" in node and isinstance(node["statements"], list):
            _walk_stmts(node["statements"], visitor, **kw)
        for v in node.values():
            _recurse(v, visitor, **kw)
    elif isinstance(node, list):
        for item in node:
            _recurse(item, visitor, **kw)


def _operand_ty(op: dict) -> dict | None:
    if isinstance(op, dict) and op:
        k = list(op.keys())[0]
        inner = op.get(k, {})
        if isinstance(inner, dict):
            return inner.get("ty")
    return None


def _zero_span() -> dict:
    return {
        "data": {"file_id": 0, "beg": {"line": 0, "col": 0}, "end": {"line": 0, "col": 0}},
        "generated_from_span": None,
    }
