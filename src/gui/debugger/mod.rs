mod disasm;

use crate::gb::FrontendMessage;
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::{EmulatorDebugMessage, FrontendDebugMessage};
use crate::gui::debugger::disasm::Disassembler;
use eframe::emath::Align;
use egui::{CentralPanel, Frame, Layout, SidePanel, TopBottomPanel, Ui, Vec2};
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
}

impl DebuggerFrontend {
    pub fn new(sender: Sender<FrontendMessage>) -> Self {
        Self {
            sender,
            state: EmulatorState::default(),
            disassembler: Disassembler::default(),
        }
    }

    /// Updates the UI of the debugger.
    pub fn update(&mut self, ctx: &egui::Context) {
        CentralPanel::default().show(ctx, |ui| {
            TopBottomPanel::top("actions")
                .resizable(false)
                .show_inside(ui, |ui| {
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
                });

            SidePanel::left("instructions")
                .resizable(false)
                .min_width(250.0)
                .show_inside(ui, |ui| {
                    if self.disassembler.update_assembly(&self.state, ui) {
                        self.send_message(FrontendDebugMessage::Breakpoints(
                            self.disassembler.breakpoints.clone(),
                        ));
                    }
                });

            CentralPanel::default().show_inside(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Central Panel");
                });
                self.update_cpu_registers(ui);
            });
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

    /// Draws a CPU register with its name and value.
    fn draw_cpu_register(&self, ui: &mut Ui, name: &str, value: u8) {
        Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.monospace(format!("{} = {:#04X}", name, value));
                    ui.monospace(format!("{:08b}", value));
                });
            });
    }

    /// Updates the CPU registers in the UI.
    fn update_cpu_registers(&self, ui: &mut Ui) {
        ui.allocate_ui_with_layout(
            Vec2::new(400.0, 400.0),
            Layout::top_down(Align::Center),
            |ui| {
                CentralPanel::default()
                    .frame(Frame::default())
                    .show_inside(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label("CPU Registers");
                            ui.horizontal(|ui| {
                                self.draw_cpu_register(ui, "A", self.state.cpu.r.a);
                                self.draw_cpu_register(ui, "F", self.state.cpu.r.f.bits());
                            });
                            ui.horizontal(|ui| {
                                self.draw_cpu_register(ui, "B", self.state.cpu.r.b);
                                self.draw_cpu_register(ui, "C", self.state.cpu.r.c);
                            });
                            ui.horizontal(|ui| {
                                self.draw_cpu_register(ui, "D", self.state.cpu.r.d);
                                self.draw_cpu_register(ui, "E", self.state.cpu.r.e);
                            });
                            ui.horizontal(|ui| {
                                self.draw_cpu_register(ui, "H", self.state.cpu.r.h);
                                self.draw_cpu_register(ui, "L", self.state.cpu.r.l);
                            });
                        });
                    });
            },
        );
    }
}
