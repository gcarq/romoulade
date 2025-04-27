mod disasm;

mod registers;

use crate::gb::FrontendMessage;
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::{EmulatorDebugMessage, FrontendDebugMessage};
use crate::gui::debugger::disasm::Disassembler;
use crate::gui::debugger::registers::Registers;
use egui::{CentralPanel, SidePanel, TopBottomPanel, Ui};
use std::collections::BTreeMap;
use std::sync::mpsc::Sender;

/// Holds the current state of the emulator that is relevant for debugging.
#[derive(Default, Clone)]
struct EmulatorState {
    pub cpu: CPU,
    pub instructions: BTreeMap<u16, Option<Instruction>>,
}

pub struct DebuggerFrontend {
    sender: Sender<FrontendMessage>,
    state: EmulatorState,
    disassembler: Disassembler,
    registers: Registers,
}

impl DebuggerFrontend {
    pub fn new(sender: Sender<FrontendMessage>) -> Self {
        Self {
            sender,
            state: EmulatorState::default(),
            disassembler: Disassembler::default(),
            registers: Registers,
        }
    }

    /// Updates the UI of the debugger.
    pub fn update(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("actions")
                .resizable(false)
                .show_inside(ui, |ui| {
                    self.draw_top_panel(ui);
                });

            SidePanel::left("instructions")
                .resizable(false)
                .show_inside(ui, |ui| {
                    if self.disassembler.update(&self.state, ui) {
                        self.send_message(FrontendDebugMessage::Breakpoints(
                            self.disassembler.breakpoints.clone(),
                        ));
                    }
                });

            CentralPanel::default().show_inside(ui, |ui| {
                self.registers.update(&self.state, ui);
            });
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
        self.sender
            .send(FrontendMessage::Debug(message))
            .expect("Unable to send debug message");
    }

    /// Handles the given `EmulatorDebugMessage` message from the emulator.
    #[inline]
    pub fn handle_message(&mut self, msg: EmulatorDebugMessage) {
        match msg {
            EmulatorDebugMessage::Cpu(cpu) => {
                self.state.cpu = cpu;
                self.disassembler.scroll_to_address(self.state.cpu.pc);
            }
            EmulatorDebugMessage::Instructions(instructions) => {
                self.state.instructions = instructions;
            }
        }
    }
}
