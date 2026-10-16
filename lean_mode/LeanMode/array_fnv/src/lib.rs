//! FNV-1a 64-bit hash (Fowler–Noll–Vo).
//!
//! The FNV-1a variant: for each symbol `b`, `hash = (hash XOR b) · prime`, where
//! the multiply is wrapping mod 2^64.  Uses the `bitvec` int-model (unsigned
//! `u64` → `BitVec 64`) so the XOR and wrapping multiply are first-class.

const FNV_OFFSET: u64 = 14695981039346656037;
const FNV_PRIME: u64 = 1099511628211;

/// FNV-1a 64-bit hash of a sequence of integer-encoded bytes (`u64` values).
pub fn fnv1a(data: &[u64]) -> u64 {
    let mut hash = FNV_OFFSET;
    let n = data.len();
    let mut i = 0;
    while i < n {
        hash = (hash ^ data[i]).wrapping_mul(FNV_PRIME);
        i += 1;
    }
    hash
}
