use crate::gui::debugger::EmulatorState;
use eframe::egui::{Align, Color32, RichText, Sense, Ui};
use egui_extras::{Column, TableBuilder};
use itertools::Itertools;
use std::collections::HashSet;

/// Represents the disassembler view in the debugger UI.
#[derive(Default)]
pub struct Disassembler {
    pub breakpoints: HashSet<u16>,
    pub scroll_to_address: Option<u16>,
}

impl Disassembler {
    /// Updates the assembly view with the current state.
    /// Returns true if breakpoints were modified.
    pub fn update(&mut self, state: &EmulatorState, ui: &mut Ui) -> bool {
        let mut breakpoints_changed = false;
        let text_height = 16.0;

        let mut table = TableBuilder::new(ui)
            .resizable(false)
            .column(Column::exact(16.0))
            .column(Column::exact(60.0))
            .column(Column::exact(76.0))
            .column(Column::exact(140.0))
            .sense(Sense::click());

        let instructions = state.instructions.as_slice();
        if let Some(address) = self.scroll_to_address.take() {
            if let Ok(index) = instructions.binary_search_by_key(&address, |ctx| ctx.address) {
                table = table.scroll_to_row(index, Some(Align::Center));
            }
        }

        table.body(|body| {
            body.rows(text_height, instructions.len(), |mut row| {
                let ctx = &instructions[row.index()];
                row.set_selected(self.breakpoints.contains(&ctx.address));
                // Draw PC indicator
                row.col(|ui| {
                    if ctx.address == state.cpu.r.pc {
                        ui.label(RichText::new("->").monospace().color(Color32::LIGHT_RED));
                    }
                });
                // Draw address column
                row.col(|ui| {
                    let text = RichText::new(format!("{:#06x}", ctx.address));
                    let color = match ctx.address == state.cpu.r.pc {
                        true => Color32::LIGHT_RED,
                        false => Color32::LIGHT_GREEN,
                    };
                    ui.label(text.monospace().color(color));
                });
                // Draw raw bytes column
                row.col(|ui| {
                    let text =
                        RichText::new(ctx.bytes.iter().map(|b| format!("{b:02x}")).join(" "));
                    let color = match ctx.address == state.cpu.r.pc {
                        true => Color32::LIGHT_RED,
                        false => Color32::GRAY,
                    };
                    ui.label(text.monospace().color(color));
                });
                // Draw instruction column
                row.col(|ui| {
                    let text = RichText::new(ctx.instruction.to_string()).monospace();
                    let color = match ctx.address == state.cpu.r.pc {
                        true => Color32::LIGHT_RED,
                        false => Color32::WHITE,
                    };
                    ui.label(text.monospace().color(color));
                });
                if row.response().clicked() {
                    self.toggle_breakpoint(ctx.address);
                    breakpoints_changed = true;
                }
            });
        });
        breakpoints_changed
    }

    /// Scrolls the disassembly view to the given address.
    #[inline]
    pub const fn scroll_to_address(&mut self, address: u16) {
        self.scroll_to_address = Some(address);
    }

    /// Toggles a breakpoint at the given address.
    #[inline]
    fn toggle_breakpoint(&mut self, address: u16) {
        if self.breakpoints.contains(&address) {
            self.breakpoints.remove(&address);
        } else {
            self.breakpoints.insert(address);
        }
    }
}
