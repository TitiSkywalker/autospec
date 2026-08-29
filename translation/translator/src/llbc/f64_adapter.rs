//! LLBC adapter: transform f64 operations in .llbc JSON before aeneas processes it.
//!
//! Port of ``translation/tools/f64_adapter.py``.

use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::common::{
    call_stmt, count_dedup_in_locals, discover_bool_id, is_target_ty, make_fun_decl,
    operand_ty, transform_switch_conditions, walk_stmts,
};

// ── Operation maps ────────────────────────────────────────────────────────────

const BINOP_MAP: &[(&str, &str)] = &[
    ("Add", "add"),
    ("Sub", "sub"),
    ("Mul", "mul"),
    ("Div", "div"),
    ("Eq", "eq"),
    ("Gt", "gt"),
    ("Lt", "lt"),
    ("Ge", "ge"),
    ("Le", "le"),
];

const HELPER_SIGS: &[(&str, &[usize], usize)] = &[
    ("add", &[0, 0], 0),
    ("sub", &[0, 0], 0),
    ("mul", &[0, 0], 0),
    ("div", &[0, 0], 0),
    ("neg", &[0], 0),
    ("abs", &[0], 0),
    ("eq", &[0, 0], 2),
    ("gt", &[0, 0], 2),
    ("lt", &[0, 0], 2),
    ("ge", &[0, 0], 2),
    ("le", &[0, 0], 2),
    ("from_usize", &[1], 0),
];

// ── Type discovery ────────────────────────────────────────────────────────────

/// Find all dedup IDs whose definition is ``Literal(Float(F64))``.
pub fn discover_f64_ids(raw_json: &str) -> HashSet<i64> {
    let mut ids = HashSet::new();
    let re = regex::Regex::new(
        r#""HashConsedValue"\s*:\s*\[(\d+)\s*,\s*\{?\s*"Literal"\s*:\s*\{\s*"Float"\s*:\s*"F64"\s*\}"#,
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
    f64_ids: &HashSet<i64>,
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
                            if !f64_ids.contains(&did) && Some(did) != bool_id {
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

    // bool fallback: scan all locals
    if bool_id.is_none() {
        bool_id = find_bool_in_locals(translated);
    }

    // usize fallback: most common dedup in locals
    if usize_id.is_none() {
        let local_counts = count_dedup_in_locals(translated);
        let mut by_count: Vec<_> = local_counts.iter().collect();
        by_count.sort_by_key(|(_, c)| std::cmp::Reverse(**c));
        for (did, _) in by_count {
            if !f64_ids.contains(did) && Some(*did) != bool_id {
                usize_id = Some(*did);
                break;
            }
        }
    }

    (usize_id, bool_id, !f64_ids.is_empty())
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
    }
    None
}

// ── Float constant expansion ──────────────────────────────────────────────────

/// Convert a float literal text to a safe identifier suffix.
pub fn float_to_name(value: &str) -> String {
    let value = if value.contains('E') || value.contains('e') {
        normalize_scientific(value)
    } else {
        value.to_string()
    };
    "const_".to_string()
        + &value
            .replace('.', "_")
            .replace('-', "neg_")
}

fn normalize_scientific(value: &str) -> String {
    // Simple manual normalization: parse "1.0E-10" → "0.0000000001"
    // Python uses Decimal + format(d, 'f') — we replicate that via string parsing.
    let upper = value.to_uppercase();
    if let Some(e_pos) = upper.find('E') {
        let mantissa = &value[..e_pos];
        let exponent: i32 = match value[e_pos + 1..].parse() {
            Ok(e) => e,
            Err(_) => return value.to_string(),
        };
        // Split mantissa into integer and fractional parts
        let (int_part, frac_part) = match mantissa.split_once('.') {
            Some((i, f)) => (i, f),
            None => (mantissa, ""),
        };
        let combined = format!("{}{}", int_part, frac_part);
        let frac_len = frac_part.len() as i32;
        let num_digits = combined.len();

        if exponent >= 0 {
            let shift = exponent - frac_len;
            if shift >= 0 {
                format!("{}{}", combined, "0".repeat(shift as usize))
            } else {
                let dot_pos = num_digits as i32 + shift;
                if dot_pos <= 0 {
                    format!("0.{}{}", "0".repeat((-dot_pos) as usize), combined)
                } else {
                    let dot_pos = dot_pos as usize;
                    format!("{}.{}", &combined[..dot_pos], &combined[dot_pos..])
                }
            }
        } else {
            let abs_exp = (-exponent) as usize;
            if abs_exp >= frac_len as usize {
                format!("0.{}{}", "0".repeat(abs_exp - frac_len as usize), combined)
            } else {
                let dot_pos = num_digits - abs_exp;
                format!("{}.{}", &combined[..dot_pos], &combined[dot_pos..])
            }
        }
    } else {
        value.to_string()
    }
}

fn collect_float_values(stmts: &[Value]) -> HashSet<String> {
    let mut vals = HashSet::new();
    fn rec(node: &Value, vals: &mut HashSet<String>) {
        match node {
            Value::Object(map) => {
                if let Some(float_val) = map.get("Float") {
                    if let Some(float_obj) = float_val.as_object() {
                        if float_obj.get("ty").and_then(|v| v.as_str()) == Some("F64") {
                            if let Some(v) = float_obj.get("value").and_then(|v| v.as_str()) {
                                vals.insert(v.to_string());
                            }
                        }
                    }
                }
                for v in map.values() {
                    rec(v, vals);
                }
            }
            Value::Array(arr) => {
                for item in arr {
                    rec(item, vals);
                }
            }
            _ => {}
        }
    }
    for stmt in stmts {
        rec(stmt, &mut vals);
    }
    vals
}

/// Hoist Float(F64) constants to const_X_Y() calls at top of block.
fn expand_float_constants(
    stmts: &mut Vec<Value>,
    f64: i64,
    const_ids: &HashMap<String, i64>,
    locals_info: &mut Value,
) -> usize {
    let all_vals = collect_float_values(stmts);
    if all_vals.is_empty() {
        return 0;
    }

    let span0 = stmts
        .first()
        .and_then(|s| s.get("span"))
        .cloned()
        .unwrap_or_else(|| {
            serde_json::json!({
                "data": {
                    "file_id": 0,
                    "beg": {"line": 0, "col": 0},
                    "end": {"line": 0, "col": 0},
                },
                "generated_from_span": null,
            })
        });

    let mut val_to_local: HashMap<String, i64> = HashMap::new();
    let mut init_stmts: Vec<Value> = Vec::new();

    let mut sorted_vals: Vec<&String> = all_vals.iter().collect();
    sorted_vals.sort();
    for val in sorted_vals {
        let cname = float_to_name(val);
        if !const_ids.contains_key(&cname) {
            continue;
        }
        let local_idx = locals_info
            .get("locals")
            .and_then(|v| v.as_array())
            .map(|a| a.len() as i64)
            .unwrap_or(0);
        val_to_local.insert(val.to_string(), local_idx);

        if let Some(locals_arr) = locals_info.get_mut("locals").and_then(|v| v.as_array_mut()) {
            locals_arr.push(serde_json::json!({
                "index": local_idx,
                "name": format!("_f64_{}", cname),
                "span": span0,
                "ty": {"Deduplicated": f64},
            }));
        }

        init_stmts.push(call_stmt(
            const_ids[&cname],
            &[],
            &serde_json::json!({"kind": {"Local": local_idx}, "ty": {"Deduplicated": f64}}),
            Some(&span0),
        ));
    }

    if !init_stmts.is_empty() {
        init_stmts.append(stmts);
        *stmts = init_stmts;
    }

    for stmt in stmts.iter_mut() {
        replace_float_nodes(stmt, &val_to_local, f64);
    }
    all_vals.len()
}

fn replace_float_nodes(node: &mut Value, val_to_local: &HashMap<String, i64>, f64: i64) {
    match node {
        Value::Object(map) => {
            if let Some(const_val) = map.get("Const").cloned() {
                if let Some(kind) = const_val.get("kind") {
                    if let Some(lit) = kind.get("Literal") {
                        let val_str = extract_float_value(lit);
                        if let Some(val) = val_str {
                            if let Some(local_idx) = val_to_local.get(&val) {
                                map.clear();
                                map.insert(
                                    "Copy".to_string(),
                                    serde_json::json!({
                                        "kind": {"Local": local_idx},
                                        "ty": {"Deduplicated": f64},
                                    }),
                                );
                                return;
                            }
                        }
                    }
                }
            }
            for (_k, v) in map.iter_mut() {
                replace_float_nodes(v, val_to_local, f64);
            }
        }
        Value::Array(arr) => {
            for i in 0..arr.len() {
                let item = &arr[i];
                if let Some(const_val) = item.get("Const").cloned() {
                    if let Some(kind) = const_val.get("kind") {
                        if let Some(lit) = kind.get("Literal") {
                            let val_str = extract_float_value(lit);
                            if let Some(val) = val_str {
                                if let Some(local_idx) = val_to_local.get(&val) {
                                    arr[i] = serde_json::json!({
                                        "Copy": {
                                            "kind": {"Local": local_idx},
                                            "ty": {"Deduplicated": f64},
                                        }
                                    });
                                }
                            }
                        }
                    }
                }
                replace_float_nodes(&mut arr[i], val_to_local, f64);
            }
        }
        _ => {}
    }
}

fn extract_float_value(lit: &Value) -> Option<String> {
    if let Some(float_obj) = lit.get("Float") {
        if float_obj.get("ty").and_then(|v| v.as_str()) == Some("F64") {
            return float_obj.get("value").and_then(|v| v.as_str()).map(|s| s.to_string());
        }
    }
    if let Some(scalar) = lit.get("Scalar") {
        if let Some(float_obj) = scalar.get("Float") {
            if float_obj.get("ty").and_then(|v| v.as_str()) == Some("F64") {
                return float_obj
                    .get("value")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
            }
        }
    }
    None
}

// ── Main entry point ──────────────────────────────────────────────────────────

/// Transform f64 operations in an LLBC JSON file. Returns the total number of
/// transformations applied.
pub fn transform_llbc(path: &Path, output: &Path) -> anyhow::Result<usize> {
    let raw_json = std::fs::read_to_string(path)?;
    let mut data: Value = serde_json::from_str(&raw_json)?;

    let trans = data
        .get_mut("translated")
        .ok_or_else(|| anyhow::anyhow!("missing 'translated' key"))?;

    // ── Phase 1: analysis (immutable reads) ────────────────────────────────

    let f64_ids = discover_f64_ids(&raw_json);
    let (usize_id, bool_id, _has_f64) = discover_types(trans, &f64_ids, &raw_json);

    if f64_ids.is_empty() {
        crate::log::dbg("[adapter] no f64 type found — nothing to do");
        return Ok(0);
    }

    let local_counts = count_dedup_in_locals(trans);
    let primary_f64 = f64_ids
        .iter()
        .max_by_key(|d| local_counts.get(d).copied().unwrap_or(0))
        .copied()
        .unwrap_or(0);
    let types_list: [Option<i64>; 3] = [Some(primary_f64), usize_id, bool_id];

    crate::log::dbg(&format!(
        "[adapter] f64={} (ids={:?}), usize={:?}, bool={:?}",
        primary_f64,
        Vec::from_iter(f64_ids.iter().copied()),
        usize_id,
        bool_id,
    ));

    // Collect all float constants
    let mut all_float_vals: HashSet<String> = HashSet::new();
    for fd in trans.get("fun_decls").and_then(|v| v.as_array()).into_iter().flatten() {
        if fd.is_null() || fd.get("body").is_none() {
            continue;
        }
        if let Some(stmts) = fd["body"]["Structured"]["body"]["statements"].as_array() {
            all_float_vals.extend(collect_float_values(stmts));
        }
    }

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
    let crate_name = "f64_helpers";

    for (i, (name, param_idxs, ret_idx)) in HELPER_SIGS.iter().enumerate() {
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

    let const_start = base_id + HELPER_SIGS.len() as i64;
    let mut sorted_float_vals: Vec<&String> = all_float_vals.iter().collect();
    sorted_float_vals.sort();
    for (j, val) in sorted_float_vals.iter().enumerate() {
        let did = const_start + j as i64;
        let cname = float_to_name(val);
        helper_ids.insert(cname.clone(), did);
        all_decls.push((
            did,
            crate_name.to_string(),
            cname,
            vec![],
            serde_json::json!({"Deduplicated": primary_f64}),
        ));
    }

    // Clone the fun_decls array for body transformation (we'll process and
    // write back later, after all the declaration insertions).
    let mut fun_decls_to_process: Vec<Value> = trans
        .get("fun_decls")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();

    // ── Phase 2: mutation (sequential, one mutable field at a time) ────────

    let trans_map = trans.as_object_mut().unwrap();

    // Extend fun_decls
    {
        let fd_list = trans_map
            .get_mut("fun_decls")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| anyhow::anyhow!("missing fun_decls"))?;
        for (did, crate_name, name, inputs, ret_ty) in all_decls.iter().rev() {
            let decl = make_fun_decl(*did, crate_name, name, inputs.clone(), ret_ty.clone(), None);
            while fd_list.len() <= *did as usize {
                fd_list.push(Value::Null);
            }
            fd_list[*did as usize] = decl;
        }
    }

    // Insert into ordered_decls
    {
        let od_list = trans_map
            .get_mut("ordered_decls")
            .and_then(|v| v.as_array_mut())
            .ok_or_else(|| anyhow::anyhow!("missing ordered_decls"))?;
        for (did, _, _, _, _) in all_decls.iter().rev() {
            od_list.insert(insert_at, serde_json::json!({"Fun": {"NonRec": did}}));
        }
    }

    // Push to item_names
    {
        let item_names = trans_map
            .get_mut("item_names")
            .and_then(|v| v.as_array_mut());
        if let Some(inames) = item_names {
            for (did, crate_name, name, _, _) in all_decls.iter().rev() {
                inames.push(serde_json::json!({
                    "key": {"Fun": did},
                    "value": [{"Ident": [crate_name, 0]}, {"Ident": [name, 0]}],
                }));
            }
        }
    }

    // Push to short_names
    {
        let short_names = trans_map
            .get_mut("short_names")
            .and_then(|v| v.as_array_mut());
        if let Some(snames) = short_names {
            for (did, _, name, _, _) in all_decls.iter().rev() {
                snames.push(serde_json::json!({
                    "key": {"Fun": did},
                    "value": [{"Ident": [name, 0]}],
                }));
            }
        }
    }

    crate::log::dbg(&format!(
        "[adapter] registered {} helpers ({} consts)",
        helper_ids.len(),
        all_float_vals.len(),
    ));

    // ── Phase 3: transform function bodies ─────────────────────────────────

    let binop_map: HashMap<String, String> = BINOP_MAP
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
    let mut total_binop = 0usize;
    let mut total_float = 0usize;

    let binop_map_ref = &binop_map;
    let f64_ids_ref = &f64_ids;
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

        total_float +=
            expand_float_constants(&mut stmts, primary_f64, &helper_ids, &mut locals_info);

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
                        if lt.map(|t| is_target_ty(t, f64_ids_ref)).unwrap_or(false)
                            || rt.map(|t| is_target_ty(t, f64_ids_ref)).unwrap_or(false)
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
                    if op_name == "Neg" {
                        if let Some(operand) = unop.get(1) {
                            if operand_ty(operand)
                                .map(|t| is_target_ty(t, f64_ids_ref))
                                .unwrap_or(false)
                            {
                                if let Some(hid) = helper_ids_ref.get("neg") {
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
            &f64_ids,
            &helper_ids,
            &binop_map,
        );

        // Write back
        fun_decls_to_process[fd_idx]["body"]["Structured"]["body"]["statements"] =
            serde_json::json!(stmts);
        fun_decls_to_process[fd_idx]["body"]["Structured"]["locals"] = locals_info;
    }

    // Write back the processed fun_decls
    {
        let fd_list = trans_map
            .get_mut("fun_decls")
            .and_then(|v| v.as_array_mut())
            .unwrap();
        // Only update indices that exist in our processed list
        for (idx, fd) in fun_decls_to_process.iter().enumerate() {
            if idx < fd_list.len() && !fd.is_null() {
                fd_list[idx] = fd.clone();
            }
        }
    }

    crate::log::dbg(&format!(
        "[adapter] {} float consts, {} binop/neg transformed",
        total_float, total_binop,
    ));

    let out_str = serde_json::to_string_pretty(&data)?;
    std::fs::write(output, out_str)?;
    Ok(total_float + total_binop)
}