use crate::gb::cpu::ImeState;
use crate::gb::cpu::registers::FlagsRegister;
use crate::gui::debugger::EmulatorState;
use bitvec::order::Lsb0;
use bitvec::view::BitView;
use eframe::epaint::Color32;
use egui::text::LayoutJob;
use egui::{FontId, TextFormat, Ui};

/// Represents the registers view in the debugger UI.
pub struct Registers;

impl Registers {
    pub fn update(&mut self, state: &EmulatorState, ui: &mut Ui) {
        self.update_flags(state, ui);
        ui.separator();
        self.update_cpu_registers(state, ui);
        ui.separator();
        self.update_sp_pc(state, ui);
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_flag(ui, "IME", state.cpu.ime == ImeState::Enabled);
            ui.separator();
            self.draw_flag(ui, "HALT", state.cpu.is_halted);
        });
        ui.separator();
    }

    /// Updates the CPUS flags in the UI (lower 8 bits of AF register).
    fn update_flags(&self, state: &EmulatorState, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.draw_flag(ui, "Z", state.cpu.r.f.contains(FlagsRegister::ZERO));
            ui.separator();
            self.draw_flag(ui, "N", state.cpu.r.f.contains(FlagsRegister::SUBTRACTION));
        });
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_flag(ui, "H", state.cpu.r.f.contains(FlagsRegister::HALF_CARRY));
            ui.separator();
            self.draw_flag(ui, "C", state.cpu.r.f.contains(FlagsRegister::CARRY));
        });
    }

    /// Updates the CPU registers in the UI.
    fn update_cpu_registers(&self, state: &EmulatorState, ui: &mut Ui) {
        ui.horizontal(|ui| {
            self.draw_cpu_register(ui, "A", state.cpu.r.a);
            ui.separator();
            self.draw_cpu_register(ui, "F", state.cpu.r.f.bits());
        });
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_cpu_register(ui, "B", state.cpu.r.b);
            ui.separator();
            self.draw_cpu_register(ui, "C", state.cpu.r.c);
        });
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_cpu_register(ui, "D", state.cpu.r.d);
            ui.separator();
            self.draw_cpu_register(ui, "E", state.cpu.r.e);
        });
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_cpu_register(ui, "H", state.cpu.r.h);
            ui.separator();
            self.draw_cpu_register(ui, "L", state.cpu.r.l);
        });
    }

    /// Updates the Stack Pointer (SP) and Program Counter (PC) in the UI.
    fn update_sp_pc(&self, state: &EmulatorState, ui: &mut Ui) {
        self.draw_u16_register(ui, "SP", state.cpu.sp);
        ui.separator();
        self.draw_u16_register(ui, "PC", state.cpu.pc);
    }

    /// Draw a boolean flag with its name and value.
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
                &format!(" = {value:#04X}"),
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
    }

    /// Draws an integer value in binary format with colored bits.
    fn draw_bits<T: BitView>(&self, ui: &mut Ui, value: T) {
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
        ui.vertical_centered(|ui| {
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
                &format!(" = {value:#06X}"),
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
    }
}
