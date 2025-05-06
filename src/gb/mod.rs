use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::{CPU, ImeState};
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
pub mod interrupt;
pub mod joypad;
mod oam;
pub mod ppu;
mod serial;
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
    Debug(DebugMessage),
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
    bus: Bus,
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
    ) -> GBResult<Self> {
        let display = Display::new(sender.clone(), config.upscale)?;
        let cpu = CPU::default();
        let mut bus = Bus::with_cartridge(cartridge, display);
        let debugger = match config.debug {
            true => Some(Debugger::new(&cpu, &mut bus, sender.clone())),
            false => None,
        };
        Ok(Self {
            is_running: true,
            fastboot: config.fastboot,
            cpu,
            bus,
            debugger,
            sender,
            receiver,
        })
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
                debugger
                    .maybe_step(&mut self.cpu, &mut self.bus)
                    .expect("Unable to step debugger");
            } else {
                self.step().expect("Unable to step emulator");
            }
        }
    }

    /// Attaches a `Debugger` to the emulator.
    #[inline]
    fn attach_debugger(&mut self) {
        self.debugger = Some(Debugger::new(&self.cpu, &mut self.bus, self.sender.clone()))
    }

    /// Steps the `CPU` once and handles interrupts if any.
    #[inline]
    fn step(&mut self) -> GBResult<()> {
        self.cpu.step(&mut self.bus)?;
        interrupt::handle(&mut self.cpu, &mut self.bus);
        self.sanity_check();
        Ok(())
    }

    /// Sanity check to verify boot ROM executed successfully
    #[inline]
    fn sanity_check(&self) {
        if self.bus.is_boot_rom_active && self.cpu.pc == BOOT_END - 1 {
            assert_eq!(self.cpu.r.get_af(), 0x01B0, "SANITY: AF is invalid!");
            assert_eq!(self.cpu.r.get_bc(), 0x0013, "SANITY: BC is invalid!");
            assert_eq!(self.cpu.r.get_de(), 0x00D8, "SANITY: DE is invalid!");
            assert_eq!(self.cpu.r.get_hl(), 0x014D, "SANITY: HL is invalid!");
            assert_eq!(self.cpu.sp, 0xFFFE, "SANITY: SP is invalid!");
            println!("Done with processing boot ROM. Switching to Cartridge ...");
        }
    }

    /// Fastboot the emulator by setting the CPU registers as if it had booted normally.
    fn fastboot(&mut self) {
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
                FrontendMessage::Input(input) => self.bus.handle_joypad_event(input),
                FrontendMessage::AttachDebugger => self.attach_debugger(),
                FrontendMessage::DetachDebugger => self.debugger = None,
                FrontendMessage::Debug(message) => {
                    if let Some(debugger) = &mut self.debugger {
                        debugger.handle_message(message)
                    }
                }
            }
        }
    }
}
