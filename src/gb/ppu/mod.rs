mod fetcher;
pub mod misc;

use crate::gb::bus::Bus;
use crate::gb::bus::constants::*;
use crate::gb::display::Display;
use crate::gb::ppu::fetcher::Fetcher;
use crate::gb::timer::Clock;
use crate::gb::{AddressSpace, SCREEN_HEIGHT, SCREEN_WIDTH, VERTICAL_BLANK_SCAN_LINE_MAX};

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

impl From<LCDMode> for u8 {
    fn from(value: LCDMode) -> u8 {
        match value {
            LCDMode::HBlank => 0b00,
            LCDMode::VBlank => 0b01,
            LCDMode::OAMSearch => 0b10,
            LCDMode::PixelTransfer => 0b11,
        }
    }
}

impl From<u8> for LCDMode {
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
    fetcher: Fetcher,
    display: &'a mut Display,
    x: u8,
}

impl<'a> PPU<'a> {
    pub fn new(display: &'a mut Display) -> Self {
        Self {
            clock: Clock::new(),
            fetcher: Fetcher::new(),
            display,
            x: 0,
        }
    }

    pub fn step(&mut self, bus: &mut Bus, cycles: u16) {
        if !self.read_ctrl(bus).contains(LCDControl::LCD_EN) {
            self.set_lcd_mode(bus, LCDMode::VBlank);
            // Screen is off, PPU remains idle.
            return;
        }

        self.clock.advance(cycles);

        let cur_mode = self.lcd_mode(bus);
        let (mode, irq) = match cur_mode {
            // In this state, the PPU would scan the OAM (Objects Attribute Memory)
            // from 0xfe00 to 0xfe9f to mix sprite pixels in the current line later.
            // This always takes 40 ticks.
            LCDMode::OAMSearch if self.clock.ticks() >= 40 => self.handle_oam_search(bus),
            LCDMode::PixelTransfer => self.handle_pixel_transfer(bus),
            // Nothing much to do here but wait the proper number of clock cycles.
            // A full scanline takes 456 clock cycles to complete. At the end of a
            // scanline, the PPU goes back to the initial OAM Search state.
            // When we reach line 144, we switch to VBlank state instead.
            LCDMode::HBlank if self.clock.ticks() >= 456 => self.handle_hblank(bus),
            // Nothing much to do here either. VBlank is when the CPU is supposed to
            // do stuff that takes time. It takes as many cycles as would be needed
            // to keep displaying scanlines up to line 153.
            LCDMode::VBlank if self.clock.ticks() >= 456 => self.handle_vblank(bus),
            // No mode change occurred
            mode => (mode, false),
        };

        // handle pending interrupts if mode changed
        if mode != cur_mode && irq {
            bus.interrupt_flag.lcd = true;
        }
        self.set_lcd_mode(bus, mode);

        self.handle_coincidence_flag(bus);
    }

    /// Checks the coincidence flag and requests an interrupt if required.
    fn handle_coincidence_flag(&mut self, bus: &mut Bus) {
        let state = self.read_stat(bus);
        if bus.read(PPU_LY) == bus.read(PPU_LYC) {
            if state.contains(LCDState::LY_INT) {
                bus.interrupt_flag.lcd = true;
            }
            self.write_stat(bus, state - LCDState::LYC_STAT);
        } else {
            self.write_stat(bus, state | LCDState::LYC_STAT);
        }
    }

    /// Handles the OAMSearch mode.
    /// Returns a tuple with the new LCDMode and whether a interrupt has been requested.
    fn handle_oam_search(&mut self, bus: &mut Bus) -> (LCDMode, bool) {
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
        let y = bus.read(PPU_SCY).wrapping_add(bus.read(PPU_LY));
        let x = bus.read(PPU_SCX).wrapping_add(self.x);

        let bg_address = match self.read_ctrl(bus).contains(LCDControl::BG_MAP) {
            true => 0x9C00,
            false => 0x9800,
        };

        let tile_row = u16::from(y / 8) * 32;
        let tile_column = u16::from(x / 8);
        let tile_map_row_addr = bg_address + tile_row + tile_column;

        let tile_line = y % 8;
        self.fetcher.start(bus, tile_map_row_addr, tile_line);
        (LCDMode::PixelTransfer, false)
    }

    /// Handles the HBlank mode.
    /// Returns a tuple with the new LCDMode and whether an interrupt has been requested.
    fn handle_hblank(&mut self, bus: &mut Bus) -> (LCDMode, bool) {
        self.clock.reset();
        bus.write(PPU_LY, bus.read(PPU_LY).wrapping_add(1));

        let state = self.read_stat(bus);
        if bus.read(PPU_LY) == SCREEN_HEIGHT {
            self.display.render_screen();
            return (LCDMode::VBlank, state.contains(LCDState::V_BLANK_INT));
        }
        (LCDMode::OAMSearch, state.contains(LCDState::OAM_INT))
    }

    /// Handles the VBlank mode.
    /// Returns a tuple with the new LCDMode and whether a interrupt has been requested.
    fn handle_vblank(&mut self, bus: &mut Bus) -> (LCDMode, bool) {
        self.clock.reset();
        bus.write(PPU_LY, bus.read(PPU_LY).wrapping_add(1));

        let state = self.read_stat(bus);
        if bus.read(PPU_LY) == VERTICAL_BLANK_SCAN_LINE_MAX {
            bus.write(PPU_LY, 0);
            return (LCDMode::OAMSearch, state.contains(LCDState::OAM_INT));
        }
        (LCDMode::VBlank, false)
    }

    /// Handles the PixelTransfer mode.
    /// Returns a tuple with the new LCDMode and whether a interrupt has been requested.
    fn handle_pixel_transfer(&mut self, bus: &mut Bus) -> (LCDMode, bool) {
        // Fetch pixel data into our pixel FIFO.
        self.fetcher.step(bus);
        // Stop here if the FIFO isn't holding at least 8 pixels. This will
        // be used to mix in sprite data when we implement these. It also
        // guarantees the FIFO will always have data to Pop() later.
        if self.fetcher.fifo.len() <= 8 {
            return (LCDMode::PixelTransfer, false);
        }
        // Put a pixel from the FIFO on screen if we have any.
        if let Some(color) = self.fetcher.fifo.pop_front() {
            self.display.write_pixel(self.x, bus.read(PPU_LY), color);
        }
        // Check when the scanline is complete (160 pixels).
        self.x = self.x.wrapping_add(1);
        match self.x == SCREEN_WIDTH {
            true => (
                LCDMode::HBlank,
                self.read_stat(bus).contains(LCDState::H_BLANK_INT),
            ),
            false => (LCDMode::PixelTransfer, false),
        }
    }

    /// Fetches the current LCD_MODE from PPU_STAT register
    fn lcd_mode(&self, bus: &mut Bus) -> LCDMode {
        LCDMode::from(bus.read(PPU_STAT) & 0b11)
    }

    /// Updates LCD_MODE in PPU_STAT register
    pub fn set_lcd_mode(&mut self, bus: &mut Bus, mode: LCDMode) {
        let stat_bits = (self.read_stat(bus).bits() & 0xFC) | u8::from(mode);
        bus.write(PPU_STAT, stat_bits);
    }

    fn read_ctrl(&self, bus: &mut Bus) -> LCDControl {
        LCDControl::from_bits(bus.read(PPU_LCDC)).expect("Got invalid value for LCDControl!")
    }

    fn write_stat(&mut self, bus: &mut Bus, stat: LCDState) {
        bus.write(PPU_STAT, stat.bits());
    }

    fn read_stat(&self, bus: &mut Bus) -> LCDState {
        LCDState::from_bits(bus.read(PPU_STAT)).expect("Got invalid value for LCDState!")
    }
}
