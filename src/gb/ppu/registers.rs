use crate::gb::ppu::misc::Palette;

/// Holds all PPU Registers
#[derive(Copy, Clone)]
pub struct Registers {
    pub lcd_control: LCDControl, // PPU_LCDC
    pub lcd_stat: LCDState,      // PPU_STAT
    pub ly: u8,                  // PPU_LY
    pub lyc: u8,                 // PPU_LYC
    pub scy: u8,                 // PPU_SCY
    pub scx: u8,                 // PPU_SCX
    pub bg_palette: Palette,     // PPU_BGP
    pub obj_palette0: Palette,   // PPU_OBP0
    pub obj_palette1: Palette,   // PPU_OBP1
    pub wy: u8,                  // PPU_WY
    pub wx: u8,                  // PPU_WX
}

impl Default for Registers {
    #[inline]
    fn default() -> Self {
        Self {
            lcd_control: LCDControl::empty(),
            lcd_stat: LCDState::empty(),
            ly: 0,
            lyc: 0,
            scy: 0,
            scx: 0,
            bg_palette: Palette::default(),
            obj_palette0: Palette::default(),
            obj_palette1: Palette::default(),
            wy: 0,
            wx: 0,
        }
    }
}

bitflags! {
    /// Represents PPU_LCDC at 0xFF40
    #[derive(Copy, Clone)]
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
    #[derive(Copy, Clone)]
    pub struct LCDState: u8 {
        const LCD_MODE1   = 0b00000001; // LCD Mode
        const LCD_MODE2   = 0b00000010; // LCD Mode
        const LYC_STAT    = 0b00000100; // LY Flag
        const H_BLANK_INT = 0b00001000; // Mode 0 H-Blank Interrupt
        const V_BLANK_INT = 0b00010000; // Mode 1 V-Blank Interrupt
        const OAM_INT     = 0b00100000; // Mode 2 OAM Interrupt
        const LY_INT      = 0b01000000; // LY Interrupt
    }
}

impl LCDState {
    /// Returns the `LCDMode` based on the first two bits of PPU_STAT.
    #[inline]
    pub fn get_lcd_mode(&self) -> LCDMode {
        LCDMode::from(self.bits() & 0b11)
    }

    /// Sets the first two bits of PPU_STAT to the given `LCDMode`.
    #[inline]
    pub fn set_lcd_mode(&mut self, mode: LCDMode) {
        *self = LCDState::from_bits_truncate((self.bits() & 0b11111100) | u8::from(mode));
    }
}

/// Represents the first two bits in LCDState for convenience.
#[derive(Copy, Clone, PartialEq)]
pub enum LCDMode {
    HBlank,        // 0b00
    VBlank,        // 0b01
    OAMSearch,     // 0b10
    PixelTransfer, // 0b11
}

impl From<LCDMode> for u8 {
    #[inline]
    fn from(value: LCDMode) -> u8 {
        match value {
            LCDMode::HBlank => 0b00,
            LCDMode::VBlank => 0b01,
            LCDMode::OAMSearch => 0b10,
            LCDMode::PixelTransfer => 0b11,
        }
    }
}

impl From<u8> for LCDMode {
    #[inline]
    fn from(value: u8) -> Self {
        match value {
            0b00 => LCDMode::HBlank,
            0b01 => LCDMode::VBlank,
            0b10 => LCDMode::OAMSearch,
            0b11 => LCDMode::PixelTransfer,
            _ => unimplemented!(),
        }
    }
}
