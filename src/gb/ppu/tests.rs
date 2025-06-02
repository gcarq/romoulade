use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::misc::{ColoredPixel, Palette, Pixel};
use crate::gb::ppu::pixel_from_line;
use crate::gb::ppu::registers::{LCDState, PPUMode};
use eframe::egui::Color32;

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

#[test]
fn test_frame_buffer_no_upscale() {
    let screen_width = 160;
    let screen_height = 144;

    let mut frame = FrameBuffer::new(1);
    assert_eq!(frame.buffer.len(), screen_width * screen_height);
    frame.write_pixel(0, 0, ColoredPixel::Black);
    frame.write_pixel(10, 0, ColoredPixel::White);
    frame.write_pixel(0, 10, ColoredPixel::LightGrey);
    frame.write_pixel(10, 10, ColoredPixel::DarkGrey);

    // Check that the pixels are written correctly
    assert_eq!(frame.buffer[0], Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(frame.buffer[10], Color32::from_rgb(0xff, 0xff, 0xff));
    assert_eq!(
        frame.buffer[screen_width * 10],
        Color32::from_rgb(0xab, 0xab, 0xab)
    );
    assert_eq!(
        frame.buffer[screen_width * 10 + 10],
        Color32::from_rgb(0x55, 0x55, 0x55)
    );
}

#[test]
fn test_frame_buffer_with_upscale() {
    let screen_width = 160;
    let screen_height = 144;

    let mut frame = FrameBuffer::new(2);
    assert_eq!(frame.buffer.len(), screen_width * 2 * screen_height * 2);

    frame.write_pixel(0, 0, ColoredPixel::Black);
    frame.write_pixel(10, 0, ColoredPixel::White);
    frame.write_pixel(0, 10, ColoredPixel::LightGrey);
    frame.write_pixel(10, 10, ColoredPixel::DarkGrey);

    // Check that the pixels are written correctly
    assert_eq!(frame.buffer[0], Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(frame.buffer[1], Color32::from_rgb(0x00, 0x00, 0x00));
    assert_eq!(frame.buffer[2], Color32::WHITE);

    assert_eq!(
        frame.buffer[screen_width * 2],
        Color32::from_rgb(0x00, 0x00, 0x00)
    );
    assert_eq!(
        frame.buffer[screen_width * 2 + 1],
        Color32::from_rgb(0x00, 0x00, 0x00)
    );
    assert_eq!(frame.buffer[screen_width * 2 + 2], Color32::WHITE);

    assert_eq!(frame.buffer[screen_width * 3], Color32::WHITE);
    assert_eq!(frame.buffer[screen_width * 3 + 1], Color32::WHITE);
}

#[test]
fn test_palette() {
    let palette = Palette::from(0b11_10_01_00);
    assert_eq!(palette.colorize(Pixel::Zero), ColoredPixel::White);
    assert_eq!(palette.colorize(Pixel::One), ColoredPixel::LightGrey);
    assert_eq!(palette.colorize(Pixel::Two), ColoredPixel::DarkGrey);
    assert_eq!(palette.colorize(Pixel::Three), ColoredPixel::Black);
    assert_eq!(u8::from(palette), 0b11_10_01_00);
}

#[test]
fn test_pixel() {
    let data = vec![
        (0b00, Pixel::Zero),
        (0b01, Pixel::One),
        (0b10, Pixel::Two),
        (0b11, Pixel::Three),
    ];
    for (value, pixel) in data {
        assert_eq!(u8::from(pixel), value);
        assert_eq!(Pixel::from(value), pixel);
    }

    assert_eq!(Pixel::from(0b1111_1111), Pixel::Three);
    assert_eq!(Pixel::from(0b0101_1100), Pixel::Zero);
}

#[test]
fn test_colored_pixel() {
    let data = vec![
        (0b00, ColoredPixel::White),
        (0b01, ColoredPixel::LightGrey),
        (0b10, ColoredPixel::DarkGrey),
        (0b11, ColoredPixel::Black),
    ];
    for (value, pixel) in data {
        assert_eq!(u8::from(pixel), value);
        assert_eq!(ColoredPixel::from(value), pixel);
    }

    assert_eq!(ColoredPixel::from(0b1111_1111), ColoredPixel::Black);
    assert_eq!(ColoredPixel::from(0b0101_1100), ColoredPixel::White);
}

#[test]
fn test_pixel_from_line() {
    let data = vec![
        (0b0000_0000, 0b0000_0000, 0, Pixel::Zero),
        (0b1111_1111, 0b1111_1111, 1, Pixel::Three),
        (0b1010_1010, 0b0101_0101, 2, Pixel::Two),
        (0b1100_1100, 0b0011_0011, 3, Pixel::One),
        (0b1111_0000, 0b0000_1111, 4, Pixel::One),
        (0b0000_1111, 0b1111_0000, 5, Pixel::Two),
        (0b1100_0011, 0b0011_1100, 6, Pixel::One),
        (0b0011_1100, 0b0100_0011, 7, Pixel::Zero),
    ];
    for (byte1, byte2, index, expected) in data {
        let pixel = pixel_from_line(byte1, byte2, index);
        assert_eq!(pixel, expected);
    }
}
