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


# ── LLBC helpers ─────────────────────────────────────────────────────────────

def _is_f64_ty(ty: dict | None, f64_id: int) -> bool:
    if isinstance(ty, dict):
        return ty.get("Deduplicated") == f64_id
    return False


# ── Type discovery ───────────────────────────────────────────────────────────

def _discover_types(translated: dict) -> tuple[int, int, int]:
    """Return (f64_dedup_id, usize_dedup_id, bool_dedup_id)."""
    f64 = None
    usize = None
    bool_id = None

    for fd in translated.get("fun_decls") or []:
        if fd is None:
            continue
        sig = fd.get("signature")
        if not sig:
            continue

        # f64: gather candidates from multiple sources, then pick best
        f64_candidates: set[int] = set()
        for inp in sig.get("inputs") or []:
            elem = _extract_slice_elem(inp)
            if elem is not None:
                f64_candidates.add(elem)
        src = fd["item_meta"].get("source_text") or ""
        if "f64" in src:
            for inp in sig.get("inputs") or []:
                if isinstance(inp, dict) and "Deduplicated" in inp:
                    f64_candidates.add(inp["Deduplicated"])
        # Discard candidates that appear many times in locals (likely usize)
        local_counts = _count_dedup_in_locals(translated)
        if len(f64_candidates) > 1:
            # Pick the candidate with fewest local occurrences (f64)
            best = min(f64_candidates,
                       key=lambda d: local_counts.get(d, 0), default=None)
            if best is not None:
                f64 = best
        elif len(f64_candidates) == 1:
            f64 = f64_candidates.pop()

        # bool: look at the return-local (index 0) of a bool-returning function
        if bool_id is None:
            src = fd["item_meta"].get("source_text") or ""
            if "-> bool" in src or "-> Bool" in src:
                body = fd.get("body")
                if isinstance(body, dict) and "Structured" in body:
                    for loc in body["Structured"].get("locals", {}).get("locals") or []:
                        if loc["index"] == 0:
                            ty = loc.get("ty")
                            if isinstance(ty, dict) and "Deduplicated" in ty:
                                bool_id = ty["Deduplicated"]
                                break

        # usize: plain Dedup params that aren't f64 or bool
        if usize is None and f64 is not None:
            for inp in sig.get("inputs") or []:
                if isinstance(inp, dict) and "Deduplicated" in inp:
                    did = inp["Deduplicated"]
                    if did != f64 and did != bool_id:
                        usize = did
                        break

        if f64 and usize and bool_id:
            break

    # Most reliable: find f64 via "Wrap" mode BinaryOps
    # (f64 uses Wrap, usize uses Panic) — only in user functions
    wrap_f64 = None
    wrap_usize = None
    wrap_counts: dict[int, int] = {}
    for fd in translated.get("fun_decls") or []:
        if fd is None or fd.get("body") is None:
            continue
        if not fd["item_meta"].get("is_local", True):
            continue
        body_str = json.dumps(fd["body"])
        for m in re.finditer(r'"Wrap"', body_str):
            chunk = body_str[max(0, m.start()-80):m.end()+80]
            for dm in re.finditer(r'"Deduplicated":\s*(\d+)', chunk):
                did = int(dm.group(1))
                wrap_counts[did] = wrap_counts.get(did, 0) + 1
    if len(wrap_counts) >= 1:
        wrap_f64 = max(wrap_counts, key=wrap_counts.get)
    # Also find usize via "Panic" mode BinaryOps
    panic_counts: dict[int, int] = {}
    for fd in translated.get("fun_decls") or []:
        if fd is None or fd.get("body") is None:
            continue
        if not fd["item_meta"].get("is_local", True):
            continue
        body_str = json.dumps(fd["body"])
        for m in re.finditer(r'"Panic"', body_str):
            chunk = body_str[max(0, m.start()-80):m.end()+80]
            for dm in re.finditer(r'"Deduplicated":\s*(\d+)', chunk):
                did = int(dm.group(1))
                panic_counts[did] = panic_counts.get(did, 0) + 1
    if len(panic_counts) >= 1:
        wrap_usize = max(panic_counts, key=panic_counts.get)
    if wrap_f64 is not None:
        f64 = wrap_f64
    if wrap_usize is not None and usize is None:
        usize = wrap_usize
    if usize is None:
        by_count = sorted(
            _count_dedup_in_locals(translated).items(), key=lambda x: -x[1]
        )
        if len(by_count) >= 1:
            usize = by_count[0][0]

    return f64, usize, bool_id


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
            if isinstance(elem, dict) and "Deduplicated" in elem:
                return elem["Deduplicated"]
    return None


# ── Function declaration builder ─────────────────────────────────────────────

def _make_fun_decl(def_id: int, crate: str, name: str,
                   inputs: list[dict], output: dict,
                   template: dict | None = None) -> dict:
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
            "data": {"file_id": 0, "beg": {"line": 0, "col": 0},
                     "end": {"line": 0, "col": 0}},
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

BINOP_MAP = {
    "Add": "add", "Sub": "sub", "Mul": "mul", "Div": "div",
    "Eq": "eq", "Gt": "gt", "Lt": "lt", "Ge": "ge", "Le": "le",
}

HELPER_SIGS = [
    ("add", [0, 0], 0),       # (f64, f64) -> f64
    ("sub", [0, 0], 0),
    ("mul", [0, 0], 0),
    ("div", [0, 0], 0),
    ("neg", [0], 0),
    ("abs", [0], 0),
    ("eq",  [0, 0], 2),       # (f64, f64) -> bool
    ("gt",  [0, 0], 2),
    ("lt",  [0, 0], 2),
    ("ge",  [0, 0], 2),
    ("le",  [0, 0], 2),
    ("from_usize", [1], 0),   # (usize) -> f64
]
# Index 0=f64, 1=usize, 2=bool in the type array


# ── Float constant expansion ─────────────────────────────────────────────────

def _float_to_name(value: str) -> str:
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


def _expand_float_constants(stmts: list, f64: int,
                            const_ids: dict[str, int],
                            locals_info: dict) -> int:
    """Hoist Float(F64) constants to const_X_Y() calls at top of block."""
    all_vals = _collect_float_values(stmts)
    if not all_vals:
        return 0

    val_to_local: dict[str, int] = {}
    span0 = stmts[0].get("span") if stmts else None
    if span0 is None:
        span0 = {
            "data": {"file_id": 0, "beg": {"line": 0, "col": 0},
                     "end": {"line": 0, "col": 0}},
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
        locals_info["locals"].append({
            "index": local_idx,
            "name": f"_f64_{cname}",
            "span": span0,
            "ty": {"Deduplicated": f64},
        })

        init_stmts.append(_call_stmt(
            const_ids[cname], [],
            {"kind": {"Local": local_idx}, "ty": {"Deduplicated": f64}},
            span0,
        ))

    if init_stmts:
        stmts[:0] = init_stmts

    # Replace Const(Float) nodes with Move(local)
    _replace_float_nodes(stmts, val_to_local, f64)
    return len(all_vals)


def _replace_float_nodes(node: Any, val_to_local: dict[str, int],
                         f64: int) -> None:
    """Replace Const(kind=Literal(Float(...))) with Move(local)."""
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
                            node["Move"] = {
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
                                node["Move"] = {
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
                                        "Move": {
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

    with open(path) as f:
        data = json.load(f)

    trans = data["translated"]
    f64, usize, bool_id = _discover_types(trans)
    types_list = [f64, usize, bool_id]

    if f64 is None:
        print("  [adapter] no f64 type found — nothing to do", file=sys.stderr)
        return 0

    print(f"  [adapter] f64={f64}, usize={usize}, bool={bool_id}",
          file=sys.stderr)

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
    for lst in (trans.get("fun_decls") or []):
        if lst is not None:
            base_id = max(base_id, lst["def_id"])
    for lst in (trans.get("global_decls") or []):
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
        all_decls.append((did, crate_name, cname, [], {"Deduplicated": f64}))

    # Insert all at the correct position (reverse order for correct insertion)
    for did, crate_name, name, inputs, ret_ty in reversed(all_decls):
        decl = _make_fun_decl(did, crate_name, name, inputs, ret_ty)
        # Extend fun_decls list so that fun_decls[def_id] points to this decl
        fd_list = trans.setdefault("fun_decls", [])
        while len(fd_list) <= did:
            fd_list.append(None)
        fd_list[did] = decl
        od_list.insert(insert_at, {"Fun": {"NonRec": did}})
        trans.setdefault("item_names", []).append({
            "key": {"Fun": did}, "value": [{"Ident": [crate_name, 0]},
                                            {"Ident": [name, 0]}],
        })
        trans.setdefault("short_names", []).append({
            "key": {"Fun": did}, "value": [{"Ident": [name, 0]}],
        })

    print(f"  [adapter] registered {len(helper_ids)} helpers "
          f"({len(all_float_vals)} consts)", file=sys.stderr)

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

        # Use a generated span (not the original) to avoid confusing aeneas
        # which may look up function names from source spans.
        span = None  # _call_stmt will generate a dummy span

        # BinaryOp on f64
        if "BinaryOp" in rhs:
            binop = rhs["BinaryOp"]
            if len(binop) == 3:
                op, left, right = binop
                op_name = list(op.keys())[0] if isinstance(op, dict) else str(op)
                if op_name in BINOP_MAP:
                    lt = _operand_ty(left)
                    rt = _operand_ty(right)
                    if _is_f64_ty(lt, f64) or _is_f64_ty(rt, f64):
                        hname = BINOP_MAP[op_name]
                        if hname in helper_ids:
                            new_s = _call_stmt(helper_ids[hname], [left, right],
                                               target)
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
                if op_name == "Neg" and _is_f64_ty(_operand_ty(operand), f64):
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
        total_float += _expand_float_constants(stmts, f64, helper_ids, locals_info)
        _walk_stmts(stmts, _visitor)

    print(f"  [adapter] {total_float} float consts, {total_binop} binop/neg "
          "transformed", file=sys.stderr)

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
