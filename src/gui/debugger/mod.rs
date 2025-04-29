mod disasm;

mod registers;

use crate::gb::FrontendMessage;
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::bus::DebugBus;
use crate::gb::debugger::{DebugMessage, FrontendDebugMessage};
use crate::gui::debugger::disasm::Disassembler;
use crate::gui::debugger::registers::Registers;
use egui::{CentralPanel, SidePanel, TopBottomPanel, Ui};
use egui_extras::{Size, StripBuilder};
use std::sync::mpsc::Sender;

/// Holds the current state of the emulator that is relevant for debugging.
#[derive(Clone)]
struct EmulatorState {
    pub cpu: CPU,
    pub bus: DebugBus,
    pub instructions: Vec<(u16, Option<Instruction>)>,
}

impl EmulatorState {
    pub fn new(cpu: CPU, bus: DebugBus) -> Self {
        let mut bus = bus;
        let instructions = bus.fetch_instructions();

        Self {
            cpu,
            bus,
            instructions,
        }
    }
}

pub struct DebuggerFrontend {
    sender: Sender<FrontendMessage>,
    state: Option<EmulatorState>,
    disassembler: Disassembler,
    registers: Registers,
}

impl DebuggerFrontend {
    #[inline]
    pub fn new(sender: Sender<FrontendMessage>) -> Self {
        Self {
            sender,
            state: None,
            disassembler: Disassembler::default(),
            registers: Registers,
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
        CentralPanel::default().show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::exact(145.0))
                .size(Size::remainder())
                .horizontal(|mut strip| {
                    strip.strip(|builder| {
                        builder.sizes(Size::remainder(), 1).horizontal(|mut strip| {
                            strip.cell(|ui| {
                                if let Some(state) = &self.state {
                                    self.registers.update(state, ui);
                                }
                            });
                        });
                    });
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
        // TODO: Handle errors properly, currently panics if the channel is closed.
        self.sender
            .send(FrontendMessage::Debug(message))
            .expect("Unable to send debug message");
    }

    /// Handles the given `EmulatorDebugMessage` message from the emulator.
    #[inline]
    pub fn handle_message(&mut self, msg: DebugMessage) {
        let pc = msg.cpu.pc;
        self.disassembler.scroll_to_address(pc);
        self.state = Some(EmulatorState::new(msg.cpu, msg.bus));
    }
}
