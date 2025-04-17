use crate::gb::ppu::registers::{LCDState, PPUMode};

#[test]
fn test_get_lcd_mode() {
    let mut state = LCDState::empty();
    assert_eq!(state.ppu_mode(), PPUMode::HBlank);

    state = LCDState::PPU_MODE1;
    assert_eq!(state.ppu_mode(), PPUMode::VBlank);

    state = LCDState::PPU_MODE2;
    assert_eq!(state.ppu_mode(), PPUMode::AccessOAM);

    state = LCDState::PPU_MODE1 | LCDState::PPU_MODE2;
    assert_eq!(state.ppu_mode(), PPUMode::AccessVRAM);
}

#[test]
fn test_set_lcd_mode() {
    let mut state = LCDState::empty();
    state.set_ppu_mode(PPUMode::HBlank);
    assert_eq!(state.bits(), 0b00000000);

    state.set_ppu_mode(PPUMode::VBlank);
    assert_eq!(state.bits(), 0b00000001);

    state.set_ppu_mode(PPUMode::AccessOAM);
    assert_eq!(state.bits(), 0b00000010);

    state.set_ppu_mode(PPUMode::AccessVRAM);
    assert_eq!(state.bits(), 0b00000011);
}
