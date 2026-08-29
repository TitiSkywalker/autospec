//! Simplify Rust source so aeneas can handle it.
//!
//! Port of ``translation/pipeline/simplify.py``.

use std::path::Path;

use regex::Regex;

/// Transform Rust source in-place for aeneas compatibility.
pub fn simplify_for_aeneas(project_dir: &Path) {
    let re_copy_from_slice = Regex::new(r"(\w+)\.copy_from_slice\((\w+)\);").unwrap();
    let re_min = Regex::new(r"(\w+)\.min\((\w+)\)").unwrap();
    let re_iter_all = Regex::new(r"(\w+)\.iter\(\)\.all\(\|&(\w+)\|\s*(\w+)\)").unwrap();

    for entry in walkdir::WalkDir::new(project_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            let p = e.path();
            p.extension().map_or(false, |ext| ext == "rs")
                && !p.to_string_lossy().contains("target")
        })
    {
        let path = entry.path();
        let mut text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(_) => continue,
        };
        let mut changed = false;

        // .copy_from_slice(other) → while loop
        if let Some(caps) = re_copy_from_slice.captures(&text) {
            let dst = caps.get(1).unwrap().as_str();
            let src = caps.get(2).unwrap().as_str();
            let replacement = format!(
                "{{ let mut __i: usize = 0; while __i < {dst}.len() \
                 {{ {dst}[__i] = {src}[__i]; __i += 1; }}}}"
            );
            text = text.replace(&caps[0], &replacement);
            changed = true;
        }

        // a.min(b) for usize → if a < b { a } else { b }
        if let Some(caps) = re_min.captures(&text) {
            let a = caps.get(1).unwrap().as_str();
            let b = caps.get(2).unwrap().as_str();
            let replacement = format!(
                "(if {a} < {b} {{ {a} }} else {{ {b} }})"
            );
            text = text.replace(&caps[0], &replacement);
            changed = true;
        }

        // &mut *var → var (explicit deref+ref)
        // NOTE: hardcoded variable names as in Python version
        if text.contains("&mut *") {
            text = text.replace("&mut *lu_data", "lu_data");
            text = text.replace("&mut *ipiv", "ipiv");
            changed = true;
        }

        // <vec>.iter().all(|&<binder>| <var>) → manual while loop
        if let Some(caps) = re_iter_all.captures(&text) {
            let vec = caps.get(1).unwrap().as_str();
            let binder = caps.get(2).unwrap().as_str();
            let cond = caps.get(3).unwrap().as_str();
            if binder == cond {
                let replacement = format!(
                    "{{ let mut __all = true; let mut __i: usize = 0; \
                     while __i < {vec}.len() \
                     {{ if !{vec}[__i] \
                     {{ __all = false; break; }} __i += 1; }} __all }}"
                );
                text = text.replace(&caps[0], &replacement);
                changed = true;
            }
        }

        if changed {
            let _ = std::fs::write(path, &text);
        }
    }
}