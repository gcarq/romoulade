bitflags! {
    /// Represents the attributes of a background map in CGB mode.
    #[derive(Debug, Copy, Clone)]
    pub struct BgMapAttributes: u8 {
        const PALETTE_MASK = 0b0000_0111;
        const BANK         = 0b0000_1000;
        const X_FLIP       = 0b0010_0000;
        const Y_FLIP       = 0b0100_0000;
        const PRIORITY     = 0b1000_0000;
    }
}

bitflags! {
    /// This register is used to address a byte in the CGB’s bg or obj palette RAM.
    /// Since there are 8 palettes, 8 palettes × 4 colors/palette × 2 bytes/color = 64 bytes
    /// can be addressed.
    #[derive(Copy, Clone, PartialEq, Default)]
    pub struct ColorSpec: u8 {
        const ADDRESS_MASK   = 0b0011_1111;
        const AUTO_INCREMENT = 0b1000_0000;
    }
}

impl ColorSpec {
    /// Returns the address part of the BCPs register.
    #[inline]
    pub fn address(&self) -> u8 {
        self.bits() & Self::ADDRESS_MASK.bits()
    }

    /// Checks whether auto-increment is enabled and increments the address if so.
    #[inline]
    pub fn maybe_increment(&mut self) {
        if !self.contains(Self::AUTO_INCREMENT) {
            return;
        }
        let value = self.bits() & !Self::ADDRESS_MASK.bits() | (self.address() + 1);
        *self = ColorSpec::from_bits_truncate(value);
    }
}

bitflags! {
    /// This register allows to read/write data to the CGBs background palette memory,
    /// addressed through BCPS/BGPI. Each color is stored as little-endian RGB555.
    #[derive(Copy, Clone, PartialEq, Default)]
    pub struct BgColorData: u16 {
        const RED_COLOR_MASK   = 0b0000_0000_0001_1111;
        const GREEN_COLOR_MASK = 0b0000_0011_1110_0000;
        const BLUE_COLOR_MASK  = 0b0111_1100_0000_0000;
    }
}

impl BgColorData {
    pub fn set_red(&mut self, value: u8) {
        let value = self.bits() | (Self::RED_COLOR_MASK.bits() & value as u16);
        *self = BgColorData::from_bits_truncate(value);
    }

    pub fn set_green(&mut self, value: u8) {
        let value = self.bits() | (Self::GREEN_COLOR_MASK.bits() & value as u16);
        *self = BgColorData::from_bits_truncate(value);
    }

    pub fn set_blue(&mut self, value: u8) {
        let value = self.bits() | (Self::BLUE_COLOR_MASK.bits() & value as u16);
        *self = BgColorData::from_bits_truncate(value);
    }

    /// Returns the red component of the color.
    #[inline]
    pub fn red(&self) -> u8 {
        (self.bits() & Self::RED_COLOR_MASK.bits()) as u8
    }

    /// Returns the green component of the color.
    #[inline]
    pub fn green(&self) -> u8 {
        ((self.bits() & Self::GREEN_COLOR_MASK.bits()) >> 5) as u8
    }

    /// Returns the blue component of the color.
    #[inline]
    pub fn blue(&self) -> u8 {
        ((self.bits() & Self::BLUE_COLOR_MASK.bits()) >> 10) as u8
    }
}
