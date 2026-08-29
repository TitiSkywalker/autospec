//! Pipeline logging helpers.

use std::sync::atomic::{AtomicBool, Ordering};

static VERBOSE: AtomicBool = AtomicBool::new(false);

pub fn set_verbose(v: bool) {
    VERBOSE.store(v, Ordering::Relaxed);
}

pub fn is_verbose() -> bool {
    VERBOSE.load(Ordering::Relaxed)
}

/// Print debug info when verbose mode is on.
pub fn dbg(msg: &str) {
    if is_verbose() {
        let _ = std::io::Write::flush(&mut std::io::stdout());
        eprintln!("  {:<10} \x1b[2m{}\x1b[0m", "debug", msg);
    }
}

/// Print a pipeline phase status line with consistent alignment.
pub fn phase(tag: &str, msg: &str) {
    println!("  {:<10} {}", tag, msg);
}

/// Print a phase failure message to stderr.
pub fn fail(tag: &str, reason: &str) {
    let _ = std::io::Write::flush(&mut std::io::stdout());
    eprintln!("  {:<10} \x1b[31mFAIL\x1b[0m  {}", tag, reason);
}

/// Print a warning message.
pub fn warn(msg: &str) {
    println!("  {:<10} \x1b[33m{}\x1b[0m", "warn", msg);
}