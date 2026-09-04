//! Re-translate all Rust test cases in LeanMode/ concurrently (skip failure/).
//!
//! Port of ``scripts/translate_all.py``.

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
    #[arg(short = 'w', long, default_value = "16")]
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

/// Truncate a string to at most ``max`` chars (char-safe), appending an
/// ellipsis when clipped. Keeps failure output readable instead of dumping
/// multi-MB JSON that aeneas emits on error.
fn truncate(s: &str, max: usize) -> String {
    let count = s.chars().count();
    if count <= max {
        s.to_string()
    } else {
        let clipped: String = s.chars().take(max).collect();
        format!("{}…", clipped)
    }
}

/// Extract up to ``limit`` relevant error lines from a subprocess's stderr,
/// each truncated to ``max`` chars.
fn collect_errors(stderr: &str, limit: usize, max: usize) -> Vec<String> {
    stderr
        .lines()
        .map(str::trim)
        .filter(|l| l.contains("FAIL") || l.contains("Error") || l.contains("error"))
        .take(limit)
        .map(|l| truncate(l, max))
        .collect()
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

    // Collect crate directories (skip failure/)
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

    // Concurrency control: Condvar-based semaphore
    let slots = Arc::new((Mutex::new(args.workers), Condvar::new()));

    // Guards the progress lines + counters so output doesn't interleave.
    let out = Arc::new(Mutex::new((0usize, 0usize))); // (passed, failed)

    let mut handles = Vec::with_capacity(dirs.len());
    for d in dirs {
        let name = d.file_name().unwrap().to_string_lossy().to_string();
        let translator = translator.clone();
        let charon = charon.clone();
        let aeneas = aeneas.clone();
        let slots = Arc::clone(&slots);
        let out = Arc::clone(&out);

        handles.push(thread::spawn(move || {
            // Acquire slot
            {
                let (lock, cvar) = &*slots;
                let mut n = lock.lock().unwrap();
                while *n == 0 {
                    n = cvar.wait(n).unwrap();
                }
                *n -= 1;
            }

            let (int_model, float_model, ok, errors) =
                match Command::new(&translator)
                    .arg(&d)
                    .arg("--charon")
                    .arg(&charon)
                    .arg("--aeneas")
                    .arg(&aeneas)
                    .output()
                {
                    Ok(outp) => {
                        let stdout = String::from_utf8_lossy(&outp.stdout);
                        let stderr = String::from_utf8_lossy(&outp.stderr);
                        let ok = outp.status.success();
                        let errors = if ok {
                            Vec::new()
                        } else {
                            collect_errors(&stderr, 5, 300)
                        };
                        (parse_model(&stdout), parse_float_model(&stdout), ok, errors)
                    }
                    Err(e) => (
                        "?".to_string(),
                        "?".to_string(),
                        false,
                        vec![format!("failed to run: {}", e)],
                    ),
                };

            // Print immediately (real-time), under lock to avoid interleaving.
            {
                let mut guard = out.lock().unwrap();
                let status = if ok { "OK" } else { "FAIL" };
                println!(
                    "{:<30} int-model={:<6} float-model={:<4} {}",
                    name, int_model, float_model, status
                );
                for line in &errors {
                    println!("    {}", line);
                }
                use std::io::Write;
                std::io::stdout().flush().ok();
                if ok {
                    guard.0 += 1;
                } else {
                    guard.1 += 1;
                }
            }

            // Release slot
            {
                let (lock, cvar) = &*slots;
                let mut n = lock.lock().unwrap();
                *n += 1;
                cvar.notify_one();
            }
        }));
    }

    for h in handles {
        h.join().ok();
    }

    let (passed, failed) = *out.lock().unwrap();
    println!();
    println!("Passed: {}, Failed: {}", passed, failed);

    if failed > 0 {
        std::process::exit(1);
    }
}