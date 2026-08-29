//! Translate Rust projects to Lean via Charon + Aeneas.
//!
//! Port of ``translation/translate.py`` Translator class.

use std::path::{Path, PathBuf};

use crate::log::{self, fail, phase, warn};
use crate::{
    charon_aeneas, extract_test, lean_post, simplify, utils,
};

/// Read the ``[package.metadata.autospec].<key>`` from a crate's Cargo.toml.
fn read_autospec_key(project_path: &Path, key: &str) -> Option<String> {
    let cargo = project_path.join("Cargo.toml");
    if !cargo.exists() {
        return None;
    }
    let text = std::fs::read_to_string(&cargo).ok()?;
    let val: toml::Value = toml::from_str(&text).ok()?;
    val.get("package")?
        .get("metadata")?
        .get("autospec")?
        .get(key)?
        .as_str()
        .map(|s| s.to_string())
}

/// Translate Rust projects to Lean via Charon + Aeneas.
pub struct Translator {
    pub charon_bin: String,
    pub aeneas_bin: String,
    pub verbose: bool,
    pub float_model: Option<String>,
    pub loops_to_rec: bool,
    pub int_model: Option<String>,
}

impl Translator {
    pub fn new(
        charon_bin: Option<String>,
        aeneas_bin: Option<String>,
        verbose: bool,
        float_model: Option<String>,
        loops_to_rec: bool,
        int_model: Option<String>,
    ) -> Self {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();

        let charon = charon_bin.unwrap_or_else(|| {
            root.join("aeneas")
                .join("charon")
                .join("bin")
                .join("charon")
                .to_string_lossy()
                .to_string()
        });

        let aeneas = aeneas_bin.unwrap_or_else(|| {
            root.join("aeneas")
                .join("bin")
                .join("aeneas")
                .to_string_lossy()
                .to_string()
        });

        Self {
            charon_bin: charon,
            aeneas_bin: aeneas,
            verbose,
            float_model,
            loops_to_rec,
            int_model,
        }
    }

    /// Translate a Rust project to Lean.
    ///
    /// ``src/lib.rs`` (implementation) → ``<Crate>.lean``.
    /// ``tests/*.rs`` (concrete tests) → ``Test_<name>.lean``.
    pub fn translate(
        &mut self,
        project_dir: &Path,
        output_dir: Option<&Path>,
    ) -> bool {
        log::set_verbose(self.verbose);

        let project_path = project_dir.canonicalize().unwrap_or_else(|_| project_dir.to_path_buf());
        let output_path = match output_dir {
            Some(d) => d.canonicalize().unwrap_or_else(|_| d.to_path_buf()),
            None => project_path.clone(),
        };
        if let Err(e) = std::fs::create_dir_all(&output_path) {
            fail("mkdir", &format!("{}", e));
            return false;
        }

        if !project_path.join("Cargo.toml").exists() {
            fail("validate", &format!("no Cargo.toml in {}", project_path.display()));
            return false;
        }

        // Resolve models
        if self.int_model.is_none() {
            self.int_model = Some(read_autospec_key(&project_path, "int-model").unwrap_or_else(|| "int".to_string()));
        }
        if self.float_model.is_none() {
            self.float_model = Some(read_autospec_key(&project_path, "float-model").unwrap_or_else(|| "Rat".to_string()));
        }

        let int_model = self.int_model.as_ref().unwrap();
        let float_model = self.float_model.as_ref().unwrap();
        phase("model", &format!("int-model={} float-model={}", int_model, float_model));

        println!(
            "autospec  \x1b[1m{}\x1b[0m  →  {}",
            project_path.file_name().unwrap_or_default().to_string_lossy(),
            output_path.display(),
        );

        let mut results: Vec<(String, String, usize)> = Vec::new();

        // 1. Translate implementation
        let mut float_model = self.float_model.clone().unwrap();
        let outcome = self.translate_inner(&project_path, &output_path, &float_model);
        let outcome = match outcome {
            Some(o) => o,
            None => {
                warn("auto-switching to --float-model Real");
                float_model = "Real".to_string();
                self.float_model = Some(float_model.clone());
                match self.translate_inner(&project_path, &output_path, "Real") {
                    Some(o) => o,
                    None => return false,
                }
            }
        };

        let (impl_defs, impl_files) = outcome;
        for name in {
            let mut sorted = impl_files;
            sorted.sort();
            sorted
        } {
            results.push(("impl".to_string(), name, impl_defs));
        }

        // 2. Translate test files (skip for Real)
        if float_model != "Real" {
            let tests_dir = project_path.join("tests");
            if tests_dir.is_dir() {
                let mut test_files: Vec<_> = std::fs::read_dir(&tests_dir)
                    .ok()
                    .into_iter()
                    .flat_map(|rd| rd.filter_map(|e| e.ok()))
                    .map(|e| e.path())
                    .filter(|p| p.extension().map_or(false, |ext| ext == "rs"))
                    .collect();
                test_files.sort();

                for test_file in &test_files {
                    let test_defs = extract_test::translate_test_file(
                        |p, o| self.translate_inner(p, o, &float_model),
                        &project_path,
                        &output_path,
                        test_file,
                        &float_model,
                        int_model,
                    );
                    if let Some(defs) = test_defs {
                        let stem = test_file.file_stem().unwrap_or_default().to_string_lossy();
                        results.push(("test".to_string(), format!("Test_{}.lean", stem), defs));
                    }
                }
            }
        }

        // Summary
        for (tag, filename, count) in &results {
            phase(tag, &format!("{}  {} defs", filename, count));
        }
        let total_files = results.len();
        let total_defs = results.iter().map(|r| r.2).sum::<usize>();
        phase(
            "done",
            &format!("{} file(s), {} defs, float-model={}", total_files, total_defs, float_model),
        );
        true
    }

    // ── core pipeline ───────────────────────────────────────────────────────

    fn translate_inner(
        &self,
        project_path: &Path,
        output_path: &Path,
        float_model: &str,
    ) -> Option<(usize, Vec<String>)> {
        let workdir = tempfile::tempdir().ok()?;
        let workdir_path = workdir.path().to_path_buf();

        let result = (|| -> Option<(usize, Vec<String>)> {
            // Copy project to workdir
            utils::copy_dir_ignore(project_path, &workdir_path, &["target", "lean", "*.lean"])
                .ok()?;

            if !workdir_path.join("Cargo.toml").exists() {
                fail("copy", "target missing Cargo.toml");
                return None;
            }

            simplify::simplify_for_aeneas(&workdir_path);

            if !charon_aeneas::run_charon(&self.charon_bin, &workdir_path) {
                return None;
            }

            if charon_aeneas::run_f64_adapter(&workdir_path).is_none() {
                return None;
            }

            if charon_aeneas::run_i64_adapter(&workdir_path).is_none() {
                return None;
            }

            let aeneas_warnings = charon_aeneas::run_aeneas(
                &self.aeneas_bin,
                &workdir_path,
                self.loops_to_rec,
            )?;

            let (def_count, written) = lean_post::write_lean_output(
                &workdir_path,
                output_path,
                float_model,
                self.int_model.as_deref().unwrap_or("int"),
            );

            if def_count == 0 {
                fail("output", "no definitions written");
                return None;
            }

            for w in &aeneas_warnings {
                warn(w);
            }

            Some((def_count, written))
        })();

        // workdir is cleaned up when tempdir drops
        result
    }
}

