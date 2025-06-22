bitflags! {
    /// Represents the CPU mode select register (KEY0) at 0xFF4C in CGB mode.
    #[derive(Copy, Clone, PartialEq)]
    pub struct CpuMode: u8 {
        const DMG_COMPAT = 0b0000_0100;
    }
}

impl Default for CpuMode {
    fn default() -> Self {
        Self::from_bits_truncate(0b0000_0100)
    }
}
