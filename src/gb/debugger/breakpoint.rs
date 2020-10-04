use crate::gb::debugger::utils::centered_rect;
use std::collections::BTreeSet;
use std::error::Error;
use termion::event::Key;
use tui::backend::Backend;
use tui::style::{Color, Style};
use tui::widgets::{Block, Borders, Paragraph};
use tui::Frame;
use unicode_width::UnicodeWidthStr;

pub struct BreakpointHandler {
    pub breakpoints: BTreeSet<u16>,
    pub active: bool,
    pub input: String,
}

impl BreakpointHandler {
    pub fn new() -> Self {
        Self {
            breakpoints: BTreeSet::new(),
            active: false,
            input: String::new(),
        }
    }

    /// Shows "Add Breakpoint" dialog
    /// TODO: create dialog with fixed minimum size
    pub fn show_dialog<B: Backend>(&mut self, f: &mut Frame<B>) {
        let area = centered_rect(20, 10, f.size());
        let input = Paragraph::new(format!("> {}", self.input))
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Add Breakpoint"),
            );
        f.render_widget(input, area);
        f.set_cursor(area.x + self.input.width() as u16 + 3, area.y + 1);
        self.active = true;
    }

    pub fn handle_dialog_input(&mut self, key: Key) -> Result<(), Box<dyn Error>> {
        assert!(self.active);
        match key {
            Key::Char('\n') => {
                if let Ok(address) = self.parse_input() {
                    self.breakpoints.insert(address);
                    self.active = false;
                }
            }

            Key::Char(c) => {
                self.input.push(c);
            }
            Key::Backspace => {
                self.input.pop();
            }
            _ => {}
        }
        Ok(())
    }

    /// Checks if a given address is marked as breakpoint
    pub fn contains(&self, address: u16) -> bool {
        self.breakpoints.contains(&address)
    }

    fn parse_input(&mut self) -> Result<u16, Box<dyn Error>> {
        let bp = self.input.drain(..).collect::<String>();
        let bp = bp.trim_start_matches("0x");
        let bp = u16::from_str_radix(bp, 16)?;
        Ok(bp)
    }
}
