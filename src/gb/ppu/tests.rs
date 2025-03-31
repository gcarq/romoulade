use crate::gb::ppu::registers::{LCDMode, LCDState};

#[test]
fn test_get_lcd_mode() {
    let mut state = LCDState::empty();
    assert_eq!(state.get_lcd_mode(), LCDMode::HBlank);

    state = LCDState::LCD_MODE1;
    assert_eq!(state.get_lcd_mode(), LCDMode::VBlank);

    state = LCDState::LCD_MODE2;
    assert_eq!(state.get_lcd_mode(), LCDMode::OAMSearch);

    state = LCDState::LCD_MODE1 | LCDState::LCD_MODE2;
    assert_eq!(state.get_lcd_mode(), LCDMode::PixelTransfer);
}

#[test]
fn test_set_lcd_mode() {
    let mut state = LCDState::empty();
    state.set_lcd_mode(LCDMode::HBlank);
    assert_eq!(state.bits(), 0b00000000);

    state.set_lcd_mode(LCDMode::VBlank);
    assert_eq!(state.bits(), 0b00000001);

    state.set_lcd_mode(LCDMode::OAMSearch);
    assert_eq!(state.bits(), 0b00000010);

    state.set_lcd_mode(LCDMode::PixelTransfer);
    assert_eq!(state.bits(), 0b00000011);
}
