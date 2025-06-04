use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::misc::ColoredPixel;
use crate::gb::{DISPLAY_REFRESH_RATE, EmulatorConfig, EmulatorMessage};
use eframe::egui::Color32;
use std::sync::mpsc::SyncSender;
use std::time::{Duration, Instant};

/// The display holds the current frame in the `FrameBuffer` and sends it to the frontend.
/// It also takes care of syncing the frame rate.
pub struct Display {
    sender: SyncSender<EmulatorMessage>,
    buffer: FrameBuffer,
    frame_limiter: FrameLimiter,
}

impl Display {
    /// Creates a new display with the given int upscale.
    pub fn new(sender: SyncSender<EmulatorMessage>, upscale: usize) -> Self {
        Self {
            sender,
            buffer: FrameBuffer::new(upscale),
            frame_limiter: FrameLimiter::new(DISPLAY_REFRESH_RATE),
        }
    }

    /// Sends the current frame to the frontend and syncs the frame rate.
    pub fn send_frame(&mut self) {
        let buffer = self.buffer.clone();
        self.sender.send(EmulatorMessage::Frame(buffer)).ok();
        self.frame_limiter.wait();
    }

    /// Writes a pixel to the given coordinates
    #[inline]
    pub fn write_pixel(&mut self, x: u8, y: u8, color: ColoredPixel) {
        self.buffer.write_pixel(x, y, translate_color(color));
    }

    pub fn update_config(&mut self, config: &EmulatorConfig) {
        self.buffer = FrameBuffer::new(config.upscale);
    }
}

/// Limits FPS with the configured refresh rate.
#[derive(Clone)]
struct FrameLimiter {
    frame_duration: Duration,
    last_call: Instant,
}

impl FrameLimiter {
    /// Creates a new frame limiter with the given refresh rate.
    #[inline]
    pub fn new(refresh_rate: f64) -> Self {
        Self {
            frame_duration: Duration::from_secs_f64(1.0 / refresh_rate),
            last_call: Instant::now(),
        }
    }

    /// Blocks the current thread until the allotted frame time has passed.
    #[inline]
    pub fn wait(&mut self) {
        let elapsed = self.last_call.elapsed();
        if elapsed < self.frame_duration {
            spin_sleep::sleep(self.frame_duration - elapsed);
        }
        self.last_call = Instant::now();
    }
}

/// Translates a `ColoredPixel` to `egui::Color32`.
#[inline]
const fn translate_color(color: ColoredPixel) -> Color32 {
    match color {
        ColoredPixel::White => Color32::WHITE,
        ColoredPixel::LightGrey => Color32::from_rgb(0xab, 0xab, 0xab),
        ColoredPixel::DarkGrey => Color32::from_rgb(0x55, 0x55, 0x55),
        ColoredPixel::Black => Color32::BLACK,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_translate_color() {
        assert_eq!(translate_color(ColoredPixel::White), Color32::WHITE);
        assert_eq!(
            translate_color(ColoredPixel::LightGrey),
            Color32::from_rgb(0xab, 0xab, 0xab)
        );
        assert_eq!(
            translate_color(ColoredPixel::DarkGrey),
            Color32::from_rgb(0x55, 0x55, 0x55)
        );
        assert_eq!(translate_color(ColoredPixel::Black), Color32::BLACK);
    }
}
