use sdl2::video::Window;

use crate::gb::ppu::Color;
use crate::gb::{DISPLAY_REFRESH_RATE, SCREEN_HEIGHT, SCREEN_WIDTH};
use sdl2::pixels;
use sdl2::rect::{Point, Rect};
use sdl2::render::Canvas;
use sdl2::EventPump;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use std::{process, thread};

const NAME: &str = "Romoulade";

pub struct Display {
    canvas: Canvas<Window>,
    event_pump: EventPump,
    upscale: u8,
    last_second_frames: VecDeque<Instant>,
    limiter: FrameLimiter,
}

impl Display {
    pub fn new(upscale: u8) -> Self {
        let sdl = sdl2::init().unwrap();
        let up = 1 << (upscale as usize);

        let xres = SCREEN_WIDTH as u32 * up;
        let yres = SCREEN_HEIGHT as u32 * up;

        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window(NAME, xres, yres)
            .position_centered()
            .build()
            .unwrap();
        let canvas = window.into_canvas().build().unwrap();

        Self {
            canvas,
            event_pump: sdl.event_pump().unwrap(),
            upscale,
            last_second_frames: VecDeque::with_capacity(60),
            limiter: FrameLimiter::new(DISPLAY_REFRESH_RATE),
        }
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
        let color = match value {
            Color::White => pixels::Color::RGB(0xff, 0xff, 0xff),
            Color::LightGrey => pixels::Color::RGB(0xab, 0xab, 0xab),
            Color::DarkGrey => pixels::Color::RGB(0x55, 0x55, 0x55),
            Color::Black => pixels::Color::RGB(0x00, 0x00, 0x00),
        };

        self.canvas.set_draw_color(color);

        if self.upscale == 0 {
            self.canvas
                .draw_point(Point::new(x as i32, y as i32))
                .unwrap();
        } else {
            let up = 1 << (self.upscale as usize);

            // Translate coordinates
            let x = x as i32 * up;
            let y = y as i32 * up;

            self.canvas
                .fill_rect(Rect::new(x, y, up as u32, up as u32))
                .unwrap();
        }
    }

    /// Returns the current frames per second
    fn calc_fps(&mut self) -> usize {
        let now = Instant::now();
        let a_second_ago = now - Duration::from_secs(1);

        while self
            .last_second_frames
            .front()
            .map_or(false, |t| *t < a_second_ago)
        {
            self.last_second_frames.pop_front();
        }

        self.last_second_frames.push_back(now);
        self.last_second_frames.len()
    }

    fn update(&mut self) {
        self.canvas.present();
        for event in self.event_pump.poll_iter() {
            match event {
                sdl2::event::Event::Quit { .. } => {
                    println!("Got Event::Quit");
                    process::exit(0)
                }
                _ => {}
            }
        }
    }
}

/// Limits FPS with thread::sleep().
pub struct FrameLimiter {
    frame_duration: Duration,
    last_call: Instant,
}

impl FrameLimiter {
    /// Creates a new frame limiter.
    pub fn new(fps: u32) -> Self {
        Self {
            frame_duration: Duration::from_secs(1) / fps,
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
