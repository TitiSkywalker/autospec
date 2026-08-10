#!/usr/bin/env python3
"""
LLBC adapter: transform f64 operations in .llbc JSON before aeneas processes it.

Replaces BinaryOp/UnaryOp on f64 with calls to Foreign f64_helpers functions,
and expands Float(F64) constants into const_X_Y() calls.
Injects properly-formatted Foreign function declarations matching charon's format.
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path
from typing import Any

# ── Imports from shared LLBC utilities ───────────────────────────────────────

from llbc_common import (  # noqa: E402
    _is_target_ty,
    _discover_bool_id,
    _count_dedup_in_locals,
    _make_fun_decl,
    _call_stmt,
    _transform_switch_conditions,
    _walk_stmts,
    _operand_ty,
)

# ── Type discovery ───────────────────────────────────────────────────────────


def _discover_f64_ids(raw_json: str) -> set:
    """Find all dedup IDs whose definition is ``Literal(Float(F64))``.

    Scans the raw LLBC JSON for patterns like::

        "HashConsedValue":[298,{"Literal":{"Float":"F64"}}]

    and returns the set of dedup IDs (both the HashConsedValue wrapper
    and the underlying Deduplicated reference, which can differ in some
    contexts).
    """
    ids: set[int] = set()
    for m in re.finditer(
        r'"HashConsedValue"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Literal"\s*:\s*\{\s*"Float"\s*:\s*"F64"\s*\}',
        raw_json,
    ):
        ids.add(int(m.group(1)))
    return ids


def _discover_types(translated: dict, f64_ids: set, raw_json: str = "") -> tuple[int, int, bool]:
    """Return (usize_dedup_id, bool_dedup_id, has_f64).

    *f64_ids* must be pre-discovered via :func:`_discover_f64_ids`.
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

        # usize: find a plain Dedup param that isn't f64 or bool
        if usize is None:
            for inp in sig.get("inputs") or []:
                if isinstance(inp, dict) and "Deduplicated" in inp:
                    did = inp["Deduplicated"]
                    if did not in f64_ids and did != bool_id:
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

    # usize fallback: most common dedup in locals that isn't f64 or bool
    if usize is None:
        local_counts = _count_dedup_in_locals(translated)
        by_count = sorted(local_counts.items(), key=lambda x: -x[1])
        for did, _ in by_count:
            if did not in f64_ids and did != bool_id:
                usize = did
                break

    return usize, bool_id, bool(f64_ids)


def _extract_slice_elem(ty_node: Any) -> int | None:
    inner = ty_node
    if isinstance(inner, dict) and "HashConsedValue" in inner:
        inner = inner["HashConsedValue"][1]
    if isinstance(inner, dict):
        if "Ref" in inner:
            inner = inner["Ref"][1]
            if isinstance(inner, dict) and "HashConsedValue" in inner:
                inner = inner["HashConsedValue"][1]
        if isinstance(inner, dict) and "Slice" in inner:
            elem = inner["Slice"]
            if isinstance(elem, dict) and "HashConsedValue" in elem:
                elem = elem["HashConsedValue"][1]
            if isinstance(elem, dict) and "Deduplicated" in elem:
                return elem["Deduplicated"]
    return None


# ── Operation maps ────────────────────────────────────────────────────────────

BINOP_MAP = {
    "Add": "add",
    "Sub": "sub",
    "Mul": "mul",
    "Div": "div",
    "Eq": "eq",
    "Gt": "gt",
    "Lt": "lt",
    "Ge": "ge",
    "Le": "le",
}

HELPER_SIGS = [
    ("add", [0, 0], 0),  # (f64, f64) -> f64
    ("sub", [0, 0], 0),
    ("mul", [0, 0], 0),
    ("div", [0, 0], 0),
    ("neg", [0], 0),
    ("abs", [0], 0),
    ("eq", [0, 0], 2),  # (f64, f64) -> bool
    ("gt", [0, 0], 2),
    ("lt", [0, 0], 2),
    ("ge", [0, 0], 2),
    ("le", [0, 0], 2),
    ("from_usize", [1], 0),  # (usize) -> f64
]
# Index 0=f64, 1=usize, 2=bool in the type array

# ── Float constant expansion ─────────────────────────────────────────────────


def _float_to_name(value: str) -> str:
    """Convert a float literal text to a safe identifier suffix.

    Normalizes scientific notation (e.g. ``1.0E-10``) to a decimal
    fraction so the resulting name contains only digits and underscores.
    """
    # Normalize scientific notation to avoid 'E' in the name
    if "E" in value or "e" in value:
        try:
            from decimal import Decimal
            d = Decimal(value)
            value = format(d, "f")
        except Exception:
            pass
    return "const_" + value.replace(".", "_").replace("-", "neg_")


def _collect_float_values(stmts: list) -> set[str]:
    """Recursively find all Float(F64) values in stmts."""
    vals: set[str] = set()

    def _rec(n: Any):
        if isinstance(n, dict):
            if "Float" in n and isinstance(n["Float"], dict):
                if n["Float"].get("ty") == "F64":
                    vals.add(n["Float"]["value"])
            for v in n.values():
                _rec(v)
        elif isinstance(n, list):
            for item in n:
                _rec(item)

    for stmt in stmts:
        _rec(stmt)
    return vals


def _expand_float_constants(
    stmts: list, f64: int, const_ids: dict[str, int], locals_info: dict
) -> int:
    """Hoist Float(F64) constants to const_X_Y() calls at top of block."""
    all_vals = _collect_float_values(stmts)
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
        cname = _float_to_name(val)
        if cname not in const_ids:
            continue
        # Create a proper local entry
        local_idx = len(locals_info["locals"])
        val_to_local[val] = local_idx
        locals_info["locals"].append(
            {
                "index": local_idx,
                "name": f"_f64_{cname}",
                "span": span0,
                "ty": {"Deduplicated": f64},
            }
        )

        init_stmts.append(
            _call_stmt(
                const_ids[cname],
                [],
                {"kind": {"Local": local_idx}, "ty": {"Deduplicated": f64}},
                span0,
            )
        )

    if init_stmts:
        stmts[:0] = init_stmts

    # Replace Const(Float) nodes with Move(local)
    _replace_float_nodes(stmts, val_to_local, f64)
    return len(all_vals)


def _replace_float_nodes(node: Any, val_to_local: dict[str, int], f64: int) -> None:
    """Replace Const(kind=Literal(Float(...))) with Copy(local).

    Uses ``Copy`` (not ``Move``) because f64 is a Copy type in LLBC.
    ``Move`` would consume the local, causing ⊥ when the same float
    constant appears multiple times (e.g. duplicate values in vec!)."""
    if isinstance(node, dict):
        if "Const" in node:
            c = node["Const"]
            if isinstance(c, dict) and "kind" in c:
                k = c["kind"]
                if isinstance(k, dict) and "Literal" in k:
                    lit = k["Literal"]
                    if isinstance(lit, dict) and "Float" in lit:
                        val = lit["Float"]["value"]
                        if val in val_to_local:
                            node.clear()
                            node["Copy"] = {
                                "kind": {"Local": val_to_local[val]},
                                "ty": {"Deduplicated": f64},
                            }
                            return
                    # Also handle Scalar→Float path
                    if isinstance(lit, dict) and "Scalar" in lit:
                        sc = lit["Scalar"]
                        if isinstance(sc, dict) and "Float" in sc:
                            val = sc["Float"]["value"]
                            if val in val_to_local:
                                node.clear()
                                node["Copy"] = {
                                    "kind": {"Local": val_to_local[val]},
                                    "ty": {"Deduplicated": f64},
                                }
                                return
        for v in node.values():
            _replace_float_nodes(v, val_to_local, f64)
    elif isinstance(node, list):
        for i, item in enumerate(node):
            if isinstance(item, dict) and "Const" in item:
                # Check if this list item can be replaced in-place
                c = item["Const"]
                if isinstance(c, dict) and "kind" in c:
                    k = c["kind"]
                    if isinstance(k, dict) and "Literal" in k:
                        lit = k["Literal"]
                        if isinstance(lit, dict):
                            sc = lit.get("Scalar", lit)
                            if isinstance(sc, dict) and "Float" in sc:
                                val = sc["Float"]["value"]
                                if val in val_to_local:
                                    node[i] = {
                                        "Copy": {
                                            "kind": {"Local": val_to_local[val]},
                                            "ty": {"Deduplicated": f64},
                                        }
                                    }
            _replace_float_nodes(item, val_to_local, f64)


# ── Main entry point ─────────────────────────────────────────────────────────


def transform_llbc(path: str | Path, output: str | Path | None = None) -> int:
    path = Path(path)
    if output is None:
        output = path

    raw_json = path.read_text()
    data = json.loads(raw_json)

    trans = data["translated"]
    f64_ids = _discover_f64_ids(raw_json)
    usize, bool_id, has_f64 = _discover_types(trans, f64_ids, raw_json)
    if has_f64 and not f64_ids:
        f64_ids = _discover_f64_ids(raw_json)  # retry after processing

    if not f64_ids:
        print("  [adapter] no f64 type found — nothing to do", file=sys.stderr)
        return 0

    # Pick the primary f64 dedup ID for creating new type references.
    # Prefer the one that appears most often in function bodies.
    local_counts = _count_dedup_in_locals(trans)
    primary_f64 = max(f64_ids, key=lambda d: local_counts.get(d, 0))
    types_list = [primary_f64, usize, bool_id]

    print(
        f"  [adapter] f64={primary_f64} (ids={sorted(f64_ids)}),"
        f" usize={usize}, bool={bool_id}",
        file=sys.stderr,
    )

    # 1. Collect all float constants used across all function bodies
    all_float_vals: set[str] = set()
    for fd in trans.get("fun_decls") or []:
        if fd is None or fd.get("body") is None:
            continue
        body = fd["body"]
        if isinstance(body, dict) and "Structured" in body:
            stmts = body["Structured"]["body"]["statements"]
            all_float_vals |= _collect_float_values(stmts)

    # 2. Register f64_helpers + const functions
    base_id = 0
    for lst in trans.get("fun_decls") or []:
        if lst is not None:
            base_id = max(base_id, lst["def_id"])
    for lst in trans.get("global_decls") or []:
        if lst is not None:
            base_id = max(base_id, lst["def_id"])
    base_id += 1

    helper_ids: dict[str, int] = {}

    # Discover the crate name for proper function naming
    crate_name = "f64_helpers"

    # Insert our helpers right before the first Transparent function.
    # This way: all Dedup IDs from original Foreign/TraitDecl entries
    # are registered, then our helpers are registered, then Transparent
    # bodies are analyzed and can find our helpers.
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
    for i, (name, param_idxs, ret_idx) in enumerate(HELPER_SIGS):
        # Check that all required types are known
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

    const_start = base_id + len(HELPER_SIGS)
    for j, val in enumerate(sorted(all_float_vals)):
        did = const_start + j
        cname = _float_to_name(val)
        helper_ids[cname] = did
        all_decls.append((did, crate_name, cname, [], {"Deduplicated": primary_f64}))

    # Insert all at the correct position (reverse order for correct insertion)
    for did, crate_name, name, inputs, ret_ty in reversed(all_decls):
        decl = _make_fun_decl(did, crate_name, name, inputs, ret_ty)
        # Extend fun_decls list so that fun_decls[def_id] points to this decl
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
        f"  [adapter] registered {len(helper_ids)} helpers "
        f"({len(all_float_vals)} consts)",
        file=sys.stderr,
    )

    # 3. Transform function bodies
    total_binop = 0
    total_float = 0

    def _visitor(stmt: dict) -> bool:
        nonlocal total_binop
        kind = stmt.get("kind", {})
        if "Assign" not in kind:
            return False

        target, rhs = kind["Assign"]
        if not isinstance(rhs, dict):
            return False

        # BinaryOp on f64
        if "BinaryOp" in rhs:
            binop = rhs["BinaryOp"]
            if len(binop) == 3:
                op, left, right = binop
                op_name = list(op.keys())[0] if isinstance(op, dict) else str(op)
                if op_name in BINOP_MAP:
                    lt = _operand_ty(left)
                    rt = _operand_ty(right)
                    if _is_target_ty(lt, f64_ids) or _is_target_ty(rt, f64_ids):
                        hname = BINOP_MAP[op_name]
                        if hname in helper_ids:
                            new_s = _call_stmt(helper_ids[hname], [left, right], target)
                            stmt.clear()
                            stmt.update(new_s)
                            total_binop += 1
                            return True

        # UnaryOp Neg on f64
        if "UnaryOp" in rhs:
            unop = rhs["UnaryOp"]
            if len(unop) == 2:
                op, operand = unop
                op_name = list(op.keys())[0] if isinstance(op, dict) else str(op)
                if op_name == "Neg" and _is_target_ty(_operand_ty(operand), f64_ids):
                    if "neg" in helper_ids:
                        new_s = _call_stmt(helper_ids["neg"], [operand], target)
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
        total_float += _expand_float_constants(stmts, primary_f64, helper_ids, locals_info)
        _walk_stmts(stmts, _visitor)
        total_binop += _transform_switch_conditions(
            stmts, locals_info, f64_ids, helper_ids, BINOP_MAP
        )

    print(
        f"  [adapter] {total_float} float consts, {total_binop} binop/neg "
        "transformed",
        file=sys.stderr,
    )

    with open(output, "w") as f:
        json.dump(data, f, indent=2)

    return total_float + total_binop


if __name__ == "__main__":
    import argparse

    p = argparse.ArgumentParser()
    p.add_argument("llbc_file")
    p.add_argument("-o", "--output", default=None)
    args = p.parse_args()
    n = transform_llbc(args.llbc_file, args.output)
    if n < 0:
        sys.exit(1)
    print(f"Transformed {n} f64 operations/constants.")
