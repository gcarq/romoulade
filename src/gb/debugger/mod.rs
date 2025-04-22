#[cfg(test)]
mod tests;

use crate::gb::bus::Bus;
use crate::gb::constants::{BOOT_END, ROM_BANK_0_BEGIN, ROM_BANK_0_END};
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::{EmulatorMessage, GBResult, interrupt};
use std::collections::{BTreeMap, HashSet};
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

/// This enum defines the possible debug messages that can be sent from
/// the emulator to the frontend.
pub enum EmulatorDebugMessage {
    Cpu(CPU),
    Instructions(BTreeMap<u16, Option<Instruction>>),
}

/// The debugger is responsible for handling the debug state of the emulator.
pub struct Debugger {
    message: Option<FrontendDebugMessage>,
    instructions: BTreeMap<u16, Option<Instruction>>,
    breakpoints: HashSet<u16>,
    sender: Sender<EmulatorMessage>,
}

impl Debugger {
    /// Creates a new debugger instance and sends the initial state to the frontend.
    pub fn new(cpu: &CPU, bus: &mut Bus, sender: Sender<EmulatorMessage>) -> Self {
        let instance = Self {
            message: None,
            instructions: populate_instructions(bus),
            breakpoints: HashSet::new(),
            sender,
        };
        // Send the initial state to the frontend
        // TODO: find a better solution
        instance.send_message(EmulatorDebugMessage::Cpu(cpu.clone()));
        instance.send_instructions();
        instance
    }

    /// Checks the current debugger state and steps the CPU if necessary.
    /// Only does a single step per call.
    pub fn maybe_step(&mut self, cpu: &mut CPU, bus: &mut Bus) -> GBResult<()> {
        match &self.message {
            Some(FrontendDebugMessage::Step) => {
                cpu.step(bus)?;
                interrupt::handle(cpu, bus);
                self.send_message(EmulatorDebugMessage::Cpu(cpu.clone()));
                self.message = None;
            }
            Some(FrontendDebugMessage::Continue) => {
                cpu.step(bus)?;
                interrupt::handle(cpu, bus);
                if self.breakpoints.contains(&cpu.pc) {
                    self.send_message(EmulatorDebugMessage::Cpu(cpu.clone()));
                    self.message = None;
                }
            }
            Some(FrontendDebugMessage::Pause) => {
                self.send_message(EmulatorDebugMessage::Cpu(cpu.clone()));
                self.message = None;
            }
            Some(FrontendDebugMessage::SkipBootRom) => {
                if cpu.pc == BOOT_END + 1 {
                    self.send_message(EmulatorDebugMessage::Cpu(cpu.clone()));
                    self.message = None;
                } else {
                    cpu.step(bus)?;
                    interrupt::handle(cpu, bus);
                }
            }
            Some(FrontendDebugMessage::Breakpoints(breakpoints)) => {
                self.breakpoints = breakpoints.clone();
            }
            None => {}
        }

        // After the boot ROM is finished we have to populate
        // the instructions from the cartridge ROM
        if cpu.pc == BOOT_END + 1 {
            self.instructions = populate_instructions(bus);
            self.send_instructions();
        }

        Ok(())
    }

    /// Handles a message from the frontend.
    #[inline]
    pub fn handle_message(&mut self, message: FrontendDebugMessage) {
        self.message = Some(message);
    }

    /// Sends a message to the frontend.
    #[inline]
    fn send_message(&self, message: EmulatorDebugMessage) {
        self.sender
            .send(EmulatorMessage::Debug(message))
            .expect("Unable to send debug message");
    }

    /// Sends the current instructions to the frontend.
    #[inline]
    fn send_instructions(&self) {
        self.send_message(EmulatorDebugMessage::Instructions(
            self.instructions.clone(),
        ));
    }
}

/// Prefetch all instructions from ROM bank 0.
/// TODO: adapt for banking
fn populate_instructions(bus: &mut Bus) -> BTreeMap<u16, Option<Instruction>> {
    let mut instructions = BTreeMap::new();
    let mut cur_pc = ROM_BANK_0_BEGIN;

    while cur_pc < ROM_BANK_0_END {
        cur_pc = match simulate_step(bus, cur_pc) {
            (Some(instruction), pc) => {
                instructions.insert(cur_pc, Some(instruction));
                pc
            }
            (None, pc) => {
                instructions.insert(cur_pc, None);
                pc
            }
        }
    }
    instructions
}

/// Simulates a single step of the CPU.
/// Returns the instruction and the next program counter.
fn simulate_step(bus: &mut Bus, pc: u16) -> (Option<Instruction>, u16) {
    // Read next opcode from memory
    let opcode = bus.read_raw(pc);
    let (opcode, prefixed) = match opcode == 0xCB {
        true => (bus.read_raw(pc + 1), true),
        false => (opcode, false),
    };

    // Parse instruction from opcode and return it together with the next program counter
    match Instruction::from_byte(opcode, prefixed) {
        Some(instruction) => (
            Some(instruction),
            pc + resolve_byte_length(opcode, prefixed) as u16,
        ),
        None => (None, pc + 1),
    }
}

/// Resolves the instruction byte length for the given opcode
fn resolve_byte_length(opcode: u8, prefixed: bool) -> u8 {
    // All prefixed opcodes have a length of 2 bytes
    if prefixed {
        return 2;
    }

    match opcode {
        0x01 => 3,
        0x06 => 2,
        0x08 => 3,
        0x0E => 2,
        0x10 => 2,
        0x11 => 3,
        0x16 => 2,
        0x18 => 2,
        0x1E => 2,
        0x20 => 2,
        0x21 => 3,
        0x26 => 2,
        0x28 => 2,
        0x2E => 2,
        0x30 => 2,
        0x31 => 3,
        0x36 => 2,
        0x38 => 2,
        0x3E => 2,
        0xC2 => 3,
        0xC3 => 3,
        0xC4 => 3,
        0xC6 => 2,
        0xCA => 3,
        0xCC => 3,
        0xCD => 3,
        0xCE => 2,
        0xD2 => 3,
        0xD4 => 3,
        0xD6 => 2,
        0xDA => 3,
        0xDC => 3,
        0xDE => 2,
        0xE0 => 2,
        0xE6 => 2,
        0xE8 => 2,
        0xEA => 3,
        0xEE => 2,
        0xF0 => 2,
        0xF6 => 2,
        0xF8 => 2,
        0xFA => 3,
        0xFE => 2,
        _ => 1,
    }
}
