use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::debugger::{DebugMessage, Debugger, FrontendDebugMessage};
use crate::gb::joypad::JoypadInput;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::display::Display;
use std::error;
use std::path::PathBuf;
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
#[derive(Clone)]
pub struct EmulatorConfig {
    pub rom: Option<PathBuf>, // Path to the ROM file
    pub upscale: usize,       // Scale factor for the display
    pub fastboot: bool,       // Skip boot ROM
    pub print_serial: bool,   // Print serial data to stdout
    pub headless: bool,       // Run in headless mode (no display)
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            rom: None,
            upscale: 3,
            fastboot: false,
            print_serial: false,
            headless: false,
        }
    }
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
        let cpu = CPU::default();
        let display = (!config.headless).then_some(Display::new(sender.clone(), config.upscale));
        let bus = MainBus::with_cartridge(cartridge, config.clone(), display);

        Self {
            is_running: true,
            fastboot: config.fastboot,
            debugger: None,
            cpu,
            bus,
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
        self.cpu.r.sp = 0xFFFE;
        self.cpu.r.pc = BOOT_END + 1;
        self.bus.is_boot_rom_active = false;
    }

    /// Initializes the debugger and sends the initial state to the frontend.
    fn attach_debugger(&mut self) {
        let debugger = Debugger::new(self.sender.clone());
        debugger.send_message(DebugMessage::new(&self.cpu, &self.bus));
        self.debugger = Some(debugger);
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
