import LeanMode.crypto_mpc_gc.CryptoMpcGc
import Mathlib

open Aeneas Aeneas.Std Result
open crypto_mpc_gc

set_option linter.unusedVariables false

/-!
# Specification: Half-Gate Garbled Circuit (Single AND Gate)

Traced from the Swanky `fancy-garbling` crate (GaloisInc):
`edge/fancy-garbling/src/garble/binary_and.rs`
(Zahur-Rosulek-Evans, ePrint 2014/756).

The half-gate scheme garbles an AND gate with only 2 ciphertexts (vs. 4
in classical Yao).  XOR gates are free via the global offset `delta`:
`label_1 = label_0 ^ delta`.  The hash function is tweaked by `tweak2(gate_num, 0)`
to ensure different gates get independent hash outputs.

## Protocol

Garbler:
- Picks random 0-labels `A`, `B` (LSB = color bit).
- Runs `garble_and_gate(gate_num, A, B, delta)` → `(gate0, gate1, C0)`.
- Sends `(gate0, gate1)` to the evaluator.

Evaluator (knows input bits a, b):
- Receives active labels `A' = A ^ (a*delta)`, `B' = B ^ (b*delta)`.
- Runs `evaluate_and_gate(gate_num, A', B', gate0, gate1)` → output label.
- Output label = `C0 ^ ((a & b) * delta)`.

Conformance is checked by `#eval` in `Test_test.lean`.
-/

/-- Half-gate AND gate correctness: evaluation returns the correct output
    label for every input combination.

    Given garbled table `(gate0, gate1, c0)` from `garble_and_gate` with
    gate number `gate_num` and 0-labels `a`, `b`, the evaluator's result
    equals `c0 ^ ((a_bit & b_bit) * delta)`. -/
theorem garble_and_gate_correct
    (gate_num a b delta gate0 gate1 c0 : BitVec 64)
    (a_bit b_bit : BitVec 64)
    (h_garble : garble_and_gate gate_num a b delta = ok (gate0, gate1, c0))
    (h_a_val : a_bit = 0#64 || a_bit = 1#64)
    (h_b_val : b_bit = 0#64 || b_bit = 1#64)
    (h_eval : evaluate_and_gate gate_num
            (a ^^^ (a_bit * delta)) (b ^^^ (b_bit * delta))
            gate0 gate1 = ok out) :
    out = c0 ^^^ ((a_bit &&& b_bit) * delta) := by
  sorry
