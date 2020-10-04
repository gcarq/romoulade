mod event;
pub mod format;
mod utils;

use crate::gb::cpu::CPU;
use crate::gb::debugger::event::{Event, Events};
use crate::gb::debugger::utils::resolve_byte_length;
use crate::gb::instruction::Instruction;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::PPU;
use crate::gb::timer::Timer;
use crate::gb::AddressSpace;
use std::cell::RefCell;
use std::error::Error;
use std::io;
use termion::event::Key;
use termion::raw::IntoRawMode;
use termion::screen::AlternateScreen;
use tui::backend::TermionBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::Terminal;

/// Starts the debugger and the emulating loop
pub fn emulate<T: AddressSpace>(
    cpu: &RefCell<CPU<T>>,
    bus: &RefCell<MemoryBus>,
    ppu: &mut PPU,
    timer: &mut Timer,
    irq_handler: &mut IRQHandler<T>,
) -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();

    loop {
        terminal.draw(|f| {
            let layout = Layout::default()
                .direction(Direction::Vertical)
                .margin(1)
                .constraints([Constraint::Percentage(95), Constraint::Percentage(5)].as_ref())
                .split(f.size());

            // Read next instructions to display
            let (selected, instructions) = read_instructions(cpu.borrow().pc, &bus);

            // Create list widget
            let list = List::new(instructions)
                .block(Block::default().title("Debugger").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
                .highlight_symbol(">> ");

            let mut state = ListState::default();
            state.select(Some(selected));

            f.render_stateful_widget(list, layout[0], &mut state);

            let paragraph = Paragraph::new(help_text())
                .style(Style::default().bg(Color::Black).fg(Color::White))
                .block(Block::default().borders(Borders::NONE))
                .alignment(Alignment::Left)
                .wrap(Wrap { trim: true });
            f.render_widget(paragraph, layout[1]);
        })?;

        match events.next()? {
            Event::Input(input) => match input {
                Key::Ctrl('c') => break,
                Key::F(3) => {
                    // Step over next instruction
                    let cycles = cpu.borrow_mut().step();
                    timer.step(cycles);
                    ppu.step(cycles);
                    irq_handler.handle();
                }
                _ => {}
            },
        }
    }
    Ok(())
}

/// Reads next instructions and returns a vector of formatted Strings
fn read_instructions(pc: u16, bus: &RefCell<MemoryBus>) -> (usize, Vec<ListItem>) {
    let mut cur_pc = pc;
    let mut frames = Vec::with_capacity(100);
    let mut current = 0;
    for i in 0..100 {
        let (instruction, new_pc) = step(cur_pc, &bus);
        if cur_pc == pc {
            current = i;
        }
        frames.push(colorize_instruction(cur_pc, instruction));
        cur_pc = new_pc;
    }
    (current, frames)
}

fn colorize_instruction(pc: u16, instruction: Instruction) -> ListItem<'static> {
    ListItem::new(Spans::from(vec![
        Span::styled(
            format!("{:#06X}: ", pc),
            Style::default().bg(Color::Black).fg(Color::Cyan),
        ),
        Span::raw(format!(" {}", instruction)),
    ]))
}

/// Emulates one CPU step without executing it.
/// Returns a tuple with the instruction and the updated program counter.
fn step(pc: u16, bus: &RefCell<MemoryBus>) -> (Instruction, u16) {
    // Read next opcode from memory
    let opcode = bus.borrow().read(pc);
    let (opcode, prefixed) = match opcode == 0xCB {
        true => (bus.borrow().read(pc + 1), true),
        false => (opcode, false),
    };

    // Parse instruction from opcode and return it together with the next program counter
    match Instruction::from_byte(opcode, prefixed) {
        Some(instruction) => (
            instruction,
            pc + resolve_byte_length(opcode, prefixed) as u16,
        ),
        None => {
            let description = format!("0x{}{:02x}", if prefixed { "cb" } else { "" }, opcode);
            panic!("Unresolved instruction: {}.\nHALTED!", description);
        }
    }
}

/// Returns static help text
fn help_text() -> Spans<'static> {
    Spans::from(vec![
        Span::styled("F3", Style::default().bg(Color::Gray).fg(Color::Black)),
        Span::raw(" Step    "),
        Span::styled("^C", Style::default().bg(Color::Gray).fg(Color::Black)),
        Span::raw(" Quit    "),
    ])
}
