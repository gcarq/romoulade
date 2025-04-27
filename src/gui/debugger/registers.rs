use crate::gb::cpu::registers::FlagsRegister;
use crate::gui::debugger::EmulatorState;
use bitvec::order::Lsb0;
use bitvec::view::BitView;
use eframe::epaint::Color32;
use egui::text::LayoutJob;
use egui::{FontId, Frame, TextFormat, Ui};

/// Represents the registers view in the debugger UI.
pub struct Registers;

impl Registers {
    pub fn update(&mut self, state: &EmulatorState, ui: &mut Ui) {
        self.update_flags(state, ui);
        self.update_cpu_registers(state, ui);
        self.update_sp_pc(state, ui);
        ui.horizontal(|ui| {
            // TODO: Add IME state
            //ui.label(format!("IME = {}", state.cpu.ime));
            ui.label(format!("HALT = {}", state.cpu.is_halted));
        });
    }

    /// Updates the CPUS flags in the UI (lower 8 bits of AF register).
    fn update_flags(&self, state: &EmulatorState, ui: &mut Ui) {
        Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        self.draw_flag(ui, "Z", state.cpu.r.f.contains(FlagsRegister::ZERO));
                        self.draw_flag(ui, "N", state.cpu.r.f.contains(FlagsRegister::SUBTRACTION));
                    });

                    ui.horizontal(|ui| {
                        self.draw_flag(ui, "H", state.cpu.r.f.contains(FlagsRegister::HALF_CARRY));
                        self.draw_flag(ui, "C", state.cpu.r.f.contains(FlagsRegister::CARRY));
                    });
                });
            });
    }

    /// Updates the CPU registers in the UI.
    fn update_cpu_registers(&self, state: &EmulatorState, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.draw_cpu_register(ui, "A", state.cpu.r.a);
                self.draw_cpu_register(ui, "F", state.cpu.r.f.bits());
            });
            ui.horizontal(|ui| {
                self.draw_cpu_register(ui, "B", state.cpu.r.b);
                self.draw_cpu_register(ui, "C", state.cpu.r.c);
            });
            ui.horizontal(|ui| {
                self.draw_cpu_register(ui, "D", state.cpu.r.d);
                self.draw_cpu_register(ui, "E", state.cpu.r.e);
            });
            ui.horizontal(|ui| {
                self.draw_cpu_register(ui, "H", state.cpu.r.h);
                self.draw_cpu_register(ui, "L", state.cpu.r.l);
            });
        });
    }

    /// Updates the Stack Pointer (SP) and Program Counter (PC) in the UI.
    #[inline]
    fn update_sp_pc(&self, state: &EmulatorState, ui: &mut Ui) {
        self.draw_u16_register(ui, "SP", state.cpu.sp);
        self.draw_u16_register(ui, "PC", state.cpu.pc);
    }

    /// Draws a CPU register flag with its name and value.
    fn draw_flag(&self, ui: &mut Ui, name: &str, value: bool) {
        let mut job = LayoutJob::default();
        job.append(
            name,
            0.0,
            TextFormat {
                font_id: FontId::monospace(11.0),
                color: Color32::CYAN,
                ..Default::default()
            },
        );
        job.append(
            &format!(" = {}", if value { "1" } else { "0" }),
            0.0,
            TextFormat {
                font_id: FontId::monospace(11.0),
                color: Color32::WHITE,
                ..Default::default()
            },
        );
        ui.label(job);
    }

    /// Draws a CPU register with its name and value.
    fn draw_cpu_register(&self, ui: &mut Ui, name: &str, value: u8) {
        Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let mut job = LayoutJob::default();
                    job.append(
                        name,
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(11.0),
                            color: Color32::ORANGE,
                            ..Default::default()
                        },
                    );
                    job.append(
                        &format!(" = {:#04X}", value),
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(11.0),
                            color: Color32::WHITE,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                    self.draw_bits(ui, value);
                });
            });
    }

    /// Draws an integer value in binary format with colored bits.
    fn draw_bits<T>(&self, ui: &mut Ui, value: T)
    where
        T: BitView,
    {
        let bits = value.view_bits::<Lsb0>();
        let mut job = LayoutJob::default();
        for (i, chunk) in bits.chunks(4).enumerate() {
            if i > 0 {
                job.append(" ", 0.0, TextFormat::default());
            }
            for bit in chunk {
                job.append(
                    if *bit { "1" } else { "0" },
                    0.0,
                    TextFormat {
                        color: match *bit {
                            true => Color32::WHITE,
                            false => Color32::GRAY,
                        },
                        font_id: FontId::monospace(11.0),
                        ..Default::default()
                    },
                );
            }
        }
        ui.label(job);
    }

    /// Draws a 16-bit register with its name and value.
    fn draw_u16_register(&self, ui: &mut Ui, name: &str, value: u16) {
        Frame::default()
            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
            .corner_radius(ui.visuals().widgets.noninteractive.corner_radius)
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let mut job = LayoutJob::default();
                    job.append(
                        name,
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(11.0),
                            color: Color32::ORANGE,
                            ..Default::default()
                        },
                    );
                    job.append(
                        &format!(" = {:#06X}", value),
                        0.0,
                        TextFormat {
                            font_id: FontId::monospace(11.0),
                            color: Color32::WHITE,
                            ..Default::default()
                        },
                    );
                    ui.label(job);
                    self.draw_bits(ui, value);
                });
            });
    }
}
