/// Holds all CPU registers
#[derive(Copy, Clone)]
pub struct Registers {
    pub pc: u16,
    pub sp: u16,
    pub a: u8,
    pub b: u8,
    pub c: u8,
    pub d: u8,
    pub e: u8,
    pub f: FlagsRegister,
    pub h: u8,
    pub l: u8,
}

impl Registers {
    #[inline(always)]
    pub fn get_af(self) -> u16 {
        (u16::from(self.a) << 8) | u16::from(self.f.bits())
    }

    #[inline(always)]
    pub const fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.f = FlagsRegister::from_bits_truncate(value as u8);
    }

    #[inline(always)]
    pub fn get_bc(self) -> u16 {
        (u16::from(self.b) << 8) | u16::from(self.c)
    }

    #[inline(always)]
    pub const fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    #[inline(always)]
    pub fn get_de(self) -> u16 {
        (u16::from(self.d) << 8) | u16::from(self.e)
    }

    #[inline(always)]
    pub const fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    #[inline(always)]
    pub fn get_hl(self) -> u16 {
        (u16::from(self.h) << 8) | u16::from(self.l)
    }

    #[inline(always)]
    pub const fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }
}

impl Default for Registers {
    #[inline]
    fn default() -> Self {
        Self {
            pc: 0x0000,
            sp: 0x0000,
            a: 0,
            b: 0,
            c: 0,
            d: 0,
            e: 0,
            f: FlagsRegister::empty(),
            h: 0,
            l: 0,
        }
    }
}

bitflags! {
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
    #[derive(Copy, Clone)]
    pub struct FlagsRegister: u8 {
        const ZERO = 0b1000_0000;
        const SUBTRACTION = 0b0100_0000;
        const HALF_CARRY = 0b0010_0000;
        const CARRY = 0b0001_0000;
    }
}

impl FlagsRegister {
    #[inline(always)]
    pub fn update(&mut self, zero: bool, subtraction: bool, half_carry: bool, carry: bool) {
        self.set(FlagsRegister::ZERO, zero);
        self.set(FlagsRegister::SUBTRACTION, subtraction);
        self.set(FlagsRegister::HALF_CARRY, half_carry);
        self.set(FlagsRegister::CARRY, carry);
    }
}
