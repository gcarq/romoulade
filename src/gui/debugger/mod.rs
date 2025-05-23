mod disasm;
mod memory;
mod registers;
#[macro_use]
mod macros;

use crate::gb::FrontendMessage;
use crate::gb::cpu::CPU;
use crate::gb::debugger::bus::DebugBus;
use crate::gb::debugger::{AnnotatedInstr, DebugMessage, FrontendDebugMessage};
use crate::gui::debugger::disasm::Disassembler;
use crate::gui::debugger::memory::MemoryMap;
use crate::gui::debugger::registers::Registers;
use eframe::egui;
use eframe::egui::{CentralPanel, SidePanel, TopBottomPanel, Ui};
use std::sync::mpsc::Sender;

/// Holds the current state of the emulator that is relevant for debugging.
struct EmulatorState {
    pub cpu: CPU,
    pub bus: DebugBus,
    pub instructions: Vec<AnnotatedInstr>,
}

impl EmulatorState {
    pub fn new(cpu: CPU, bus: DebugBus) -> Self {
        let mut bus = bus;
        Self {
            instructions: bus.fetch_instructions(),
            cpu,
            bus,
        }
    }
}

pub struct DebuggerFrontend {
    sender: Sender<FrontendMessage>,
    state: Option<EmulatorState>,
    disassembler: Disassembler,
    registers: Registers,
    memory: MemoryMap,
}

impl DebuggerFrontend {
    #[inline]
    pub fn new(sender: Sender<FrontendMessage>) -> Self {
        Self {
            state: None,
            disassembler: Disassembler::default(),
            memory: MemoryMap::default(),
            registers: Registers,
            sender,
        }
    }

    /// Updates the UI of the debugger.
    pub fn update(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("actions")
            .resizable(false)
            .show(ctx, |ui| {
                self.draw_top_panel(ui);
            });

        SidePanel::left("instructions")
            .resizable(false)
            .show(ctx, |ui| {
                if let Some(state) = &self.state {
                    if self.disassembler.update(state, ui) {
                        self.send_message(FrontendDebugMessage::Breakpoints(
                            self.disassembler.breakpoints.clone(),
                        ));
                    }
                }
            });
        SidePanel::right("memory").resizable(false).show(ctx, |ui| {
            if let Some(state) = &mut self.state {
                self.memory.update(state, ui);
            }
        });
        CentralPanel::default().show(ctx, |ui| {
            if let Some(state) = &mut self.state {
                self.registers.update(state, ui);
            }
        });
    }

    /// Draws the top panel for the debugger.
    fn draw_top_panel(&self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Step").clicked() {
                self.send_message(FrontendDebugMessage::Step);
            }
            if ui.button("Continue").clicked() {
                self.send_message(FrontendDebugMessage::Continue);
            }
            ui.separator();
            if ui.button("Pause").clicked() {
                self.send_message(FrontendDebugMessage::Pause);
            }
            if ui.button("Skip Boot ROM").clicked() {
                self.send_message(FrontendDebugMessage::SkipBootRom);
            }
        });
    }

    /// Sends a debug message to the emulator.
    #[inline]
    fn send_message(&self, message: FrontendDebugMessage) {
        self.sender.send(FrontendMessage::Debug(message)).ok();
    }

    /// Handles the given `EmulatorDebugMessage` message from the emulator.
    #[inline]
    pub fn handle_message(&mut self, msg: DebugMessage) {
        self.disassembler.scroll_to_address(msg.cpu.r.pc);
        self.state = Some(EmulatorState::new(msg.cpu, msg.bus));
    }
}
