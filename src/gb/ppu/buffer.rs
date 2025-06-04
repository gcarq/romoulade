use crate::gb::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::Color32;

/// Frame buffer to store the current screen state.
#[derive(Clone)]
pub struct FrameBuffer {
    upscale: usize,
    buffer: Vec<Color32>,
}

impl FrameBuffer {
    #[inline]
    pub fn new(upscale: usize) -> Self {
        Self {
            buffer: vec![
                Color32::WHITE;
                SCREEN_WIDTH as usize * upscale * SCREEN_HEIGHT as usize * upscale
            ],
            upscale,
        }
    }

    /// Returns the width of the frame buffer image in pixels.
    #[inline(always)]
    pub const fn width(&self) -> usize {
        SCREEN_WIDTH as usize * self.upscale
    }

    /// Returns the height of the frame buffer image in pixels.
    #[inline(always)]
    pub const fn height(&self) -> usize {
        SCREEN_HEIGHT as usize * self.upscale
    }

    /// Writes a colored pixel to the buffer with the configured upscale in mind.
    pub fn write_pixel(&mut self, x: u8, y: u8, color: Color32) {
        let scaled_x = x as usize * self.upscale;
        let scaled_y = y as usize * self.upscale;
        for y in scaled_y..scaled_y + self.upscale {
            let offset = y * self.width() + scaled_x;
            self.buffer[offset..offset + self.upscale].fill(color);
        }
    }

    /// Consumes the frame buffer and returns the underlying vector of colors.
    #[inline]
    pub fn into_vec(self) -> Vec<Color32> {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_no_upscale() {
        let screen_width = 160;
        let screen_height = 144;

        let mut frame = FrameBuffer::new(1);
        assert_eq!(frame.buffer.len(), screen_width * screen_height);
        frame.write_pixel(0, 0, Color32::BLACK);
        frame.write_pixel(10, 0, Color32::WHITE);
        frame.write_pixel(0, 10, Color32::LIGHT_GRAY);
        frame.write_pixel(10, 10, Color32::DARK_GRAY);

        // Check that the pixels are written correctly
        assert_eq!(frame.buffer[0], Color32::BLACK);
        assert_eq!(frame.buffer[10], Color32::WHITE);
        assert_eq!(frame.buffer[screen_width * 10], Color32::LIGHT_GRAY);
        assert_eq!(frame.buffer[screen_width * 10 + 10], Color32::DARK_GRAY);
    }

    #[test]
    fn test_buffer_with_upscale() {
        let screen_width = 160;
        let screen_height = 144;

        let mut frame = FrameBuffer::new(2);
        assert_eq!(frame.buffer.len(), screen_width * 2 * screen_height * 2);

        frame.write_pixel(0, 0, Color32::BLACK);
        frame.write_pixel(10, 0, Color32::WHITE);
        frame.write_pixel(0, 10, Color32::LIGHT_GRAY);
        frame.write_pixel(10, 10, Color32::DARK_GRAY);

        // Check that the pixels are written correctly
        assert_eq!(frame.buffer[0], Color32::BLACK);
        assert_eq!(frame.buffer[1], Color32::BLACK);
        assert_eq!(frame.buffer[2], Color32::WHITE);

        assert_eq!(frame.buffer[screen_width * 2], Color32::BLACK);
        assert_eq!(frame.buffer[screen_width * 2 + 1], Color32::BLACK);
        assert_eq!(frame.buffer[screen_width * 2 + 2], Color32::WHITE);

        assert_eq!(frame.buffer[screen_width * 3], Color32::WHITE);
        assert_eq!(frame.buffer[screen_width * 3 + 1], Color32::WHITE);
    }
}
