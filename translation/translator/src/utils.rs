//! Shared utilities for the translation pipeline.

use std::path::Path;

/// Read package name from Cargo.toml, or fall back to directory name.
pub fn read_crate_name(project_path: &Path) -> String {
    let toml_path = project_path.join("Cargo.toml");
    if let Ok(text) = std::fs::read_to_string(&toml_path) {
        if let Some(name) = toml::from_str::<toml::Value>(&text)
            .ok()
            .and_then(|v| {
                v.get("package")?
                    .get("name")?
                    .as_str()
                    .map(|s| s.replace('-', "_"))
            })
        {
            return name;
        }
    }
    project_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .replace('-', "_")
}

/// Uppercase the first character, leave the rest unchanged.
pub fn capitalize_first(s: &str) -> String {
    let mut chars: Vec<char> = s.chars().collect();
    if let Some(c) = chars.first_mut() {
        *c = c.to_uppercase().next().unwrap_or(*c);
    }
    chars.into_iter().collect()
}

/// Walk up from *path* to find a directory containing lakefile.toml or lakefile.lean.
pub fn find_lake_root(path: &Path) -> Option<&Path> {
    let mut current = Some(path);
    while let Some(p) = current {
        if p.join("lakefile.toml").exists() || p.join("lakefile.lean").exists() {
            return Some(p);
        }
        current = p.parent();
    }
    None
}

/// Parse adapter stderr lines like ``[adapter] f64=306, usize=311, ...``.
///
/// Returns a flat map of key-value pairs.
pub fn parse_adapter_stderr(stderr: &str) -> std::collections::HashMap<String, String> {
    let mut stats = std::collections::HashMap::new();
    let re = regex::Regex::new(r"(\d+)\s+float consts?,\s*(\d+)\s+binop/neg").unwrap();
    for line in stderr.lines() {
        let line = line.trim();
        if let Some(body) = line.strip_prefix("[adapter] ") {
            let body = body.trim();
            if body.contains('=') {
                for part in body.split(',') {
                    let part = part.trim();
                    if let Some((k, v)) = part.split_once('=') {
                        stats.insert(k.trim().to_string(), v.trim().to_string());
                    }
                }
            } else if body.contains("no f64") {
                stats.insert("f64".to_string(), "none".to_string());
            } else if let Some(caps) = re.captures(body) {
                stats.insert(
                    "consts".to_string(),
                    caps.get(1).unwrap().as_str().to_string(),
                );
                stats.insert(
                    "binop".to_string(),
                    caps.get(2).unwrap().as_str().to_string(),
                );
            }
        }
    }
    stats
}

/// Recursively copy a directory, ignoring patterns like Python's shutil.copytree(ignore=...).
pub fn copy_dir_ignore(
    src: &Path,
    dst: &Path,
    ignore_patterns: &[&str],
) -> std::io::Result<()> {
    if !dst.exists() {
        std::fs::create_dir_all(dst)?;
    }
    for entry in walkdir::WalkDir::new(src)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        let rel = path.strip_prefix(src).unwrap_or(path);

        let rel_str = rel.to_string_lossy();
        let should_ignore = ignore_patterns.iter().any(|pat| {
            if pat.contains('*') {
                let glob_pat = pat.replace('*', "");
                rel_str.contains(&glob_pat) || rel_str.ends_with(&glob_pat)
            } else {
                rel_str == *pat || rel_str.starts_with(&format!("{}/", pat))
            }
        });
        if should_ignore {
            continue;
        }

        let target = dst.join(rel);
        if path.is_dir() {
            std::fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(path, &target)?;
        }
    }
    Ok(())
}