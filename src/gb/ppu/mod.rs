mod fetcher;

use crate::gb::display::Display;
use crate::gb::interrupt::IRQ;
use crate::gb::memory::constants::{PPU_LCDC, PPU_LY, PPU_LYC, PPU_SCX, PPU_SCY, PPU_STAT};
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::fetcher::Fetcher;
use crate::gb::timer::Clock;
use crate::gb::{AddressSpace, SCREEN_HEIGHT, SCREEN_WIDTH, VERTICAL_BLANK_SCAN_LINE_MAX};
use std::cell::RefCell;
use std::convert;

bitflags! {
    /// Represents PPU_LCDC at 0xFF40
    struct LCDControl: u8 {
        const BG_EN    = 0b00000001; // BG Enable
        const OBJ_EN   = 0b00000010; // OBJ Enable
        const OBJ_SIZE = 0b00000100; // OBJ Size
        const BG_MAP   = 0b00001000; // BG Tile Map Address
        const TILE_SEL = 0b00010000; // BG & Window Tile Data
        const WIN_EN   = 0b00100000; // Window Enable
        const WIN_MAP  = 0b01000000; // Window Tile Map Address
        const LCD_EN   = 0b10000000; // LCD Display Enable
    }
}

bitflags! {
    /// Represents PPU_STAT at 0xFF41
    struct LCDState: u8 {
        const LCD_MODE1   = 0b00000001; // LCD Mode
        const LCD_MODE2   = 0b00000010; // LCD Mode
        const LYC_STAT    = 0b00000100; // LY Flag
        const H_BLANK_INT = 0b00001000; // Mode 0 H-Blank Interrupt
        const V_BLANK_INT = 0b00010000; // Mode 1 V-Blank Interrupt
        const OAM_INT     = 0b00100000; // Mode 2 OAM Interrupt
        const LY_INT      = 0b01000000; // LY Interrupt
    }
}

/// Represents the first two bits in LCDState
#[derive(Debug, Copy, Clone, PartialEq)]
#[repr(u8)]
pub enum LCDMode {
    HBlank = 0x0,
    VBlank = 0x1,
    OAMSearch = 0x2,
    PixelTransfer = 0x3,
}

impl convert::From<LCDMode> for u8 {
    fn from(value: LCDMode) -> u8 {
        match value {
            LCDMode::HBlank => 0b00,
            LCDMode::VBlank => 0b01,
            LCDMode::OAMSearch => 0b10,
            LCDMode::PixelTransfer => 0b11,
        }
    }
}

impl convert::From<u8> for LCDMode {
    fn from(value: u8) -> Self {
        match value {
            0b00 => LCDMode::HBlank,
            0b01 => LCDMode::VBlank,
            0b10 => LCDMode::OAMSearch,
            0b11 => LCDMode::PixelTransfer,
            _ => unimplemented!(),
        }
    }
}

/// Pixel Processing Unit
pub struct PPU<'a> {
    clock: Clock,
    bus: &'a RefCell<MemoryBus>,
    fetcher: Fetcher<'a>,
    display: &'a mut Display,
    x: u8,
}

impl<'a> PPU<'a> {
    pub fn new(bus: &'a RefCell<MemoryBus>, display: &'a mut Display) -> Self {
        Self {
            clock: Clock::new(),
            bus,
            fetcher: Fetcher::new(&bus),
            display,
            x: 0,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        if !self.read_ctrl().contains(LCDControl::LCD_EN) {
            self.set_lcd_mode(LCDMode::OAMSearch);
            // Screen is off, PPU remains idle.
            return;
        }

        self.clock.advance(cycles);

        let state = self.read_stat();
        let cur_mode = self.lcd_mode();
        let (mode, irq) = match cur_mode {
            // In this state, the PPU would scan the OAM (Objects Attribute Memory)
            // from 0xfe00 to 0xfe9f to mix sprite pixels in the current line later.
            // This always takes 40 ticks.
            LCDMode::OAMSearch if self.clock.ticks() >= 40 => {
                // Move to Pixel Transfer state. Initialize the fetcher to start
                // reading background tiles from VRAM. The boot ROM does nothing
                // fancy with map addresses, so we just give the fetcher the base
                // address of the row of tiles we need in video RAM, adjusted with
                // the value in our vertical scrolling register.
                //
                // In the present case, we only need to figure out in which row of
                // the background map our current line (at position Y) is. Then we
                // start fetching pixels from that row's address in VRAM, and for
                // each tile, we can tell which 8-pixel line to fetch by computing
                // Y modulo 8.
                self.x = 0;
                // TODO: add case for drawing windows
                let y = self.read(PPU_SCY).wrapping_add(self.read(PPU_LY));
                let x = self.read(PPU_SCX).wrapping_add(self.x);

                let tile_row = u16::from(y / 8) * 32;
                let tile_column = u16::from(x / 8);
                let tile_map_row_addr = 0x9800 + tile_row + tile_column;

                let tile_line = y % 8;
                self.fetcher.start(tile_map_row_addr, tile_line);
                (LCDMode::PixelTransfer, false)
            }
            LCDMode::PixelTransfer => {
                // Fetch pixel data into our pixel FIFO.
                self.fetcher.step();
                // Stop here if the FIFO isn't holding at least 8 pixels. This will
                // be used to mix in sprite data when we implement these. It also
                // guarantees the FIFO will always have data to Pop() later.
                if self.fetcher.fifo.len() <= 8 {
                    (LCDMode::PixelTransfer, false)
                } else {
                    // Put a pixel from the FIFO on screen if we have any.
                    if let Some(color) = self.fetcher.fifo.pop_front() {
                        self.display.write_pixel(self.x, self.read(PPU_LY), color);
                    }
                    // Check when the scanline is complete (160 pixels).
                    self.x = self.x.wrapping_add(1);
                    match self.x == SCREEN_WIDTH {
                        true => (LCDMode::HBlank, state.contains(LCDState::H_BLANK_INT)),
                        false => (LCDMode::PixelTransfer, false),
                    }
                }
            }
            // Nothing much to do here but wait the proper number of clock cycles.
            // A full scanline takes 456 clock cycles to complete. At the end of a
            // scanline, the PPU goes back to the initial OAM Search state.
            // When we reach line 144, we switch to VBlank state instead.
            LCDMode::HBlank if self.clock.ticks() >= 456 => {
                self.clock.reset();
                self.write(PPU_LY, self.read(PPU_LY).wrapping_add(1));
                if self.read(PPU_LY) == SCREEN_HEIGHT {
                    self.display.render_screen();
                    self.write_stat(self.read_stat() | LCDState::V_BLANK_INT);
                    (LCDMode::VBlank, state.contains(LCDState::V_BLANK_INT))
                } else {
                    (LCDMode::OAMSearch, state.contains(LCDState::OAM_INT))
                }
            }
            // Nothing much to do here either. VBlank is when the CPU is supposed to
            // do stuff that takes time. It takes as many cycles as would be needed
            // to keep displaying scanlines up to line 153.
            LCDMode::VBlank if self.clock.ticks() >= 456 => {
                self.clock.reset();
                self.write(PPU_LY, self.read(PPU_LY).wrapping_add(1));
                if self.read(PPU_LY) == VERTICAL_BLANK_SCAN_LINE_MAX {
                    self.write(PPU_LY, 0);
                    (LCDMode::OAMSearch, state.contains(LCDState::OAM_INT))
                } else {
                    (LCDMode::VBlank, false)
                }
            }
            // No mode change occurred
            mode => (mode, false),
        };

        // handle pending interrupts if mode changed
        if mode != cur_mode && irq {
            self.bus.borrow_mut().irq(IRQ::LCD);
        }
        self.set_lcd_mode(mode);

        self.handle_coincidence_flag();
    }

    /// Checks the coincidence flag and requests an interrupt if required
    fn handle_coincidence_flag(&mut self) {
        let state = self.read_stat();
        if self.read(PPU_LY) == self.read(PPU_LYC) {
            if state.contains(LCDState::LY_INT) {
                self.bus.borrow_mut().irq(IRQ::LCD);
            }
            self.write_stat(state - LCDState::LYC_STAT);
        } else {
            self.write_stat(state | LCDState::LYC_STAT);
        }
    }

    /// Fetches the current LCD_MODE from PPU_STAT register
    pub fn lcd_mode(&self) -> LCDMode {
        LCDMode::from(self.read(PPU_STAT) & 0b11)
    }

    /// Updates LCD_MODE in PPU_STAT register
    pub fn set_lcd_mode(&mut self, mode: LCDMode) {
        let stat_bits = (self.read_stat().bits & 0xFC) | u8::from(mode);
        self.bus.borrow_mut().write(PPU_STAT, stat_bits);
    }

    fn read_ctrl(&self) -> LCDControl {
        LCDControl::from_bits(self.bus.borrow().read(PPU_LCDC))
            .expect("Got invalid value for LCDControl!")
    }

    fn write_stat(&mut self, stat: LCDState) {
        self.bus.borrow_mut().write(PPU_STAT, stat.bits);
    }

    fn read_stat(&self) -> LCDState {
        LCDState::from_bits(self.bus.borrow().read(PPU_STAT))
            .expect("Got invalid value for LCDState!")
    }
}

impl<'a> AddressSpace for PPU<'a> {
    fn write(&mut self, address: u16, value: u8) {
        self.bus.borrow_mut().write(address, value);
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }
}

/// Defines a Palette to colorize a Pixel
/// Used by bgp, obp0 and obp1 registers
#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Color {
    White = 0x00,
    LightGrey = 0x01,
    DarkGrey = 0x10,
    Black = 0x11,
}

impl convert::From<Color> for u8 {
    fn from(value: Color) -> u8 {
        match value {
            Color::White => 0b00,
            Color::LightGrey => 0b01,
            Color::DarkGrey => 0b10,
            Color::Black => 0b11,
        }
    }
}

impl convert::From<u8> for Color {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Color::White,
            0b01 => Color::LightGrey,
            0b10 => Color::DarkGrey,
            0b11 => Color::Black,
            _ => unimplemented!(),
        }
    }
}
