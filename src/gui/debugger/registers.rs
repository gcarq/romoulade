use crate::gb::SubSystem;
use crate::gb::constants::*;
use crate::gb::cpu::ImeState;
use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::ppu::*;
use crate::gui::debugger::EmulatorState;
use crate::monospace_append;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use eframe::egui::text::LayoutJob;
use eframe::egui::{FontId, RichText, TextFormat, Ui};
use eframe::epaint::Color32;
use egui_extras::{Column, TableBuilder};

/// Represents the registers view in the debugger UI.
pub struct Registers;

impl Registers {
    pub fn update(&mut self, state: &mut EmulatorState, ui: &mut Ui) {
        self.update_cpu(state, ui);
        ui.separator();
        self.update_interrupts(state, ui);
        ui.separator();
        self.update_ppu(state, ui);
        ui.separator();
        self.update_timer(state, ui);
    }

    /// Updates all relevant parts of the CPU in the UI.
    fn update_cpu(&self, state: &mut EmulatorState, ui: &mut Ui) {
        ui.label(RichText::new("CPU").monospace().color(Color32::CYAN));
        self.update_cpu_registers(state, ui);
        ui.separator();
        ui.horizontal(|ui| {
            self.draw_flag(ui, "IME", state.cpu.ime == ImeState::Enabled);
            ui.separator();
            self.draw_flag(ui, "HALT", state.cpu.is_halted);
        });
    }

    /// Updates the interrupt registers in the UI.
    fn update_interrupts(&self, state: &mut EmulatorState, ui: &mut Ui) {
        self.draw_io_registers(
            "INTERRUPTS",
            ui,
            &[
                (INTERRUPT_FLAG, "IF", state.bus.read(INTERRUPT_FLAG)),
                (INTERRUPT_ENABLE, "IE", state.bus.read(INTERRUPT_ENABLE)),
            ],
        );
    }

    /// Updates the PPU registers in the UI.
    fn update_ppu(&self, state: &mut EmulatorState, ui: &mut Ui) {
        self.draw_io_registers(
            "PPU",
            ui,
            &[
                (PPU_LCDC, "LCDC", state.bus.read(PPU_LCDC)),
                (PPU_STAT, "STAT", state.bus.read(PPU_STAT)),
                (PPU_SCY, "SCY", state.bus.read(PPU_SCY)),
                (PPU_SCX, "SCX", state.bus.read(PPU_SCX)),
                (PPU_LY, "LY", state.bus.read(PPU_LY)),
                (PPU_LYC, "LYC", state.bus.read(PPU_LYC)),
                (PPU_BGP, "BGP", state.bus.read(PPU_BGP)),
                (PPU_OBP0, "OBP0", state.bus.read(PPU_OBP0)),
                (PPU_OBP1, "OBP1", state.bus.read(PPU_OBP1)),
                (PPU_WY, "WY", state.bus.read(PPU_WY)),
                (PPU_WX, "WX", state.bus.read(PPU_WX)),
            ],
        );
    }

    /// Updates the timer registers in the UI.
    fn update_timer(&self, state: &mut EmulatorState, ui: &mut Ui) {
        self.draw_io_registers(
            "TIMER",
            ui,
            &[
                (TIMER_DIVIDER, "DIV", state.bus.read(TIMER_DIVIDER)),
                (TIMER_COUNTER, "TIMA", state.bus.read(TIMER_COUNTER)),
                (TIMER_MODULO, "TMA", state.bus.read(TIMER_MODULO)),
                (TIMER_CTRL, "TAC", state.bus.read(TIMER_CTRL)),
            ],
        );
    }

    /// Updates the CPUS flags in the UI (lower 8 bits of AF register).
    fn update_cpu_flags(&self, state: &EmulatorState, ui: &mut Ui) {
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
        self.update_cpu_flags(state, ui);
        ui.separator();
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
        ui.separator();
        self.draw_u16_register(ui, "SP", state.cpu.r.sp);
        ui.separator();
        self.draw_u16_register(ui, "PC", state.cpu.r.pc);
    }

    /// Draws the given IO registers in a table format.
    /// The data is expected to be a tuple of (address, register name, value).
    fn draw_io_registers(&self, name: &str, ui: &mut Ui, data: &[(u16, &str, u8)]) {
        ui.label(RichText::new(name).monospace().color(Color32::CYAN));
        let table = TableBuilder::new(ui)
            .id_salt(name)
            .resizable(false)
            .column(Column::exact(50.0))
            .column(Column::exact(30.0))
            .column(Column::exact(30.0))
            .column(Column::exact(70.0));

        table.body(|body| {
            body.rows(16.0, data.len(), |mut row| {
                let (address, name, value) = data[row.index()];
                // Draw register address
                row.col(|ui| {
                    let text = RichText::new(format!("{address:#06X}"));
                    ui.label(text.monospace().color(Color32::LIGHT_GREEN));
                });
                // Draw register name
                row.col(|ui| {
                    let text = RichText::new(name);
                    ui.label(text.monospace().color(Color32::ORANGE));
                });
                //Draw register hex value
                row.col(|ui| {
                    let text = RichText::new(format!("{value:#04X}"));
                    ui.label(text.monospace().color(Color32::WHITE));
                });
                //Draw register binary value
                row.col(|ui| {
                    let mut job = LayoutJob::default();
                    self.draw_bits(&mut job, value);
                    ui.label(job);
                });
            });
        });
    }

    /// Draw a boolean flag with its name and value.
    fn draw_flag(&self, ui: &mut Ui, name: &str, value: bool) {
        let mut job = LayoutJob::default();
        monospace_append!(job, name, Color32::CYAN);
        monospace_append!(
            job,
            &format!(" = {}", if value { "1" } else { "0" }),
            Color32::WHITE
        );
        ui.label(job);
    }

    /// Draws a `u8` register with its name and value.
    fn draw_cpu_register(&self, ui: &mut Ui, name: &str, value: u8) {
        ui.vertical(|ui| {
            let mut job = LayoutJob::default();
            monospace_append!(job, name, Color32::ORANGE);
            monospace_append!(job, &format!(" = {value:#04X}\n"), Color32::WHITE);
            self.draw_bits(&mut job, value);
            ui.label(job);
        });
    }

    /// Draws an integer value in binary format with colored bits.
    fn draw_bits<T: BitView>(&self, job: &mut LayoutJob, value: T) {
        let bits = value.view_bits::<Msb0>();
        for (i, chunk) in bits.chunks(4).enumerate() {
            if i > 0 {
                monospace_append!(job, " ", Color32::default());
            }
            for bit in chunk {
                let color = match *bit {
                    true => Color32::WHITE,
                    false => Color32::GRAY,
                };
                monospace_append!(job, if *bit { "1" } else { "0" }, color);
            }
        }
    }

    /// Draws a 16-bit register with its name and value.
    fn draw_u16_register(&self, ui: &mut Ui, name: &str, value: u16) {
        ui.vertical_centered(|ui| {
            let mut job = LayoutJob::default();
            monospace_append!(job, name, Color32::ORANGE);
            monospace_append!(job, &format!(" = {value:#06X}\n"), Color32::WHITE);
            self.draw_bits(&mut job, value);
            ui.label(job);
        });
    }
}
