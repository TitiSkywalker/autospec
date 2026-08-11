//! Generate a random integer in `[0, n)` using an opaque random source.

unsafe extern "C" {
    /// Opaque random source — modeled axiomatically in Lean.
    fn random() -> usize;
}

/// Return a random integer in `[0, n)`, or `0` if `n == 0`.
pub fn random_integer(n: usize) -> usize {
    if n == 0 {
        return 0;
    }
    let r = unsafe { random() };
    r % n
}
