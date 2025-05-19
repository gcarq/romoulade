use crate::gb::ppu::misc::ColoredPixel;
use crate::gb::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::Color32;

/// Frame buffer to store the current screen state.
#[derive(Clone)]
pub struct FrameBuffer {
    upscale: usize,
    pub buffer: Vec<Color32>,
}

impl FrameBuffer {
    #[inline]
    pub fn new(upscale: usize) -> Self {
        Self {
            buffer: vec![
                Color32::DARK_BLUE;
                SCREEN_WIDTH as usize * upscale * SCREEN_HEIGHT as usize * upscale
            ],
            upscale,
        }
    }

    /// Writes a colored pixel to the buffer with the defined upscale in mind.
    pub fn write_pixel(&mut self, x: u8, y: u8, color: ColoredPixel) {
        let color = self.translate_color(color);
        let x = x as usize * self.upscale;
        let y = y as usize * self.upscale;
        let scaled_width = SCREEN_WIDTH as usize * self.upscale;
        for xx in x..x + self.upscale {
            for yy in y..y + self.upscale {
                self.buffer[(yy * scaled_width) + xx] = color;
            }
        }
    }

    #[inline]
    const fn translate_color(&self, color: ColoredPixel) -> Color32 {
        match color {
            ColoredPixel::White => Color32::from_rgb(0xff, 0xff, 0xff),
            ColoredPixel::LightGrey => Color32::from_rgb(0xab, 0xab, 0xab),
            ColoredPixel::DarkGrey => Color32::from_rgb(0x55, 0x55, 0x55),
            ColoredPixel::Black => Color32::from_rgb(0x00, 0x00, 0x00),
        }
    }
}
