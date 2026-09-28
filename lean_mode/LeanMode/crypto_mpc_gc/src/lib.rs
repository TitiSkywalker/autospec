//! Half-gate garbled circuit — single AND gate.
//!
//! Adapted from the Swanky `fancy-garbling` crate (GaloisInc):
//!   `edge/fancy-garbling/src/garble/binary_and.rs`
//!   (Zahur-Rosulek-Evans, ePrint 2014/756)
//!
//! The half-gate scheme garbles an AND gate with only 2 ciphertexts
//! (vs. 4 in classical Yao).  XOR gates are free (label XOR with delta).
//!
//! Under the `bitvec` model, `u64` → `BitVec 64` (fixed-width 64-bit labels).

/// Gate tweak function.  Source: `edge/fancy-garbling/src/util.rs`, `tweak2`.
pub fn tweak2(i: u64, j: u64) -> u64 {
    i ^ (j << 32u32)
}

/// Hash a wire label with a tweak.
/// Simplified from Swanky's `TweakableCircularCorrelationRobustHash`
/// to a pure mixing function.
pub fn hash_wire(label: u64, tweak: u64) -> u64 {
    let x = label ^ tweak;
    let y = x ^ (x << 13u32);
    let z = y ^ (y >> 7u32);
    z ^ (z << 17u32)
}

/// Half-gate garbling of a binary AND gate.
///
/// Source: `edge/fancy-garbling/src/garble/binary_and.rs`, `garble_and_gate`.
///
/// - `gate_num` — unique gate identifier (used as hash tweak)
/// - `a`, `b` — garbler's 0-labels for wires A and B
/// - `delta` — global free-XOR offset (LSB must be 1)
///
/// Returns `(gate0, gate1, c0)` where:
/// - `gate0`, `gate1` — 2-ciphertext garbled table (sent to evaluator)
/// - `c0` — garbler's 0-label for output C = A & B
pub fn garble_and_gate(gate_num: u64, a: u64, b: u64, delta: u64) -> (u64, u64, u64) {
    let alpha = a & 1u64;        // A.color() — LSB
    let beta = (2u64 - (b & 1u64)) % 2u64; // -B.color() mod 2 (= B.color())
    let r = b & 1u64;            // B.color() — garbler's secret

    let x1 = a ^ (delta * alpha); // X1 = A + delta*alpha
    let y1 = b ^ (delta * beta);  // Y1 = B + delta*beta

    let a_sel = a & 1u64; // A.color()
    let b_sel = b & 1u64; // B.color()

    let ad = a ^ delta; // A ^ delta
    let bd = b ^ delta; // B ^ delta

    // newA = conditional_select(&AD, A, a_sel) = if a_sel==0 { AD } else { A }
    //        = if A.color()==0 { A^delta } else { A }
    let new_a = if a_sel == 0u64 { ad } else { a };
    // B_sel = conditional_select(&BD, B, b_sel) = if b_sel==0 { BD } else { B }
    //        = if B.color()==0 { B^delta } else { B }
    let b_sel_label = if b_sel == 0u64 { bd } else { b };
    // idx = conditional_select(&(r as u8), &0u8, a_sel) = if a_sel==0 { r } else { 0 }
    //     = if A.color()==0 { B.color() } else { 0 }
    let idx = if a_sel == 0u64 { r } else { 0u64 };

    let g = tweak2(gate_num, 0u64);
    let ha = hash_wire(new_a, g);
    let hb = hash_wire(b_sel_label, g);
    let hx = hash_wire(x1, g);
    let hy = hash_wire(y1, g);

    // X = H(X1) + delta*(alpha*r mod 2)
    let x = hx ^ (delta * ((alpha * r) % 2u64));
    let y = hy;

    // gate0 = H(newA) ^ select(idx ? X^delta : X)
    let gate0 = if idx == 0u64 { ha ^ x } else { ha ^ x ^ delta };
    // gate1 = H(B_sel) ^ Y ^ A
    let gate1 = hb ^ y ^ a;

    (gate0, gate1, x ^ y)
}

/// Half-gate evaluation of a binary AND gate.
///
/// Source: `edge/fancy-garbling/src/garble/binary_and.rs`, `evaluate_and_gate`.
///
/// - `gate_num` — unique gate identifier (must match garbler's)
/// - `a`, `b` — evaluator's active labels for wires A and B
/// - `gate0`, `gate1` — garbled table from `garble_and_gate`
///
/// Returns the output label for C = A & B.
pub fn evaluate_and_gate(gate_num: u64, a: u64, b: u64, gate0: u64, gate1: u64) -> u64 {
    let a_sel = a & 1u64; // A.color()
    let b_sel = b & 1u64; // B.color()

    let g = tweak2(gate_num, 0u64);
    let ha = hash_wire(a, g);
    let hb = hash_wire(b, g);

    // L = select(A.color() ? ha ^ gate0 : ha)   — garbler's half-gate
    let l = if a_sel == 0u64 { ha } else { ha ^ gate0 };
    // R = select(B.color() ? hb ^ gate1 : hb)   — evaluator's half-gate
    let r_val = if b_sel == 0u64 { hb } else { hb ^ gate1 };

    // L ^ R ^ (A if B.color()==1 else 0)
    l ^ r_val ^ (a * b_sel)
}