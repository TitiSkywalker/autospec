//! Translate Rust projects into Lean via Charon + Aeneas.
//!
//! Usage::
//!
//!   translator <project-dir> [OPTIONS]

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "translator", about = "Translate Rust projects into Lean")]
struct Cli {
    /// Path to the Rust project directory
    project_dir: PathBuf,

    /// Output directory (defaults to project directory)
    #[arg(short = 'o', long)]
    output_dir: Option<PathBuf>,

    /// Print debug output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Path to charon binary
    #[arg(long)]
    charon: Option<String>,

    /// Path to aeneas binary
    #[arg(long)]
    aeneas: Option<String>,

    /// Float model: Rat (computable, default) or Real (computable sqrt)
    #[arg(long, value_parser = ["Rat", "Real"])]
    float_model: Option<String>,

    /// Pass -loops-to-rec to aeneas
    #[arg(long, default_value = "true")]
    loops_to_rec: bool,

    /// Integer model: int (all → Int), nat (unsigned → Nat), bitvec (unsigned → BitVec w)
    #[arg(long, value_parser = ["int", "nat", "bitvec"])]
    int_model: Option<String>,
}

fn main() {
    let args = Cli::parse();

    let mut translator = translator::Translator::new(
        args.charon,
        args.aeneas,
        args.verbose,
        args.float_model,
        args.loops_to_rec,
        args.int_model,
    );

    let ok = translator.translate(
        &args.project_dir,
        args.output_dir.as_deref(),
    );

    std::process::exit(if ok { 0 } else { 1 });
}