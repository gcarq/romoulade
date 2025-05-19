pub mod buffer;
pub mod display;
pub mod misc;
mod registers;
#[cfg(test)]
mod tests;

use crate::gb::bus::InterruptRegister;
use crate::gb::constants::*;
use crate::gb::ppu::misc::{Palette, Pixel, Sprite, SpriteAttributes};
use crate::gb::ppu::registers::{LCDControl, LCDState, PPUMode, Registers};
use crate::gb::utils::bit_at;
use crate::gb::{SubSystem, SCREEN_HEIGHT, SCREEN_WIDTH};
use display::Display;
use std::cmp::Ordering;

/// LCDC is the main LCD Control register.
/// Its bits toggle what elements are displayed on the screen, and how.
const PPU_LCDC: u16 = 0xFF40;

/// LCD Status register.
const PPU_STAT: u16 = 0xFF41;

/// These two registers specify the top-left coordinates of the visible 160×144 pixel
/// area within the 256×256 pixels BG map.
const PPU_SCY: u16 = 0xFF42;
const PPU_SCX: u16 = 0xFF43;

/// LY indicates the current horizontal line, which might be about to be drawn, being drawn,
/// or just been drawn. LY can hold any value from 0 to 153, with values from 144 to 153
/// indicating the `VBlank` period.
const PPU_LY: u16 = 0xFF44;

/// When both `PPU_LYC` and `PPU_LY` values are identical, the “LYC=LY” flag in the STAT register is set
const PPU_LYC: u16 = 0xFF45;

/// Writing to this register requests an OAM DMA transfer, but it’s just a request and the
/// actual DMA transfer starts with a delay.
const PPU_DMA: u16 = 0xFF46;

/// This register assigns gray shades to the color indices of the BG and Window tiles.
const PPU_BGP: u16 = 0xFF47;

/// These registers assigns gray shades to the color indexes of the OBJs that use the corresponding
/// palette. They work exactly like BGP, except that the lower two bits are ignored because color
/// index 0 is transparent for OBJs.
const PPU_OBP0: u16 = 0xFF48;
const PPU_OBP1: u16 = 0xFF49;

/// These two registers specify the on-screen coordinates of the Window’s top-left pixel.
const PPU_WY: u16 = 0xFF4A;
const PPU_WX: u16 = 0xFF4B;

const ACCESS_OAM_CYCLES: isize = 21;
const ACCESS_VRAM_CYCLES: isize = 43;
const HBLANK_CYCLES: isize = 51;
const VBLANK_LINE_CYCLES: isize = 114;

/// The Window is visible (if enabled) when both coordinates are in the ranges WX=0..166, WY=0..143
/// respectively. Values WX=7, WY=0 place the Window at the top left of the screen,
/// completely covering the background.
const WINDOW_X_MAX: u8 = 166;
const WINDOW_Y_MAX: u8 = 143;

/// A frame consists of 154 scanlines; during the first 144, the screen is drawn top to bottom,
/// left to right.
const VBLANK_SCAN_LINE_MAX: u8 = 153;

const TILES_PER_LINE: usize = 32;
const TILE_WIDTH: usize = 8;
const TILE_HEIGHT: usize = 8;

/// A single tile contains 8 lines, each of which is two bytes.
const TILE_BYTES: usize = 2 * 8;

/// The number of bytes a sprite takes in the OAM.
const SPRITE_BYTES: usize = 4;

/// Pixel Processing Unit
pub struct PPU {
    pub r: Registers,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    cycles: isize,
    // This line counter determines what window line is to be rendered on the current scanline.
    // See `<https://gbdev.io/pandocs/Tile_Maps.html#window>`
    wy_internal: Option<u8>,
    display: Option<Display>,
}

impl Clone for PPU {
    /// Clones everything except the `Display`.
    fn clone(&self) -> Self {
        Self {
            r: self.r,
            vram: self.vram,
            oam: self.oam,
            cycles: self.cycles,
            wy_internal: self.wy_internal,
            display: None,
        }
    }
}

impl Default for PPU {
    #[inline]
    fn default() -> Self {
        Self {
            r: Registers::default(),
            vram: [0u8; VRAM_SIZE],
            oam: [0u8; OAM_SIZE],
            cycles: ACCESS_OAM_CYCLES,
            wy_internal: None,
            display: None,
        }
    }
}

impl PPU {
    #[inline]
    pub fn new(display: Option<Display>) -> Self {
        Self {
            display,
            ..Self::default()
        }
    }

    /// Steps the PPU for a given number of cycles.
    pub fn step(&mut self, int_reg: &mut InterruptRegister) {
        if !self.r.lcd_control.contains(LCDControl::LCD_EN) {
            // Screen is off, PPU remains idle.
            return;
        }
        let mode = self.r.lcd_stat.mode();

        self.cycles -= 1;
        if self.cycles == 1 && mode == PPUMode::AccessVRAM {
            // STAT mode=0 interrupt happens one cycle before the actual mode switch!
            if self.r.lcd_stat.contains(LCDState::H_BLANK_INT) {
                int_reg.insert(InterruptRegister::STAT);
            }
        }

        if self.cycles > 0 {
            return;
        }

        match mode {
            // In this state, the PPU would scan the OAM (Objects Attribute Memory)
            // from 0xfe00 to 0xfe9f to mix sprite pixels in the current line later.
            // This always takes 40 ticks.
            PPUMode::AccessOAM => self.switch_mode(PPUMode::AccessVRAM, int_reg),
            PPUMode::AccessVRAM => {
                self.draw_line();
                self.switch_mode(PPUMode::HBlank, int_reg);
            }
            // Nothing much to do here but wait the proper number of clock cycles.
            // A full scanline takes 456 clock cycles to complete. At the end of a
            // scanline, the PPU goes back to the initial OAM Search state.
            // When we reach line 144, we switch to VBlank state instead.
            PPUMode::HBlank => self.handle_hblank(int_reg),
            // Nothing much to do here either. VBlank is when the CPU is supposed to
            // do stuff that takes time. It takes as many cycles as would be needed
            // to keep displaying scanlines up to line 153.
            PPUMode::VBlank => self.handle_vblank(int_reg),
        }
    }

    /// Switches the LCD mode and handles interrupts if needed.
    fn switch_mode(&mut self, mode: PPUMode, int_reg: &mut InterruptRegister) {
        self.r.lcd_stat.set_mode(mode);
        self.cycles += mode.cycles();
        match mode {
            PPUMode::HBlank => {}
            PPUMode::VBlank => {
                self.wy_internal = None;
                int_reg.insert(InterruptRegister::VBLANK);
                if self.r.lcd_stat.contains(LCDState::V_BLANK_INT) {
                    int_reg.insert(InterruptRegister::STAT);
                }
                if self.r.lcd_stat.contains(LCDState::OAM_INT) {
                    int_reg.insert(InterruptRegister::STAT);
                }
            }
            PPUMode::AccessOAM => {
                if self.r.lcd_stat.contains(LCDState::OAM_INT) {
                    int_reg.insert(InterruptRegister::STAT);
                }
            }
            PPUMode::AccessVRAM => {
                if self.r.lcd_control.contains(LCDControl::WIN_EN)
                    && self.wy_internal.is_none()
                    && self.r.wy == self.r.ly
                {
                    self.wy_internal = Some(0);
                }
            }
        }
    }

    /// Handles the coincidence flag, which is set when the LY register matches the LYC register.
    fn handle_coincidence_flag(&mut self, int_reg: &mut InterruptRegister) {
        if self.r.ly != self.r.lyc {
            self.r.lcd_stat.remove(LCDState::LYC_STAT);
            return;
        }
        self.r.lcd_stat.insert(LCDState::LYC_STAT);
        if self.r.lcd_stat.contains(LCDState::LY_INT) {
            int_reg.insert(InterruptRegister::STAT);
        }
    }

    /// Handles the `HBlank` mode, requests an OAM and/or STAT interrupt if needed
    fn handle_hblank(&mut self, int_reg: &mut InterruptRegister) {
        self.r.ly += 1;
        if is_on_screen_y(self.r.ly) {
            self.switch_mode(PPUMode::AccessOAM, int_reg);
        } else {
            if let Some(display) = &mut self.display {
                display.send_frame();
            }
            self.switch_mode(PPUMode::VBlank, int_reg);
        }
        self.handle_coincidence_flag(int_reg);
    }

    /// Handles the `VBlank` mode, requests an VBLANK and/or STAT interrupt if needed
    fn handle_vblank(&mut self, int_reg: &mut InterruptRegister) {
        self.r.ly += 1;
        if self.r.ly > VBLANK_SCAN_LINE_MAX {
            self.r.ly = 0;
            self.switch_mode(PPUMode::AccessOAM, int_reg);
        } else {
            self.cycles += VBLANK_LINE_CYCLES;
        }
        self.handle_coincidence_flag(int_reg);
    }

    /// Draws the background on the current scan line.
    fn draw_background_line(&mut self, bg_prio: &mut [bool; SCREEN_WIDTH as usize]) {
        let tile_map_address = self.r.lcd_control.bg_tile_map_area();
        // Y position of the pixel on the final screen
        let screen_y = self.r.ly;
        // Y position of the pixel in the tile map
        let scrolled_y = screen_y.wrapping_add(self.r.scy);
        for screen_x in 0..SCREEN_WIDTH {
            // X position of the pixel in the tile map
            let scrolled_x = screen_x.wrapping_add(self.r.scx);
            let pixel = self.fetch_pixel_from_tile(tile_map_address, scrolled_x, scrolled_y);
            bg_prio[screen_x as usize] = pixel != Pixel::Zero;
            if let Some(display) = &mut self.display {
                display.write_pixel(screen_x, screen_y, self.r.bg_palette.colorize(pixel));
            }
        }
    }

    /// Draws the window on the current scan line.
    fn draw_window_line(&mut self, bg_prio: &mut [bool; SCREEN_WIDTH as usize]) {
        if self.r.wx >= WINDOW_X_MAX || self.r.wy >= WINDOW_Y_MAX {
            return;
        }

        // Y position of the pixel in the tile map
        let window_y = match self.wy_internal {
            Some(wy) => wy,
            None => return,
        };
        self.wy_internal = Some(window_y + 1);
        // Y position of the pixel on the final screen
        let screen_y = self.r.ly;

        let tile_map_address = self.r.lcd_control.window_tile_map_area();
        let window_x_start = self.r.wx.saturating_sub(7);
        for screen_x in window_x_start..SCREEN_WIDTH {
            // X position of the pixel in the tile map
            let mut window_x = screen_x;
            if window_x >= window_x_start {
                window_x = screen_x - window_x_start;
            }
            let pixel = self.fetch_pixel_from_tile(tile_map_address, window_x, window_y);
            bg_prio[screen_x as usize] = pixel != Pixel::Zero;
            if let Some(display) = &mut self.display {
                display.write_pixel(screen_x, screen_y, self.r.bg_palette.colorize(pixel));
            }
        }
    }

    /// Draws all visible sprites on the current scan line.
    fn draw_sprites(&mut self, bg_prio: &[bool; SCREEN_WIDTH as usize]) {
        let size = match self.r.lcd_control.contains(LCDControl::OBJ_SIZE) {
            true => 16,
            false => 8,
        };

        let mut sprites: Vec<(usize, Sprite)> = self
            .oam
            .chunks_exact(SPRITE_BYTES)
            .filter_map(|chunk| match chunk {
                &[y, x, tile_num, attrs] => {
                    let y = y.wrapping_sub(16);
                    let x = x.wrapping_sub(8);
                    if self.r.ly.wrapping_sub(y) < size {
                        let attrs = SpriteAttributes::from_bits_truncate(attrs);
                        Some(Sprite::new(y, x, tile_num, attrs))
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .take(10)
            .enumerate()
            .collect();

        sprites.sort_by(|&(a_index, a), &(b_index, b)| {
            match a.x.cmp(&b.x) {
                // If X coordinates are the same, use OAM index as priority (low index => draw last)
                Ordering::Equal => a_index.cmp(&b_index).reverse(),
                // Use X coordinate as priority (low X => draw last)
                other => other.reverse(),
            }
        });

        for (_, sprite) in sprites {
            self.draw_sprite(sprite, size, bg_prio);
        }
    }

    /// Draws the given `Sprite` on the current scan line with respect to the background priority.
    fn draw_sprite(&mut self, sprite: Sprite, size: u8, bg_prio: &[bool; SCREEN_WIDTH as usize]) {
        let palette = match sprite.attrs.contains(SpriteAttributes::DMG_PALETTE) {
            true => &self.r.obj_palette1,
            false => &self.r.obj_palette0,
        };

        // Ignore bit 0 of tile index for 8x16 sprites
        let mut tile_num = match size == 16 {
            true => sprite.tile_index & 0b1111_1110,
            false => sprite.tile_index,
        } as usize;

        let screen_y = self.r.ly;
        let mut line = match sprite.attrs.contains(SpriteAttributes::Y_FLIP) {
            true => size - screen_y.wrapping_sub(sprite.y) - 1,
            false => screen_y.wrapping_sub(sprite.y),
        };

        if line >= 8 {
            tile_num += 1;
            line -= 8;
        }
        line *= 2;
        let tile_mask = tile_num << 4;
        let data1 = self.vram[(tile_mask | line as usize) & 0x1FFF];
        let data2 = self.vram[(tile_mask | (line as usize + 1)) & 0x1FFF];

        for x in (0..8).rev() {
            let bit = match sprite.attrs.contains(SpriteAttributes::X_FLIP) {
                true => 7 - x,
                false => x,
            };
            let pixel = pixel_from_line(data1, data2, bit);
            let screen_x = sprite.x.wrapping_add(7 - x);
            if !is_on_screen_x(screen_x) || pixel == Pixel::Zero {
                continue;
            }
            if !sprite.attrs.contains(SpriteAttributes::PRIORITY) || !bg_prio[screen_x as usize] {
                if let Some(display) = &mut self.display {
                    display.write_pixel(screen_x, screen_y, palette.colorize(pixel));
                }
            }
        }
    }

    /// Draws the current scan line to the display.
    fn draw_line(&mut self) {
        // This slice is used to determine the priority of the background and window over sprites.
        let mut bg_prio = [false; SCREEN_WIDTH as usize];
        if self.r.lcd_control.contains(LCDControl::BG_EN) {
            self.draw_background_line(&mut bg_prio);
        }
        if self.r.lcd_control.contains(LCDControl::WIN_EN) {
            self.draw_window_line(&mut bg_prio);
        }
        if self.r.lcd_control.contains(LCDControl::OBJ_EN) {
            self.draw_sprites(&bg_prio);
        }
    }

    /// Returns a `Pixel` from the tile data at the given coordinates.
    fn fetch_pixel_from_tile(&self, tile_map_address: u16, x: u8, y: u8) -> Pixel {
        let col = x as usize / TILE_WIDTH;
        let row = y as usize / TILE_HEIGHT;

        let tile_id_address = tile_map_address + (row * TILES_PER_LINE + col) as u16;
        let tile_id = self.vram[(tile_id_address - VRAM_BEGIN) as usize];
        let data_address = if self.r.lcd_control.contains(LCDControl::TILE_SEL) {
            0x8000 + tile_id as usize * TILE_BYTES
        } else {
            0x8800 + ((tile_id as i8 as i16) + 128) as usize * TILE_BYTES
        };

        let line = (y as usize % TILE_HEIGHT) * 2;
        let byte1 = self.vram[data_address + line - VRAM_BEGIN as usize];
        let byte2 = self.vram[data_address + line + 1 - VRAM_BEGIN as usize];

        let bit = (x % TILE_WIDTH as u8).wrapping_sub(7).wrapping_mul(0xFF);
        pixel_from_line(byte1, byte2, bit)
    }

    /// Writes to the LCD control register (`PPU_LCDC`).
    fn write_control(&mut self, value: u8) {
        let cur = self.r.lcd_control;
        let new = LCDControl::from_bits_truncate(value);
        if new.contains(LCDControl::LCD_EN) && !cur.contains(LCDControl::LCD_EN) {
            // LCD is being turned on
            self.r.lcd_stat.set_mode(PPUMode::HBlank);
            self.cycles = PPUMode::AccessOAM.cycles();
            self.r.lcd_stat.insert(LCDState::LYC_STAT);
        } else if !new.contains(LCDControl::LCD_EN) && cur.contains(LCDControl::LCD_EN) {
            // LCD is being turned off, reset the LY register to 0.
            if self.r.lcd_stat.mode() != PPUMode::VBlank {
                eprintln!("FIXME: LCD off, but not in VBlank");
            }
            self.r.ly = 0;
            self.wy_internal = None;
        }
        self.r.lcd_control = new;
    }

    /// Writes to the LCD status register (`PPU_STAT`),
    /// the first two bits are only writable by the PPU.
    #[inline]
    fn write_stat(&mut self, value: u8) {
        let cur = self.r.lcd_stat;
        let mut new = LCDState::from_bits_truncate(value);
        new.set(LCDState::PPU_MODE1, cur.contains(LCDState::PPU_MODE1));
        new.set(LCDState::PPU_MODE2, cur.contains(LCDState::PPU_MODE2));
        self.r.lcd_stat = new;
    }

    /// Reads from the OAM (Object Attribute Memory) region.
    fn read_oam(&self, address: u16) -> u8 {
        match self.r.lcd_stat.mode() {
            // In Pixel Transfer mode or during OAM Search, the OAM is not accessible.
            PPUMode::AccessOAM | PPUMode::AccessVRAM => UNDEFINED_READ,
            PPUMode::HBlank | PPUMode::VBlank => self.oam[(address - OAM_BEGIN) as usize],
        }
    }

    /// Writes to the OAM (Object Attribute Memory) region.
    fn write_oam(&mut self, address: u16, value: u8) {
        match self.r.lcd_stat.mode() {
            // OAM is not accessible during Pixel Transfer or OAM Search.
            PPUMode::AccessOAM | PPUMode::AccessVRAM => {}
            PPUMode::HBlank | PPUMode::VBlank => self.oam[(address - OAM_BEGIN) as usize] = value,
        }
    }

    /// Reads from the VRAM region.
    fn read_vram(&self, address: u16) -> u8 {
        match self.r.lcd_stat.mode() {
            // In Pixel Transfer mode, the VRAM is not accessible.
            PPUMode::AccessVRAM => UNDEFINED_READ,
            _ => self.vram[usize::from(address - VRAM_BEGIN)],
        }
    }

    /// Writes to the VRAM region.
    fn write_vram(&mut self, address: u16, value: u8) {
        if self.r.lcd_stat.mode() != PPUMode::AccessVRAM {
            // VRAM is not accessible during Pixel Transfer mode.
            self.vram[usize::from(address - VRAM_BEGIN)] = value;
        }
    }
}

impl SubSystem for PPU {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_BEGIN..=VRAM_END => self.write_vram(address, value),
            OAM_BEGIN..=OAM_END => self.write_oam(address, value),
            PPU_LCDC => self.write_control(value),
            PPU_STAT => self.write_stat(value),
            PPU_SCY => self.r.scy = value,
            PPU_SCX => self.r.scx = value,
            PPU_LY => {} // LY is read-only
            PPU_LYC => self.r.lyc = value,
            PPU_DMA => self.r.oam_dma.request(value),
            PPU_BGP => self.r.bg_palette = Palette::from(value),
            PPU_OBP0 => self.r.obj_palette0 = Palette::from(value),
            PPU_OBP1 => self.r.obj_palette1 = Palette::from(value),
            PPU_WY => self.r.wy = value,
            PPU_WX => self.r.wx = value,
            _ => panic!("Attempt to write to unmapped PPU register: {address:#06x}"),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            VRAM_BEGIN..=VRAM_END => self.read_vram(address),
            OAM_BEGIN..=OAM_END => self.read_oam(address),
            PPU_LCDC => self.r.lcd_control.bits(),
            PPU_STAT => self.r.lcd_stat.bits() | 0b1000_0000, // Undocumented bit should be 1
            PPU_SCY => self.r.scy,
            PPU_SCX => self.r.scx,
            PPU_LY => self.r.ly,
            PPU_LYC => self.r.lyc,
            PPU_DMA => self.r.oam_dma.source,
            PPU_BGP => u8::from(self.r.bg_palette),
            PPU_OBP0 => u8::from(self.r.obj_palette0),
            PPU_OBP1 => u8::from(self.r.obj_palette1),
            PPU_WY => self.r.wy,
            PPU_WX => self.r.wx,
            _ => panic!("Attempt to read from unmapped audio register: {address:#06x}"),
        }
    }
}

/// Checks whether the given x coordinate is within the `SCREEN_WIDTH`.
#[inline(always)]
const fn is_on_screen_x(x: u8) -> bool {
    x < SCREEN_WIDTH
}

/// Checks whether the given y coordinate is within the `SCREEN_HEIGHT`.
#[inline(always)]
const fn is_on_screen_y(y: u8) -> bool {
    y < SCREEN_HEIGHT
}

/// Creates a pixel from tile data at the given bit position.
#[inline]
fn pixel_from_line(byte1: u8, byte2: u8, bit: u8) -> Pixel {
    Pixel::from(((bit_at(byte2, bit) as u8) << 1) | bit_at(byte1, bit) as u8)
}
