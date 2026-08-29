//! Re-translate all Rust test cases in LeanMode/ concurrently (skip failure/).
//!
//! Port of ``scripts/translate_all.py``.

use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use clap::Parser;

/// Re-translate all LeanMode test cases concurrently.
#[derive(Parser)]
#[command(about = "Re-translate all LeanMode test cases")]
struct Args {
    /// Number of concurrent workers
    #[arg(short = 'w', long, default_value = "8")]
    workers: usize,

    /// Path to the translator binary (defaults to the release build)
    #[arg(long)]
    translator: Option<PathBuf>,

    /// Path to the LeanMode directory
    #[arg(long)]
    lean_mode: Option<PathBuf>,

    /// Path to the charon binary
    #[arg(long)]
    charon: Option<PathBuf>,

    /// Path to the aeneas binary
    #[arg(long)]
    aeneas: Option<PathBuf>,
}

fn parse_model(stdout: &str) -> String {
    // "int-model=bitvec" — take the first occurrence
    for line in stdout.lines() {
        if let Some(pos) = line.find("int-model=") {
            let rest = &line[pos + "int-model=".len()..];
            return rest.split_whitespace().next().unwrap_or("?").to_string();
        }
    }
    "?".to_string()
}

fn parse_float_model(stdout: &str) -> String {
    // "float-model=Rat" — take the last occurrence (after any auto-switch)
    stdout
        .lines()
        .filter_map(|line| {
            let pos = line.find("float-model=")?;
            Some(line[pos + "float-model=".len()..].split_whitespace().next()?.to_string())
        })
        .last()
        .unwrap_or_else(|| "?".to_string())
}

fn main() {
    let args = Args::parse();

    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();

    let lean_mode = args
        .lean_mode
        .unwrap_or_else(|| root.join("lean_mode").join("LeanMode"));

    let translator = args.translator.unwrap_or_else(|| {
        root.join("translation")
            .join("translator")
            .join("target")
            .join("release")
            .join("translator")
    });

    let charon = args.charon.unwrap_or_else(|| {
        root.join("aeneas").join("charon").join("bin").join("charon")
    });

    let aeneas = args
        .aeneas
        .unwrap_or_else(|| root.join("aeneas").join("bin").join("aeneas"));

    // Collect crate directories (sorted, skip failure/)
    let mut dirs: Vec<PathBuf> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&lean_mode) {
        for entry in rd.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir()
                && path.file_name().map_or(false, |n| n != "failure")
                && path.join("Cargo.toml").exists()
            {
                dirs.push(path);
            }
        }
    }
    dirs.sort();

    let total = dirs.len();
    println!("Translating {} projects ({} concurrent)...\n", total, args.workers);

    // Results: ordered map name → (int_model, float_model, ok)
    let results: Arc<Mutex<BTreeMap<String, (String, String, bool)>>> =
        Arc::new(Mutex::new(BTreeMap::new()));

    // Concurrency control: Condvar-based semaphore
    let slots = Arc::new((Mutex::new(args.workers), Condvar::new()));

    let mut handles = Vec::new();
    for d in dirs {
        let name = d.file_name().unwrap().to_string_lossy().to_string();
        let translator = translator.clone();
        let charon = charon.clone();
        let aeneas = aeneas.clone();
        let results = Arc::clone(&results);
        let slots = Arc::clone(&slots);

        let h = thread::spawn(move || {
            // Acquire slot
            {
                let (lock, cvar) = &*slots;
                let mut n = lock.lock().unwrap();
                while *n == 0 {
                    n = cvar.wait(n).unwrap();
                }
                *n -= 1;
            }

            let output = Command::new(&translator)
                .arg(&d)
                .arg("--charon")
                .arg(&charon)
                .arg("--aeneas")
                .arg(&aeneas)
                .output();

            match output {
                Ok(out) => {
                    let stdout = String::from_utf8_lossy(&out.stdout);
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    let ok = out.status.success();
                    let model = parse_model(&stdout);
                    let float_model = parse_float_model(&stdout);

                    let mut lock = results.lock().unwrap();
                    let status = if ok { "OK" } else { "FAIL" };
                    println!(
                        "{:<30} int-model={:<6} float-model={:<4} {}",
                        name, model, float_model, status
                    );
                    if !ok {
                        for line in stderr.lines() {
                            let line = line.trim();
                            if line.contains("FAIL")
                                || line.contains("Error")
                                || line.contains("error")
                            {
                                println!("    {}", line);
                            }
                        }
                    }
                    std::io::stdout().flush().ok();
                    lock.insert(name, (model, float_model, ok));
                }
                Err(e) => {
                    let mut lock = results.lock().unwrap();
                    println!("{:<30} int-model={:<6} float-model={:<4} FAIL", name, "?", "?");
                    println!("    failed to run: {}", e);
                    std::io::stdout().flush().ok();
                    lock.insert(name, ("?".to_string(), "?".to_string(), false));
                }
            }

            // Release slot
            {
                let (lock, cvar) = &*slots;
                let mut n = lock.lock().unwrap();
                *n += 1;
                cvar.notify_one();
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.join().ok();
    }

    let lock = results.lock().unwrap();
    let passed: Vec<_> = lock.iter().filter(|(_, (_, _, ok))| *ok).collect();
    let failed: Vec<_> = lock.iter().filter(|(_, (_, _, ok))| !*ok).collect();

    println!();
    println!("Passed: {}, Failed: {}", passed.len(), failed.len());

    if !failed.is_empty() {
        std::process::exit(1);
    }
}