use crate::gb::SubSystem;
use crate::gb::constants::*;
use crate::gui::debugger::EmulatorState;
use crate::monospace_append;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, RichText, TextFormat, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use std::sync::Arc;

#[derive(Default, PartialEq)]
enum MemoryArea {
    #[default]
    RomBank0,
    RomBank1,
    CRam,
    WRam,
    Oam,
    IO,
    HRam,
}

#[derive(Default)]
pub struct MemoryMap {
    memory_area: MemoryArea,
}

impl MemoryMap {
    /// Updates the memory map view with the current state.
    pub fn update(&mut self, state: &mut EmulatorState, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.memory_area, MemoryArea::RomBank0, "ROM0");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::RomBank1, "ROM1");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::CRam, "CRAM");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::WRam, "WRAM");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::Oam, "OAM");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::IO, "IO");
            ui.separator();
            ui.selectable_value(&mut self.memory_area, MemoryArea::HRam, "HRAM");
        });
        ui.separator();

        let text_height = 16.0;
        let table = TableBuilder::new(ui)
            .resizable(false)
            .striped(true)
            .column(Column::exact(60.0))
            .column(Column::exact(350.0))
            .column(Column::exact(140.0))
            .header(text_height, |mut header| {
                header.col(|ui| {
                    ui.label(RichText::new("ADDRESS").monospace().color(Color32::CYAN));
                });
                header.col(|ui| {
                    let offsets = RichText::new((0..16).map(|i| format!("{i:02x}")).join(" "));
                    ui.label(offsets.monospace().color(Color32::CYAN));
                });
                header.col(|ui| {
                    ui.label(RichText::new("ASCII").monospace().color(Color32::CYAN));
                });
            });

        // Determine the memory area to display
        let (mem_begin, mem_range) = match self.memory_area {
            MemoryArea::RomBank0 => (ROM_LOW_BANK_BEGIN, ROM_LOW_BANK_BEGIN..=ROM_LOW_BANK_END),
            MemoryArea::RomBank1 => (ROM_HIGH_BANK_BEGIN, ROM_HIGH_BANK_BEGIN..=ROM_HIGH_BANK_END),
            MemoryArea::CRam => (CRAM_BANK_BEGIN, CRAM_BANK_BEGIN..=CRAM_BANK_END),
            MemoryArea::WRam => (WRAM_BEGIN, WRAM_BEGIN..=WRAM_END),
            MemoryArea::Oam => (OAM_BEGIN, OAM_BEGIN..=OAM_END),
            MemoryArea::IO => (IO_BEGIN, IO_BEGIN..=IO_END),
            MemoryArea::HRam => (HRAM_BEGIN, HRAM_BEGIN..=HRAM_END),
        };

        let memory = mem_range
            .map(|addr| state.bus.read(addr))
            .collect::<Arc<_>>();

        let memory = memory.chunks(16).collect::<Arc<_>>();

        table.body(|body| {
            body.rows(text_height, memory.len(), |mut row| {
                let index = row.index();
                let bytes = &memory[index];

                // Draw address column
                row.col(|ui| {
                    let address = mem_begin + index as u16 * 16;
                    let text = RichText::new(format!("{address:#06X}"));
                    ui.label(text.monospace().color(Color32::LIGHT_GREEN));
                });
                // Draw bytes column
                row.col(|ui| {
                    let mut job = LayoutJob::default();
                    for byte in *bytes {
                        let color = match *byte != 0 {
                            true => Color32::WHITE,
                            false => Color32::GRAY,
                        };
                        monospace_append!(job, &format!("{byte:02x} "), color);
                    }
                    ui.label(job);
                });
                // Draw ASCII column
                row.col(|ui| {
                    let mut job = LayoutJob::default();
                    for byte in *bytes {
                        let (text, color) = match byte.is_ascii_graphic() {
                            true => (*byte as char, Color32::WHITE),
                            false => ('.', Color32::GRAY),
                        };
                        monospace_append!(job, text, color);
                    }
                    ui.label(job);
                });
            });
        });
    }
}
