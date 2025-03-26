use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::misc::Color;
use crate::gb::{DISPLAY_REFRESH_RATE, EmulatorMessage, GBResult};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use std::{error, thread};

/// The display holds the current screen in the framebuffer and sends it to the frontend,
/// once requested. It also takes care of syncing the frame rate.
pub struct Display {
    sender: Sender<EmulatorMessage>,
    buffer: FrameBuffer,
    frame_limiter: FrameLimiter,
}

impl Display {
    /// Creates a new display with the given int upscale.
    pub fn new(
        sender: Sender<EmulatorMessage>,
        upscale: usize,
    ) -> Result<Self, Box<dyn error::Error>> {
        Ok(Self {
            sender,
            buffer: FrameBuffer::new(upscale),
            frame_limiter: FrameLimiter::new(DISPLAY_REFRESH_RATE),
        })
    }

    /// Sends the current frame to the frontend and syncs the frame rate.
    pub fn send_frame(&mut self) -> GBResult<()> {
        let buffer = self.buffer.clone();
        self.sender.send(EmulatorMessage::Frame(buffer))?;
        self.frame_limiter.wait();
        Ok(())
    }

    /// Writes a pixel to the given coordinates
    #[inline]
    pub fn write_pixel(&mut self, x: u8, y: u8, color: Color) {
        self.buffer.write_pixel(x, y, color);
    }
}

/// Limits FPS with the configured refresh rate.
struct FrameLimiter {
    frame_duration: Duration,
    last_call: Instant,
}

impl FrameLimiter {
    /// Creates a new frame limiter with the given refresh rate.
    pub fn new(refresh_rate: u32) -> Self {
        Self {
            frame_duration: Duration::from_secs(1) / refresh_rate,
            last_call: Instant::now(),
        }
    }

    /// Blocks the current thread until the allotted frame time has passed.
    pub fn wait(&mut self) {
        let elapsed = self.last_call.elapsed();
        if elapsed < self.frame_duration {
            thread::sleep(self.frame_duration - elapsed);
        }
        self.last_call = Instant::now();
    }
}
