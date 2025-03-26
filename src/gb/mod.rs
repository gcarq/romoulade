use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::CPU;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::display::Display;
use crate::gb::ppu::PPU;
use crate::gui::FrontendMessage;
use std::error;
use std::sync::mpsc::{Receiver, Sender};

pub mod bus;
pub mod cartridge;
pub mod cpu;
pub mod interrupt;
pub mod joypad;
pub mod ppu;
#[cfg(test)]
mod tests;
pub mod timer;
mod utils;

pub const DISPLAY_REFRESH_RATE: u32 = 60; // TODO: exact refresh rate is 59.7

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const VERTICAL_BLANK_SCAN_LINE_MAX: u8 = 153;

pub type GBResult<T> = Result<T, Box<dyn error::Error>>;

/// This trait defines a common interface to interact with the memory bus.
pub trait AddressSpace {
    fn write(&mut self, address: u16, value: u8);
    fn read(&self, address: u16) -> u8;
}

pub enum EmulatorMessage {
    Frame(FrameBuffer),
}

pub struct Emulator {
    bus: Bus,
    cpu: CPU,
    ppu: PPU,
    receiver: Receiver<FrontendMessage>,
    is_running: bool,
}

impl Emulator {
    pub fn new(
        sender: Sender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        cartridge: Cartridge,
        upscale: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let display = Display::new(sender, upscale)?;
        Ok(Self {
            bus: Bus::new(cartridge),
            cpu: CPU::default(),
            ppu: PPU::new(display),
            receiver,
            is_running: true,
        })
    }

    fn step(&mut self) -> GBResult<()> {
        let cycles = self.cpu.step(&mut self.bus);
        self.bus.step(cycles);
        self.ppu.step(&mut self.bus, cycles)?;
        interrupt::handle(&mut self.cpu, &mut self.bus);
        Ok(())
    }

    /// Handles messages from the frontend.
    fn handle_messages(&mut self) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                FrontendMessage::Stop => self.is_running = false,
                FrontendMessage::Input(input) => self.bus.handle_joypad_event(input),
            }
        }
    }

    pub fn run(&mut self) {
        while self.is_running {
            if let Err(err) = self.step() {
                eprintln!("Error: {}", err);
                break;
            }
            self.handle_messages();
        }
    }
}
