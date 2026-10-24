//! Counting domino tilings of a `w × h` grid — broken-profile DP.

/// Number of ways to tile a `w × h` grid completely with `1 × 2` and `2 × 1`
/// dominoes, via the broken-profile (frontier mask) dynamic program.
///
/// Cells are processed in row-major order.  The mask holds one bit per column:
/// for the cell `(i, j)` currently being processed, bit `k` with `k < j` records
/// that cell `(i+1, k)` is already covered (by a vertical domino coming from
/// row `i`), and bit `k` with `k >= j` records that cell `(i, k)` is already
/// covered.  `dp[mask]` counts the partial coverings of the processed prefix
/// whose frontier is exactly `mask`.  After the last cell only the empty
/// frontier can complete the grid, so the answer is `dp[0]`.
///
/// Requires `w <= 8` (the mask is a `usize` bitfield and `2^w` states are
/// enumerated).
pub fn domino_tilings(w: usize, h: usize) -> usize {
    let total = 1usize << w;
    let mut dp = vec![0usize; total];
    if total == 0 {
        return 0;
    }
    dp[0] = 1;
    let mut i = 0;
    while i < h {
        let mut j = 0;
        while j < w {
            let bit = 1usize << j;
            let mut next = vec![0usize; total];
            let mut mask = 0;
            while mask < total {
                let cur = dp[mask];
                if cur != 0 {
                    let filled = mask & bit;
                    if filled != 0 {
                        // The cell is already covered from above: the cell below
                        // it is free again, so clear the bit.
                        let nm = mask ^ bit;
                        let acc = next[nm] + cur;
                        next[nm] = acc;
                    } else {
                        // Vertical domino: also covers `(i+1, j)`.
                        if i + 1 < h {
                            let nv = mask | bit;
                            let acc = next[nv] + cur;
                            next[nv] = acc;
                        }
                        // Horizontal domino: also covers `(i, j+1)`.
                        if j + 1 < w {
                            let bit2 = 1usize << (j + 1);
                            let taken = mask & bit2;
                            if taken == 0 {
                                // Bit `j` is already clear: after moving past
                                // cell `(i, j)` it would stand for `(i+1, j)`.
                                let nh = mask | bit2;
                                let acc = next[nh] + cur;
                                next[nh] = acc;
                            }
                        }
                    }
                }
                mask += 1;
            }
            dp = next;
            j += 1;
        }
        i += 1;
    }
    dp[0]
}
