mod breakpoint;
mod event;
pub mod format;
mod utils;

use crate::gb::cpu::CPU;
use crate::gb::debugger::breakpoint::BreakpointHandler;
use crate::gb::debugger::event::{Event, Events};
use crate::gb::debugger::utils::{format_instruction, resolve_byte_length};
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
use tui::backend::{Backend, TermionBackend};
use tui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap};
use tui::{Frame, Terminal};

pub struct Debugger<'a, T: AddressSpace> {
    cpu: &'a RefCell<CPU<'a, T>>,
    bus: &'a RefCell<MemoryBus>,
    ppu: &'a mut PPU<'a>,
    timer: &'a mut Timer<'a>,
    irq_handler: &'a mut IRQHandler<'a, T>,
    bp_handler: BreakpointHandler,
}

impl<'a, T: AddressSpace> Debugger<'a, T> {
    /// Creates a new debugger
    pub fn new(
        cpu: &'a RefCell<CPU<'a, T>>,
        bus: &'a RefCell<MemoryBus>,
        ppu: &'a mut PPU<'a>,
        timer: &'a mut Timer<'a>,
        irq_handler: &'a mut IRQHandler<'a, T>,
    ) -> Self {
        Self {
            cpu,
            bus,
            ppu,
            timer,
            irq_handler,
            bp_handler: BreakpointHandler::new(),
        }
    }

    /// Starts the emulating loop
    pub fn emulate(&mut self) -> Result<(), Box<dyn Error>> {
        let stdout = io::stdout().into_raw_mode()?;
        let stdout = AlternateScreen::from(stdout);
        let backend = TermionBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        let events = Events::new();

        loop {
            terminal.draw(|f| {
                // Defines root layout
                let root = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(
                        [
                            Constraint::Percentage(75),
                            Constraint::Length(6),
                            Constraint::Length(2),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                // Defines layout for assembly and breakpoints view
                let upper = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(root[0]);

                self.draw_assembly(f, upper[0]);
                self.draw_breakpoints(f, upper[1]);
                self.draw_registers(f, root[1]);
                self.draw_help(f, root[2]);
                if self.bp_handler.active {
                    self.bp_handler.show_dialog(f);
                }
            })?;

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Ctrl('c') => break,
                    Key::F(2) => {
                        while !self.bp_handler.contains(self.cpu.borrow().pc) {
                            let cycles = self.cpu.borrow_mut().step();
                            self.timer.step(cycles);
                            self.ppu.step(cycles);
                            self.irq_handler.handle();
                        }
                    }
                    Key::F(3) => {
                        // Step over next instruction
                        let cycles = self.cpu.borrow_mut().step();
                        self.timer.step(cycles);
                        self.ppu.step(cycles);
                        self.irq_handler.handle();
                    }
                    Key::Esc if self.bp_handler.active => self.bp_handler.active = false,
                    Key::F(4) => self.bp_handler.active = !self.bp_handler.active,
                    key if self.bp_handler.active => self.bp_handler.handle_dialog_input(key)?,
                    _ => {}
                },
            }
        }
        Ok(())
    }

    /// Draws instruction list
    fn draw_assembly<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Read next instructions to display
        let (selected, instructions) = self.read_instructions(area.height * 2);

        // Create list widget
        let list = List::new(instructions)
            .block(Block::default().title("Assembly").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::ITALIC))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(usize::from(selected)));

        f.render_stateful_widget(list, area, &mut state);
    }

    /// Draws registers
    fn draw_registers<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let chunks = Layout::default()
            .constraints(
                [
                    Constraint::Length(14),
                    Constraint::Length(16),
                    Constraint::Percentage(60),
                ]
                .as_ref(),
            )
            .direction(Direction::Horizontal)
            .split(area);

        let r = self.cpu.borrow().r;

        // Render register list
        let registers = List::new(vec![
            ListItem::new(format!(" AF: {:#06X}", r.get_af())),
            ListItem::new(format!(" BC: {:#06X}", r.get_bc())),
            ListItem::new(format!(" DE: {:#06X}", r.get_de())),
            ListItem::new(format!(" HL: {:#06X}", r.get_hl())),
        ])
        .block(Block::default().title("Registers").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
        f.render_widget(registers, chunks[0]);

        // Render flags list
        let flags = List::new(vec![
            ListItem::new(format!(" Zero:      {}", r.f.zero as u8)),
            ListItem::new(format!(" Negative:  {}", r.f.negative as u8)),
            ListItem::new(format!(" HalfCarry: {}", r.f.half_carry as u8)),
            ListItem::new(format!(" Carry:     {}", r.f.carry as u8)),
        ])
        .block(Block::default().title("Flags").borders(Borders::ALL))
        .style(Style::default().fg(Color::White));
        f.render_widget(flags, chunks[1]);
    }

    fn draw_breakpoints<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Create list widget
        let list = List::new(
            self.bp_handler
                .breakpoints
                .iter()
                .map(|a| ListItem::new(format!(" {:#06X}", a)))
                .collect::<Vec<ListItem>>(),
        )
        .block(Block::default().title("Breakpoints").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::ITALIC),
        );
        let mut state = ListState::default();
        let pc = self.cpu.borrow().pc;
        state.select(self.bp_handler.breakpoints.iter().position(|a| a == &pc));
        f.render_stateful_widget(list, area, &mut state);
    }

    /// Draws the static help text
    fn draw_help<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let text = Spans::from(vec![
            Span::styled("F2", Style::default().bg(Color::Gray).fg(Color::Black)),
            Span::raw(" Run     "),
            Span::styled("F3", Style::default().bg(Color::Gray).fg(Color::Black)),
            Span::raw(" Step    "),
            Span::styled("F4", Style::default().bg(Color::Gray).fg(Color::Black)),
            Span::raw(" Set Breakpoint    "),
            Span::styled("^C", Style::default().bg(Color::Gray).fg(Color::Black)),
            Span::raw(" Quit    "),
        ]);

        let paragraph = Paragraph::new(text)
            .style(Style::default().bg(Color::Black).fg(Color::White))
            .block(Block::default().borders(Borders::NONE))
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    /// Reads next n instructions and returns a tuple
    /// with the index of the current pc and a vector of formatted Strings.
    fn read_instructions(&self, count: u16) -> (u16, Vec<ListItem>) {
        let cpu_pc = self.cpu.borrow().pc;
        let mut pc = cpu_pc.saturating_sub(count / 2);
        let mut frames = Vec::with_capacity(100);
        let mut current = 0;
        for i in 0..count {
            let (instruction, new_pc) = self.step(pc);
            // Check if the instruction is an actual instruction or random data
            // TODO: maybe show byte code even if its not a valid instruction?
            if let Some(instruction) = instruction {
                if cpu_pc == pc {
                    current = i;
                }
                // Collect bytes for this instruction as string
                let bytes = (pc..new_pc)
                    .map(|i| format!("{:02X}", self.bus.borrow().read(i)))
                    .collect::<Vec<String>>()
                    .join(" ");
                frames.push(format_instruction(pc, &bytes, instruction));
                pc = new_pc;
            }
        }
        (current, frames)
    }

    /// Emulates one CPU step without executing it.
    /// Returns a tuple with the instruction and the updated program counter.
    fn step(&self, pc: u16) -> (Option<Instruction>, u16) {
        // Read next opcode from memory
        let opcode = self.bus.borrow().read(pc);
        let (opcode, prefixed) = match opcode == 0xCB {
            true => (self.bus.borrow().read(pc + 1), true),
            false => (opcode, false),
        };

        // Parse instruction from opcode and return it together with the next program counter
        match Instruction::from_byte(opcode, prefixed) {
            Some(instruction) => (
                Some(instruction),
                pc + resolve_byte_length(opcode, prefixed) as u16,
            ),
            None => (None, pc),
        }
    }
}
