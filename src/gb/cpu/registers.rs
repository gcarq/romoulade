use std::{convert, fmt};

/// Holds all CPU registers
pub struct Registers {
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
}

impl fmt::Display for Registers {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "a: {:#04x}, b: {:#04x}, c: {:#04x}, d: {:#04x}, e: {:#04x}, h: {:#04x}, l: {:#04x} | f: {}",
            self.a, self.b, self.c, self.d, self.e, self.h, self.l, self.f
        )
    }
}

impl Registers {
    pub fn get_bc(&self) -> u16 {
        (self.b as u16) << 8 | self.c as u16
    }

    pub fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    pub fn get_de(&self) -> u16 {
        (self.d as u16) << 8 | self.e as u16
    }

    pub fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    pub fn get_hl(&self) -> u16 {
        (self.h as u16) << 8 | self.l as u16
    }

    pub fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}

impl Default for Registers {
    fn default() -> Self {
        Self {
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister::from(0x00),
            h: 0,
            l: 0,
        }
    }
}

const ZERO_FLAG_BYTE_POSITION: u8 = 7;
const NEGATIVE_FLAG_BYTE_POSITION: u8 = 6;
const HALF_CARRY_FLAG_BYTE_POSITION: u8 = 5;
const CARRY_FLAG_BYTE_POSITION: u8 = 4;

/// Represents the special purpose "flags" register.
/// Only the upper 4 bits are used.
///
///    ┌-> Carry
///  ┌-+> Subtraction
///  | |
/// 1111 0000
/// | |
/// └-+> Zero
///   └-> Half Carry
#[derive(PartialEq, Debug)]
pub struct FlagsRegister {
    pub zero: bool,
    pub negative: bool,
    pub half_carry: bool,
    pub carry: bool,
}

impl FlagsRegister {
    pub fn update(&mut self, zero: bool, negative: bool, half_carry: bool, carry: bool) {
        self.zero = zero;
        self.negative = negative;
        self.half_carry = half_carry;
        self.carry = carry;
    }
}

impl fmt::Display for FlagsRegister {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "z: {:#04x}, n: {:#04x}, h: {:#04x}, c: {:#04x}",
            self.zero as u8, self.negative as u8, self.half_carry as u8, self.carry as u8
        )
    }
}

impl convert::From<FlagsRegister> for u8 {
    fn from(flag: FlagsRegister) -> u8 {
        (if flag.zero { 1 } else { 0 }) << ZERO_FLAG_BYTE_POSITION
            | (if flag.negative { 1 } else { 0 }) << NEGATIVE_FLAG_BYTE_POSITION
            | (if flag.half_carry { 1 } else { 0 }) << HALF_CARRY_FLAG_BYTE_POSITION
            | (if flag.carry { 1 } else { 0 }) << CARRY_FLAG_BYTE_POSITION
    }
}

impl convert::From<u8> for FlagsRegister {
    fn from(byte: u8) -> Self {
        let zero = ((byte >> ZERO_FLAG_BYTE_POSITION) & 0b1) != 0;
        let subtract = ((byte >> NEGATIVE_FLAG_BYTE_POSITION) & 0b1) != 0;
        let half_carry = ((byte >> HALF_CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;
        let carry = ((byte >> CARRY_FLAG_BYTE_POSITION) & 0b1) != 0;

        FlagsRegister {
            zero,
            negative: subtract,
            half_carry,
            carry,
        }
    }
}
