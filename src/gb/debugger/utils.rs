use crate::gb::instruction::Instruction;
use tui::layout::{Constraint, Direction, Layout, Rect};
use tui::style::{Color, Style};
use tui::text::{Span, Spans};
use tui::widgets::ListItem;

/// Formats and colorizes the given instruction (including raw bytes)
/// and returns a TUI compatible ListItem.
pub fn format_instruction(pc: u16, bytes: &str, instruction: Instruction) -> ListItem<'static> {
    ListItem::new(Spans::from(vec![
        Span::styled(
            format!("{:#06X}:  ", pc),
            Style::default().bg(Color::Black).fg(Color::Cyan),
        ),
        Span::styled(
            format!("{:<10}", bytes),
            Style::default().bg(Color::Black).fg(Color::Gray),
        ),
        Span::raw(format!(" {}", instruction)),
    ]))
}

/// helper function to create a centered rect using up
/// certain percentage of the available rect `r`
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

/// Resolves the instruction byte length for the given opcode
pub fn resolve_byte_length(opcode: u8, prefixed: bool) -> u8 {
    // All prefixed opcodes have a length of 2 bytes
    if prefixed {
        return 2;
    }

    NON_PREFIXED_OPCODE_MAP
        .binary_search_by(|(k, _)| k.cmp(&opcode))
        .map(|x| NON_PREFIXED_OPCODE_MAP[x].1)
        .unwrap()
}

/// Defines a mapping between non prefixed opcodes to byte length
static NON_PREFIXED_OPCODE_MAP: &[(u8, u8)] = &[
    (0x00, 1),
    (0x01, 3),
    (0x02, 1),
    (0x03, 1),
    (0x04, 1),
    (0x05, 1),
    (0x06, 2),
    (0x07, 1),
    (0x08, 3),
    (0x09, 1),
    (0x0A, 1),
    (0x0B, 1),
    (0x0C, 1),
    (0x0D, 1),
    (0x0E, 2),
    (0x0F, 1),
    //////////
    (0x10, 2),
    (0x11, 3),
    (0x12, 1),
    (0x13, 1),
    (0x14, 1),
    (0x15, 1),
    (0x16, 2),
    (0x17, 1),
    (0x18, 2),
    (0x19, 1),
    (0x1A, 1),
    (0x1B, 1),
    (0x1C, 1),
    (0x1D, 1),
    (0x1E, 2),
    (0x1F, 1),
    //////////
    (0x20, 2),
    (0x21, 3),
    (0x22, 1),
    (0x23, 1),
    (0x24, 1),
    (0x25, 1),
    (0x26, 2),
    (0x27, 1),
    (0x28, 2),
    (0x29, 1),
    (0x2A, 1),
    (0x2B, 1),
    (0x2C, 1),
    (0x2D, 1),
    (0x2E, 2),
    (0x2F, 1),
    //////////
    (0x30, 2),
    (0x31, 3),
    (0x32, 1),
    (0x33, 1),
    (0x34, 1),
    (0x35, 1),
    (0x36, 2),
    (0x37, 1),
    (0x38, 2),
    (0x39, 1),
    (0x3A, 1),
    (0x3B, 1),
    (0x3C, 1),
    (0x3D, 1),
    (0x3E, 2),
    (0x3F, 1),
    //////////
    (0x40, 1),
    (0x41, 1),
    (0x42, 1),
    (0x43, 1),
    (0x44, 1),
    (0x45, 1),
    (0x46, 1),
    (0x47, 1),
    (0x48, 1),
    (0x49, 1),
    (0x4A, 1),
    (0x4B, 1),
    (0x4C, 1),
    (0x4D, 1),
    (0x4E, 1),
    (0x4F, 1),
    //////////
    (0x50, 1),
    (0x51, 1),
    (0x52, 1),
    (0x53, 1),
    (0x54, 1),
    (0x55, 1),
    (0x56, 1),
    (0x57, 1),
    (0x58, 1),
    (0x59, 1),
    (0x5A, 1),
    (0x5B, 1),
    (0x5C, 1),
    (0x5D, 1),
    (0x5E, 1),
    (0x5F, 1),
    //////////
    (0x60, 1),
    (0x61, 1),
    (0x62, 1),
    (0x63, 1),
    (0x64, 1),
    (0x65, 1),
    (0x66, 1),
    (0x67, 1),
    (0x68, 1),
    (0x69, 1),
    (0x6A, 1),
    (0x6B, 1),
    (0x6C, 1),
    (0x6D, 1),
    (0x6E, 1),
    (0x6F, 1),
    //////////
    (0x70, 1),
    (0x71, 1),
    (0x72, 1),
    (0x73, 1),
    (0x74, 1),
    (0x75, 1),
    (0x76, 1),
    (0x77, 1),
    (0x78, 1),
    (0x79, 1),
    (0x7A, 1),
    (0x7B, 1),
    (0x7C, 1),
    (0x7D, 1),
    (0x7E, 1),
    (0x7F, 1),
    //////////
    (0x80, 1),
    (0x81, 1),
    (0x82, 1),
    (0x83, 1),
    (0x84, 1),
    (0x85, 1),
    (0x86, 1),
    (0x87, 1),
    (0x88, 1),
    (0x89, 1),
    (0x8A, 1),
    (0x8B, 1),
    (0x8C, 1),
    (0x8D, 1),
    (0x8E, 1),
    (0x8F, 1),
    //////////
    (0x90, 1),
    (0x91, 1),
    (0x92, 1),
    (0x93, 1),
    (0x94, 1),
    (0x95, 1),
    (0x96, 1),
    (0x97, 1),
    (0x98, 1),
    (0x99, 1),
    (0x9A, 1),
    (0x9B, 1),
    (0x9C, 1),
    (0x9D, 1),
    (0x9E, 1),
    (0x9F, 1),
    //////////
    (0xA0, 1),
    (0xA1, 1),
    (0xA2, 1),
    (0xA3, 1),
    (0xA4, 1),
    (0xA5, 1),
    (0xA6, 1),
    (0xA7, 1),
    (0xA8, 1),
    (0xA9, 1),
    (0xAA, 1),
    (0xAB, 1),
    (0xAC, 1),
    (0xAD, 1),
    (0xAE, 1),
    (0xAF, 1),
    //////////
    (0xB0, 1),
    (0xB1, 1),
    (0xB2, 1),
    (0xB3, 1),
    (0xB4, 1),
    (0xB5, 1),
    (0xB6, 1),
    (0xB7, 1),
    (0xB8, 1),
    (0xB9, 1),
    (0xBA, 1),
    (0xBB, 1),
    (0xBC, 1),
    (0xBD, 1),
    (0xBE, 1),
    (0xBF, 1),
    //////////
    (0xC0, 1),
    (0xC1, 1),
    (0xC2, 3),
    (0xC3, 3),
    (0xC4, 3),
    (0xC5, 1),
    (0xC6, 2),
    (0xC7, 1),
    (0xC8, 1),
    (0xC9, 1),
    (0xCA, 3),
    (0xCB, 1),
    (0xCC, 3),
    (0xCD, 3),
    (0xCE, 2),
    (0xCF, 1),
    //////////
    (0xD0, 1),
    (0xD1, 1),
    (0xD2, 3),
    (0xD4, 3),
    (0xD5, 1),
    (0xD6, 2),
    (0xD7, 1),
    (0xD8, 1),
    (0xD9, 1),
    (0xDA, 3),
    (0xDC, 3),
    (0xDE, 2),
    (0xDF, 1),
    //////////
    (0xE0, 2),
    (0xE1, 1),
    (0xE2, 1),
    (0xE5, 1),
    (0xE6, 2),
    (0xE7, 1),
    (0xE8, 2),
    (0xE9, 1),
    (0xEA, 3),
    (0xEE, 2),
    (0xEF, 1),
    //////////
    (0xF0, 2),
    (0xF1, 1),
    (0xF2, 1),
    (0xF3, 1),
    (0xF5, 1),
    (0xF6, 2),
    (0xF7, 1),
    (0xF8, 2),
    (0xF9, 1),
    (0xFA, 3),
    (0xFB, 1),
    (0xFE, 2),
    (0xFF, 1),
];
