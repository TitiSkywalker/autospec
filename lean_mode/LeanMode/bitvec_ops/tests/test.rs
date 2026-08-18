// ── rotation ────────────────────────────────────────────────────────────────

#[test]
fn test_rotate_left_1() {
    assert_eq!(bitvec_ops::rotate_left(0x0000_0001, 1), 0x0000_0002);
}

#[test]
fn test_rotate_left_wraps() {
    assert_eq!(bitvec_ops::rotate_left(0x8000_0000, 1), 0x0000_0001);
}

#[test]
fn test_rotate_right_1() {
    assert_eq!(bitvec_ops::rotate_right(0x8000_0000, 1), 0x4000_0000);
}

#[test]
fn test_rotate_roundtrip() {
    let x: u32 = 0x1234_5678;
    assert_eq!(bitvec_ops::rotate_left(bitvec_ops::rotate_right(x, 7), 7), x);
}

#[test]
fn test_rotate_by_full_width_is_identity() {
    assert_eq!(bitvec_ops::rotate_left(0x0F0F_0F0F, 32), 0x0F0F_0F0F);
}

// ── wrapping arithmetic ─────────────────────────────────────────────────────

#[test]
fn test_wrapping_add_no_overflow() {
    assert_eq!(bitvec_ops::wrapping_add(7, 13), 20);
}

#[test]
fn test_wrapping_add_overflow_wraps() {
    assert_eq!(bitvec_ops::wrapping_add(0xFFFF_FFFF, 1), 0);
}

#[test]
fn test_wrapping_sub_underflow_wraps() {
    assert_eq!(bitvec_ops::wrapping_sub(0, 1), 0xFFFF_FFFF);
}

#[test]
fn test_wrapping_sub_plain() {
    assert_eq!(bitvec_ops::wrapping_sub(5, 3), 2);
}

#[test]
fn test_wrapping_mul_overflow_wraps() {
    assert_eq!(bitvec_ops::wrapping_mul(0x8000_0000, 2), 0);
}

#[test]
fn test_wrapping_mul_plain() {
    assert_eq!(bitvec_ops::wrapping_mul(3, 4), 12);
}

#[test]
fn test_wrapping_neg_one() {
    assert_eq!(bitvec_ops::wrapping_neg(1), 0xFFFF_FFFF);
}

#[test]
fn test_wrapping_neg_zero() {
    assert_eq!(bitvec_ops::wrapping_neg(0), 0);
}

// ── bitwise logic ───────────────────────────────────────────────────────────

#[test]
fn test_bit_and() {
    assert_eq!(bitvec_ops::bit_and(0xF0F0_F0F0, 0xFF00_FF00), 0xF000_F000);
}

#[test]
fn test_bit_and_zero() {
    assert_eq!(bitvec_ops::bit_and(0x1234_5678, 0), 0);
}

#[test]
fn test_bit_or() {
    assert_eq!(bitvec_ops::bit_or(0x0F0F_0F0F, 0xF0F0_F0F0), 0xFFFF_FFFF);
}

#[test]
fn test_bit_xor() {
    assert_eq!(bitvec_ops::bit_xor(0xFFFF_0000, 0xFFFF_FFFF), 0x0000_FFFF);
}

#[test]
fn test_bit_xor_self_is_zero() {
    assert_eq!(bitvec_ops::bit_xor(0xA5A5_A5A5, 0xA5A5_A5A5), 0);
}

#[test]
fn test_bit_not_zero() {
    assert_eq!(bitvec_ops::bit_not(0), 0xFFFF_FFFF);
}

#[test]
fn test_bit_not_involutive() {
    let x: u32 = 0x0F0F_0F0F;
    assert_eq!(bitvec_ops::bit_not(bitvec_ops::bit_not(x)), x);
}

// ── shifts ──────────────────────────────────────────────────────────────────

#[test]
fn test_shift_left() {
    assert_eq!(bitvec_ops::shift_left(1, 4), 16);
}

#[test]
fn test_shift_left_wraps() {
    assert_eq!(bitvec_ops::shift_left(0x8000_0000, 1), 0);
}

#[test]
fn test_shift_right() {
    assert_eq!(bitvec_ops::shift_right(0x8000_0000, 31), 1);
}

#[test]
fn test_shift_right_plain() {
    assert_eq!(bitvec_ops::shift_right(0xF0F0_F0F0, 4), 0x0F0F_0F0F);
}

// ── population count ────────────────────────────────────────────────────────

#[test]
fn test_popcount_zero() {
    assert_eq!(bitvec_ops::popcount(0), 0);
}

#[test]
fn test_popcount_all_ones() {
    assert_eq!(bitvec_ops::popcount(0xFFFF_FFFF), 32);
}

#[test]
fn test_popcount_nibble_alternating() {
    assert_eq!(bitvec_ops::popcount(0xF0F0_F0F0), 16);
}

#[test]
fn test_popcount_mixed() {
    assert_eq!(bitvec_ops::popcount(0x1234_5678), 13);
}