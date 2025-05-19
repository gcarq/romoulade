use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::debugger::{DebugMessage, Debugger, FrontendDebugMessage};
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
pub mod joypad;
mod oam;
pub mod ppu;
mod serial;
#[cfg(test)]
pub mod tests;
pub mod timer;
mod utils;

pub const DISPLAY_REFRESH_RATE: u32 = 60; // TODO: exact refresh rate is 59.7

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;

pub type GBResult<T> = Result<T, GBError>;
pub type GBError = Box<dyn error::Error>;

/// This trait defines a common interface for all subsystems of the emulator.
pub trait SubSystem {
    /// Writes a byte to the given address.
    fn write(&mut self, address: u16, value: u8);

    /// Reads a byte from the given address.
    fn read(&mut self, address: u16) -> u8;
}

/// This trait defines a common interface to interact with the Hardware Bus.
pub trait Bus: SubSystem {
    /// Advance the bus for 1 machine cycle and write a byte to the given address.
    fn cycle_write(&mut self, address: u16, value: u8) {
        self.cycle();
        self.write(address, value);
    }

    /// Advance the bus for 1 machine cycle and read a byte from the given address.
    fn cycle_read(&mut self, address: u16) -> u8 {
        self.cycle();
        self.read(address)
    }

    /// Advance the bus for 1 machine cycle.
    fn cycle(&mut self);

    /// Indicates whether an interrupt is pending.
    fn has_irq(&self) -> bool;

    fn set_ie(&mut self, r: InterruptRegister);
    fn get_ie(&self) -> InterruptRegister;
    fn set_if(&mut self, r: InterruptRegister);
    fn get_if(&self) -> InterruptRegister;
}

/// This enum defines the possible messages that can be sent from the emulator to the frontend.
pub enum EmulatorMessage {
    Frame(FrameBuffer),
    Debug(Box<DebugMessage>),
}

/// This enum defines the possible messages that can be sent from the frontend to the emulator.
pub enum FrontendMessage {
    Stop,
    Input(JoypadInput),
    AttachDebugger,
    DetachDebugger,
    Debug(FrontendDebugMessage),
}
/// Holds the configuration for the emulator.
#[derive(Clone, Copy)]
pub struct EmulatorConfig {
    pub upscale: usize, // Scale factor for the display
    pub debug: bool,    // Enable debugger
    pub fastboot: bool, // Skip boot ROM
}

/// Holds and manages the state of the whole emulator backend.
pub struct Emulator {
    cpu: CPU,
    bus: MainBus,
    sender: Sender<EmulatorMessage>,
    receiver: Receiver<FrontendMessage>,
    debugger: Option<Debugger>,
    is_running: bool,
    fastboot: bool,
}

impl Emulator {
    /// Creates a new `Emulator` instance.
    pub fn new(
        sender: Sender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        cartridge: Cartridge,
        config: EmulatorConfig,
    ) -> Self {
        let display = Display::new(sender.clone(), config.upscale);
        let cpu = CPU::default();
        let mut bus = MainBus::with_cartridge(cartridge, Some(display));
        let debugger = match config.debug {
            true => Some(Debugger::new(&cpu, &mut bus, sender.clone())),
            false => None,
        };
        Self {
            is_running: true,
            fastboot: config.fastboot,
            cpu,
            bus,
            debugger,
            sender,
            receiver,
        }
    }

    /// Runs the emulator loop.
    pub fn run(&mut self) {
        println!("Starting emulator...");

        println!("Loaded ROM: {}", self.bus.cartridge);
        println!("DEBUG: {:?}", self.bus.cartridge.header.config);

        if self.fastboot {
            println!("Fastboot enabled. Skipping boot ROM ...");
            self.fastboot();
        }

        // TODO: implement proper error handling
        while self.is_running {
            self.handle_message();
            if let Some(debugger) = &mut self.debugger {
                debugger.maybe_step(&mut self.cpu, &mut self.bus);
            } else {
                self.step();
            }
        }
    }

    /// Attaches a `Debugger` to the emulator.
    #[inline]
    fn attach_debugger(&mut self) {
        self.debugger = Some(Debugger::new(&self.cpu, &mut self.bus, self.sender.clone()));
    }

    /// Steps the `CPU` once and handles interrupts if any.
    #[inline]
    fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    /// Fastboot the emulator by setting the CPU registers as if it had booted normally.
    const fn fastboot(&mut self) {
        self.cpu.r.set_af(0x01B0);
        self.cpu.r.set_bc(0x0013);
        self.cpu.r.set_de(0x00D8);
        self.cpu.r.set_hl(0x014D);
        self.cpu.sp = 0xFFFE;
        self.cpu.pc = BOOT_END + 1;
        self.bus.is_boot_rom_active = false;
    }

    /// Checks for a new `FrontendMessage` and handles it.
    fn handle_message(&mut self) {
        if let Ok(message) = self.receiver.try_recv() {
            match message {
                FrontendMessage::Stop => self.is_running = false,
                FrontendMessage::Input(input) => self.bus.joypad.handle_input(input),
                FrontendMessage::AttachDebugger => self.attach_debugger(),
                FrontendMessage::DetachDebugger => self.debugger = None,
                FrontendMessage::Debug(message) => {
                    if let Some(debugger) = &mut self.debugger {
                        debugger.handle_message(message);
                    }
                }
            }
        }
    }
}
