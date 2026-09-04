//! Shared utilities for LLBC JSON manipulation.
//!
//! Used by f64_adapter and i64_adapter to avoid code duplication.
//!
//! Port of ``translation/tools/llbc_common.py``.

use serde_json::{json, Value};

// ── Type helpers ───────────────────────────────────────────────────────────────

/// Check whether a type node references any of the given dedup IDs.
pub fn is_target_ty(ty: &Value, type_ids: &std::collections::HashSet<i64>) -> bool {
    if let Some(v) = ty.get("Deduplicated").and_then(|v| v.as_i64()) {
        return type_ids.contains(&v);
    }
    if let Some(arr) = ty.get("HashConsedValue").and_then(|v| v.as_array()) {
        if let Some(id) = arr.first().and_then(|v| v.as_i64()) {
            return type_ids.contains(&id);
        }
    }
    false
}

// ── Type discovery ─────────────────────────────────────────────────────────────

/// Find the dedup ID for the Bool literal type from raw JSON.
pub fn discover_bool_id(raw_json: &str) -> Option<i64> {
    let re = regex::Regex::new(
        r#""HashConsedValue"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Literal"\s*:\s*"Bool"\s*\}"#,
    )
    .ok()?;
    let caps = re.captures(raw_json)?;
    caps.get(1)?.as_str().parse().ok()
}

/// Count occurrences of each Dedup ID in function locals.
pub fn count_dedup_in_locals(translated: &Value) -> std::collections::HashMap<i64, usize> {
    let mut counts: std::collections::HashMap<i64, usize> = std::collections::HashMap::new();
    for fd in translated.get("fun_decls").and_then(|v| v.as_array()).into_iter().flatten() {
        let body = match fd {
            Value::Null => continue,
            v => v.get("body"),
        };
        let body = match body {
            Some(Value::Object(o)) if o.contains_key("Structured") => o,
            _ => continue,
        };
        let locals = match body
            .get("Structured")
            .and_then(|s| s.get("locals"))
            .and_then(|l| l.get("locals"))
            .and_then(|l| l.as_array())
        {
            Some(l) => l,
            None => continue,
        };
        for loc in locals {
            if let Some(did) = loc
                .get("ty")
                .and_then(|t| t.get("Deduplicated"))
                .and_then(|v| v.as_i64())
            {
                *counts.entry(did).or_insert(0) += 1;
            }
        }
    }
    counts
}

// ── Function declaration builder ───────────────────────────────────────────────

/// Build a Foreign function declaration in charon's format.
pub fn make_fun_decl(
    def_id: i64,
    crate_name: &str,
    name: &str,
    inputs: Vec<Value>,
    output: Value,
    template: Option<&Value>,
) -> Value {
    if let Some(t) = template {
        let mut fd = t.clone();
        fd["def_id"] = json!(def_id);
        fd["item_meta"]["name"] = json!([
            {"Ident": [crate_name, 0]},
            {"Ident": [name, 0]},
        ]);
        if let Some(_span) = fd["item_meta"]["span"].get("data") {
            fd["item_meta"]["span"]["data"]["file_id"] = json!(0);
        }
        fd["item_meta"]["lang_item"] = Value::Null;
        fd["signature"]["inputs"] = json!(inputs);
        fd["signature"]["output"] = output;
        fd["body"] = json!("Opaque");
        return fd;
    }

    json!({
        "def_id": def_id,
        "item_meta": {
            "name": [
                {"Ident": [crate_name, 0]},
                {"Ident": [name, 0]},
            ],
            "span": {
                "data": {
                    "file_id": 0,
                    "beg": {"line": 0, "col": 0},
                    "end": {"line": 0, "col": 0},
                },
                "generated_from_span": null,
            },
            "source_text": null,
            "attr_info": {
                "attributes": [],
                "inline": null,
                "rename": null,
                "public": true,
            },
            "is_local": false,
            "opacity": "Foreign",
            "lang_item": null,
            "diagnostic_item": null,
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
            "is_unsafe": false,
            "abi": "Rust",
            "is_variadic": false,
            "inputs": inputs,
            "output": output,
        },
        "src": "Normal",
        "body": "Opaque",
    })
}

// ── Statement builders ─────────────────────────────────────────────────────────

/// Build a call statement: ``Call(func_id, args, dest)``.
pub fn call_stmt(func_id: i64, args: &[Value], dest: &Value, span: Option<&Value>) -> Value {
    let span = span.cloned().unwrap_or_else(zero_span);
    json!({
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
                "on_unwind": {"span": span, "id": 0, "statements": []},
            }
        },
        "comments_before": [],
    })
}

/// Generate a zero-location span.
pub fn zero_span() -> Value {
    json!({
        "data": {"file_id": 0, "beg": {"line": 0, "col": 0}, "end": {"line": 0, "col": 0}},
        "generated_from_span": null,
    })
}

/// Walk statements recursively, calling visitor on each.
pub fn walk_stmts<F>(stmts: &mut [Value], visitor: &mut F)
where
    F: FnMut(&mut Value) -> bool,
{
    for stmt in stmts.iter_mut() {
        visitor(stmt);
        recurse(stmt, visitor);
    }
}

fn recurse<F>(node: &mut Value, visitor: &mut F)
where
    F: FnMut(&mut Value) -> bool,
{
    match node {
        Value::Object(map) => {
            if let Some(stmts) = map.get_mut("statements").and_then(|v| v.as_array_mut()) {
                walk_stmts(stmts, visitor);
            }
            for (_k, v) in map.iter_mut() {
                recurse(v, visitor);
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                recurse(item, visitor);
            }
        }
        _ => {}
    }
}

/// Extract the type annotation from an operand (Move, Copy, etc.).
pub fn operand_ty(op: &Value) -> Option<&Value> {
    if let Value::Object(map) = op {
        if let Some((_kind, inner)) = map.iter().next() {
            if let Some(ty) = inner.get("ty") {
                return Some(ty);
            }
        }
    }
    None
}

// ── Switch condition transformation ────────────────────────────────────────────

/// Replace target-type BinaryOp in Switch(If) conditions with helper calls.
pub fn transform_switch_conditions(
    stmts: &mut Vec<Value>,
    locals_info: &mut Value,
    type_ids: &std::collections::HashSet<i64>,
    helper_ids: &std::collections::HashMap<String, i64>,
    binop_map: &std::collections::HashMap<String, String>,
) -> usize {
    let bool_dedup = find_bool_dedup(locals_info);
    let mut count = 0;

    fn next_local(locals_info: &Value) -> i64 {
        let locals = locals_info
            .get("locals")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|loc| loc.get("index").and_then(|v| v.as_i64()))
                    .max()
                    .unwrap_or(-1)
            })
            .unwrap_or(-1);
        locals + 1
    }

    fn walk(
        stmts: &mut Vec<Value>,
        locals_info: &mut Value,
        type_ids: &std::collections::HashSet<i64>,
        helper_ids: &std::collections::HashMap<String, i64>,
        binop_map: &std::collections::HashMap<String, String>,
        bool_dedup: Option<i64>,
        count: &mut usize,
    ) {
        let mut i = 0;
        while i < stmts.len() {
            let kind = stmts[i].get("kind").cloned();
            let kind_obj = match &kind {
                Some(Value::Object(o)) => o.clone(),
                _ => {
                    i += 1;
                    continue;
                }
            };

            if let Some(sw) = kind_obj.get("Switch") {
                if let Some(if_val) = sw.get("If") {
                    let if_arr = if_val.as_array().map(|a| a.clone());
                    if let Some(sw_arr) = if_arr {
                        if sw_arr.len() >= 3 {
                            let cond = &sw_arr[0];
                            if let Some(binop) = cond.get("BinaryOp") {
                                let binop_arr = binop.as_array().map(|a| a.clone());
                                if let Some(binop_arr) = binop_arr {
                                    if binop_arr.len() == 3 {
                                        let op = &binop_arr[0];
                                        let op_name: String = if let Value::Object(o) = op {
                                            o.keys().next().cloned().unwrap_or_default()
                                        } else if let Value::String(s) = op {
                                            s.clone()
                                        } else {
                                            String::new()
                                        };
                                        if let Some(hname) = binop_map.get(&op_name) {
                                            let lt = operand_ty(&binop_arr[1]);
                                            let rt = operand_ty(&binop_arr[2]);
                                            let lt_match = lt
                                                .map(|t| is_target_ty(t, type_ids))
                                                .unwrap_or(false);
                                            let rt_match = rt
                                                .map(|t| is_target_ty(t, type_ids))
                                                .unwrap_or(false);
                                            if (lt_match || rt_match) && bool_dedup.is_some() {
                                                if let Some(hid) = helper_ids.get(hname) {
                                                    let tmp_id = next_local(locals_info);
                                                    let tmp_ty = json!({"Deduplicated": bool_dedup.unwrap()});
                                                    if let Some(locals_arr) = locals_info
                                                        .get_mut("locals")
                                                        .and_then(|v| v.as_array_mut())
                                                    {
                                                        locals_arr.push(json!({
                                                            "index": tmp_id,
                                                            "name": null,
                                                            "span": zero_span(),
                                                            "ty": &tmp_ty,
                                                        }));
                                                    }
                                                    // Replace condition with Move(tmp)
                                                    let mut sw = stmts[i]
                                                        .get_mut("kind")
                                                        .and_then(|k| k.get_mut("Switch"))
                                                        .and_then(|s| s.get_mut("If"))
                                                        .and_then(|a| a.as_array_mut());
                                                    if let Some(sw_arr) = &mut sw {
                                                        sw_arr[0] = json!({
                                                            "Move": {
                                                                "kind": {"Local": tmp_id},
                                                                "ty": tmp_ty,
                                                            }
                                                        });
                                                    }
                                                    let call = call_stmt(
                                                        *hid,
                                                        &[binop_arr[1].clone(), binop_arr[2].clone()],
                                                        &json!({"kind": {"Local": tmp_id}, "ty": tmp_ty}),
                                                        None,
                                                    );
                                                    stmts.insert(i, call);
                                                    i += 1;
                                                    *count += 1;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            // Walk then/else branches
                            for bi in [1, 2] {
                                if let Some(b) = sw_arr.get(bi) {
                                    if let Some(sub_stmts) =
                                        b.get("statements").and_then(|v| v.as_array())
                                    {
                                        let mut sub = sub_stmts.clone();
                                        walk(
                                            &mut sub,
                                            locals_info,
                                            type_ids,
                                            helper_ids,
                                            binop_map,
                                            bool_dedup,
                                            count,
                                        );
                                        if let Some(sw_arr) = stmts[i]
                                            .get_mut("kind")
                                            .and_then(|k| k.get_mut("Switch"))
                                            .and_then(|s| s.get_mut("If"))
                                            .and_then(|a| a.as_array_mut())
                                        {
                                            sw_arr[bi]["statements"] = json!(sub);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if let Some(loop_val) = kind_obj.get("Loop") {
                if let Some(loop_stmts) =
                    loop_val.get("statements").and_then(|v| v.as_array())
                {
                    let mut sub = loop_stmts.clone();
                    walk(
                        &mut sub,
                        locals_info,
                        type_ids,
                        helper_ids,
                        binop_map,
                        bool_dedup,
                        count,
                    );
                    if let Some(lp) = stmts[i]
                        .get_mut("kind")
                        .and_then(|k| k.get_mut("Loop"))
                    {
                        lp["statements"] = json!(sub);
                    }
                }
            }
            i += 1;
        }
    }

    walk(
        stmts,
        locals_info,
        type_ids,
        helper_ids,
        binop_map,
        bool_dedup,
        &mut count,
    );
    count
}

fn find_bool_dedup(locals_info: &Value) -> Option<i64> {
    for loc in locals_info
        .get("locals")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let ty = loc.get("ty")?;
        let ty_str = ty.to_string();
        if ty_str.contains(r#""Literal":"Bool""#) || ty_str.contains(r#""Literal": "Bool""#) {
            if let Some(did) = ty.get("Deduplicated").and_then(|v| v.as_i64()) {
                return Some(did);
            }
            if let Some(arr) = ty.get("HashConsedValue").and_then(|v| v.as_array()) {
                if let Some(id) = arr.first().and_then(|v| v.as_i64()) {
                    return Some(id);
                }
            }
        }
    }
    None
}