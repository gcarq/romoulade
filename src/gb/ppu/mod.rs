pub mod buffer;
pub mod display;
mod fetcher;
pub mod misc;
mod registers;
#[cfg(test)]
mod tests;

use crate::gb::constants::*;
use crate::gb::ppu::fetcher::Fetcher;
use crate::gb::ppu::misc::Palette;
use crate::gb::ppu::registers::{LCDControl, LCDMode, LCDState, Registers};
use crate::gb::timer::Clock;
use crate::gb::{AddressSpace, SCREEN_HEIGHT, SCREEN_WIDTH, VERTICAL_BLANK_SCAN_LINE_MAX};
use display::Display;

/// Pixel Processing Unit
pub struct PPU {
    r: Registers,
    vram: [u8; VRAM_SIZE],
    clock: Clock,
    fetcher: Fetcher,
    display: Display,
    x: u8,
}

impl PPU {
    pub fn new(display: Display) -> Self {
        Self {
            r: Registers::default(),
            vram: [0u8; VRAM_SIZE],
            clock: Clock::default(),
            fetcher: Fetcher::default(),
            display,
            x: 0,
        }
    }

    /// Steps the PPU and returns true if an LCD interrupt has been requested.
    pub fn step(&mut self, cycles: u16) -> bool {
        if !self.r.lcd_control.contains(LCDControl::LCD_EN) {
            // Screen is off, PPU remains idle.
            return false;
        }

        self.clock.advance(cycles);

        let cur_mode = self.r.lcd_stat.get_lcd_mode();
        let (mode, irq) = match cur_mode {
            // In this state, the PPU would scan the OAM (Objects Attribute Memory)
            // from 0xfe00 to 0xfe9f to mix sprite pixels in the current line later.
            // This always takes 40 ticks.
            LCDMode::OAMSearch if self.clock.ticks() >= 40 => (self.handle_oam_search(), false),
            LCDMode::PixelTransfer => self.handle_pixel_transfer(),
            // Nothing much to do here but wait the proper number of clock cycles.
            // A full scanline takes 456 clock cycles to complete. At the end of a
            // scanline, the PPU goes back to the initial OAM Search state.
            // When we reach line 144, we switch to VBlank state instead.
            LCDMode::HBlank if self.clock.ticks() >= 456 => self.handle_hblank(),
            // Nothing much to do here either. VBlank is when the CPU is supposed to
            // do stuff that takes time. It takes as many cycles as would be needed
            // to keep displaying scanlines up to line 153.
            LCDMode::VBlank if self.clock.ticks() >= 456 => self.handle_vblank(),
            // No mode change occurred
            mode => (mode, false),
        };

        let mut interrupt = false;
        // request an interrupts if mode changed
        if mode != cur_mode && irq {
            interrupt = true;
        }
        self.r.lcd_stat.set_lcd_mode(mode);
        if self.handle_coincidence_flag() {
            interrupt = true;
        }
        interrupt
    }

    /// Checks the coincidence flag and returns true if an interrupt should be requested.
    fn handle_coincidence_flag(&mut self) -> bool {
        if self.r.ly != self.r.lyc {
            self.r.lcd_stat |= LCDState::LYC_STAT;
            return false;
        }
        self.r.lcd_stat -= LCDState::LYC_STAT;
        self.r.lcd_stat.contains(LCDState::LY_INT)
    }

    /// Handles the OAMSearch mode.
    fn handle_oam_search(&mut self) -> LCDMode {
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
        let y = self.r.scy.wrapping_add(self.r.ly);
        let x = self.r.scx.wrapping_add(self.x);

        let bg_address = match self.r.lcd_control.contains(LCDControl::BG_MAP) {
            true => 0x9C00,
            false => 0x9800,
        };

        let tile_row = u16::from(y / 8) * 32;
        let tile_column = u16::from(x / 8);
        let tile_map_row_addr = bg_address + tile_row + tile_column;

        let tile_line = y % 8;
        self.fetcher.start(&self.r, tile_map_row_addr, tile_line);
        LCDMode::PixelTransfer
    }

    /// Handles the HBlank mode.
    /// Returns a tuple with the new LCDMode and whether an interrupt should be requested.
    fn handle_hblank(&mut self) -> (LCDMode, bool) {
        self.clock.reset();
        self.r.ly = self.r.ly.wrapping_add(1);

        let state = self.r.lcd_stat;
        if self.r.ly == SCREEN_HEIGHT {
            self.display.send_frame();
            (LCDMode::VBlank, state.contains(LCDState::V_BLANK_INT))
        } else {
            (LCDMode::OAMSearch, state.contains(LCDState::OAM_INT))
        }
    }

    /// Handles the VBlank mode.
    /// Returns a tuple with the new LCDMode and whether an interrupt should be requested.
    fn handle_vblank(&mut self) -> (LCDMode, bool) {
        self.clock.reset();
        self.r.ly = self.r.ly.wrapping_add(1);
        if self.r.ly == VERTICAL_BLANK_SCAN_LINE_MAX {
            self.r.ly = 0;
            (
                LCDMode::OAMSearch,
                self.r.lcd_stat.contains(LCDState::OAM_INT),
            )
        } else {
            (LCDMode::VBlank, false)
        }
    }

    /// Handles the PixelTransfer mode.
    /// Returns a tuple with the new LCDMode and whether an interrupt should be requested.
    fn handle_pixel_transfer(&mut self) -> (LCDMode, bool) {
        // Fetch pixel data into our pixel FIFO.
        self.fetcher.step(&self.r, &self.vram);
        // Stop here if the FIFO isn't holding at least 8 pixels. This will
        // be used to mix in sprite data when we implement these. It also
        // guarantees the FIFO will always have data to Pop() later.
        if self.fetcher.fifo.len() <= 8 {
            return (LCDMode::PixelTransfer, false);
        }
        // Put a pixel from the FIFO on screen if we have any.
        if let Some(color) = self.fetcher.fifo.pop_front() {
            self.display.write_pixel(self.x, self.r.ly, color);
        }
        // Check when the scanline is complete (160 pixels).
        self.x = self.x.wrapping_add(1);
        if self.x == SCREEN_WIDTH {
            let irq = self.r.lcd_stat.contains(LCDState::H_BLANK_INT);
            (LCDMode::HBlank, irq)
        } else {
            (LCDMode::PixelTransfer, false)
        }
    }
}

impl AddressSpace for PPU {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[usize::from(address - VRAM_BEGIN)] = value,
            PPU_LCDC => self.r.lcd_control = LCDControl::from_bits_truncate(value),
            PPU_STAT => self.r.lcd_stat = LCDState::from_bits_truncate(value),
            PPU_SCY => self.r.scy = value,
            PPU_SCX => self.r.scx = value,
            PPU_LY => self.r.ly = value,
            PPU_LYC => self.r.lyc = value,
            // TODO: implement me PPU_DMA => self.fetcher.dma_transfer(value),
            PPU_BGP => self.r.bg_palette = Palette::from(value),
            PPU_OBP0 => self.r.obj_palette0 = Palette::from(value),
            PPU_OBP1 => self.r.obj_palette1 = Palette::from(value),
            PPU_WY => self.r.wy = value,
            PPU_WX => self.r.wx = value,
            _ => panic!("Attempt to write to unmapped PPU register: 0x{:X}", address),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            VRAM_BEGIN..=VRAM_END => self.vram[usize::from(address - VRAM_BEGIN)],
            PPU_LCDC => self.r.lcd_control.bits(),
            PPU_STAT => self.r.lcd_stat.bits() | 0b1000_0000, // Undocumented bit should be 1
            PPU_SCY => self.r.scy,
            PPU_SCX => self.r.scx,
            PPU_LY => self.r.ly,
            PPU_LYC => self.r.lyc,
            PPU_DMA => 0, // TODO: implement DMA
            PPU_BGP => u8::from(self.r.bg_palette),
            PPU_OBP0 => u8::from(self.r.obj_palette0),
            PPU_OBP1 => u8::from(self.r.obj_palette1),
            PPU_WY => self.r.wy,
            PPU_WX => self.r.wx,
            _ => panic!(
                "Attempt to read from unmapped audio register: 0x{:X}",
                address
            ),
        }
    }
}
