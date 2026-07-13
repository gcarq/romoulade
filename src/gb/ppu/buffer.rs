use crate::gb::{SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui::Color32;

/// Frame buffer to store the current screen state.
#[derive(Clone)]
pub struct FrameBuffer {
    buffer: Vec<Color32>,
}

impl FrameBuffer {
    #[inline]
    pub fn new() -> Self {
        Self {
            buffer: vec![Color32::WHITE; SCREEN_WIDTH as usize * SCREEN_HEIGHT as usize],
        }
    }

    /// Returns the width of the frame buffer image in pixels.
    #[inline(always)]
    pub const fn width(&self) -> usize {
        SCREEN_WIDTH as usize
    }

    /// Returns the height of the frame buffer image in pixels.
    #[inline(always)]
    pub const fn height(&self) -> usize {
        SCREEN_HEIGHT as usize
    }

    /// Writes a colored pixel to the buffer.
    #[inline]
    pub fn write_pixel(&mut self, x: u8, y: u8, color: Color32) {
        let index = y as usize * SCREEN_WIDTH as usize + x as usize;
        self.buffer[index] = color;
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
    fn test_frame_buffer_handling() {
        let screen_width = 160;
        let screen_height = 144;

        let mut frame = FrameBuffer::new();
        assert_eq!(frame.width(), screen_width);
        assert_eq!(frame.height(), screen_height);
        assert_eq!(frame.buffer.len(), screen_width * screen_height);

        frame.write_pixel(0, 0, Color32::BLACK);
        frame.write_pixel(10, 0, Color32::LIGHT_GRAY);
        frame.write_pixel(0, 10, Color32::DARK_GRAY);

        assert_eq!(frame.buffer[0], Color32::BLACK);
        assert_eq!(frame.buffer[10], Color32::LIGHT_GRAY);
        assert_eq!(frame.buffer[screen_width * 10], Color32::DARK_GRAY);
        assert_eq!(frame.buffer[1], Color32::WHITE);
    }
}
