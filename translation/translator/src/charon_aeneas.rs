//! Charon → LLBC adapter → Aeneas pipeline stage.
//!
//! Port of ``translation/pipeline/charon_aeneas.py``.

use std::path::Path;
use std::process::Command;

use crate::log;

/// Run charon to produce ``.llbc`` files. Returns true on success.
pub fn run_charon(charon_bin: &str, workdir: &Path) -> bool {
    let result = Command::new(charon_bin)
        .args(["cargo", "--preset=aeneas", "--", "--lib"])
        .current_dir(workdir)
        .output();

    match result {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.trim().is_empty() {
                log::dbg(stdout.trim());
            }
            if output.status.success() {
                log::phase("charon", "ok");
                true
            } else {
                log::fail("charon", "failed");
                for line in String::from_utf8_lossy(&output.stderr).lines() {
                    let trimmed = line.trim();
                    if !trimmed.is_empty() {
                        eprintln!("           {}", trimmed);
                    }
                }
                false
            }
        }
        Err(e) => {
            log::fail("charon", &format!("failed to run: {}", e));
            false
        }
    }
}

/// Run the f64 LLBC adapter (Rust native) on every ``.llbc`` file in *workdir*.
pub fn run_f64_adapter(workdir: &Path) -> Option<()> {
    run_rust_adapter(workdir, "f64", |input, output| {
        crate::llbc::f64_adapter::transform_llbc(input, output).map(|_| ())
    })
}

/// Run the i64 LLBC adapter (Rust native) on every ``.llbc`` file in *workdir*.
pub fn run_i64_adapter(workdir: &Path) -> Option<()> {
    run_rust_adapter(workdir, "i64", |input, output| {
        crate::llbc::i64_adapter::transform_i64_llbc(input, output).map(|_| ())
    })
}

fn run_rust_adapter(
    workdir: &Path,
    tag: &str,
    transform: impl Fn(&Path, &Path) -> anyhow::Result<()>,
) -> Option<()> {
    let llbc_files: Vec<_> = std::fs::read_dir(workdir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "llbc"))
        .collect();

    if llbc_files.is_empty() {
        log::fail(tag, "no .llbc file found");
        return None;
    }

    for f in &llbc_files {
        let tmp = f.with_extension("llbc.tmp");
        match transform(f, &tmp) {
            Ok(()) => {
                // If the adapter didn't write to tmp (e.g. no matching types),
                // just copy the original so the rename always succeeds.
                if !tmp.exists() {
                    if let Err(e) = std::fs::copy(f, &tmp) {
                        log::fail(tag, &format!("copy failed: {}", e));
                        return None;
                    }
                }
                if let Err(e) = std::fs::rename(&tmp, f) {
                    log::fail(tag, &format!("rename failed: {}", e));
                    return None;
                }
                log::phase(tag, "ok");
            }
            Err(e) => {
                log::fail(tag, &format!("{:?}", e));
                return None;
            }
        }
    }
    Some(())
}

/// Run aeneas on every ``.llbc`` file in *workdir*.
///
/// Returns a list of warning strings, or None on error.
pub fn run_aeneas(
    aeneas_bin: &str,
    workdir: &Path,
    loops_to_rec: bool,
) -> Option<Vec<String>> {
    let llbc_files: Vec<_> = std::fs::read_dir(workdir)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "llbc"))
        .collect();

    let mut warnings: Vec<String> = Vec::new();

    for f in &llbc_files {
        let mut cmd = Command::new(aeneas_bin);
        cmd.args(["-all-computable", "-sequential", "-backend", "lean"]);
        if loops_to_rec {
            cmd.arg("-loops-to-rec");
        }
        cmd.arg(f);
        cmd.current_dir(workdir);

        let result = match cmd.output() {
            Ok(r) => r,
            Err(e) => {
                log::fail("aeneas", &format!("failed to run: {}", e));
                return None;
            }
        };

        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );

        for line in combined.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            log::dbg(&format!("[aeneas] {}", line));
            if line.contains("[Error") || line.to_lowercase().contains("error:") {
                log::fail("aeneas", line);
                return None;
            }
            if let Some(rest) = line
                .strip_prefix("[Warn] ")
                .or_else(|| line.strip_prefix("[Warning] "))
            {
                if !rest.contains("external, unknown definitions") {
                    warnings.push(rest.to_string());
                }
            }
        }
    }

    // Check for .lean files
    let has_lean = std::fs::read_dir(workdir)
        .ok()
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .any(|e| e.path().extension().map_or(false, |ext| ext == "lean"))
        })
        .unwrap_or(false);

    if !has_lean {
        log::fail("aeneas", "no .lean file generated");
        return None;
    }

    log::phase("aeneas", "ok");
    Some(warnings)
}