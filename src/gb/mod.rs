use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::{CPU, ImeState};
use crate::gb::debugger::{Debugger, EmulatorDebugMessage, FrontendDebugMessage};
use crate::gb::joypad::JoypadInput;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::display::Display;
use std::error;
use std::sync::mpsc::{Receiver, Sender};

mod audio;
pub mod bus;
pub mod cartridge;
pub mod constants;
pub mod cpu;
pub mod debugger;
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

pub type GBResult<T> = Result<T, GBError>;
pub type GBError = Box<dyn error::Error>;

/// This trait defines a common interface to interact with the hardware context.
pub trait HardwareContext {
    fn set_ime(&mut self, ime: ImeState);
    fn ime(&self) -> ImeState;
    fn tick(&mut self);
}

/// This trait defines a common interface to interact with the memory bus.
pub trait AddressSpace {
    fn write(&mut self, address: u16, value: u8);
    fn read(&mut self, address: u16) -> u8;
}

/// This enum defines the possible messages that can be sent from the emulator to the frontend.
pub enum EmulatorMessage {
    Frame(FrameBuffer),
    Debug(EmulatorDebugMessage),
}

/// This enum defines the possible messages that can be sent from the frontend to the emulator.
pub enum FrontendMessage {
    Stop,
    Input(JoypadInput),
    AttachDebugger,
    Debug(FrontendDebugMessage),
}

pub struct Emulator {
    cpu: CPU,
    bus: Bus,
    sender: Sender<EmulatorMessage>,
    receiver: Receiver<FrontendMessage>,
    is_running: bool,
    debugger: Option<Debugger>,
}

impl Emulator {
    pub fn new(
        sender: Sender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        cartridge: Cartridge,
        upscale: usize,
        debug: bool,
    ) -> GBResult<Self> {
        let display = Display::new(sender.clone(), upscale)?;
        let cpu = CPU::default();
        let mut bus = Bus::new(cartridge, display);
        let debugger = match debug {
            true => Some(Debugger::new(&cpu, &mut bus, sender.clone())),
            false => None,
        };
        Ok(Self {
            is_running: true,
            cpu,
            bus,
            debugger,
            sender,
            receiver,
        })
    }

    /// Runs the emulator loop.
    /// TODO: implement proper error handling
    pub fn run(&mut self) {
        println!("Starting emulator...");
        println!("Loaded ROM: {}", self.bus.cartridge);

        while self.is_running {
            self.handle_message();
            if let Some(debugger) = &mut self.debugger {
                debugger
                    .maybe_step(&mut self.cpu, &mut self.bus)
                    .expect("Unable to step debugger");
            } else {
                self.step().expect("Unable to step emulator");
            }
        }
    }

    /// Attaches a debugger to the emulator.
    #[inline]
    fn attach_debugger(&mut self) {
        self.debugger = Some(Debugger::new(&self.cpu, &mut self.bus, self.sender.clone()))
    }

    #[inline]
    fn step(&mut self) -> GBResult<()> {
        self.cpu.step(&mut self.bus)?;
        interrupt::handle(&mut self.cpu, &mut self.bus);
        Ok(())
    }

    /// Handles messages from the frontend.
    fn handle_message(&mut self) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                FrontendMessage::Stop => self.is_running = false,
                FrontendMessage::Input(input) => self.bus.handle_joypad_event(input),
                FrontendMessage::AttachDebugger => self.attach_debugger(),
                FrontendMessage::Debug(message) => {
                    if let Some(debugger) = &mut self.debugger {
                        debugger.handle_message(message)
                    }
                }
            }
        }
    }
}
