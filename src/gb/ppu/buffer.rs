use crate::gb::ppu::misc::Color;
use crate::gb::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::Color32;

/// Frame buffer to store the current screen state.
#[derive(Clone)]
pub struct FrameBuffer {
    upscale: usize,
    pub buffer: Vec<Color32>,
}

impl FrameBuffer {
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
    pub fn write_pixel(&mut self, x: u8, y: u8, color: Color) {
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
    fn translate_color(&self, color: Color) -> Color32 {
        match color {
            Color::White => Color32::from_rgb(0xff, 0xff, 0xff),
            Color::LightGrey => Color32::from_rgb(0xab, 0xab, 0xab),
            Color::DarkGrey => Color32::from_rgb(0x55, 0x55, 0x55),
            Color::Black => Color32::from_rgb(0x00, 0x00, 0x00),
        }
    }
}
