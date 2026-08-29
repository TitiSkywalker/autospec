//! Translate Rust projects into Lean via Charon + Aeneas.
//!
//! Pipeline::
//!
//!   Rust source → [simplify] → [flatten-loops] → charon → .llbc
//!   → [LLBC adapter] → aeneas → .lean → [F64→Rat + inline] → output
//!
//! The original project is never modified — all transformations happen on a
//! temporary copy.

pub mod charon_aeneas;
pub mod extract_test;
pub mod lean_inline;
pub mod lean_post;
pub mod llbc;
pub mod log;
pub mod simplify;
pub mod translator;
pub mod utils;

pub use translator::Translator;