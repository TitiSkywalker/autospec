//! I64 LLBC adapter: transform i64 operations in .llbc JSON before aeneas.
//!
//! Port of ``translation/tools/i64_adapter.py``.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::common::{
    call_stmt, count_dedup_in_locals, discover_bool_id, is_target_ty, make_fun_decl,
    operand_ty, transform_switch_conditions, walk_stmts,
};

// ── Operation maps ────────────────────────────────────────────────────────────

const I64_BINOP_MAP: &[(&str, &str)] = &[
    ("Add", "add"),
    ("Sub", "sub"),
    ("Mul", "mul"),
    ("Div", "div"),
    ("Rem", "rem"),
    ("Eq", "eq"),
    ("Ne", "ne"),
    ("Gt", "gt"),
    ("Lt", "lt"),
    ("Ge", "ge"),
    ("Le", "le"),
    ("BitAnd", "bitand"),
    ("BitOr", "bitor"),
    ("BitXor", "bitxor"),
    ("Shl", "shl"),
    ("Shr", "shr"),
    ("AddChecked", "add"),
    ("MulChecked", "mul"),
];

const I64_UNARY_MAP: &[(&str, &str)] = &[("Neg", "neg"), ("Not", "not")];

const I64_HELPER_SIGS: &[(&str, &[usize], usize)] = &[
    ("add", &[0, 0], 0),
    ("sub", &[0, 0], 0),
    ("mul", &[0, 0], 0),
    ("div", &[0, 0], 0),
    ("rem", &[0, 0], 0),
    ("eq", &[0, 0], 2),
    ("ne", &[0, 0], 2),
    ("gt", &[0, 0], 2),
    ("lt", &[0, 0], 2),
    ("ge", &[0, 0], 2),
    ("le", &[0, 0], 2),
    ("bitand", &[0, 0], 0),
    ("bitor", &[0, 0], 0),
    ("bitxor", &[0, 0], 0),
    ("shl", &[0, 1], 0),
    ("shr", &[0, 1], 0),
    ("neg", &[0], 0),
    ("not", &[0], 0),
    ("from_usize", &[1], 0),
];

// ── Type discovery ────────────────────────────────────────────────────────────

pub fn discover_i64_ids(raw_json: &str) -> HashSet<i64> {
    let mut ids = HashSet::new();
    let re = regex::Regex::new(
        r#""Value"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Scalar"\s*:\s*\{\s*"Integer"\s*:\s*\{\s*"Signed"\s*:\s*"I64"\s*\}\s*\}"#,
    )
    .unwrap();
    for caps in re.captures_iter(raw_json) {
        if let Some(m) = caps.get(1) {
            if let Ok(id) = m.as_str().parse() {
                ids.insert(id);
            }
        }
    }
    ids
}

pub fn discover_types(
    translated: &Value,
    i64_ids: &HashSet<i64>,
    raw_json: &str,
) -> (Option<i64>, Option<i64>, bool) {
    let mut usize_id: Option<i64> = None;
    let mut bool_id: Option<i64> = if !raw_json.is_empty() {
        discover_bool_id(raw_json)
    } else {
        None
    };

    let fun_decls = translated
        .get("fun_decls")
        .and_then(|v| v.as_array())
        .map(|a| a.iter().filter(|v| !v.is_null()));

    if let Some(fds) = fun_decls {
        for fd in fds {
            let sig = match fd.get("signature") {
                Some(s) => s,
                None => continue,
            };

            if usize_id.is_none() {
                if let Some(inputs) = sig.get("inputs").and_then(|v| v.as_array()) {
                    for inp in inputs {
                        if let Some(did) = inp.get("Deduplicated").and_then(|v| v.as_i64()) {
                            if !i64_ids.contains(&did) && Some(did) != bool_id {
                                usize_id = Some(did);
                                break;
                            }
                        }
                    }
                }
            }

            if usize_id.is_some() && bool_id.is_some() {
                break;
            }
        }
    }

    if bool_id.is_none() {
        bool_id = find_bool_in_locals(translated);
    }

    if usize_id.is_none() {
        let local_counts = count_dedup_in_locals(translated);
        let mut by_count: Vec<_> = local_counts.iter().collect();
        by_count.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
        for (did, _) in by_count {
            if !i64_ids.contains(did) && Some(*did) != bool_id {
                usize_id = Some(*did);
                break;
            }
        }
    }

    (usize_id, bool_id, !i64_ids.is_empty())
}

fn find_bool_in_locals(translated: &Value) -> Option<i64> {
    for fd in translated
        .get("fun_decls")
        .and_then(|v| v.as_array())
        .into_iter()
        .flatten()
    {
        let body = match fd.get("body") {
            Some(Value::Object(o)) if o.contains_key("Structured") => o,
            _ => continue,
        };
        for loc in body["Structured"]["locals"]
            .get("locals")
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
        {
            let ty = match loc.get("ty") {
                Some(t) => t,
                None => continue,
            };
            let ty_str = ty.to_string();
            if ty_str.contains(r#""Scalar":"Bool""#) || ty_str.contains(r#""Scalar": "Bool""#) {
                if let Some(did) = ty.get("Deduplicated").and_then(|v| v.as_i64()) {
                    return Some(did);
                }
                if let Some(arr) = ty.get("Value").and_then(|v| v.as_array()) {
                    if let Some(id) = arr.first().and_then(|v| v.as_i64()) {
                        return Some(id);
                    }
                }
            }
        }
    }
    None
}

// ── Main entry point ──────────────────────────────────────────────────────────

pub fn transform_i64_llbc(path: &Path, output: &Path) -> anyhow::Result<usize> {
    let raw_json = std::fs::read_to_string(path)?;
    let mut data: Value = serde_json::from_str(&raw_json)?;

    let trans = data
        .get_mut("translated")
        .ok_or_else(|| anyhow::anyhow!("missing 'translated' key"))?;

    // ── Phase 1: analysis ──────────────────────────────────────────────────

    let i64_ids = discover_i64_ids(&raw_json);
    let (usize_id, bool_id, _has_i64) = discover_types(trans, &i64_ids, &raw_json);

    if i64_ids.is_empty() {
        crate::log::dbg("[adapter] no i64 type found — nothing to do");
        return Ok(0);
    }

    let local_counts = count_dedup_in_locals(trans);
    let primary_i64 = i64_ids
        .iter()
        .max_by_key(|d| local_counts.get(d).copied().unwrap_or(0))
        .copied()
        .unwrap_or(0);
    let types_list: [Option<i64>; 3] = [Some(primary_i64), usize_id, bool_id];

    crate::log::dbg(&format!(
        "[adapter] i64={} (ids={:?}), usize={:?}, bool={:?}",
        primary_i64,
        Vec::from_iter(i64_ids.iter().copied()),
        usize_id,
        bool_id,
    ));

    // Compute base_id
    let mut base_id: i64 = 0;
    for fd in trans.get("fun_decls").and_then(|v| v.as_array()).into_iter().flatten() {
        if !fd.is_null() {
            base_id = base_id.max(fd["def_id"].as_i64().unwrap_or(0));
        }
    }
    for gd in trans.get("global_decls").and_then(|v| v.as_array()).into_iter().flatten() {
        if !gd.is_null() {
            base_id = base_id.max(gd["def_id"].as_i64().unwrap_or(0));
        }
    }
    base_id += 1;

    // Find insertion point
    let od_list = trans.get("ordered_decls").and_then(|v| v.as_array());
    let mut insert_at = od_list.map(|a| a.len()).unwrap_or(0);
    if let Some(od_list) = od_list {
        for (idx, od) in od_list.iter().enumerate() {
            if let Some(fid) = od.get("Fun").and_then(|f| f.get("NonRec")).and_then(|v| v.as_i64()) {
                for fd in trans.get("fun_decls").and_then(|v| v.as_array()).into_iter().flatten() {
                    if !fd.is_null() && fd["def_id"].as_i64() == Some(fid) {
                        if fd["item_meta"]["opacity"].as_str() == Some("Transparent") {
                            insert_at = idx;
                            break;
                        }
                    }
                }
                if insert_at < od_list.len() {
                    break;
                }
            }
        }
    }

    // Build all helper declarations
    let mut helper_ids: HashMap<String, i64> = HashMap::new();
    let mut all_decls: Vec<(i64, String, String, Vec<Value>, Value)> = Vec::new();
    let crate_name = "i64_helpers";

    for (i, (name, param_idxs, ret_idx)) in I64_HELPER_SIGS.iter().enumerate() {
        let types_ok = param_idxs
            .iter()
            .chain(std::iter::once(ret_idx))
            .all(|idx| types_list[*idx].is_some());
        if !types_ok {
            continue;
        }
        let did = base_id + i as i64;
        helper_ids.insert(name.to_string(), did);
        let inputs: Vec<Value> = param_idxs
            .iter()
            .map(|idx| serde_json::json!({"Deduplicated": types_list[*idx].unwrap()}))
            .collect();
        let ret_ty = serde_json::json!({"Deduplicated": types_list[*ret_idx].unwrap()});
        all_decls.push((did, crate_name.to_string(), name.to_string(), inputs, ret_ty));
    }

    // Clone fun_decls for body processing
    let mut fun_decls_to_process: Vec<Value> = trans
        .get("fun_decls")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // ── Phase 2: mutation ──────────────────────────────────────────────────

    let trans_map = trans.as_object_mut().unwrap();

    {
        let fd_list = trans_map
            .get_mut("fun_decls")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| anyhow::anyhow!("missing fun_decls"))?;
        for (did, cname, name, inputs, ret_ty) in all_decls.iter().rev() {
            let decl = make_fun_decl(*did, cname, name, inputs.clone(), ret_ty.clone(), None);
            while fd_list.len() <= *did as usize {
                fd_list.push(Value::Null);
            }
            fd_list[*did as usize] = decl;
        }
    }

    {
        let od_list = trans_map
            .get_mut("ordered_decls")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| anyhow::anyhow!("missing ordered_decls"))?;
        for (did, _, _, _, _) in all_decls.iter().rev() {
            od_list.insert(insert_at, serde_json::json!({"Fun": {"NonRec": did}}));
        }
    }

    if let Some(inames) = trans_map
        .get_mut("item_names")
        .and_then(|v| v.as_array_mut())
    {
        for (did, cname, name, _, _) in all_decls.iter().rev() {
            inames.push(serde_json::json!({
                "key": {"Fun": did},
                "value": [{"Ident": [cname, 0]}, {"Ident": [name, 0]}],
            }));
        }
    }

    if let Some(snames) = trans_map
        .get_mut("short_names")
        .and_then(|v| v.as_array_mut())
    {
        for (did, _, name, _, _) in all_decls.iter().rev() {
            snames.push(serde_json::json!({
                "key": {"Fun": did},
                "value": [{"Ident": [name, 0]}],
            }));
        }
    }

    crate::log::dbg(&format!(
        "[adapter] registered {} helpers",
        helper_ids.len(),
    ));

    // ── Phase 3: transform function bodies ─────────────────────────────────

    let binop_map: HashMap<String, String> = I64_BINOP_MAP
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let unary_map: HashMap<String, String> = I64_UNARY_MAP
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let mut total_binop = 0usize;

    let binop_map_ref = &binop_map;
    let unary_map_ref = &unary_map;
    let i64_ids_ref = &i64_ids;
    let helper_ids_ref = &helper_ids;

    for fd_idx in 0..fun_decls_to_process.len() {
        if fun_decls_to_process[fd_idx].is_null()
            || fun_decls_to_process[fd_idx].get("body").is_none()
        {
            continue;
        }
        let body = &fun_decls_to_process[fd_idx]["body"];
        if !body.is_object() || !body.as_object().unwrap().contains_key("Structured") {
            continue;
        }

        let mut stmts = body["Structured"]["body"]["statements"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut locals_info = body["Structured"]["locals"].clone();

        let mut visitor = |stmt: &mut Value| -> bool {
            let kind = match stmt.get("kind").and_then(|v| v.as_object()) {
                Some(k) => k.clone(),
                None => return false,
            };
            let assign = match kind.get("Assign").and_then(|v| v.as_array()) {
                Some(a) if a.len() == 2 => a,
                _ => return false,
            };
            let target = &assign[0];
            let rhs = &assign[1];

            // BinaryOp on i64
            if let Some(binop) = rhs.get("BinaryOp").and_then(|v| v.as_array()) {
                if binop.len() == 3 {
                    let op = &binop[0];
                    let op_name: String = if let Value::Object(o) = op {
                        o.keys().next().cloned().unwrap_or_default()
                    } else if let Value::String(s) = op {
                        s.clone()
                    } else {
                        return false;
                    };
                    if let Some(hname) = binop_map_ref.get(&op_name) {
                        let lt = operand_ty(&binop[1]);
                        let rt = operand_ty(&binop[2]);
                        if lt.map(|t| is_target_ty(t, i64_ids_ref)).unwrap_or(false)
                            || rt.map(|t| is_target_ty(t, i64_ids_ref)).unwrap_or(false)
                        {
                            if let Some(hid) = helper_ids_ref.get(hname) {
                                let new_s = call_stmt(
                                    *hid,
                                    &[binop[1].clone(), binop[2].clone()],
                                    target,
                                    None,
                                );
                                stmt.as_object_mut().unwrap().clear();
                                stmt.as_object_mut()
                                    .unwrap()
                                    .extend(new_s.as_object().unwrap().clone());
                                total_binop += 1;
                                return true;
                            }
                        }
                    }
                }
            }

            // UnaryOp on i64
            if let Some(unop) = rhs.get("UnaryOp").and_then(|v| v.as_array()) {
                if unop.len() == 2 {
                    let op = &unop[0];
                    let op_name: String = if let Value::Object(o) = op {
                        o.keys().next().cloned().unwrap_or_default()
                    } else if let Value::String(s) = op {
                        s.clone()
                    } else {
                        return false;
                    };
                    if let Some(hname) = unary_map_ref.get(&op_name) {
                        if let Some(operand) = unop.get(1) {
                            if operand_ty(operand)
                                .map(|t| is_target_ty(t, i64_ids_ref))
                                .unwrap_or(false)
                            {
                                if let Some(hid) = helper_ids_ref.get(hname) {
                                    let new_s =
                                        call_stmt(*hid, &[operand.clone()], target, None);
                                    stmt.as_object_mut().unwrap().clear();
                                    stmt.as_object_mut()
                                        .unwrap()
                                        .extend(new_s.as_object().unwrap().clone());
                                    total_binop += 1;
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
            false
        };

        walk_stmts(&mut stmts, &mut visitor);
        total_binop += transform_switch_conditions(
            &mut stmts,
            &mut locals_info,
            &i64_ids,
            &helper_ids,
            &binop_map,
        );

        fun_decls_to_process[fd_idx]["body"]["Structured"]["body"]["statements"] =
            serde_json::json!(stmts);
        fun_decls_to_process[fd_idx]["body"]["Structured"]["locals"] = locals_info;
    }

    // Write back processed fun_decls
    {
        let fd_list = trans_map
            .get_mut("fun_decls")
            .and_then(|v| v.as_array_mut())
            .unwrap();
        for (idx, fd) in fun_decls_to_process.iter().enumerate() {
            if idx < fd_list.len() && !fd.is_null() {
                fd_list[idx] = fd.clone();
            }
        }
    }

    crate::log::dbg(&format!(
        "[adapter] {} binop/unop transformed",
        total_binop,
    ));

    let out_str = serde_json::to_string_pretty(&data)?;
    std::fs::write(output, out_str)?;
    Ok(total_binop)
}