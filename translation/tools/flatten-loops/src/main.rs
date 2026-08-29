//! Thin binary shell — the real logic is in the library crate.

use anyhow::{Context, Result};
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: flatten-loops <file.rs> [--in-place]");
        std::process::exit(1);
    }

    let path = PathBuf::from(&args[1]);
    let in_place = args.get(2).map_or(false, |a| a == "--in-place");

    let src = std::fs::read_to_string(&path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let transformed = flatten_loops::transform_file(&src)?;

    if in_place {
        std::fs::write(&path, &transformed)?;
        eprintln!("Wrote {}", path.display());
    } else {
        println!("{}", transformed);
    }
    Ok(())
}