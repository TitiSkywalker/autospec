// ── Independent brute force: enumerate tilings cell by cell ─────────────────

/// Recursively count completions: find the first uncovered cell and try the two
/// placements that can cover it (vertical / horizontal).
fn brute_count(occ: &mut Vec<usize>, w: usize, h: usize, cell: usize) -> usize {
    let ncells = w * h;
    if cell >= ncells {
        return 1;
    }
    if occ[cell] != 0 {
        return brute_count(occ, w, h, cell + 1);
    }
    let row = cell / w;
    let col = cell - row * w;
    let mut count: usize = 0;
    if row + 1 < h {
        let below = cell + w;
        if occ[below] == 0 {
            occ[cell] = 1;
            occ[below] = 1;
            let sub = brute_count(occ, w, h, cell + 1);
            occ[cell] = 0;
            occ[below] = 0;
            count += sub;
        }
    }
    if col + 1 < w {
        let right = cell + 1;
        if occ[right] == 0 {
            occ[cell] = 1;
            occ[right] = 1;
            let sub = brute_count(occ, w, h, cell + 1);
            occ[cell] = 0;
            occ[right] = 0;
            count += sub;
        }
    }
    count
}

/// Brute-force tiling count for a `w × h` grid.
fn brute_tilings(w: usize, h: usize) -> usize {
    let mut occ = vec![0usize; w * h];
    brute_count(&mut occ, w, h, 0)
}

// ── Known-answer tests ──────────────────────────────────────────────────────

#[test]
fn test_2x2() {
    assert_eq!(dp_domino_tiling::domino_tilings(2, 2), 2);
}

#[test]
fn test_2x4() {
    // 2×n tilings are the Fibonacci numbers: 1, 2, 3, 5, 8, 13, 21, 34
    assert_eq!(dp_domino_tiling::domino_tilings(2, 4), 5);
    assert_eq!(dp_domino_tiling::domino_tilings(2, 8), 34);
}

#[test]
fn test_4x4() {
    assert_eq!(dp_domino_tiling::domino_tilings(4, 4), 36);
}

#[test]
fn test_6x6() {
    assert_eq!(dp_domino_tiling::domino_tilings(6, 6), 6728);
}

#[test]
fn test_3x4() {
    assert_eq!(dp_domino_tiling::domino_tilings(3, 4), 11);
    assert_eq!(dp_domino_tiling::domino_tilings(4, 3), 11);
}

#[test]
fn test_1x8() {
    // A single row can only be tiled by horizontal dominoes.
    assert_eq!(dp_domino_tiling::domino_tilings(1, 8), 1);
    assert_eq!(dp_domino_tiling::domino_tilings(8, 1), 1);
}

// ── Edge cases ──────────────────────────────────────────────────────────────

#[test]
fn test_odd_area_is_zero() {
    assert_eq!(dp_domino_tiling::domino_tilings(1, 1), 0);
    assert_eq!(dp_domino_tiling::domino_tilings(3, 3), 0);
    assert_eq!(dp_domino_tiling::domino_tilings(5, 5), 0);
}

#[test]
fn test_empty_grid() {
    // The empty grid has exactly one (empty) tiling.
    assert_eq!(dp_domino_tiling::domino_tilings(0, 0), 1);
    assert_eq!(dp_domino_tiling::domino_tilings(0, 5), 1);
    assert_eq!(dp_domino_tiling::domino_tilings(5, 0), 1);
}

// ── Cross-check against the brute force ─────────────────────────────────────

#[test]
fn test_agrees_with_brute_force() {
    let mut w = 1;
    while w <= 4 {
        let mut h = 1;
        while h <= 4 {
            let got = dp_domino_tiling::domino_tilings(w, h);
            let want = brute_tilings(w, h);
            assert_eq!(got, want);
            h += 1;
        }
        w += 1;
    }
}
