/// Gets the bit at position `n`.
/// Bits are numbered from 0 (least significant) to 7 (most significant).
pub fn bit_at(input: u8, n: u8) -> bool {
    match n < 8 {
        true => input & (1 << n) != 0,
        false => false,
    }
}

/// Sets the bit at position `n` to the given state.
/// Bits are numbered from 0 (least significant) to 7 (most significant).
pub fn set_bit(input: u8, n: u8, state: bool) -> u8 {
    match state {
        true => input | (1 << n),
        false => input & !(1 << n),
    }
}

/// Checks if half carry from bit 3 to bit 4 occurred.
pub fn half_carry_u8(x: u8, y: u8) -> bool {
    ((x & 0x0F) + (y & 0x0F)) & 0x10 == 0x10
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_at() {
        let x = 0b11110000u8;
        assert!(!bit_at(x, 3));
        assert!(bit_at(x, 4));
    }

    #[test]
    fn test_set_bit() {
        let x = 0b11110000u8;
        assert_eq!(set_bit(x, 0, true), 0b11110001u8);
        assert_eq!(set_bit(x, 1, true), 0b11110010u8);
        assert_eq!(set_bit(x, 0, false), 0b11110000u8);
        assert_eq!(set_bit(x, 7, false), 0b01110000u8);
    }

    #[test]
    fn test_half_carry_u8_true() {
        let x = 62;
        let y = 34;
        assert!(half_carry_u8(x, y));
    }

    #[test]
    fn test_half_carry_u8_false() {
        let x = 34;
        let y = 34;
        assert!(!half_carry_u8(x, y));
    }
}
