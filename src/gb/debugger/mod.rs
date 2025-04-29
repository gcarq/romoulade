pub mod bus;

use crate::gb::bus::Bus;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::debugger::bus::DebugBus;
use crate::gb::{EmulatorMessage, GBResult, interrupt};
use std::collections::HashSet;
use std::sync::mpsc::Sender;

/// This enum defines the possible debug messages that can be sent from
/// the frontend to the emulator.
pub enum FrontendDebugMessage {
    Step,
    Continue,
    Pause,
    SkipBootRom,
    Breakpoints(HashSet<u16>),
}

/// This struct holds the current state of the emulator that is relevant for debugging.
pub struct DebugMessage {
    pub cpu: CPU,
    pub bus: DebugBus,
}

impl DebugMessage {
    #[inline]
    pub fn new(cpu: &CPU, bus: &Bus) -> Self {
        Self {
            cpu: cpu.clone(),
            bus: DebugBus::from(bus.clone()),
        }
    }
}

/// The debugger is responsible for handling the debug state of the emulator.
pub struct Debugger {
    message: Option<FrontendDebugMessage>,
    breakpoints: HashSet<u16>,
    sender: Sender<EmulatorMessage>,
}

impl Debugger {
    /// Creates a new debugger instance and sends the initial state to the frontend.
    pub fn new(cpu: &CPU, bus: &mut Bus, sender: Sender<EmulatorMessage>) -> Self {
        let instance = Self {
            message: None,
            breakpoints: HashSet::new(),
            sender,
        };
        // Send the initial state to the frontend
        // TODO: find a better solution
        instance.send_message(DebugMessage::new(cpu, bus));
        instance
    }

    /// Checks the current debugger state and steps the CPU if necessary.
    /// Only does a single step per call.
    pub fn maybe_step(&mut self, cpu: &mut CPU, bus: &mut Bus) -> GBResult<()> {
        match &self.message {
            // The frontend requested a single step
            Some(FrontendDebugMessage::Step) => {
                self.step(cpu, bus)?;
                self.send_message(DebugMessage::new(cpu, bus));
                self.message = None;
            }
            // The frontend requested normal execution, the cpu should step until breakpoint is hit
            Some(FrontendDebugMessage::Continue) => {
                self.step(cpu, bus)?;
                if self.breakpoints.contains(&cpu.pc) {
                    self.send_message(DebugMessage::new(cpu, bus));
                    self.message = None;
                }
            }
            // The frontend requested a pause, the cpu should stop executing
            Some(FrontendDebugMessage::Pause) => {
                self.send_message(DebugMessage::new(cpu, bus));
                self.message = None;
            }
            // The frontend requested to skip the boot ROM,
            // the cpu should step until the end of the boot ROM
            Some(FrontendDebugMessage::SkipBootRom) => {
                if bus.is_boot_rom_active && cpu.pc == BOOT_END - 1 {
                    self.step(cpu, bus)?;
                    self.send_message(DebugMessage::new(cpu, bus));
                    self.message = None;
                } else {
                    self.step(cpu, bus)?;
                }
            }
            // The frontend sent a new set of breakpoints
            Some(FrontendDebugMessage::Breakpoints(breakpoints)) => {
                self.breakpoints = breakpoints.clone();
            }
            None => {}
        }
        Ok(())
    }

    /// Handles a message from the frontend.
    #[inline(always)]
    pub fn handle_message(&mut self, message: FrontendDebugMessage) {
        self.message = Some(message);
    }

    /// Steps the CPU and handles interrupts.
    #[inline]
    fn step(&mut self, cpu: &mut CPU, bus: &mut Bus) -> GBResult<()> {
        cpu.step(bus)?;
        interrupt::handle(cpu, bus);
        Ok(())
    }

    /// Sends a message to the frontend.
    #[inline]
    fn send_message(&self, message: DebugMessage) {
        self.sender
            .send(EmulatorMessage::Debug(message))
            .expect("Unable to send debug message");
    }
}
