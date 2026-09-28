// Half-gate garbled circuit test vectors.
// Traced from Swanky `edge/fancy-garbling/src/garble/binary_and.rs`.

// --- tweak2 ---

#[test]
fn test_tweak2() {
    assert_eq!(crypto_mpc_gc::tweak2(0, 0), 0);
    assert_eq!(crypto_mpc_gc::tweak2(1, 0), 1);
    assert_eq!(crypto_mpc_gc::tweak2(0, 1), 1u64 << 32u32);
}

// --- Single AND gate correctness (all 4 input combinations) ---

#[test]
fn test_garble_and_gate_00() {
    let delta = 0x0123456789ABCDEFu64; // LSB = 1
    let a0 = 0xAAAAAAAAAAAAAAAAu64;
    let b0 = 0xBBBBBBBBBBBBBBBBu64;
    let gate_num = 0u64;

    let (gate0, gate1, c0) = crypto_mpc_gc::garble_and_gate(gate_num, a0, b0, delta);

    // a=0, b=0 → C = 0
    let a = a0;
    let b = b0;
    let out = crypto_mpc_gc::evaluate_and_gate(gate_num, a, b, gate0, gate1);
    assert_eq!(out, c0); // C0 (output for 0)
}

#[test]
fn test_garble_and_gate_10() {
    let delta = 0x0123456789ABCDEFu64;
    let a0 = 0xAAAAAAAAAAAAAAAAu64;
    let b0 = 0xBBBBBBBBBBBBBBBBu64;
    let gate_num = 0u64;

    let (gate0, gate1, c0) = crypto_mpc_gc::garble_and_gate(gate_num, a0, b0, delta);

    // a=1, b=0 → C = 0
    let a = a0 ^ delta;
    let b = b0;
    let out = crypto_mpc_gc::evaluate_and_gate(gate_num, a, b, gate0, gate1);
    assert_eq!(out, c0);
}

#[test]
fn test_garble_and_gate_01() {
    let delta = 0x0123456789ABCDEFu64;
    let a0 = 0xAAAAAAAAAAAAAAAAu64;
    let b0 = 0xBBBBBBBBBBBBBBBBu64;
    let gate_num = 0u64;

    let (gate0, gate1, c0) = crypto_mpc_gc::garble_and_gate(gate_num, a0, b0, delta);

    // a=0, b=1 → C = 0
    let a = a0;
    let b = b0 ^ delta;
    let out = crypto_mpc_gc::evaluate_and_gate(gate_num, a, b, gate0, gate1);
    assert_eq!(out, c0);
}

#[test]
fn test_garble_and_gate_11() {
    let delta = 0x0123456789ABCDEFu64;
    let a0 = 0xAAAAAAAAAAAAAAAAu64;
    let b0 = 0xBBBBBBBBBBBBBBBBu64;
    let gate_num = 0u64;

    let (gate0, gate1, c0) = crypto_mpc_gc::garble_and_gate(gate_num, a0, b0, delta);

    // a=1, b=1 → C = 1
    let a = a0 ^ delta;
    let b = b0 ^ delta;
    let out = crypto_mpc_gc::evaluate_and_gate(gate_num, a, b, gate0, gate1);
    assert_eq!(out, c0 ^ delta); // C1 = C0 ^ delta
}

// --- Different gate_num ---

#[test]
fn test_garble_and_gate_different_num() {
    let delta = 0x123456789ABCDEF1u64;
    let a0 = 0x5555555555555555u64;
    let b0 = 0x3333333333333333u64;

    let (g0, g1, c0) = crypto_mpc_gc::garble_and_gate(7u64, a0, b0, delta);

    // All 4 cases with gate_num=7
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(7u64, a0, b0, g0, g1), c0);
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(7u64, a0 ^ delta, b0, g0, g1), c0);
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(7u64, a0, b0 ^ delta, g0, g1), c0);
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(7u64, a0 ^ delta, b0 ^ delta, g0, g1), c0 ^ delta);
}

// --- Different labels, same delta ---

#[test]
fn test_garble_and_gate_consistency() {
    let delta = 0x9876543210FEDCBBu64; // LSB=1
    // Gate 0
    let a0_0 = 0x1111111111111111u64;
    let b0_0 = 0x3333333333333333u64;
    let (g0_0, g1_0, c0_0) = crypto_mpc_gc::garble_and_gate(0u64, a0_0, b0_0, delta);
    // Gate 1
    let a0_1 = 0x2222222222222222u64;
    let b0_1 = 0x4444444444444444u64;
    let (g0_1, g1_1, c0_1) = crypto_mpc_gc::garble_and_gate(1u64, a0_1, b0_1, delta);

    // Both evaluate correctly
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(0u64, a0_0, b0_0, g0_0, g1_0), c0_0);
    assert_eq!(crypto_mpc_gc::evaluate_and_gate(1u64, a0_1 ^ delta, b0_1 ^ delta, g0_1, g1_1), c0_1 ^ delta);
}