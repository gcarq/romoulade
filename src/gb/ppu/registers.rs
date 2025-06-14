use crate::gb::oam::OamDmaController;
use crate::gb::ppu::misc::Palette;
use crate::gb::ppu::{HBLANK_CYCLES, OAM_SCAN_CYCLES, PIXEL_TRANSFER_CYCLES, VBLANK_CYCLES};

/// Holds all PPU Registers
#[derive(Clone, Copy, Default)]
pub struct Registers {
    pub lcd_control: LCDControl,   // PPU_LCDC
    pub lcd_stat: LCDState,        // PPU_STAT
    pub ly: u8,                    // PPU_LY
    pub lyc: u8,                   // PPU_LYC
    pub oam_dma: OamDmaController, // PPU_DMA
    pub scy: u8,                   // PPU_SCY
    pub scx: u8,                   // PPU_SCX
    pub bg_palette: Palette,       // PPU_BGP
    pub obj_palette0: Palette,     // PPU_OBP0
    pub obj_palette1: Palette,     // PPU_OBP1
    pub wy: u8,                    // PPU_WY
    pub wx: u8,                    // PPU_WX
}

bitflags! {
    /// Represents PPU_LCDC at 0xFF40
    #[derive(Copy, Clone, Default)]
    pub struct LCDControl: u8 {
        const BG_EN    = 0b00000001; // BG Enable
        const OBJ_EN   = 0b00000010; // OBJ Enable
        const OBJ_SIZE = 0b00000100; // OBJ Size
        const BG_MAP   = 0b00001000; // BG Tile Map Address
        const TILE_SEL = 0b00010000; // BG & Window Tile Data
        const WIN_EN   = 0b00100000; // Window Enable
        const WIN_MAP  = 0b01000000; // Window Tile Map Address
        const LCD_EN   = 0b10000000; // LCD Display Enable
    }

    /// Represents PPU_STAT at 0xFF41
    #[derive(Copy, Clone, Default)]
    pub struct LCDState: u8 {
        const PPU_MODE1   = 0b00000001; // Indicates the PPUs current status
        const PPU_MODE2   = 0b00000010; // Indicates the PPUs current status
        const LYC_STAT    = 0b00000100; // Set when LY contains the same value as LYC
        const H_BLANK_INT = 0b00001000; // Selects the Mode 0 for the STAT interrupt
        const V_BLANK_INT = 0b00010000; // Selects the Mode 1 for the STAT interrupt
        const OAM_INT     = 0b00100000; // Selects the Mode 2 for the STAT interrupt
        const LY_INT      = 0b01000000; // selects the LYC == LY condition for the STAT interrupt
    }
}

impl LCDControl {
    /// `LCDControl::WIN_MAP` controls which background map the Window uses for rendering.
    /// When it’s clear (0), the 0x9800 tilemap is used, otherwise it’s the 0x9C00 one.
    #[inline]
    pub const fn window_tile_map_area(&self) -> u16 {
        match self.contains(LCDControl::WIN_MAP) {
            true => 0x9C00,
            false => 0x9800,
        }
    }

    /// `LCDControl::BG_MAP` works similarly to `LCDControl::WIN_MAP`: if the bit is clear (0),
    /// the BG uses tilemap 0x9800, otherwise tilemap 0x9C00.
    #[inline]
    pub const fn bg_tile_map_area(&self) -> u16 {
        match self.contains(LCDControl::BG_MAP) {
            true => 0x9C00,
            false => 0x9800,
        }
    }
}

impl LCDState {
    /// Returns the `PPUMode` based on the first two bits of `PPU_STAT`.
    #[inline]
    pub fn mode(&self) -> PPUMode {
        PPUMode::from(self.bits())
    }

    /// Sets the first two bits of `PPU_STAT` to the given `PPUMode`.
    #[inline]
    pub const fn set_mode(&mut self, mode: PPUMode) {
        *self = LCDState::from_bits_truncate((self.bits() & 0b1111_1100) | mode as u8);
    }
}

/// Represents the first two bits in `LCDState` for convenience.
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum PPUMode {
    HBlank = 0b00,        // Mode 0
    VBlank = 0b01,        // Mode 1
    OAMScan = 0b10,       // Mode 2
    PixelTransfer = 0b11, // Mode 3
}

impl PPUMode {
    /// Returns the number of cycles for the current mode.
    #[inline]
    pub const fn cycles(self) -> usize {
        match self {
            PPUMode::OAMScan => OAM_SCAN_CYCLES,
            PPUMode::PixelTransfer => PIXEL_TRANSFER_CYCLES,
            PPUMode::HBlank => HBLANK_CYCLES,
            PPUMode::VBlank => VBLANK_CYCLES,
        }
    }
}

impl From<u8> for PPUMode {
    #[inline]
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => PPUMode::HBlank,
            0b01 => PPUMode::VBlank,
            0b10 => PPUMode::OAMScan,
            0b11 => PPUMode::PixelTransfer,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_lcd_mode() {
        let mut state = LCDState::empty();
        assert_eq!(state.mode(), PPUMode::HBlank);

        state = LCDState::PPU_MODE1;
        assert_eq!(state.mode(), PPUMode::VBlank);

        state = LCDState::PPU_MODE2;
        assert_eq!(state.mode(), PPUMode::OAMScan);

        state = LCDState::PPU_MODE1 | LCDState::PPU_MODE2;
        assert_eq!(state.mode(), PPUMode::PixelTransfer);
    }

    #[test]
    fn test_set_lcd_mode() {
        let mut state = LCDState::empty();
        state.set_mode(PPUMode::HBlank);
        assert_eq!(state.bits(), 0b00000000);

        state.set_mode(PPUMode::VBlank);
        assert_eq!(state.bits(), 0b00000001);

        state.set_mode(PPUMode::OAMScan);
        assert_eq!(state.bits(), 0b00000010);

        state.set_mode(PPUMode::PixelTransfer);
        assert_eq!(state.bits(), 0b00000011);
    }
}
