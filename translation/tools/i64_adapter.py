#!/usr/bin/env python3
"""
I64 LLBC adapter: transform i64 operations in .llbc JSON before aeneas processes it.

Replaces BinaryOp/UnaryOp on i64 with calls to Foreign i64_helpers functions,
and expands Scalar(Signed(I64)) constants into const_X() calls.
Injects properly-formatted Foreign function declarations matching charon's format.

This is the i64 analogue of ``llbc_adapter.py`` (which handles f64 → Rat).
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

# ── LLBC helpers ─────────────────────────────────────────────────────────────


def _is_i64_ty(ty: dict | None, i64_ids: set) -> bool:
    """Check whether a type node references an i64 dedup ID."""
    if isinstance(ty, dict):
        if "Deduplicated" in ty:
            return ty["Deduplicated"] in i64_ids
        if "HashConsedValue" in ty:
            return ty["HashConsedValue"][0] in i64_ids
    return False


def _op_name(op: Any) -> str:
    """Extract the operator name from a BinaryOp/UnaryOp head.

    Handles both forms:
      - plain string: ``"Eq"``, ``"BitAnd"``
      - object with overflow mode: ``{"Add":"Wrap"}``, ``{"Neg":"Panic"}``
    """
    if isinstance(op, str):
        return op
    if isinstance(op, dict):
        return list(op.keys())[0]
    return str(op)


# ── Type discovery ───────────────────────────────────────────────────────────


def _discover_i64_ids(raw_json: str) -> set:
    """Find all dedup IDs whose definition is ``Literal(Int(I64))``.

    Scans the raw LLBC JSON for patterns like::

        "HashConsedValue":[298,{"Literal":{"Int":"I64"}}]

    and returns the set of dedup IDs.
    """
    ids: set[int] = set()
    for m in re.finditer(
        r'"HashConsedValue"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Literal"\s*:\s*\{\s*"Int"\s*:\s*"I64"\s*\}',
        raw_json,
    ):
        ids.add(int(m.group(1)))
    return ids


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


def _discover_types(translated: dict, i64_ids: set, raw_json: str = "") -> tuple[int | None, int | None, bool]:
    """Return (usize_dedup_id, bool_dedup_id, has_i64).

    *i64_ids* must be pre-discovered via :func:`_discover_i64_ids`.
    *raw_json* is used for reliable Bool type discovery.
    """
    usize = None
    bool_id = None

    # Discover Bool type from the raw JSON (more reliable than return-local scan)
    if raw_json:
        bool_id = _discover_bool_id(raw_json)

    for fd in translated.get("fun_decls") or []:
        if fd is None:
            continue
        sig = fd.get("signature")
        if not sig:
            continue

        # usize: find a plain Dedup param that isn't i64 or bool
        if usize is None:
            for inp in sig.get("inputs") or []:
                if isinstance(inp, dict) and "Deduplicated" in inp:
                    did = inp["Deduplicated"]
                    if did not in i64_ids and did != bool_id:
                        usize = did
                        break

        if usize is not None and bool_id is not None:
            break

    # bool fallback: scan all locals for Bool literal type
    if bool_id is None:
        for fd in translated.get("fun_decls") or []:
            if fd is None:
                continue
            body = fd.get("body")
            if not isinstance(body, dict) or "Structured" not in body:
                continue
            for loc in body["Structured"].get("locals", {}).get("locals") or []:
                ty = loc.get("ty", {})
                ty_str = json.dumps(ty)
                if '"Literal":"Bool"' in ty_str or '"Literal": "Bool"' in ty_str:
                    if isinstance(ty, dict):
                        if "Deduplicated" in ty:
                            bool_id = ty["Deduplicated"]
                        elif "HashConsedValue" in ty:
                            bool_id = ty["HashConsedValue"][0]
                    if bool_id:
                        break
            if bool_id:
                break

    # usize fallback: most common dedup in locals that isn't i64 or bool
    if usize is None:
        local_counts = _count_dedup_in_locals(translated)
        by_count = sorted(local_counts.items(), key=lambda x: -x[1])
        for did, _ in by_count:
            if did not in i64_ids and did != bool_id:
                usize = did
                break

    return usize, bool_id, bool(i64_ids)


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
        span = {
            "data": {
                "file_id": 0,
                "beg": {"line": 0, "col": 0},
                "end": {"line": 0, "col": 0},
            },
            "generated_from_span": None,
        }
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
    stmts: list, locals_info: dict, i64_ids: set, helper_ids: dict[str, int]
) -> int:
    """Replace i64 BinaryOp in Switch(If) conditions with helper calls."""
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
                            op_name = _op_name(op)
                            if op_name in I64_BINOP_MAP:
                                lt = _operand_ty(binop[1])
                                rt = _operand_ty(binop[2])
                                if (_is_i64_ty(lt, i64_ids) or _is_i64_ty(rt, i64_ids)) and bool_dedup is not None:
                                    hname = I64_BINOP_MAP[op_name]
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


def _zero_span() -> dict:
    return {
        "data": {"file_id": 0, "beg": {"line": 0, "col": 0}, "end": {"line": 0, "col": 0}},
        "generated_from_span": None,
    }


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


# ── Operation maps ────────────────────────────────────────────────────────────

# BinaryOp → helper name.
# Covers arithmetic (Add/Sub/Mul/Div/Rem with Wrap/Panic/UB modes),
# comparisons (Eq/Ne/Gt/Lt/Ge/Le plain strings),
# bitwise (BitAnd/BitOr/BitXor plain strings),
# and shifts (Shl/Shr with Panic mode).
I64_BINOP_MAP = {
    # arithmetic
    "Add": "add",
    "Sub": "sub",
    "Mul": "mul",
    "Div": "div",
    "Rem": "rem",
    # comparisons
    "Eq": "eq",
    "Ne": "ne",
    "Gt": "gt",
    "Lt": "lt",
    "Ge": "ge",
    "Le": "le",
    # bitwise
    "BitAnd": "bitand",
    "BitOr": "bitor",
    "BitXor": "bitxor",
    # shifts
    "Shl": "shl",
    "Shr": "shr",
    # checked arithmetic (from .ullbc)
    "AddChecked": "add",
    "MulChecked": "mul",
}

# UnaryOp → helper name.
I64_UNARY_MAP = {
    "Neg": "neg",
    "Not": "not",
}

# Helper signatures: (name, param_type_indices, return_type_index).
# Index 0=i64, 1=usize, 2=bool in the type array.
I64_HELPER_SIGS = [
    # arithmetic — (i64, i64) → i64
    ("add", [0, 0], 0),
    ("sub", [0, 0], 0),
    ("mul", [0, 0], 0),
    ("div", [0, 0], 0),
    ("rem", [0, 0], 0),
    # comparisons — (i64, i64) → bool
    ("eq", [0, 0], 2),
    ("ne", [0, 0], 2),
    ("gt", [0, 0], 2),
    ("lt", [0, 0], 2),
    ("ge", [0, 0], 2),
    ("le", [0, 0], 2),
    # bitwise — (i64, i64) → i64
    ("bitand", [0, 0], 0),
    ("bitor", [0, 0], 0),
    ("bitxor", [0, 0], 0),
    # shifts — (i64, usize) → i64
    ("shl", [0, 1], 0),
    ("shr", [0, 1], 0),
    # unary
    ("neg", [0], 0),
    ("not", [0], 0),
    # conversion — (usize) → i64
    ("from_usize", [1], 0),
]

# ── Integer constant expansion ────────────────────────────────────────────────


def _i64_const_name(value: str) -> str:
    """Convert an i64 literal to a safe identifier suffix.

    >>> _i64_const_name("42")
    'const_42'
    >>> _i64_const_name("-5")
    'const_neg_5'
    """
    if value.startswith("-"):
        return "const_neg_" + value[1:]
    return "const_" + value


def _collect_i64_constants(stmts: list) -> set[str]:
    """Recursively find all Scalar(Signed(I64, ...)) values in stmts."""
    vals: set[str] = set()

    def _rec(n: Any):
        if isinstance(n, dict):
            if "Scalar" in n and isinstance(n["Scalar"], dict):
                signed = n["Scalar"].get("Signed")
                if isinstance(signed, list) and len(signed) == 2 and signed[0] == "I64":
                    vals.add(signed[1])
            for v in n.values():
                _rec(v)
        elif isinstance(n, list):
            for item in n:
                _rec(item)

    for stmt in stmts:
        _rec(stmt)
    return vals


def _expand_i64_constants(
    stmts: list, i64: int, const_ids: dict[str, int], locals_info: dict
) -> int:
    """Hoist Scalar(Signed(I64)) constants to const_X() calls at top of block."""
    all_vals = _collect_i64_constants(stmts)
    if not all_vals:
        return 0

    val_to_local: dict[str, int] = {}
    span0 = stmts[0].get("span") if stmts else None
    if span0 is None:
        span0 = {
            "data": {
                "file_id": 0,
                "beg": {"line": 0, "col": 0},
                "end": {"line": 0, "col": 0},
            },
            "generated_from_span": None,
        }
    init_stmts = []

    for val in sorted(all_vals):
        cname = _i64_const_name(val)
        if cname not in const_ids:
            continue
        local_idx = len(locals_info["locals"])
        val_to_local[val] = local_idx
        locals_info["locals"].append(
            {
                "index": local_idx,
                "name": f"_i64_{cname}",
                "span": span0,
                "ty": {"Deduplicated": i64},
            }
        )

        init_stmts.append(
            _call_stmt(
                const_ids[cname],
                [],
                {"kind": {"Local": local_idx}, "ty": {"Deduplicated": i64}},
                span0,
            )
        )

    if init_stmts:
        stmts[:0] = init_stmts

    # Replace Const(Scalar(Signed(I64))) nodes with Copy(local)
    _replace_i64_nodes(stmts, val_to_local, i64)
    return len(all_vals)


def _replace_i64_nodes(node: Any, val_to_local: dict[str, int], i64: int) -> None:
    """Replace Const(kind=Literal(Scalar(Signed(I64)))) with Copy(local).

    Uses ``Copy`` (not ``Move``) because i64 is a Copy type in LLBC.
    ``Move`` would consume the local, causing issues when the same
    constant appears multiple times.
    """
    if isinstance(node, dict):
        if "Const" in node:
            c = node["Const"]
            if isinstance(c, dict) and "kind" in c:
                k = c["kind"]
                if isinstance(k, dict) and "Literal" in k:
                    lit = k["Literal"]
                    if isinstance(lit, dict) and "Scalar" in lit:
                        sc = lit["Scalar"]
                        if isinstance(sc, dict) and "Signed" in sc:
                            signed = sc["Signed"]
                            if isinstance(signed, list) and len(signed) == 2 and signed[0] == "I64":
                                val = signed[1]
                                if val in val_to_local:
                                    node.clear()
                                    node["Copy"] = {
                                        "kind": {"Local": val_to_local[val]},
                                        "ty": {"Deduplicated": i64},
                                    }
                                    return
        for v in node.values():
            _replace_i64_nodes(v, val_to_local, i64)
    elif isinstance(node, list):
        for i, item in enumerate(node):
            if isinstance(item, dict) and "Const" in item:
                c = item["Const"]
                if isinstance(c, dict) and "kind" in c:
                    k = c["kind"]
                    if isinstance(k, dict) and "Literal" in k:
                        lit = k["Literal"]
                        if isinstance(lit, dict) and "Scalar" in lit:
                            sc = lit["Scalar"]
                            if isinstance(sc, dict) and "Signed" in sc:
                                signed = sc["Signed"]
                                if isinstance(signed, list) and len(signed) == 2 and signed[0] == "I64":
                                    val = signed[1]
                                    if val in val_to_local:
                                        node[i] = {
                                            "Copy": {
                                                "kind": {"Local": val_to_local[val]},
                                                "ty": {"Deduplicated": i64},
                                            }
                                        }
            _replace_i64_nodes(item, val_to_local, i64)


# ── Main entry point ─────────────────────────────────────────────────────────


def transform_i64_llbc(path: str | Path, output: str | Path | None = None) -> int:
    """Transform i64 operations in an LLBC JSON file.

    Reads *path*, discovers i64 types and constants, replaces BinaryOp/UnaryOp
    on i64 with ``i64_helpers.*`` Foreign calls, and writes the result to
    *output* (defaults to overwriting *path*).

    Returns the total number of transformations applied (constants + ops).
    """
    path = Path(path)
    if output is None:
        output = path

    raw_json = path.read_text()
    data = json.loads(raw_json)

    trans = data["translated"]
    i64_ids = _discover_i64_ids(raw_json)
    usize, bool_id, has_i64 = _discover_types(trans, i64_ids, raw_json)
    if has_i64 and not i64_ids:
        i64_ids = _discover_i64_ids(raw_json)

    if not i64_ids:
        print("  [adapter] no i64 type found — nothing to do", file=sys.stderr)
        return 0

    # Pick the primary i64 dedup ID (most frequent in function bodies).
    local_counts = _count_dedup_in_locals(trans)
    primary_i64 = max(i64_ids, key=lambda d: local_counts.get(d, 0))
    types_list = [primary_i64, usize, bool_id]

    print(
        f"  [adapter] i64={primary_i64} (ids={sorted(i64_ids)}),"
        f" usize={usize}, bool={bool_id}",
        file=sys.stderr,
    )

    # 1. Compute base_id for new declarations
    base_id = 0
    for lst in trans.get("fun_decls") or []:
        if lst is not None:
            base_id = max(base_id, lst["def_id"])
    for lst in trans.get("global_decls") or []:
        if lst is not None:
            base_id = max(base_id, lst["def_id"])
    base_id += 1

    helper_ids: dict[str, int] = {}
    crate_name = "i64_helpers"

    # Insert helpers right before the first Transparent function.
    od_list = trans.setdefault("ordered_decls", [])
    insert_at = len(od_list)
    for idx, od in enumerate(od_list):
        if "Fun" in od:
            fid = od["Fun"]["NonRec"]
            for fd in trans.get("fun_decls") or []:
                if fd is not None and fd["def_id"] == fid:
                    if fd["item_meta"]["opacity"] == "Transparent":
                        insert_at = idx
                        break
            if insert_at < len(od_list):
                break

    # Build all helper declarations — skip any whose types aren't available
    all_decls = []
    for i, (name, param_idxs, ret_idx) in enumerate(I64_HELPER_SIGS):
        types_ok = True
        for idx in param_idxs + [ret_idx]:
            if types_list[idx] is None:
                types_ok = False
                break
        if not types_ok:
            continue
        did = base_id + i
        helper_ids[name] = did
        inputs = [{"Deduplicated": types_list[idx]} for idx in param_idxs]
        ret_ty = {"Deduplicated": types_list[ret_idx]}
        all_decls.append((did, crate_name, name, inputs, ret_ty))

    # Insert all at the correct position (reverse for correct order)
    for did, crate_name, name, inputs, ret_ty in reversed(all_decls):
        decl = _make_fun_decl(did, crate_name, name, inputs, ret_ty)
        fd_list = trans.setdefault("fun_decls", [])
        while len(fd_list) <= did:
            fd_list.append(None)
        fd_list[did] = decl
        od_list.insert(insert_at, {"Fun": {"NonRec": did}})
        trans.setdefault("item_names", []).append(
            {
                "key": {"Fun": did},
                "value": [{"Ident": [crate_name, 0]}, {"Ident": [name, 0]}],
            }
        )
        trans.setdefault("short_names", []).append(
            {
                "key": {"Fun": did},
                "value": [{"Ident": [name, 0]}],
            }
        )

    print(
        f"  [adapter] registered {len(helper_ids)} helpers",
        file=sys.stderr,
    )

    # 2. Transform function bodies
    total_binop = 0

    def _visitor(stmt: dict) -> bool:
        nonlocal total_binop
        kind = stmt.get("kind", {})
        if "Assign" not in kind:
            return False

        target, rhs = kind["Assign"]
        if not isinstance(rhs, dict):
            return False

        # BinaryOp on i64
        if "BinaryOp" in rhs:
            binop = rhs["BinaryOp"]
            if len(binop) == 3:
                op, left, right = binop
                op_name = _op_name(op)
                if op_name in I64_BINOP_MAP:
                    lt = _operand_ty(left)
                    rt = _operand_ty(right)
                    if _is_i64_ty(lt, i64_ids) or _is_i64_ty(rt, i64_ids):
                        hname = I64_BINOP_MAP[op_name]
                        if hname in helper_ids:
                            new_s = _call_stmt(helper_ids[hname], [left, right], target)
                            stmt.clear()
                            stmt.update(new_s)
                            total_binop += 1
                            return True

        # UnaryOp on i64
        if "UnaryOp" in rhs:
            unop = rhs["UnaryOp"]
            if len(unop) == 2:
                op, operand = unop
                op_name = _op_name(op)
                if op_name in I64_UNARY_MAP:
                    if _is_i64_ty(_operand_ty(operand), i64_ids):
                        hname = I64_UNARY_MAP[op_name]
                        if hname in helper_ids:
                            new_s = _call_stmt(helper_ids[hname], [operand], target)
                            stmt.clear()
                            stmt.update(new_s)
                            total_binop += 1
                            return True
        return False

    for fd in trans.get("fun_decls") or []:
        if fd is None or fd.get("body") is None:
            continue
        body = fd["body"]
        if not isinstance(body, dict) or "Structured" not in body:
            continue

        stmts = body["Structured"]["body"]["statements"]
        locals_info = body["Structured"]["locals"]
        _walk_stmts(stmts, _visitor)
        total_binop += _transform_switch_conditions(
            stmts, locals_info, i64_ids, helper_ids
        )

    print(
        f"  [adapter] {total_binop} binop/unop transformed",
        file=sys.stderr,
    )

    with open(output, "w") as f:
        json.dump(data, f, indent=2)

    return total_binop


if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser(description="Transform i64 operations in LLBC JSON")
    p.add_argument("llbc_file")
    p.add_argument("-o", "--output", default=None)
    args = p.parse_args()
    n = transform_i64_llbc(args.llbc_file, args.output)
    if n < 0:
        sys.exit(1)
    print(f"Transformed {n} i64 operations/constants.")
