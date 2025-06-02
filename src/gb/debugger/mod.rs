pub mod bus;

use crate::gb::EmulatorMessage;
use crate::gb::bus::MainBus;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::bus::DebugBus;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::mpsc::SyncSender;

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
    pub fn new(cpu: &CPU, bus: &MainBus) -> Self {
        Self {
            cpu: cpu.clone(),
            bus: DebugBus::from(bus.clone()),
        }
    }
}

/// Represents a single instruction with context information for the disassembler.
pub struct AnnotatedInstr {
    pub address: u16,     // address of the opcode
    pub bytes: Arc<[u8]>, // all raw bytes of the instruction
    pub instruction: Instruction,
}

impl AnnotatedInstr {
    #[inline]
    pub const fn new(address: u16, bytes: Arc<[u8]>, instruction: Instruction) -> Self {
        Self {
            address,
            bytes,
            instruction,
        }
    }
}

/// The debugger is responsible for handling the debug state of the emulator.
pub struct Debugger {
    message: Option<FrontendDebugMessage>,
    breakpoints: HashSet<u16>,
    sender: SyncSender<EmulatorMessage>,
}

impl Debugger {
    /// Creates a new debugger instance and sends the initial state to the frontend.
    #[inline]
    pub fn new(sender: SyncSender<EmulatorMessage>) -> Self {
        Self {
            message: None,
            breakpoints: HashSet::new(),
            sender,
        }
    }

    /// Checks the current debugger state and steps the CPU if necessary.
    /// Only does a single step per call.
    pub fn maybe_step(&mut self, cpu: &mut CPU, bus: &mut MainBus) {
        match &self.message {
            // The frontend requested a single step
            Some(FrontendDebugMessage::Step) => {
                self.step(cpu, bus);
                self.send_message(DebugMessage::new(cpu, bus));
                self.message = None;
            }
            // The frontend requested normal execution, the cpu should step until breakpoint is hit
            Some(FrontendDebugMessage::Continue) => {
                self.step(cpu, bus);
                if self.breakpoints.contains(&cpu.r.pc) {
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
                if bus.is_boot_rom_active && cpu.r.pc == BOOT_END - 1 {
                    self.step(cpu, bus);
                    self.send_message(DebugMessage::new(cpu, bus));
                    self.message = None;
                } else {
                    self.step(cpu, bus);
                }
            }
            // The frontend sent a new set of breakpoints
            Some(FrontendDebugMessage::Breakpoints(breakpoints)) => {
                self.breakpoints = breakpoints.clone();
            }
            None => {}
        }
    }

    /// Sends a message to the frontend.
    #[inline]
    pub fn send_message(&self, message: DebugMessage) {
        self.sender
            .send(EmulatorMessage::Debug(Box::new(message)))
            .expect("Unable to send debug message");
    }

    /// Handles a message from the frontend.
    #[inline(always)]
    pub fn handle_message(&mut self, message: FrontendDebugMessage) {
        self.message = Some(message);
    }

    /// Steps the CPU and handles interrupts.
    #[inline]
    fn step(&mut self, cpu: &mut CPU, bus: &mut MainBus) {
        cpu.step(bus);
    }
}
