use anyhow::{Context, Result, anyhow};
use eframe::egui;
use log::{error, info};

use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::debugger::{DebugMessage, Debugger, FrontendDebugMessage};
use crate::gb::joypad::JoypadInputState;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::ppu::display::Display;
use std::path::PathBuf;
use std::sync::mpsc::{Receiver, SyncSender};
use std::time::Instant;

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
mod test_support;
pub mod timer;
mod utils;

pub const DISPLAY_REFRESH_RATE: f64 = 59.727500569606;

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;

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

    #[cfg(test)]
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
    Input(JoypadInputState),
    UpdateConfig(EmulatorConfig),
    WriteSaveFile,
    AttachDebugger,
    DetachDebugger,
    Debug(FrontendDebugMessage),
}
/// Holds the configuration for the emulator.
#[derive(Clone)]
pub struct EmulatorConfig {
    pub rom: Option<PathBuf>,      // Path to the ROM file
    pub upscale: usize,            // Scale factor for the display
    pub fastboot: bool,            // Skip boot ROM
    pub print_serial: bool,        // Print serial data to stdout
    pub headless: bool,            // Run in headless mode (no display)
    pub savefile: Option<PathBuf>, // Path to the save file
    pub autosave: bool,            // Automatically loads and saves SRAM
}

impl Default for EmulatorConfig {
    fn default() -> Self {
        Self {
            rom: None,
            upscale: 3,
            fastboot: false,
            print_serial: false,
            headless: false,
            savefile: None,
            autosave: true,
        }
    }
}

/// Holds and manages the state of the whole emulator backend.
pub struct Emulator {
    cpu: CPU,
    bus: MainBus,
    sender: SyncSender<EmulatorMessage>,
    receiver: Receiver<FrontendMessage>,
    debugger: Option<Debugger>,
    is_running: bool,
    config: EmulatorConfig,
    last_autosave: Option<Instant>, // Last time the autosave was performed
}

impl Emulator {
    /// Creates a new `Emulator` instance with a display.
    pub fn with_display(
        sender: SyncSender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        cartridge: Cartridge,
        repaint_ctx: egui::Context,
        config: EmulatorConfig,
    ) -> Self {
        let display = Display::new(sender.clone(), repaint_ctx);
        let bus = MainBus::with_cartridge(cartridge, config.clone(), Some(display));
        Self::new(sender, receiver, config, bus)
    }

    /// Creates a new `Emulator` instance in headless mode (no display).
    pub fn headless(
        sender: SyncSender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        cartridge: Cartridge,
        config: EmulatorConfig,
    ) -> Self {
        let bus = MainBus::with_cartridge(cartridge, config.clone(), None);
        Self::new(sender, receiver, config, bus)
    }

    /// Runs the emulator loop.
    pub fn run(&mut self) -> Result<()> {
        info!("Starting emulator...");
        info!("Loaded ROM: {}", self.bus.cartridge);

        if let Some(ref savefile) = self.config.savefile {
            self.bus
                .cartridge
                .load_savefile(savefile)
                .with_context(|| anyhow!("Failed to load save file: {}", savefile.display()))?;
            info!("Loaded save file: {}", savefile.display());
        }
        if self.config.fastboot {
            info!("Fastboot enabled. Skipping boot ROM ...");
            self.fastboot();
        }

        while self.is_running {
            self.handle_message();
            self.maybe_autosave();
            if let Some(debugger) = &mut self.debugger {
                debugger.maybe_step(&mut self.cpu, &mut self.bus);
            } else {
                self.step();
            }
        }
        Ok(())
    }

    /// Creates a new `Emulator` instance with the given `MainBus`.
    fn new(
        sender: SyncSender<EmulatorMessage>,
        receiver: Receiver<FrontendMessage>,
        config: EmulatorConfig,
        bus: MainBus,
    ) -> Self {
        Self {
            cpu: CPU::default(),
            is_running: true,
            debugger: None,
            last_autosave: config.autosave.then(std::time::Instant::now),
            config,
            bus,
            sender,
            receiver,
        }
    }

    /// Steps the `CPU` once and handles interrupts if any.
    #[inline]
    fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    /// Updates the emulator configuration.
    #[inline]
    fn update_config(&mut self, config: EmulatorConfig) {
        info!("Updating emulator configuration...");
        self.config = config;
        self.bus.update_config(&self.config);
    }

    /// Triggers an autosave if the configuration allows it
    /// and the last autosave was more than 60 seconds ago.
    fn maybe_autosave(&mut self) {
        if !self.config.autosave || !self.bus.cartridge.header.config.is_savable() {
            return;
        }
        if let Some(last_autosave) = self.last_autosave
            && last_autosave.elapsed().as_secs() < 60
        {
            return;
        }
        let path = match self.config.savefile {
            Some(ref path) => path.as_path(),
            None => return,
        };
        match self.bus.cartridge.write_savefile(path) {
            Ok(()) => info!("Autosaved to: {}", path.display()),
            Err(msg) => error!("Failed to autosave: {msg}"),
        }
        self.last_autosave = Some(std::time::Instant::now());
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
                FrontendMessage::Stop => {
                    // This should trigger an autosave if enabled
                    self.last_autosave = None;
                    self.is_running = false;
                }
                FrontendMessage::Input(input) => self.bus.handle_joypad_input(input),
                FrontendMessage::UpdateConfig(config) => self.update_config(config),
                FrontendMessage::WriteSaveFile => {
                    if let Some(ref path) = self.config.savefile {
                        match self.bus.cartridge.write_savefile(path) {
                            Ok(()) => info!("Save file written to: {}", path.display()),
                            Err(msg) => error!("Failed to write save file: {msg}"),
                        }
                    }
                }
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

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::gb::cpu::ImeState;

    use super::*;

    #[test]
    fn test_boot_rom() {
        let header = vec![
            0x00, 0xc3, 0x13, 0x02, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73,
            0x00, 0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e,
            0xdc, 0xcc, 0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e,
            0xec, 0xcc, 0xdd, 0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00,
            0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x66, 0x4d, 0xeb,
        ];
        let mut rom = vec![0x00; 0x100];
        rom.extend(header);

        let cartridge = Cartridge::try_from(Arc::from(rom.into_boxed_slice())).unwrap();

        let mut cpu = CPU::default();
        let mut bus = MainBus::with_cartridge(cartridge, EmulatorConfig::default(), None);

        while cpu.r.pc < BOOT_END + 1 {
            cpu.step(&mut bus);
        }

        assert!(!bus.is_boot_rom_active, "Boot ROM is still active");
        assert_eq!(cpu.r.get_af(), 0x01B0, "AF is invalid");
        assert_eq!(cpu.r.get_bc(), 0x0013, "BC is invalid");
        assert_eq!(cpu.r.get_de(), 0x00D8, "DE is invalid");
        assert_eq!(cpu.r.get_hl(), 0x014D, "HL is invalid");
        assert_eq!(cpu.r.sp, 0xFFFE, "SP is invalid");
        assert_eq!(cpu.r.pc, 0x0100, "PC is invalid");
        assert_eq!(cpu.ime, ImeState::Disabled, "IME should be disabled");
    }
}
