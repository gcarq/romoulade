use crate::gui::debugger::EmulatorState;
use egui::{Align, Color32, Label, RichText, Ui, Widget};
use egui_extras::{Column, TableBuilder};
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

        let text_height = egui::TextStyle::Monospace
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);

        let instructions = state.instructions.as_slice();

        let mut table = TableBuilder::new(ui)
            .resizable(false)
            .column(Column::auto().at_least(300.0))
            .sense(egui::Sense::click());

        if let Some(address) = self.scroll_to_address {
            if let Some(index) = instructions.iter().position(|(addr, _)| *addr == address) {
                table = table.scroll_to_row(index, Some(Align::Center));
                self.scroll_to_address = None;
            }
        }

        table.body(|body| {
            body.rows(text_height, instructions.len(), |mut row| {
                let index = row.index();
                let (addr, instruction) = instructions[index];
                let text = if addr == state.cpu.pc {
                    RichText::from(format!("-> {addr:#06X}\t\t{instruction}"))
                        .monospace()
                        .color(Color32::LIGHT_GREEN)
                } else {
                    RichText::from(format!("   {addr:#06X}\t\t{instruction}"))
                        .monospace()
                        .color(Color32::WHITE)
                };

                row.set_selected(self.breakpoints.contains(&addr));
                row.col(|ui| {
                    Label::new(text).selectable(false).ui(ui);
                });
                if row.response().clicked() {
                    self.toggle_breakpoint(addr);
                    breakpoints_changed = true;
                }
            });
        });
        breakpoints_changed
    }

    /// Scrolls the disassembly view to the given address.
    #[inline]
    pub fn scroll_to_address(&mut self, address: u16) {
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
