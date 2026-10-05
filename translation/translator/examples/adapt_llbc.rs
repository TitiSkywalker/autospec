//! Debug helper: run the f64/i64 LLBC adapters standalone on a charon output.
//!
//! Usage:
//!   cargo run --example adapt_llbc -- <input.llbc> <out.f64.llbc> <out.i64.llbc>

use std::path::PathBuf;

fn main() {
    let input = PathBuf::from(std::env::args().nth(1).expect("input .llbc"));
    let out_f64 = PathBuf::from(std::env::args().nth(2).expect("out.f64.llbc"));
    let out_i64 = PathBuf::from(std::env::args().nth(3).expect("out.i64.llbc"));

    let n = translator::llbc::f64_adapter::transform_llbc(&input, &out_f64).unwrap();
    // The f64 adapter skips writing when it finds no f64 type; mirror the real
    // pipeline (run_rust_adapter) and pass the input through unchanged.
    if !out_f64.exists() {
        std::fs::copy(&input, &out_f64).unwrap();
    }
    println!("f64 adapter: {} transforms → {}", n, out_f64.display());

    let m = translator::llbc::i64_adapter::transform_i64_llbc(&out_f64, &out_i64).unwrap();
    if !out_i64.exists() {
        std::fs::copy(&out_f64, &out_i64).unwrap();
    }
    println!("i64 adapter: {} transforms → {}", m, out_i64.display());
}
