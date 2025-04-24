mod bus;

use crate::gb::bus::Bus;
use crate::gb::constants::{BOOT_END, ROM_BANK_0_BEGIN, ROM_BANK_0_END};
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::bus::DebugBus;
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
    let mut bus = DebugBus::from(bus.clone());
    let mut instructions = BTreeMap::new();
    let mut pc = ROM_BANK_0_BEGIN;

    while pc < ROM_BANK_0_END {
        let (instruction, next_pc) = Instruction::new(pc, &mut bus);
        instructions.insert(pc, instruction);
        pc = next_pc;
    }
    instructions
}
