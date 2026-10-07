//! Extract and translate test files (``tests/*.rs``) as separate Lean specs.
//!
//! Port of ``translation/pipeline/extract_test.py``.

use std::collections::HashSet;
use std::path::Path;

use regex::Regex;

use crate::lean_inline;
use crate::lean_post;
use crate::log;
use crate::utils;

/// Translate a test file and return the number of definitions produced.
pub fn translate_test_file<F>(
    translate_inner: F,
    project_path: &Path,
    output_path: &Path,
    test_file: &Path,
    float_model: &str,
    int_model: &str,
) -> Option<usize>
where
    F: Fn(&Path, &Path) -> Option<(usize, Vec<String>)>,
{
    let raw = match std::fs::read_to_string(test_file) {
        Ok(r) => r,
        Err(e) => {
            log::warn(&format!("cannot read test file {}: {}", test_file.display(), e));
            return None;
        }
    };
    let crate_name = utils::read_crate_name(project_path);

    // Record #[test]-annotated function names
    let eval_names: HashSet<String> = {
        let re = Regex::new(r#"#\[test\]\s*\n(?:\s*#\[[^\]]*\]\s*\n)*\s*fn\s+(\w+)"#).unwrap();
        re.captures_iter(&raw)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect()
    };

    // Strip #[test] and make every fn pub
    let re_test = Regex::new(r"#\[test\]\s*\n?").unwrap();
    let mut raw = re_test.replace_all(&raw, "").to_string();
    let re_fn = Regex::new(r"\bfn\s+(\w+)").unwrap();
    raw = re_fn.replace_all(&raw, "pub fn $1").to_string();

    // Remove `use` items that refer to the parent crate or the crate itself
    // (`use crypto_lwe;`, `use crypto_lwe::foo;`, `use super::foo;`, …).
    // Once the test body is inlined into the crate these imports are either
    // self-referential (unresolvable) or redundant.
    let re_use = Regex::new(&format!(
        r"(?m)^\s*use\s+(?:super|self|{})\b[^\n]*\n?",
        regex::escape(&crate_name)
    ))
    .unwrap();
    raw = re_use.replace_all(&raw, "").to_string();

    // Replace crate-prefixed calls (e.g. matmul::matmul → matmul)
    let re_crate = Regex::new(&format!(r"\b{}::", regex::escape(&crate_name))).unwrap();
    raw = re_crate.replace_all(&raw, "").to_string();

    // Inner doc comments (`//! …`) are only legal at the top of a crate, but
    // this test file is appended after the library body below. Rewrite them as
    // plain line comments so the combined crate still compiles.
    let re_inner_doc = Regex::new(r"(?m)^(\s*)//!").unwrap();
    raw = re_inner_doc.replace_all(&raw, "$1//").to_string();

    let stem = test_file.file_stem().unwrap_or_default().to_string_lossy();

    // Create tmp dir and copy project
    let tmp_dir = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            log::warn(&format!("cannot create temp dir: {}", e));
            return None;
        }
    };
    let tmp_path = tmp_dir.path().to_path_buf();

    // Copy project to tmp dir (ignoring tests/)
    if let Err(e) = utils::copy_dir_ignore(project_path, &tmp_path, &["target", "lean", "*.lean", "tests"]) {
        log::warn(&format!("cannot copy project: {}", e));
        return None;
    }

    // Append test functions to lib.rs
    let lib_rs = tmp_path.join("src").join("lib.rs");
    match std::fs::read_to_string(&lib_rs) {
        Ok(lib_content) => {
            let combined = format!("{}\n\n{}\n", lib_content.trim_end(), raw);
            if let Err(e) = std::fs::write(&lib_rs, &combined) {
                log::warn(&format!("cannot write combined lib.rs: {}", e));
                return None;
            }
        }
        Err(e) => {
            log::warn(&format!("cannot read lib.rs: {}", e));
            return None;
        }
    }

    // Run translation on the combined project
    let combined_output = match tempfile::tempdir() {
        Ok(d) => d,
        Err(e) => {
            log::warn(&format!("cannot create combined output dir: {}", e));
            return None;
        }
    };
    let combined_path = combined_output.path().to_path_buf();

    let outcome = translate_inner(&tmp_path, &combined_path);
    if outcome.is_none() {
        log::warn(&format!("test translation failed for {}", test_file.display()));
        return None;
    }

    // Collect test function names (original names from the test file)
    let test_fn_names: HashSet<String> = {
        let re = Regex::new(r"pub\s+fn\s+(\w+)").unwrap();
        re.captures_iter(&raw)
            .filter_map(|c| c.get(1).map(|m| m.as_str().to_string()))
            .collect()
    };

    let use_real = float_model == "Real";

    let lean_files: Vec<_> = std::fs::read_dir(&combined_path)
        .ok()
        .into_iter()
        .flat_map(|rd| rd.filter_map(|e| e.ok()))
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "lean"))
        .collect();

    for lf in &lean_files {
        let mut combined_text = match std::fs::read_to_string(lf) {
            Ok(t) => t,
            Err(e) => {
                log::warn(&format!("cannot read {}: {}", lf.display(), e));
                continue;
            }
        };
        
        if use_real {
            combined_text = lean_post::lean_f64_to_real(&combined_text);
        } else {
            combined_text = match lean_post::lean_f64_to_rat(&combined_text) {
                Ok(t) => t,
                Err(_) => return None,
            };
        }

        combined_text = lean_post::lean_integers_to_model(&combined_text, int_model);
        combined_text = lean_inline::inline_f64_helpers(&combined_text);
        combined_text = lean_inline::inline_i64_helpers(&combined_text);

        if use_real {
            combined_text = combined_text.replace("Rat", "ℝ");
            combined_text = combined_text.replace("RatClone", "RealClone");
        }

        // Deduplicate set_option lines
        let re_opt = Regex::new(r"(set_option linter\.style\.nameCheck false\n)+").unwrap();
        combined_text = re_opt
            .replace_all(&combined_text, "set_option linter.style.nameCheck false\n")
            .to_string();

        let test_text = extract_defs_by_prefix(&combined_text, &test_fn_names);
        if test_text.trim().is_empty() {
            continue;
        }

        // Drop the `module`/`@[expose]` header so the impl-module import uses
        // plain `import` and `#eval` can interpret the impl module's
        // `partial_fixpoint` defs (see `lean_post::strip_module_header`).
        let test_text = lean_post::strip_module_header(&test_text);

        let impl_module = compute_module_path(project_path);
        let test_text = add_import(&test_text, &impl_module);
        let test_text = add_evals(&test_text, &eval_names);

        let target = output_path.join(format!("Test_{}.lean", stem));
        if let Err(e) = std::fs::write(&target, test_text.trim_end().to_string() + "\n") {
            log::warn(&format!("cannot write {}: {}", target.display(), e));
            return None;
        }

        let def_re = Regex::new(
            r"(?m)^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+",
        )
        .unwrap();
        return Some(def_re.find_iter(&test_text).count());
    }

    None
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn snake_to_camel(s: &str) -> String {
    s.split('_')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut chars: Vec<char> = p.chars().collect();
            if let Some(c) = chars.first_mut() {
                *c = c.to_uppercase().next().unwrap_or(*c);
            }
            chars.into_iter().collect::<String>()
        })
        .collect()
}

fn compute_module_path(project_path: &Path) -> String {
    let lake_root = utils::find_lake_root(project_path);
    if let Some(lake_root) = lake_root {
        if let Ok(rel) = project_path.strip_prefix(lake_root) {
            let mut parts: Vec<String> = rel
                .components()
                .map(|c| c.as_os_str().to_string_lossy().replace('-', "_"))
                .collect();
            if let Some(p) = parts.first_mut() {
                *p = utils::capitalize_first(p);
            }
            let mod_prefix = parts.join(".");
            let crate_stem = project_path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .replace('-', "_");
            return format!("{}.{}", mod_prefix, snake_to_camel(&crate_stem));
        }
    }
    snake_to_camel(
        &project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .replace('-', "_"),
    )
}

fn add_import(lean_text: &str, module: &str) -> String {
    let mut lines: Vec<String> = lean_text.lines().map(|s| s.to_string()).collect();
    let mut last_import: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        // Old aeneas emits `import Aeneas`; the module-system aeneas emits
        // `public import Aeneas` inside a `module` block.  Anchor on either so
        // the impl-module import is appended after the existing imports.
        if trimmed.starts_with("public import ") || trimmed.starts_with("import ") {
            last_import = Some(i);
        }
    }
    if let Some(idx) = last_import {
        // Mirror the anchor's visibility: a module-system (`public import`)
        // header must also import the impl module publicly, otherwise the impl
        // module's defs are not exposed to this module.
        let trimmed = lines[idx].trim_start();
        let public = trimmed.starts_with("public import ");
        let indent: String = lines[idx].chars().take_while(|c| c.is_whitespace()).collect();
        let kw = if public { "public import" } else { "import" };
        lines.insert(idx + 1, format!("{}{} {}", indent, kw, module));
    }
    lines.join("\n")
}

fn add_evals(lean_text: &str, fn_names: &HashSet<String>) -> String {
    if lean_text.contains("noncomputable section") {
        return lean_text.to_string();
    }

    let mut result = lean_text.to_string();

    let mut sorted: Vec<&String> = fn_names.iter().collect();
    sorted.sort();

    for name in &sorted {
        // Skip if this specific def is marked noncomputable
        let re = Regex::new(&format!(
            r"noncomputable\s+def\s+{}\b",
            regex::escape(name)
        ))
        .unwrap();
        if re.is_match(&result) {
            continue;
        }

        // Insert #eval before the final `end <ns>`
        let re_end = Regex::new(r"(\nend\s+\S+\s*)$").unwrap();
        if re_end.is_match(&result) {
            result = re_end
                .replace(&result, |caps: &regex::Captures| {
                    format!("\n#eval {}\n{}", name, &caps[1])
                })
                .to_string();
        }
    }
    result
}

fn extract_defs_by_prefix(combined_text: &str, prefixes: &HashSet<String>) -> String {
    let lines: Vec<&str> = combined_text.lines().collect();

    // Find preamble
    let mut preamble_end = 0;
    let def_re = Regex::new(r"(?m)^\s*(?:noncomputable\s+)?(?:def|theorem|axiom)\s+\S+").unwrap();
    for (i, line) in lines.iter().enumerate() {
        if line.starts_with("/-- [") || def_re.is_match(line) {
            preamble_end = i;
            break;
        }
        preamble_end = i + 1;
    }
    let preamble = lines[..preamble_end].join("\n").trim_end().to_string();

    let ns_match = Regex::new(r"namespace\s+(\S+)")
        .unwrap()
        .captures(&preamble);
    let ns = ns_match
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "«crate»".to_string());

    // Split into definition blocks
    let mut blocks: Vec<String> = Vec::new();
    let mut current: Vec<&str> = Vec::new();

    for line in lines.iter().skip(preamble_end) {
        let stripped = line.trim();
        if stripped.starts_with("end ") && Regex::new(r"^end\s+\S+$").unwrap().is_match(stripped) {
            break;
        }
        if line.starts_with("/-- [") && !current.is_empty() {
            blocks.push(current.join("\n").trim_end().to_string());
            current.clear();
        }
        current.push(line);
    }
    if !current.is_empty() {
        let text = current.join("\n").trim_end().to_string();
        let text = Regex::new(r"\nend\s+\S+\s*$")
            .unwrap()
            .replace(&text, "")
            .to_string();
        if !text.trim().is_empty() {
            blocks.push(text.trim().to_string());
        }
    }

    let mut kept: Vec<String> = Vec::new();
    for block in &blocks {
        if let Some(caps) = def_re.captures(block) {
            let name = caps.get(0).unwrap().as_str().trim();
            let name = name.trim_start_matches("noncomputable ");
            let name = name
                .strip_prefix("def ")
                .or_else(|| name.strip_prefix("theorem "))
                .or_else(|| name.strip_prefix("axiom "))
                .unwrap_or(name);
            let name = name.trim_end_matches(':');
            if prefixes.iter().any(|p| name.starts_with(p.as_str())) {
                kept.push(block.clone());
            }
        }
    }

    if kept.is_empty() {
        return String::new();
    }

    format!(
        "{}\n\n{}\n\nend {}\n",
        preamble,
        kept.join("\n\n"),
        ns
    )
}

