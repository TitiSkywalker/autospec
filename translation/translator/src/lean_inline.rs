//! Inline f64_helpers / i64_helpers calls in generated Lean code.
//!
//! Port of ``translation/tools/lean_inline.py``.

use regex::Regex;

// ── f64 operator tables ────────────────────────────────────────────────────────

const F64_BINARY: &[(&str, &str)] = &[
    ("add", "+"),
    ("sub", "-"),
    ("mul", "*"),
    ("div", "/"),
];

const F64_COMPARISON: &[(&str, &str)] = &[
    ("eq", "="),
    ("gt", ">"),
    ("lt", "<"),
    ("ge", "≥"),
    ("le", "≤"),
];

const F64_UNARY: &[(&str, &str, &str)] = &[
    ("neg", "-", ""),
    ("abs", "|", "|"),
];

// ── i64 operator tables ────────────────────────────────────────────────────────

const I64_BINARY: &[(&str, &str)] = &[
    ("add", "+"),
    ("sub", "-"),
    ("mul", "*"),
    ("div", "/"),
    ("rem", "%"),
];

const I64_COMPARISON: &[(&str, &str)] = &[
    ("eq", "="),
    ("ne", "≠"),
    ("gt", ">"),
    ("lt", "<"),
    ("ge", "≥"),
    ("le", "≤"),
];

const I64_BITWISE: &[(&str, &str, bool)] = &[
    ("bitand", "Int.land", false),
    ("bitor", "Int.lor", false),
    ("bitxor", "Int.xor", false),
    ("shl", "Int.shiftLeft", true),
    ("shr", "Int.shiftRight", true),
];

const I64_UNARY: &[(&str, &str, &str)] = &[
    ("neg", "-", ""),
    ("not", "(-(", " + 1))"),
];

// ── public API ─────────────────────────────────────────────────────────────────

/// Inline all f64_helpers.* calls and remove their definitions.
pub fn inline_f64_helpers(content: &str) -> String {
    let mut content = content.to_string();

    // Step 1: binary arithmetic (add / sub / mul / div)
    for (name, op) in F64_BINARY {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+f64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }

    // Step 2: comparisons
    for (name, op) in F64_COMPARISON {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+f64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }

    // Step 3: unary (neg / abs)
    for (name, prefix, suffix) in F64_UNARY {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+f64_helpers\.{}\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let p = *prefix;
        let s = *suffix;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {}{}{}", &caps[1], p, &caps[2], s)
            })
            .to_string();
    }

    // Step 4: from_usize
    let re = Regex::new(r"(let\s+\w+\s+)←\s+f64_helpers\.from_usize\s+(\S+)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}:= ({} : Rat)", &caps[1], format!("{}.val", &caps[2]))
        })
        .to_string();

    // Step 5: floating-point constants (const_3_14, const_neg_1_0, ...)
    let re = Regex::new(r"(let\s+\w+\s+)←\s+f64_helpers\.(const_\w+)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}:= {}", &caps[1], const_to_rat(&caps[2]))
        })
        .to_string();

    // Step 6: handle tail-position f64_helpers calls (return value of do block)
    for (name, op) in F64_BINARY {
        let re = Regex::new(&format!(
            r"(?m)^(\s+)f64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({} {} {})", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }
    for (name, op) in F64_COMPARISON {
        let re = Regex::new(&format!(
            r"(?m)^(\s+)f64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({} {} {})", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }
    for (name, prefix, suffix) in F64_UNARY {
        let re = Regex::new(&format!(
            r"(?m)^(\s+)f64_helpers\.{}\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let p = *prefix;
        let s = *suffix;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({}{}{})", &caps[1], p, &caps[2], s)
            })
            .to_string();
    }
    let re = Regex::new(r"(?m)^(\s+)f64_helpers\.from_usize\s+(\S+)[ \t]*$").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}ok (({} : Rat))", &caps[1], format!("{}.val", &caps[2]))
        })
        .to_string();

    // Step 7: handle f64_helpers.neg / f64_helpers.abs outside a let binding
    content = Regex::new(r"f64_helpers\.neg\s+(\S+)")
        .unwrap()
        .replace_all(&content, "ok (-$1)")
        .to_string();
    content = Regex::new(r"f64_helpers\.abs\s+(\S+)")
        .unwrap()
        .replace_all(&content, "|$1|")
        .to_string();

    // Step 8: remove now-unused def f64_helpers.* definitions
    content = remove_f64_helper_defs(&content);

    content
}

/// Inline all i64_helpers.* calls and remove their definitions.
pub fn inline_i64_helpers(content: &str) -> String {
    let mut content = content.to_string();

    // Step 1: binary arithmetic
    for (name, op) in I64_BINARY {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+i64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }

    // Step 2: comparisons
    for (name, op) in I64_COMPARISON {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+i64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }

    // Step 3: bitwise
    for (name, func, _needs_val) in I64_BITWISE {
        if *name == "shl" || *name == "shr" {
            continue;
        }
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+i64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let func = *func;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}", &caps[1], func, &caps[2], &caps[3])
            })
            .to_string();
    }

    // Step 4: shifts
    for (name, func, _) in I64_BITWISE {
        if *name != "shl" && *name != "shr" {
            continue;
        }
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+i64_helpers\.{}\s+(\S+)\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let func = *func;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {} {} {}.val", &caps[1], func, &caps[2], &caps[3])
            })
            .to_string();
    }

    // Step 5: unary
    for (name, prefix, suffix) in I64_UNARY {
        let re = Regex::new(&format!(
            r"(let\s+\w+\s+)←\s+i64_helpers\.{}\s+(\S+)",
            regex::escape(name)
        ))
        .unwrap();
        let p = *prefix;
        let s = *suffix;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}:= {}{}{}", &caps[1], p, &caps[2], s)
            })
            .to_string();
    }

    // Step 6: from_usize
    let re = Regex::new(r"(let\s+\w+\s+)←\s+i64_helpers\.from_usize\s+(\S+)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}:= ({} : Int)", &caps[1], format!("{}.val", &caps[2]))
        })
        .to_string();

    // Step 7: integer constants
    let re = Regex::new(r"(let\s+\w+\s+)←\s+i64_helpers\.(const_\w+)").unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}:= {}", &caps[1], i64_const_to_int(&caps[2]))
        })
        .to_string();

    // Step 8: tail-position calls
    for (name, op) in I64_BINARY {
        let re = Regex::new(&format!(
            r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({} {} {})", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }
    for (name, op) in I64_COMPARISON {
        let re = Regex::new(&format!(
            r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let op = *op;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({} {} {})", &caps[1], &caps[2], op, &caps[3])
            })
            .to_string();
    }
    for (name, func, needs_val) in I64_BITWISE {
        let pat = if *needs_val {
            format!(
                r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
                regex::escape(name)
            )
        } else {
            format!(
                r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.{}\s+(\S+)\s+(\S+)[ \t]*$",
                regex::escape(name)
            )
        };
        let re = Regex::new(&pat).unwrap();
        let func = *func;
        let needs_val = *needs_val;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                if needs_val {
                    format!("{}ok ({} {} {}.val)", &caps[1], func, &caps[2], &caps[3])
                } else {
                    format!("{}ok ({} {} {})", &caps[1], func, &caps[2], &caps[3])
                }
            })
            .to_string();
    }
    for (name, prefix, suffix) in I64_UNARY {
        let re = Regex::new(&format!(
            r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.{}\s+(\S+)[ \t]*$",
            regex::escape(name)
        ))
        .unwrap();
        let p = *prefix;
        let s = *suffix;
        content = re
            .replace_all(&content, |caps: &regex::Captures| {
                format!("{}ok ({}{}{})", &caps[1], p, &caps[2], s)
            })
            .to_string();
    }
    let re = Regex::new(
        r"(?m)^([ \t]*(?:then[ \t]+|else[ \t]+)?)i64_helpers\.from_usize\s+(\S+)[ \t]*$",
    )
    .unwrap();
    content = re
        .replace_all(&content, |caps: &regex::Captures| {
            format!("{}ok (({} : Int))", &caps[1], format!("{}.val", &caps[2]))
        })
        .to_string();

    // Step 9: handle i64_helpers calls outside a let binding
    // (?!:) is handled by checking the line doesn't start a definition
    content = by_line_replace(&content, "i64_helpers.neg", "ok (-$1)");
    content = by_line_replace(&content, "i64_helpers.not", "ok (-($1 + 1))");

    // Step 10: remove now-unused def i64_helpers.* definitions
    content = remove_i64_helper_defs(&content);

    content
}

/// Line-based replacement that avoids matching definition lines (lines with `:`).
fn by_line_replace(content: &str, pattern: &str, replacement: &str) -> String {
    let re = Regex::new(&format!(r"{}\s+(\S+)", regex::escape(pattern))).unwrap();
    let mut out = String::with_capacity(content.len());
    for line in content.lines() {
        if line.contains(':') && line.contains(pattern) {
            // Skip definition lines (axiom/def i64_helpers.neg : ...)
            out.push_str(line);
            out.push('\n');
            continue;
        }
        let replaced = re.replace(line, replacement);
        out.push_str(&replaced);
        out.push('\n');
    }
    out
}

// ── constant conversion ────────────────────────────────────────────────────────

fn const_to_rat(name: &str) -> String {
    let v = name.strip_prefix("const_").unwrap_or(name);
    let neg = v.starts_with("neg_");
    let v = if neg { &v[4..] } else { v };

    // Handle scientific notation (e.g. "1_0Eneg_10" = 1.0 * 10^-10)
    let em = regex::Regex::new(r"^(\d+)_(\d+)E([nN][eE][gG])_(\d+)$").unwrap();
    if let Some(caps) = em.captures(v) {
        let int_part = caps.get(1).unwrap().as_str();
        let frac_part = caps.get(2).unwrap().as_str();
        let exp_sign = caps.get(3).unwrap().as_str().to_lowercase();
        let exp_val: i64 = caps.get(4).unwrap().as_str().parse().unwrap_or(0);

        let _mantissa: i64 = format!("{}{}", int_part, frac_part).parse().unwrap_or(0);
        let _frac_len = frac_part.len() as i64;

        let mantissa: i128 = format!("{}{}", int_part, frac_part).parse().unwrap_or(0);
        let frac_len = frac_part.len() as i64;

        let (num, denom) = if exp_sign == "neg" {
            let pow = 10_i128.checked_pow((exp_val + frac_len) as u32);
            match pow {
                Some(d) => (mantissa, d),
                None => return format!("({} : Rat)", v),
            }
        } else {
            let shift = exp_val - frac_len;
            if shift >= 0 {
                match 10_i128.checked_pow(shift as u32) {
                    Some(pow) => (mantissa * pow, 1),
                    None => return format!("({} : Rat)", v),
                }
            } else {
                match 10_i128.checked_pow((-shift) as u32) {
                    Some(d) => (mantissa, d),
                    None => return format!("({} : Rat)", v),
                }
            }
        };

        let expr = if denom == 1 {
            format!("({} : Rat)", num)
        } else {
            format!("({}/{} : Rat)", num, denom)
        };
        return if neg { format!("(-{})", expr) } else { expr };
    }

    let parts: Vec<&str> = v.split('_').collect();
    if parts.len() != 2 {
        let expr = format!("({} : Rat)", v);
        return if neg { format!("(-{})", expr) } else { expr };
    }

    let int_part = parts[0];
    let frac_part = parts[1];
    let denom = 10_i128.pow(frac_part.len() as u32);
    let int_val: i128 = int_part.parse().unwrap_or(0);
    let frac_val: i128 = frac_part.parse().unwrap_or(0);
    let num = int_val * denom + frac_val;
    let expr = format!("({}/{} : Rat)", num, denom);
    if neg {
        format!("(-{})", expr)
    } else {
        expr
    }
}

fn i64_const_to_int(name: &str) -> String {
    let v = name.strip_prefix("const_").unwrap_or(name);
    let neg = v.starts_with("neg_");
    let v = if neg { &v[4..] } else { v };
    if neg {
        format!("(-{} : Int)", v)
    } else {
        format!("({} : Int)", v)
    }
}

// ── definition removal ─────────────────────────────────────────────────────────

fn remove_helper_defs(content: &str, prefix: &str) -> String {
    let escaped = regex::escape(prefix);
    // Remove the entire block: doc comment → attribute → axiom/def
    let re = Regex::new(&format!(
        r#"(?s)\n*/--\s+\[{}::\S+\]:\s*.*?\n(?:.*?\n)*?@\[rust_fun\s+"{}::\S+"\][ \t]*(?:\n[ \t]*)?(?:noncomputable\s+)?(?:axiom|def)\s+{}\.\S+[ \t]*[^\n]*\n?"#,
        escaped, escaped, escaped
    ))
    .unwrap();
    let mut result = re.replace_all(content, "").to_string();

    // Fallback: remove any remaining bare helper lines
    let mut out: Vec<&str> = Vec::new();
    for line in result.lines() {
        let stripped = line.trim();
        if stripped.starts_with(&format!("def {}.", prefix))
            || stripped.starts_with(&format!("axiom {}.", prefix))
        {
            continue;
        }
        out.push(line);
    }
    result = out.join("\n");

    // Collapse 4+ consecutive blank lines → 3
    let re = Regex::new(r"\n{4,}").unwrap();
    result = re.replace_all(&result, "\n\n\n").to_string();

    result
}

fn remove_f64_helper_defs(content: &str) -> String {
    let content = content.replace(
        "/- F64 replaced with Rat by autospec -/",
        "/- F64 replaced with Rat; f64_helpers inlined by autospec -/",
    );
    remove_helper_defs(&content, "f64_helpers")
}

fn remove_i64_helper_defs(content: &str) -> String {
    remove_helper_defs(content, "i64_helpers")
}