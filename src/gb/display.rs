use sdl2::video::Window;

use crate::gb::ppu::misc::Color;
use crate::gb::{SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::pixels;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::EventPump;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::{error, process, thread};

const NAME: &str = "Romoulade";

/// Display with sdl2 backend to emulate the LCD.
pub struct Display {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    upscale: u8,
    last_second_frames: VecDeque<Instant>,
    limiter: FrameLimiter,
}

impl Display {
    /// Creates a new display with the given int upscale.
    pub fn new(upscale: u8, fps_limit: u32) -> Result<Self, Box<dyn error::Error>> {
        let sdl = sdl2::init()?;
        let up = 1 << (upscale as usize);

        let x_res = SCREEN_WIDTH as u32 * up;
        let y_res = SCREEN_HEIGHT as u32 * up;

        let video_subsystem = sdl.video()?;
        let window = video_subsystem
            .window(NAME, x_res, y_res)
            .position_centered()
            .build()?;

        let canvas = window.into_canvas().build()?;
        let limiter = match fps_limit {
            0 => FrameLimiter::new(LimitStrategy::Disabled),
            _ => FrameLimiter::new(LimitStrategy::Sleep(Duration::from_secs(1) / fps_limit)),
        };
        Ok(Self {
            canvas,
            event_pump: sdl.event_pump()?,
            upscale,
            last_second_frames: VecDeque::with_capacity(60),
            limiter,
        })
    }

    /// Renders the current canvas to screen
    pub fn render_screen(&mut self) {
        self.update();
        self.limiter.wait();

        let fps = self.calc_fps();
        self.canvas
            .window_mut()
            .set_title(&format!("{} - FPS: {}", NAME, fps))
            .expect("Unable to update title");
    }

    /// Writes a pixel to the given coordinates
    pub fn write_pixel(&mut self, x: u8, y: u8, value: Color) {
        let color = self.translate_color(value);
        self.canvas.set_draw_color(color);
        if self.upscale == 0 {
            self.canvas
                .draw_point(Point::new(x as i32, y as i32))
                .unwrap();
            return;
        }

        // Translate coordinates
        let up = 1 << (self.upscale as usize);
        let x = x as i32 * up;
        let y = y as i32 * up;

        self.canvas
            .fill_rect(Rect::new(x, y, up as u32, up as u32))
            .unwrap();
    }

    /// Translates given color to sdl2::pixels::Color
    fn translate_color(&self, color: Color) -> pixels::Color {
        match color {
            Color::White => pixels::Color::RGB(0xff, 0xff, 0xff),
            Color::LightGrey => pixels::Color::RGB(0xab, 0xab, 0xab),
            Color::DarkGrey => pixels::Color::RGB(0x55, 0x55, 0x55),
            Color::Black => pixels::Color::RGB(0x00, 0x00, 0x00),
        }
    }

    /// Returns the current frames per second
    fn calc_fps(&mut self) -> usize {
        let now = Instant::now();
        let a_second_ago = now - Duration::from_secs(1);

        while self
            .last_second_frames
            .front().is_some_and(|t| *t < a_second_ago)
        {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push_back(now);
        self.last_second_frames.len()
    }

    /// Renders the canvas to screen and checks
    /// if there are any events that need to be handled.
    fn update(&mut self) {
        self.canvas.present();
        for event in self.event_pump.poll_iter() {
            if let sdl2::event::Event::Quit { .. } = event { process::exit(0) }
        }
    }
}

/// Defines FrameLimit strategies
enum LimitStrategy {
    Disabled,
    Sleep(Duration),
}

/// Limits FPS with the configured strategy.
struct FrameLimiter {
    strategy: LimitStrategy,
    last_call: Instant,
}

impl FrameLimiter {
    /// Creates a new frame limiter.
    pub fn new(strategy: LimitStrategy) -> Self {
        Self {
            strategy,
            last_call: Instant::now(),
        }
    }

    /// Blocks the current thread until the allotted frame time has passed.
    pub fn wait(&mut self) {
        match self.strategy {
            LimitStrategy::Disabled => return,
            LimitStrategy::Sleep(frame_duration) => {
                let elapsed = self.last_call.elapsed();
                if elapsed < frame_duration {
                    thread::sleep(frame_duration - elapsed);
                }
            }
        }
        self.last_call = Instant::now();
    }
}
