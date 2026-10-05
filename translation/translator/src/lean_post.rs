//! Post-process generated Lean files: F64→Rat/ℝ, inline helpers, write output.
//!
//! Port of ``translation/pipeline/lean_post.py``.

use std::path::Path;

use regex::Regex;

use crate::lean_inline;
use crate::log;

// ── exceptions ─────────────────────────────────────────────────────────────────

/// Rat float-model cannot handle non-computable operations like sqrt.
#[derive(Debug)]
pub struct NonComputableError {
    pub functions: Vec<String>,
}

impl std::fmt::Display for NonComputableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} not computable in Rat mode", self.functions.join(", "))
    }
}

impl std::error::Error for NonComputableError {}

// ── integer types table ────────────────────────────────────────────────────────

struct IntType {
    ty: &'static str,
    suffix: &'static str,
    width: Option<u32>,
    is_signed: bool,
}

const INT_TYPES: &[IntType] = &[
    IntType { ty: "Std.I8",   suffix: "#i8",    width: Some(8),  is_signed: true  },
    IntType { ty: "Std.I16",  suffix: "#i16",   width: Some(16), is_signed: true  },
    IntType { ty: "Std.I32",  suffix: "#i32",   width: Some(32), is_signed: true  },
    IntType { ty: "Std.I64",  suffix: "#i64",   width: Some(64), is_signed: true  },
    IntType { ty: "Std.I128", suffix: "#i128",  width: Some(128),is_signed: true  },
    IntType { ty: "Std.Isize",suffix: "#isize", width: None,     is_signed: true  },
    IntType { ty: "Std.U8",   suffix: "#u8",    width: Some(8),  is_signed: false },
    IntType { ty: "Std.U16",  suffix: "#u16",   width: Some(16), is_signed: false },
    IntType { ty: "Std.U32",  suffix: "#u32",   width: Some(32), is_signed: false },
    IntType { ty: "Std.U64",  suffix: "#u64",   width: Some(64), is_signed: false },
    IntType { ty: "Std.U128", suffix: "#u128",  width: Some(128),is_signed: false },
];

fn int_target_type(is_signed: bool, width: Option<u32>, model: &str) -> String {
    match model {
        "int" => "Int".to_string(),
        "nat" => {
            if is_signed { "Int".to_string() } else { "Nat".to_string() }
        }
        "bitvec" => {
            if is_signed {
                "Int".to_string()
            } else {
                format!("BitVec {}", width.unwrap_or(64))
            }
        }
        _ => "Int".to_string(),
    }
}

fn int_helper_stem(target_type: &str) -> String {
    target_type.replace(' ', "")
}

fn type_arg(target_type: &str) -> String {
    if target_type.contains(' ') {
        format!("({})", target_type)
    } else {
        target_type.to_string()
    }
}

// ── f64 → Rat / ℝ ──────────────────────────────────────────────────────────────

const F64_SKIP_PATTERNS: &[&str] = &[
    "f64_helpers::",
    "core::f64::",
    "core::clone::impls",
    "Trait implementation:",
];

fn skip_f64_blocks(lines: &[&str]) -> Vec<usize> {
    let mut skip = vec![false; lines.len()];
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i];
        if line.starts_with("/-- [")
            || (line.starts_with("/-- Trait") && line.contains("Clone for f64"))
        {
            if F64_SKIP_PATTERNS.iter().any(|p| line.contains(p)) {
                let start = i;
                // Skip doc comment lines
                while i < lines.len() && !lines[i].trim_end().ends_with("-/") {
                    i += 1;
                }
                i += 1;
                // Skip attributes
                while i < lines.len() && lines[i].starts_with("@[") {
                    i += 1;
                }
                // Skip the definition body
                while i < lines.len() && !lines[i].trim().is_empty() {
                    i += 1;
                }
                if i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }
                for j in start..i.min(lines.len()) {
                    skip[j] = true;
                }
                continue;
            }
        }
        i += 1;
    }
    skip.iter().enumerate().filter(|(_, v)| **v).map(|(i, _)| i).collect()
}

fn replace_f64_base(content: &str, target: &str, clone_name: &str) -> String {
    if !content.contains("F64") {
        return content.to_string();
    }

    let mut result = content.to_string();
    result = result.replace(
        "F64.Insts.CoreCloneClone.clone",
        &format!("{}.clone", clone_name),
    );
    result = result.replace("F64.Insts.CoreCloneClone", clone_name);

    let lines: Vec<&str> = result.lines().collect();
    let skip: std::collections::HashSet<usize> = skip_f64_blocks(&lines).into_iter().collect();

    let mut out: Vec<String> = Vec::new();
    for (j, line) in lines.iter().enumerate() {
        if skip.contains(&j) {
            continue;
        }
        out.push(line.replace("F64", target));
    }
    result = out.join("\n");

    // Replace abs call sites
    let re = Regex::new(r"core\.f64\.\S+\.abs\b").unwrap();
    result = re.replace_all(&result, "f64_helpers.abs").to_string();

    // is_nan / is_infinite → always false
    let re = Regex::new(r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_nan\s+(\S+)").unwrap();
    result = re.replace_all(&result, "let $1 := false").to_string();
    let re = Regex::new(r"\blet\s+(\w+)\s+←\s+core\.f64\.\S+\.is_infinite\s+(\S+)").unwrap();
    result = re.replace_all(&result, "let $1 := false").to_string();

    // Add Clone helper if referenced but not defined
    if result.contains(clone_name) && !result.contains(&format!("def {}.clone", clone_name)) {
        let ta = type_arg(target);
        let clone_def = format!(
            "/- F64 replaced with {} by autospec -/\n\
             def {}.clone (x : {}) : Result {} := ok x\n\
             \n\
             @[reducible]\n\
             def {} : core.clone.Clone {} := {{\n  clone := {}.clone\n}}\n",
            target, clone_name, ta, ta, clone_name, ta, clone_name
        );
        let re = Regex::new(r"(namespace \w+\n)").unwrap();
        if let Some(caps) = re.captures(&result) {
            let ns = caps.get(1).unwrap().as_str().to_string();
            result = result.replace(&ns, &format!("{}\n{}", ns, clone_def));
        }
    }

    result
}

pub fn lean_f64_to_rat(content: &str) -> Result<String, NonComputableError> {
    let mut result = content.replace("noncomputable section\n", "");
    result = replace_f64_base(&result, "Rat", "RatClone");

    // Mark remaining axioms as noncomputable
    // Use line-based approach (no lookahead in Rust regex)
    let mut out_lines: Vec<String> = Vec::new();
    for line in result.lines() {
        if line.trim().starts_with("axiom ") && !line.contains("noncomputable") {
            out_lines.push(format!("noncomputable {}", line));
        } else {
            out_lines.push(line.to_string());
        }
    }
    result = out_lines.join("\n");

    // Check for non-computable functions
    let re = Regex::new(r"std\.f64\.Rat\.(\w+)").unwrap();
    let non_comp: Vec<String> = re
        .captures_iter(&result)
        .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
        .collect();
    if !non_comp.is_empty() {
        let mut funcs: Vec<String> = non_comp.into_iter().collect();
        funcs.sort();
        funcs.dedup();
        return Err(NonComputableError { functions: funcs });
    }

    Ok(result)
}

pub fn lean_f64_to_real(content: &str) -> String {
    let mut result = replace_f64_base(content, "ℝ", "RealClone");

    // Add Mathlib import and noncomputable section. New aeneas emits a
    // `module` + `public import Aeneas` header, so insert the (top-level)
    // `import Mathlib` before the `module` declaration.
    let re = Regex::new(r"module\npublic import Aeneas\n").unwrap();
    result = re
        .replace(&result, "import Mathlib\n\nmodule\npublic import Aeneas\n")
        .to_string();
    result = result.replace("noncomputable section\n", "");
    result = result.replace(
        "set_option linter.style.nameCheck false\n",
        "set_option linter.style.nameCheck false\nnoncomputable section\n",
    );

    // Replace transcendental functions
    result = replace_transcendental(&result, "sqrt", "Real.sqrt");
    result = replace_transcendental(&result, "exp", "Real.exp");
    result = replace_transcendental(&result, "ln", "Real.log");

    // Replace f64 constants
    result = replace_f64_const(&result, "core.f64.consts.PI", "Real.pi");

    result
}

fn replace_transcendental(result: &str, func_name: &str, mathlib_name: &str) -> String {
    let marker = format!("std.f64.ℝ.{}", func_name);
    if !result.contains(&marker) {
        return result.to_string();
    }

    let mut content = result.to_string();

    // Remove the axiom definition block.
    // Regex is line-matched to Python: literal spaces, not \s+/\s*,
    // so that adjacent blank lines are not consumed.
    let re = Regex::new(&format!(
        r#"(?s)\n*/-- \[std::f64::\{{f64\}}::{}\]:.*?\n(?:.*?\n)*?@\[rust_fun "std::f64::\{{f64\}}::{}"\] (?:noncomputable )?axiom {} : ℝ → Result ℝ\n"#,
        regex::escape(func_name),
        regex::escape(func_name),
        regex::escape(&marker),
    ))
    .unwrap();
    content = re.replace(&content, "\n").to_string();

    // Replace let-binding call sites
    let re = Regex::new(&format!(
        r"\blet\s+(\w+)\s+←\s+{}\s+(\S+)",
        regex::escape(&marker)
    ))
    .unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("let {} := {} {}", &caps[1], mathlib_name, &caps[2])
        })
        .to_string();

    // Replace tail-position call sites
    let re = Regex::new(&format!(
        r"(?m)^([ \t]+){}\s+(\S+)[ \t]*$",
        regex::escape(&marker)
    ))
    .unwrap();
    let mn = mathlib_name;
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}ok ({} {})", &caps[1], mn, &caps[2])
        })
        .to_string();

    content
}

fn replace_f64_const(result: &str, const_name: &str, mathlib_name: &str) -> String {
    if !result.contains(const_name) {
        return result.to_string();
    }

    let mut content = result.to_string();
    let escaped = regex::escape(const_name);

    // Replace let-binding call sites
    let re = Regex::new(&format!(r"\blet\s+(\w+)\s+←\s+{}\b", escaped)).unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("let {} := {}", &caps[1], mathlib_name)
        })
        .to_string();

    // Replace tail-position call sites
    let re = Regex::new(&format!(r"(?m)^([ \t]+){}\s*$", escaped)).unwrap();
    let mn = mathlib_name;
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}{}", &caps[1], mn)
        })
        .to_string();

    // Add a definition
    let re = Regex::new(r"(namespace \w+\n)").unwrap();
    if let Some(caps) = re.captures(&content) {
        let ns = caps.get(1).unwrap().as_str().to_string();
        let def = format!(
            "\n/- {} replaced with {} by autospec -/\n\
             noncomputable def {} : ℝ := {}\n",
            const_name, mathlib_name, const_name, mathlib_name
        );
        content = content.replace(&ns, &format!("{}{}", ns, def));
    }

    content
}

// ── integer model replacement ──────────────────────────────────────────────────

pub fn lean_integers_to_model(content: &str, model: &str) -> String {
    // Check if any integer types are present
    let markers: Vec<&str> = INT_TYPES
        .iter()
        .flat_map(|it| vec![it.ty, it.suffix])
        .collect();
    if !markers.iter().any(|m| content.contains(m)) {
        return content.to_string();
    }

    let mut result = content.to_string();

    let produced: Vec<String> = INT_TYPES
        .iter()
        .map(|it| int_target_type(it.is_signed, it.width, model))
        .collect();

    for it in INT_TYPES {
        let tgt = int_target_type(it.is_signed, it.width, model);
        let clone_helper = format!("{}Clone", int_helper_stem(&tgt));
        let peq_helper = format!("PartialEq{}", int_helper_stem(&tgt));
        let clone_name = it.ty.strip_prefix("Std.").unwrap_or(it.ty);

        result = result.replace(it.ty, &type_arg(&tgt));
        result = result.replace(
            &format!("core.clone.Clone{}", clone_name),
            &clone_helper,
        );
        result = result.replace(
            &format!("core.cmp.PartialEq{}", clone_name),
            &peq_helper,
        );

        if model == "bitvec" && !it.is_signed {
            let re = Regex::new(&format!(
                r"(\d+){}\b",
                regex::escape(it.suffix)
            ))
            .unwrap();
            let w = it.width.unwrap_or(64);
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    format!("({} : BitVec {})", &caps[1], w)
                })
                .to_string();
        } else {
            let re = Regex::new(&format!(r"{}\b", regex::escape(it.suffix))).unwrap();
            result = re.replace_all(&result, "").to_string();
        }
    }

    result = monadic_ops_to_pure(&result, &produced);

    // Add Clone / PartialEq helpers
    let mut extras: Vec<String> = Vec::new();
    let mut seen = std::collections::HashSet::new();
    for tgt in &produced {
        if seen.contains(tgt) { continue; }
        seen.insert(tgt.clone());
        let stem = int_helper_stem(tgt);
        let clone_helper = format!("{}Clone", stem);
        let peq_helper = format!("PartialEq{}", stem);
        let need_clone = result.contains(&clone_helper)
            && !result.contains(&format!("def {}.clone", clone_helper));
        let need_peq = result.contains(&peq_helper)
            && !result.contains(&format!("def {}.eq", peq_helper));

        if need_clone || need_peq {
            if !result.contains(&format!("def {}.clone", clone_helper)) {
                let ta = type_arg(tgt);
                extras.push(format!(
                    "/- Integer types replaced with {} by autospec -/\n\
                     def {}.clone (x : {}) : Result {} := ok x\n\
                     \n\
                     @[reducible]\n\
                     def {} : core.clone.Clone {} := {{\n  clone := {}.clone\n}}\n",
                    tgt, clone_helper, ta, ta, clone_helper, ta, clone_helper
                ));
            }
            if !result.contains(&format!("def {}.eq", peq_helper)) {
                let ta = type_arg(tgt);
                extras.push(format!(
                    "def {}.eq (x y : {}) : Result Bool := ok (x = y)\n\
                     \n\
                     @[reducible]\n\
                     def {} : core.cmp.PartialEq {} {} := {{\n  eq := {}.eq\n}}\n",
                    peq_helper, ta, peq_helper, ta, ta, peq_helper
                ));
            }
        }
    }

    if !extras.is_empty() {
        let block = extras.join("\n");
        let re = Regex::new(r"(namespace \w+\n)").unwrap();
        if let Some(caps) = re.captures(&result) {
            let ns = caps.get(1).unwrap().as_str().to_string();
            result = result.replace(&ns, &format!("{}\n{}", ns, block));
        }
    }

    result
}

// ── monadic → pure conversion ──────────────────────────────────────────────────

fn monadic_ops_to_pure(content: &str, ctx_types: &[String]) -> String {
    let mut int_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut usize_vars: std::collections::HashSet<String> = std::collections::HashSet::new();
    let def_re = Regex::new(r"^\s*def\s+\w+").unwrap();
    let sig_end_re = Regex::new(r":=\s*(do\b)?").unwrap();
    let param_re = Regex::new(r"\((\w+)\s*:\s*([^)]*)\)").unwrap();

    let mut out: Vec<String> = Vec::new();
    let mut in_signature = false;
    let mut sig_lines: Vec<String> = Vec::new();

    for line in content.lines() {
        if def_re.is_match(line) {
            int_vars.clear();
            usize_vars.clear();
            in_signature = true;
            sig_lines = vec![line.to_string()];
            if sig_end_re.is_match(line) {
                in_signature = false;
                seed_params_from_lines(&sig_lines, ctx_types, &mut int_vars, &mut usize_vars, &param_re);
            }
            out.push(line.to_string());
        } else if in_signature {
            sig_lines.push(line.to_string());
            if sig_end_re.is_match(line) {
                in_signature = false;
                seed_params_from_lines(&sig_lines, ctx_types, &mut int_vars, &mut usize_vars, &param_re);
            }
            out.push(line.to_string());
        } else {
            out.push(convert_let_line(line, &mut int_vars, &mut usize_vars));
        }
    }
    out.join("\n")
}

fn seed_params_from_lines(
    lines: &[String],
    ctx_types: &[String],
    int_vars: &mut std::collections::HashSet<String>,
    usize_vars: &mut std::collections::HashSet<String>,
    param_re: &Regex,
) {
    for line in lines {
        for caps in param_re.captures_iter(line) {
            let name = caps.get(1).unwrap().as_str().to_string();
            let ty = caps.get(2).unwrap().as_str().trim().trim_start_matches('(');
            if ty == "Std.Usize" {
                usize_vars.insert(name);
            } else if ctx_types.iter().any(|t| ty == *t) {
                int_vars.insert(name);
            }
        }
    }
}

fn is_int(tok: &str, int_vars: &std::collections::HashSet<String>) -> bool {
    int_vars.contains(tok) || tok.parse::<i64>().is_ok()
        || (tok.starts_with('(') && tok.ends_with(')') && tok.contains("BitVec"))
}
fn is_usize(tok: &str, usize_vars: &std::collections::HashSet<String>) -> bool {
    usize_vars.contains(tok) || tok.ends_with("#usize")
}
fn is_bare_id(tok: &str) -> bool {
    tok.chars().all(|c| c.is_alphanumeric() || c == '_')
        && tok.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
}
fn track_vars(
    x: &str, a: &str, b: &str,
    int_vars: &mut std::collections::HashSet<String>,
    usize_vars: &mut std::collections::HashSet<String>,
) {
    if is_usize(a, usize_vars) || is_usize(b, usize_vars) {
        usize_vars.insert(x.to_string());
    } else if is_int(a, int_vars) && is_int(b, int_vars) {
        int_vars.insert(x.to_string());
    }
}

/// Full apply logic matching Python's ``apply`` function.
/// Returns `Some(new_line)` if the line should be rewritten, `None` if unchanged.
fn apply_let_arrow(
    prefix: &str, x: &str, a: &str, op: &str, b: &str,
    int_vars: &mut std::collections::HashSet<String>,
    usize_vars: &mut std::collections::HashSet<String>,
    original_line: &str,
) -> String {
    // Both operands are int-typed and neither is usize → pure
    if is_int(a, int_vars) && is_int(b, int_vars) && !is_usize(a, usize_vars) && !is_usize(b, usize_vars) {
        int_vars.insert(x.to_string());
        return format!("{}:= {} {} {}", prefix, a, op, b);
    }
    if is_usize(a, usize_vars) || is_usize(b, usize_vars) {
        usize_vars.insert(x.to_string());
    }
    // One operand is int-typed and the other is a bare identifier
    if is_int(a, int_vars) && !is_int(b, int_vars) && !is_usize(b, usize_vars) && is_bare_id(b) {
        int_vars.insert(b.to_string());
        int_vars.insert(x.to_string());
        return format!("{}:= {} {} {}", prefix, a, op, b);
    }
    if !is_int(a, int_vars) && is_int(b, int_vars) && !is_usize(a, usize_vars) && is_bare_id(a) {
        int_vars.insert(a.to_string());
        int_vars.insert(x.to_string());
        return format!("{}:= {} {} {}", prefix, a, op, b);
    }
    // Both operands are unknown bare identifiers and neither is usize
    if !is_usize(a, usize_vars) && !is_usize(b, usize_vars) && is_bare_id(a) && is_bare_id(b) {
        int_vars.insert(a.to_string());
        int_vars.insert(b.to_string());
        int_vars.insert(x.to_string());
        return format!("{}:= {} {} {}", prefix, a, op, b);
    }
    original_line.to_string()
}

fn convert_let_line(
    line: &str,
    int_vars: &mut std::collections::HashSet<String>,
    usize_vars: &mut std::collections::HashSet<String>,
) -> String {
    let op_token = r"(\w+|\([^()]*\)|\d+#usize|\d+)";

    // ── `←` patterns (monadic → pure) ──

    // arithmetic:  let x ← A + B  /  let x ← A - B  /  etc.
    let re_arith = Regex::new(&format!(
        r"^(\s*(?:then\s+|else\s+)?let\s+(\w+)\s+)←\s+{op}\s*([+\-*/%])\s*{op}\s*$",
        op = op_token
    )).unwrap();
    if let Some(caps) = re_arith.captures(line) {
        let prefix = caps.get(1).unwrap().as_str();
        let x = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        return apply_let_arrow(prefix, x, a, op, b, int_vars, usize_vars, line);
    }

    // bitwise:  let x ← lift (A &&& B)
    let re_bitwise = Regex::new(&format!(
        r"^(\s*(?:then\s+|else\s+)?let\s+(\w+)\s+)←\s+lift\s*\(\s*{op}\s*(&&&|\|\|\||\^\^\^)\s*{op}\s*\)\s*$",
        op = op_token
    )).unwrap();
    if let Some(caps) = re_bitwise.captures(line) {
        let prefix = caps.get(1).unwrap().as_str();
        let x = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        return apply_let_arrow(prefix, x, a, op, b, int_vars, usize_vars, line);
    }

    // shift:  let x ← A <<< B
    let re_shift = Regex::new(&format!(
        r"^(\s*(?:then\s+|else\s+)?let\s+(\w+)\s+)←\s+{op}\s*(<<<|>>>)\s*{op}\s*$",
        op = op_token
    )).unwrap();
    if let Some(caps) = re_shift.captures(line) {
        let prefix = caps.get(1).unwrap().as_str();
        let x = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        return apply_let_arrow(prefix, x, a, op, b, int_vars, usize_vars, line);
    }

    // ── `:=` patterns (already pure — just track types) ──

    let pure_prefix = r"^\s*(?:then\s+|else\s+)?let\s+(\w+)\s+:=\s+";

    // arithmetic:  let x := A + B
    let re_pure_arith = Regex::new(&format!(
        r"{}{}\s*([+\-*/%])\s*{}\s*$",
        pure_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_pure_arith.captures(line) {
        let x = caps.get(1).unwrap().as_str();
        let a = caps.get(2).unwrap().as_str();
        let b = caps.get(4).unwrap().as_str();
        if is_usize(a, usize_vars) || is_usize(b, usize_vars) {
            usize_vars.insert(x.to_string());
            return line.replacen(":=", "←", 1);
        }
        track_vars(x, a, b, int_vars, usize_vars);
        return line.to_string();
    }

    // bitwise:  let x := A &&& B
    let re_pure_bitwise = Regex::new(&format!(
        r"{}{}\s*(&&&|\|\|\||\^\^\^)\s*{}\s*$",
        pure_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_pure_bitwise.captures(line) {
        let x = caps.get(1).unwrap().as_str();
        let a = caps.get(2).unwrap().as_str();
        let b = caps.get(4).unwrap().as_str();
        track_vars(x, a, b, int_vars, usize_vars);
        return line.to_string();
    }

    // shift:  let x := A <<< B
    let re_pure_shift = Regex::new(&format!(
        r"{}{}\s*(<<<|>>>)\s*{}\s*$",
        pure_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_pure_shift.captures(line) {
        let x = caps.get(1).unwrap().as_str();
        let a = caps.get(2).unwrap().as_str();
        let b = caps.get(4).unwrap().as_str();
        track_vars(x, a, b, int_vars, usize_vars);
        return line.to_string();
    }

    // ── tail-position expressions (no `let`) — wrap in `ok (…)` ──

    let tail_prefix = r"^(\s*)((?:then\s+|else\s+)?)";

    let tail_ok = |a: &str, op: &str, b: &str| -> Option<String> {
        if is_int(a, int_vars) && is_int(b, int_vars) && !is_usize(a, usize_vars) && !is_usize(b, usize_vars) {
            Some(format!("ok ({} {} {})", a, op, b))
        } else {
            None
        }
    };

    // arithmetic tail
    let re_tail_arith = Regex::new(&format!(
        r"{}{}\s*([+\-*/%])\s*{}\s*$",
        tail_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_tail_arith.captures(line) {
        let indent = caps.get(1).unwrap().as_str();
        let prefix = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        if let Some(r) = tail_ok(a, op, b) {
            return format!("{}{}{}", indent, prefix, r);
        }
    }

    // bitwise tail: lift (A &&& B)
    let re_tail_bitwise = Regex::new(&format!(
        r"{}lift\s*\(\s*{}\s*(&&&|\|\|\||\^\^\^)\s*{}\s*\)\s*$",
        tail_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_tail_bitwise.captures(line) {
        let indent = caps.get(1).unwrap().as_str();
        let prefix = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        if let Some(r) = tail_ok(a, op, b) {
            return format!("{}{}{}", indent, prefix, r);
        }
    }

    // shift tail
    let re_tail_shift = Regex::new(&format!(
        r"{}{}\s*(<<<|>>>)\s*{}\s*$",
        tail_prefix, op_token, op_token
    )).unwrap();
    if let Some(caps) = re_tail_shift.captures(line) {
        let indent = caps.get(1).unwrap().as_str();
        let prefix = caps.get(2).unwrap().as_str();
        let a = caps.get(3).unwrap().as_str();
        let op = caps.get(4).unwrap().as_str();
        let b = caps.get(5).unwrap().as_str();
        if let Some(r) = tail_ok(a, op, b) {
            return format!("{}{}{}", indent, prefix, r);
        }
    }

    line.to_string()
}

// ── bitvec scalar-op rewriting ─────────────────────────────────────────────────

fn bitvec_ops_to_bv(content: &str) -> String {
    if !content.contains("core.num.U") && !content.contains("UScalar.cast") && !content.contains("IScalar.cast") {
        return content.to_string();
    }

    let mut result = content.to_string();
    let arg = r"(?:\([^()]*\)|[\w.]+)";

    for (fn_name, op) in &[("wrapping_add", "+"), ("wrapping_sub", "-"), ("wrapping_mul", "*")] {
        let re = Regex::new(&format!(
            r"core\.num\.U(?:8|16|32|64|128)\.{}\s+({})\s+({})",
            regex::escape(fn_name), arg, arg
        ))
        .unwrap();
        result = re.replace_all(&result, format!("$1 {} $2", op)).to_string();
    }

    for (fn_name, bv) in &[("rotate_left", "rotateLeft"), ("rotate_right", "rotateRight")] {
        let re = Regex::new(&format!(
            r"core\.num\.U(?:8|16|32|64|128)\.{}\s+({})\s+({})",
            regex::escape(fn_name), arg, arg
        ))
        .unwrap();
        let bv = *bv;
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                format!("BitVec.{} {} {}.toNat", bv, &caps[1], &caps[2])
            })
            .to_string();
    }

    if result.contains("UScalar.cast") || result.contains("IScalar.cast") {
        let re = Regex::new(&format!(r"UScalar\.cast\s+\.Usize\s+({})", arg)).unwrap();
        result = re
            .replace_all(&result, "UScalar.mk (BitVec.ofNat (UScalarTy.Usize.numBits) $1.toNat)")
            .to_string();
        for w in [8u32, 16, 32, 64, 128] {
            let re = Regex::new(&format!(r"UScalar\.cast\s+\.U{}\s+({})", w, arg)).unwrap();
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    format!("(BitVec.ofNat {} {}.toNat)", w, &caps[1])
                })
                .to_string();
        }
        for w in [8u32, 16, 32, 64, 128] {
            let re = Regex::new(&format!(r"IScalar\.cast\s+\.I{}\s+({})", w, arg)).unwrap();
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    format!("(BitVec.ofInt {} {}.toInt)", w, &caps[1])
                })
                .to_string();
        }
    }

    result
}

// ── misc fixups ────────────────────────────────────────────────────────────────

fn mark_noncomputable_defs(text: &str) -> String {
    let mut axiom_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    let axiom_re = Regex::new(r"(?m)^\s*(?:noncomputable\s+)?axiom\s+([\w.]+)").unwrap();
    for caps in axiom_re.captures_iter(text) {
        axiom_names.insert(caps.get(1).unwrap().as_str().to_string());
    }
    if axiom_names.is_empty() {
        return text.to_string();
    }

    let mut decls: Vec<(String, String, usize, usize)> = Vec::new();
    let decl_re = Regex::new(
        r"(?m)^(?:@\[[^\]]*\][ \t]*\n\s*)?(?:(noncomputable[ \t]+)?(def|theorem)[ \t]+(\w+))",
    )
    .unwrap();
    for caps in decl_re.captures_iter(text) {
        let name = caps.get(3).unwrap().as_str().to_string();
        let is_noncomp = caps.get(1).is_some();
        let body_start = caps.get(0).unwrap().end();
        let body_end = {
            let rest = &text[body_start..];
            let next = Regex::new(
                r"(?m)^\s*(?:end\b|namespace\b|(?:noncomputable\s+)?(?:def|axiom|theorem)\s+)",
            )
            .unwrap();
            next.find(rest).map(|m| body_start + m.start()).unwrap_or(text.len())
        };
        let body = &text[body_start..body_end];
        let body_clean = Regex::new(r"(?s)/-.*?-/").unwrap().replace_all(body, "");
        let body_clean = body_clean.to_string();
        decls.push((name, body_clean, caps.get(0).unwrap().start(), caps.get(0).unwrap().end()));
        if is_noncomp {
            axiom_names.insert(decls.last().unwrap().0.clone());
        }
    }

    // Fixed-point propagation
    loop {
        let mut changed = false;
        for (name, body, _, _) in &decls {
            if axiom_names.contains(name) {
                continue;
            }
            if axiom_names.iter().any(|n| body.contains(n.as_str())) {
                axiom_names.insert(name.clone());
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }

    if axiom_names.is_empty() {
        return text.to_string();
    }

    // Find non-axiom defs that need noncomputable
    let axiom_only: std::collections::HashSet<String> = axiom_re
        .captures_iter(text)
        .map(|c| c.get(1).unwrap().as_str().to_string())
        .collect();

    let noncomp_names: Vec<String> = axiom_names
        .difference(&axiom_only)
        .cloned()
        .collect();

    let mut result = text.to_string();
    // Sort by length descending to avoid partial matches
    let mut sorted = noncomp_names;
    sorted.sort_by_key(|b| std::cmp::Reverse(b.len()));
    for name in &sorted {
        let re = Regex::new(&format!(
            r"(?m)^(@\[[^\]]*\]\s*\n\s*)?(def\s+{})",
            regex::escape(name)
        ))
        .unwrap();
        result = re
            .replace_all(&result, |caps: &regex::Captures| {
                let prefix = caps.get(1).map(|m| m.as_str()).unwrap_or("");
                format!("{}noncomputable {}", prefix, &caps[2])
            })
            .to_string();
    }

    result
}

fn fix_partial_mutual(text: &str) -> String {
    let mut result = text.to_string();
    for (tag, label) in &[
        (r"@\[rust_loop\]\n", "_quick_sort_loop"),
        (r"@\[rust_loop_body\]\n", r"_quick_sort_loop\.body"),
        (r"@\[reducible\]\n", "_quick_sort"),
    ] {
        let re = Regex::new(&format!(
            r"(?s){}(def {}\b.*?)partial_fixpoint",
            tag, label
        ))
        .unwrap();
        result = re.replace_all(&result, "partial $1").to_string();
    }
    result
}

fn fix_vec_is_empty(text: &str) -> String {
    if !text.contains("alloc.vec.Vec.is_empty") {
        return text.to_string();
    }
    let mut result = text.to_string();
    let re = Regex::new(r"let\s+(\w+)\s+←\s+alloc\.vec\.Vec\.is_empty\s+Global\s+(\w+)").unwrap();
    result = re
        .replace_all(&result, "let $1 := (alloc.vec.Vec.deref $2).length = 0")
        .to_string();
    let re = Regex::new(r"alloc\.vec\.Vec\.is_empty\s+Global\s+(\w+)").unwrap();
    result = re
        .replace_all(&result, "ok ((alloc.vec.Vec.deref $1).length = 0)")
        .to_string();
    result
}

// ── main entry point ───────────────────────────────────────────────────────────

/// Post-process and copy Lean files from *workdir* to *output_path*.
///
/// Returns (def_count, written_filenames).
pub fn write_lean_output(
    workdir: &Path,
    output_path: &Path,
    float_model: &str,
    int_model: &str,
) -> (usize, Vec<String>) {
    let lean_files: Vec<_> = std::fs::read_dir(workdir)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.filter_map(|e| e.ok()))
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "lean"))
        .collect();

    let mut def_count = 0usize;
    let mut written: Vec<String> = Vec::new();
    let use_real = float_model == "Real";

    for f in &lean_files {
        let text = match std::fs::read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                log::warn(&format!("cannot read {}: {}", f.display(), e));
                continue;
            }
        };

        let mut text = text.replace(
            "set_option linter.unusedVariables false",
            "set_option linter.unusedVariables false\nset_option linter.style.nameCheck false",
        );

        if use_real {
            text = lean_f64_to_real(&text);
        } else {
            text = match lean_f64_to_rat(&text) {
                Ok(t) => t,
                Err(_e) => {
                    return (0, vec![]); // caller should catch and retry with Real
                }
            };
        }

        text = lean_integers_to_model(&text, int_model);

        if int_model == "bitvec" {
            text = bitvec_ops_to_bv(&text);
            // Run monadic_ops_to_pure again after bitvec rewrite
            let produced: Vec<String> = INT_TYPES
                .iter()
                .map(|it| int_target_type(it.is_signed, it.width, int_model))
                .collect();
            let mut seen = std::collections::HashSet::new();
            let unique: Vec<String> = produced.into_iter().filter(|t| seen.insert(t.clone())).collect();
            text = monadic_ops_to_pure(&text, &unique);
        }

        text = lean_inline::inline_f64_helpers(&text);
        text = lean_inline::inline_i64_helpers(&text);

        if use_real {
            text = text.replace("Rat", "ℝ");
            text = text.replace("RatClone", "RealClone");
        }

        text = fix_partial_mutual(&text);
        text = fix_vec_is_empty(&text);
        text = mark_noncomputable_defs(&text);

        let fname = f.file_name().unwrap_or_default().to_string_lossy().to_string();
        let out_path = output_path.join(&fname);
        if let Err(e) = std::fs::write(&out_path, text.trim_end().to_string() + "\n") {
            log::warn(&format!("cannot write {}: {}", out_path.display(), e));
            continue;
        }
        written.push(fname);

        // Count definitions
        let def_re = Regex::new(
            r"(?m)^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+",
        )
        .unwrap();
        def_count += def_re.find_iter(&text).count();
    }

    if def_count == 0 {
        log::fail("output", "no definitions written");
    } else {
        log::phase("output", &format!("{} defs", def_count));
    }

    (def_count, written)
}
