/// Gets the bit at position `n`.
/// Bits are numbered from 0 (least significant) to 7 (most significant).
#[inline]
pub fn bit_at(input: u8, n: u8) -> bool {
    match n < 8 {
        true => input & (1 << n) != 0,
        false => false,
    }
}

/// Sets the bit at position `n` to the given state.
/// Bits are numbered from 0 (least significant) to 7 (most significant).
#[inline]
pub fn set_bit(input: u8, n: u8, state: bool) -> u8 {
    match state {
        true => input | (1 << n),
        false => input & !(1 << n),
    }
}

/// Checks if half carry from bit 3 to bit 4 occurred.
#[inline]
pub fn half_carry_u8(x: u8, y: u8) -> bool {
    ((x & 0x0F) + (y & 0x0F)) & 0x10 == 0x10
}
