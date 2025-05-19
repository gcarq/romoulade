use crate::gb::cpu::instruction::Instruction;
use crate::gui::debugger::EmulatorState;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Align, Color32, FontId, Label, Sense, TextFormat, Ui, Widget};
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

        let instructions = state.instructions.as_slice();

        let mut table = TableBuilder::new(ui)
            .resizable(false)
            .column(Column::auto().at_least(310.0))
            .sense(Sense::click());

        if let Some(address) = self.scroll_to_address.take() {
            if let Some(index) = instructions
                .iter()
                .position(|(addr, _, _)| *addr == address)
            {
                table = table.scroll_to_row(index, Some(Align::Center));
            }
        }

        table.body(|body| {
            body.rows(text_height, instructions.len(), |mut row| {
                let index = row.index();
                let (addr, opcodes, instruction) = &instructions[index];
                row.set_selected(self.breakpoints.contains(addr));
                row.col(|ui| {
                    Self::row_label(state, *addr, opcodes, instruction).ui(ui);
                });
                if row.response().clicked() {
                    self.toggle_breakpoint(*addr);
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

    /// Creates a row label for the disassembly view.
    fn row_label(
        state: &EmulatorState,
        address: u16,
        opcodes: &[u8],
        instruction: &Instruction,
    ) -> Label {
        let opcodes = opcodes.iter().map(|b| format!("{b:02x}")).join(" ");
        let mut job = LayoutJob::default();
        if address == state.cpu.r.pc {
            job.append(
                &format!("-> {address:#06x}    "),
                0.0,
                text_format(Color32::LIGHT_RED),
            );
        } else {
            job.append(
                &format!("   {address:#06x}    "),
                0.0,
                text_format(Color32::LIGHT_GREEN),
            );
        }
        job.append(
            &format!("{opcodes:<12}"),
            0.0,
            text_format(match address == state.cpu.r.pc {
                true => Color32::LIGHT_RED,
                false => Color32::GRAY,
            }),
        );
        job.append(
            &format!("{instruction}"),
            0.0,
            text_format(match address == state.cpu.r.pc {
                true => Color32::LIGHT_RED,
                false => Color32::WHITE,
            }),
        );
        Label::new(job).selectable(false)
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

/// Returns a `TextFormat` that will be used for each row.
fn text_format(color: Color32) -> TextFormat {
    TextFormat {
        font_id: FontId::monospace(12.0),
        color,
        ..Default::default()
    }
}
