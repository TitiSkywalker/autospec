//! Flatten nested for-loops in Rust source by extracting inner loops into
//! separate functions.  Aeneas cannot translate functions that contain a for-loop
//! inside another for-loop, so this tool guarantees every function has at most
//! one level of for-loop nesting.
//!
//! Re-exported from the binary; use ``transform_file`` as the public API.

mod flatten_impl;

pub use flatten_impl::transform_file;