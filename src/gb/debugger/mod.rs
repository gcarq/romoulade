mod breakpoint;
mod event;
pub mod format;
mod utils;

use crate::gb::cpu::CPU;
use crate::gb::debugger::breakpoint::BreakpointHandler;
use crate::gb::debugger::event::{Event, Events};
use crate::gb::debugger::utils::resolve_byte_length;
use crate::gb::instruction::Instruction;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::constants::*;
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
use tui::widgets::{Block, Borders, List, ListItem, ListState, Paragraph};
use tui::{Frame, Terminal};

pub struct Debugger<'a, T: AddressSpace> {
    cpu: &'a RefCell<CPU<'a, T>>,
    bus: &'a RefCell<MemoryBus>,
    ppu: &'a mut PPU<'a>,
    timer: &'a mut Timer<'a>,
    irq_handler: &'a mut IRQHandler<'a, T>,
    bp_handler: BreakpointHandler,
    memory_offset: u16,
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
            memory_offset: 0,
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
                let (upper, middle, lower) = self.create_layouts(f);
                self.draw_assembly(f, upper[0]);
                self.draw_memory(f, upper[1]);
                self.draw_breakpoints(f, upper[2]);
                self.draw_cpu_registers(f, middle[0]);
                self.draw_cpu_flags(f, middle[1]);
                self.draw_ppu_flags(f, middle[2]);
                self.draw_timer_registers(f, middle[3]);
                self.draw_help(f, lower[0]);
                if self.bp_handler.active {
                    self.bp_handler.show_dialog(f);
                }
            })?;

            match events.next()? {
                Event::Input(input) => match input {
                    Key::Ctrl('c') => break,
                    Key::F(2) => {
                        while !self.bp_handler.contains(self.cpu.borrow().pc) {
                            self.execute();
                        }
                    }
                    Key::F(3) => {
                        self.execute();
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

    /// Creates and returns layouts for upper, middle and lower sections.
    fn create_layouts<B: Backend>(&self, f: &mut Frame<B>) -> (Vec<Rect>, Vec<Rect>, Vec<Rect>) {
        // Defines root vertical layout
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Percentage(80),
                    Constraint::Length(6),
                    Constraint::Length(2),
                ]
                .as_ref(),
            )
            .split(f.size());
        // Defines layout for assembly, memory and breakpoints widget
        let upper = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints(
                [
                    Constraint::Percentage(45),
                    Constraint::Percentage(45),
                    Constraint::Percentage(10),
                ]
                .as_ref(),
            )
            .split(root[0]);
        // Defines layout for register widget
        let middle = Layout::default()
            .direction(Direction::Horizontal)
            .horizontal_margin(1)
            .constraints(
                [
                    Constraint::Length(14),
                    Constraint::Length(16),
                    Constraint::Length(41),
                    Constraint::Length(14),
                    Constraint::Percentage(50),
                ]
                .as_ref(),
            )
            .split(root[1]);
        // Defines layout for help view
        let lower = Layout::default()
            .direction(Direction::Horizontal)
            .margin(1)
            .constraints([Constraint::Percentage(100)].as_ref())
            .split(root[2]);
        (upper, middle, lower)
    }

    /// Draws assembly widget
    fn draw_assembly<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        // Read next instructions to display
        let (selected, instructions) = self.read_instructions(area.height * 2);

        let list = List::new(instructions)
            .block(Block::default().title("Assembly").borders(Borders::ALL))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        let mut state = ListState::default();
        state.select(Some(usize::from(selected)));

        f.render_stateful_widget(list, area, &mut state);
    }

    /// Draws memory widget
    fn draw_memory<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let count = area.height;
        let bus = self.bus.borrow();

        // Read memory region to display
        let memory = (self.memory_offset..self.memory_offset + count * 16)
            .step_by(16)
            .map(|offset| {
                ListItem::new(Spans::from(vec![
                    Span::styled(
                        format!("{:#06x}:  ", offset),
                        Style::default().bg(Color::Black).fg(Color::Cyan),
                    ),
                    Span::raw(
                        (offset..offset + 16)
                            .map(|l| format!("{:02x}", bus.read(l)))
                            .collect::<Vec<String>>()
                            .join(" "),
                    ),
                ]))
            })
            .collect::<Vec<ListItem>>();

        let list = List::new(memory)
            .block(Block::default().title("Memory").borders(Borders::ALL))
            .style(Style::default().fg(Color::White));
        f.render_widget(list, area);
    }

    /// Draws CPU registers
    fn draw_cpu_registers<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let r = self.cpu.borrow().r;
        let text = vec![
            Spans::from(format!(" AF: {:#06x}", r.get_af())),
            Spans::from(format!(" BC: {:#06x}", r.get_bc())),
            Spans::from(format!(" DE: {:#06x}", r.get_de())),
            Spans::from(format!(" HL: {:#06x}", r.get_hl())),
        ];
        let block = Block::default().title("CPU Reg.").borders(Borders::ALL);
        let registers = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(registers, area);
    }

    /// Draws CPU flags
    fn draw_cpu_flags<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let r = self.cpu.borrow().r;
        let text = vec![
            Spans::from(format!(" Zero:      {}", r.f.zero as u8)),
            Spans::from(format!(" Negative:  {}", r.f.negative as u8)),
            Spans::from(format!(" HalfCarry: {}", r.f.half_carry as u8)),
            Spans::from(format!(" Carry:     {}", r.f.carry as u8)),
        ];
        let block = Block::default().title("CPU Flags").borders(Borders::ALL);
        let registers = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(registers, area);
    }

    /// Draws PPU registers
    fn draw_ppu_flags<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let bus = self.bus.borrow();
        let text = vec![
            Spans::from(Span::raw(format!(
                " LCDC: {:#04x}    LY:  {:#04x}    OBP0: {:#04x}",
                bus.read(PPU_LCDC),
                bus.read(PPU_LY),
                bus.read(PPU_OBP0)
            ))),
            Spans::from(Span::raw(format!(
                " STAT: {:#04x}    LCY: {:#04x}    OBP1: {:#04x}",
                bus.read(PPU_STAT),
                bus.read(PPU_LYC),
                bus.read(PPU_OBP1)
            ))),
            Spans::from(Span::raw(format!(
                " SCY:  {:#04x}    DMA: {:#04x}    WY:   {:#04x}",
                bus.read(PPU_SCY),
                bus.read(PPU_DMA),
                bus.read(PPU_WY)
            ))),
            Spans::from(Span::raw(format!(
                " SCX:  {:#04x}    BGP: {:#04x}    WX:   {:#04x}",
                bus.read(PPU_SCX),
                bus.read(PPU_BGP),
                bus.read(PPU_WX)
            ))),
        ];
        let block = Block::default()
            .title("PPU Registers")
            .borders(Borders::ALL);
        let registers = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(registers, area);
    }

    /// Draws Timer registers
    fn draw_timer_registers<B: Backend>(&mut self, f: &mut Frame<B>, area: Rect) {
        let bus = self.bus.borrow();
        let text = vec![
            Spans::from(format!(" DIV:  {:#04x}", bus.read(TIMER_DIVIDER))),
            Spans::from(format!(" TIMA: {:#04x}", bus.read(TIMER_COUNTER))),
            Spans::from(format!(" TMA:  {:#04x}", bus.read(TIMER_MODULO))),
            Spans::from(format!(" TAC:  {:#04x}", bus.read(TIMER_CTRL))),
        ];
        let block = Block::default().title("Timer Reg.").borders(Borders::ALL);
        let registers = Paragraph::new(text)
            .block(block)
            .style(Style::default().fg(Color::White).bg(Color::Black));
        f.render_widget(registers, area);
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
        .highlight_style(Style::default().fg(Color::Green));
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
            .alignment(Alignment::Left);
        f.render_widget(paragraph, area);
    }

    /// Reads next n instructions and returns a tuple
    /// with the index of the current pc and a vector of formatted Strings.
    fn read_instructions(&self, count: u16) -> (u16, Vec<ListItem>) {
        let cpu_pc = self.cpu.borrow().pc;
        let mut pc = cpu_pc;
        let mut frames = Vec::with_capacity(usize::from(count));
        let mut pc_index = 0;
        for i in 0..count {
            let (instruction, new_pc) = self.simulate_step(pc);
            if cpu_pc == pc {
                pc_index = i;
            }
            // Collect bytes for this instruction as string
            let bytes = (pc..new_pc)
                .map(|i| format!("{:02x}", self.bus.borrow().read(i)))
                .collect::<Vec<String>>()
                .join(" ");
            frames.push(self.format_instruction(pc, &bytes, instruction));
            pc = new_pc;
        }
        (pc_index, frames)
    }

    /// Formats and colorizes the given instruction (including raw bytes)
    /// and returns a TUI compatible ListItem.
    fn format_instruction(
        &self,
        pc: u16,
        bytes: &str,
        instruction: Option<Instruction>,
    ) -> ListItem<'static> {
        let address_style = match self.bp_handler.contains(pc) {
            true => Style::default().bg(Color::Black).fg(Color::Red),
            false => Style::default().bg(Color::Black).fg(Color::Cyan),
        };
        let bytes_style = Style::default().bg(Color::Black).fg(Color::Gray);
        let instruction_span = match instruction {
            Some(i) => Span::raw(format!(" {}", i)),
            None => Span::styled(" DATA", Style::default().fg(Color::Red)),
        };
        ListItem::new(Spans::from(vec![
            Span::styled(format!("{:#06x}:  ", pc), address_style),
            Span::styled(format!("{:<10}", bytes), bytes_style),
            instruction_span,
        ]))
    }

    /// Simulates one CPU step without executing it.
    /// Returns a tuple with the instruction and the updated program counter.
    fn simulate_step(&self, pc: u16) -> (Option<Instruction>, u16) {
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

    /// Executes a single step
    fn execute(&mut self) {
        let cycles = self.cpu.borrow_mut().step();
        self.timer.step(cycles);
        self.ppu.step(cycles);
        self.irq_handler.handle();
    }
}
