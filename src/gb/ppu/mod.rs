pub mod buffer;
pub mod display;
pub mod misc;
mod registers;
#[cfg(test)]
mod tests;

use crate::gb::constants::*;
use crate::gb::interrupt::InterruptRegister;
use crate::gb::ppu::misc::{Palette, Pixel, Sprite, SpriteAttributes};
use crate::gb::ppu::registers::{LCDControl, LCDMode, LCDState, Registers};
use crate::gb::timer::Clock;
use crate::gb::utils::bit_at;
use crate::gb::{AddressSpace, SCREEN_HEIGHT, SCREEN_WIDTH, VERTICAL_BLANK_SCAN_LINE_MAX};
use display::Display;
use std::cmp::Ordering;

/// Pixel Processing Unit
pub struct PPU {
    r: Registers,
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    clock: Clock,
    display: Display,
    window_line_counter: u8,
}

impl PPU {
    pub fn new(display: Display) -> Self {
        Self {
            r: Registers::default(),
            vram: [0u8; VRAM_SIZE],
            oam: [0u8; OAM_SIZE],
            clock: Clock::default(),
            display,
            window_line_counter: 0,
        }
    }

    /// Steps the PPU for a given number of cycles.
    pub fn step(&mut self, int_reg: &mut InterruptRegister, cycles: u16) {
        if !self.r.lcd_control.contains(LCDControl::LCD_EN) {
            // Screen is off, PPU remains idle.
            return;
        }

        self.clock.advance(cycles);

        let cur_mode = self.r.lcd_stat.get_lcd_mode();

        match cur_mode {
            // In this state, the PPU would scan the OAM (Objects Attribute Memory)
            // from 0xfe00 to 0xfe9f to mix sprite pixels in the current line later.
            // This always takes 40 ticks.
            LCDMode::AccessOAM if self.clock.ticks() >= 40 => {
                self.switch_mode(LCDMode::AccessVRAM, int_reg)
            }
            LCDMode::AccessVRAM => {
                self.draw_line();
                self.switch_mode(LCDMode::HBlank, int_reg)
            }
            // Nothing much to do here but wait the proper number of clock cycles.
            // A full scanline takes 456 clock cycles to complete. At the end of a
            // scanline, the PPU goes back to the initial OAM Search state.
            // When we reach line 144, we switch to VBlank state instead.
            LCDMode::HBlank if self.clock.ticks() >= 456 => self.handle_hblank(int_reg),
            // Nothing much to do here either. VBlank is when the CPU is supposed to
            // do stuff that takes time. It takes as many cycles as would be needed
            // to keep displaying scanlines up to line 153.
            LCDMode::VBlank if self.clock.ticks() >= 456 => self.handle_vblank(int_reg),
            _ => {}
        };
    }

    /// Switches the LCD mode and handles interrupts if needed.
    fn switch_mode(&mut self, mode: LCDMode, int_reg: &mut InterruptRegister) {
        self.r.lcd_stat.set_lcd_mode(mode);
        match mode {
            LCDMode::AccessOAM => {
                if self.r.lcd_stat.contains(LCDState::OAM_INT) {
                    int_reg.insert(InterruptRegister::STAT);
                }
            }
            LCDMode::VBlank => {
                int_reg.insert(InterruptRegister::VBLANK);
                if self.r.lcd_stat.contains(LCDState::V_BLANK_INT) {
                    int_reg.insert(InterruptRegister::STAT);
                }
            }
            _ => {}
        }
    }

    /// Handles the coincidence flag, which is set when the LY register matches the LYC register.
    fn handle_coincidence_flag(&mut self, int_reg: &mut InterruptRegister) {
        if self.r.ly != self.r.lyc {
            self.r.lcd_stat.remove(LCDState::LYC_STAT);
            return;
        }
        self.r.lcd_stat.insert(LCDState::LYC_STAT);
        int_reg.insert(InterruptRegister::STAT);
    }

    /// Handles the HBlank mode, requests an OAM and/or STAT interrupt if needed
    fn handle_hblank(&mut self, int_reg: &mut InterruptRegister) {
        self.clock.reset();
        self.r.ly += 1;
        if self.r.ly >= SCREEN_HEIGHT {
            self.display.send_frame();
            self.switch_mode(LCDMode::VBlank, int_reg);
        } else {
            self.switch_mode(LCDMode::AccessOAM, int_reg);
        }
        self.handle_coincidence_flag(int_reg);
    }

    /// Handles the VBlank mode, requests an VBLANK and/or STAT interrupt if needed
    fn handle_vblank(&mut self, int_reg: &mut InterruptRegister) {
        self.clock.reset();
        self.r.ly += 1;
        if self.r.ly > VERTICAL_BLANK_SCAN_LINE_MAX {
            self.r.ly = 0;
            self.window_line_counter = 0;
            self.switch_mode(LCDMode::AccessOAM, int_reg);
        } else {
            // TODO: use constant for VBlank cycle duration
            self.clock.advance(114);
        }
        self.handle_coincidence_flag(int_reg);
    }

    /// Draws the background on the current scan line.
    fn draw_background(&mut self, bg_prio: &mut [bool; SCREEN_WIDTH as usize]) {
        let map_mask = self.r.lcd_control.bg_tile_map_area();

        let y = self.r.ly.wrapping_add(self.r.scy);
        let row = (y / 8) as usize;
        for i in 0..SCREEN_WIDTH {
            let x = i.wrapping_add(self.r.scx);
            let col = (x / 8) as usize;

            let tile_num = self.vram[((row * 32 + col) | map_mask as usize) & 0x1fff] as usize;
            let tile_num = if self.r.lcd_control.contains(LCDControl::TILE_SEL) {
                tile_num
            } else {
                128 + ((tile_num as i8 as i16) + 128) as usize
            };

            let line = ((y % 8) * 2) as usize;
            let tile_mask = tile_num << 4;
            let data1 = self.vram[(tile_mask | line) & 0x1fff];
            let data2 = self.vram[(tile_mask | (line + 1)) & 0x1fff];

            let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xff) as usize;

            let color_value =
                ((bit_at(data2, bit as u8) as u8) << 1) | bit_at(data1, bit as u8) as u8;
            let pixel = Pixel::from(color_value);
            let color = self.r.bg_palette.colorize(pixel);
            bg_prio[i as usize] = pixel != Pixel::Zero;
            self.display.write_pixel(i, self.r.ly, color);
        }
    }

    /// Draws the window on the current scan line.
    fn draw_window(&mut self, bg_prio: &mut [bool; SCREEN_WIDTH as usize]) {
        let map_mask = self.r.lcd_control.window_tile_map_area();
        let window_x = self.r.wx.wrapping_sub(7);

        //let y = self.r.ly - self.r.wy;
        let y = self.window_line_counter;
        let row = (y / 8) as usize;

        for i in window_x..SCREEN_WIDTH {
            let mut x = i.wrapping_add(self.r.scx);
            if x >= window_x {
                x = i - window_x;
            }
            let col = (x / 8) as usize;

            let tile_num = self.vram[((row * 32 + col) | map_mask as usize) & 0x1fff] as usize;
            let tile_num = if self.r.lcd_control.contains(LCDControl::TILE_SEL) {
                tile_num
            } else {
                128 + ((tile_num as i8 as i16) + 128) as usize
            };

            let line = ((y % 8) * 2) as usize;
            let tile_mask = tile_num << 4;
            let data1 = self.vram[(tile_mask | line) & 0x1fff];
            let data2 = self.vram[(tile_mask | (line + 1)) & 0x1fff];

            let bit = (x % 8).wrapping_sub(7).wrapping_mul(0xff) as usize;
            let color_value =
                ((bit_at(data2, bit as u8) as u8) << 1) | bit_at(data1, bit as u8) as u8;
            let pixel = Pixel::from(color_value);
            let color = self.r.bg_palette.colorize(pixel);
            bg_prio[i as usize] = pixel != Pixel::Zero;
            self.display.write_pixel(i, self.r.ly, color);
        }
    }

    /// Draws the sprites on the current scan line.
    fn draw_sprites(&mut self, bg_prio: &[bool; SCREEN_WIDTH as usize]) {
        let size = if self.r.lcd_control.contains(LCDControl::OBJ_SIZE) {
            16
        } else {
            8
        };

        let current_line = self.r.ly;

        let mut sprites_to_draw: Vec<(usize, Sprite)> = self
            .oam
            .chunks(4)
            .filter_map(|chunk| match chunk {
                &[y, x, tile_num, flags] => {
                    let y = y.wrapping_sub(16);
                    let x = x.wrapping_sub(8);
                    let flags = SpriteAttributes::from_bits_truncate(flags);
                    if current_line.wrapping_sub(y) < size {
                        let sprite = Sprite::new(y, x, tile_num, flags);
                        Some(sprite)
                    } else {
                        None
                    }
                }
                _ => None,
            })
            .take(10)
            .enumerate()
            .collect();

        sprites_to_draw.sort_by(|&(a_index, a), &(b_index, b)| {
            match a.x.cmp(&b.x) {
                // If X coordinates are the same, use OAM index as priority (low index => draw last)
                Ordering::Equal => a_index.cmp(&b_index).reverse(),
                // Use X coordinate as priority (low X => draw last)
                other => other.reverse(),
            }
        });

        for (_, sprite) in sprites_to_draw {
            let palette = if sprite.attributes.contains(SpriteAttributes::DMG_PALETTE) {
                &self.r.obj_palette1
            } else {
                &self.r.obj_palette0
            };

            // Ignore bit 0 of tile index for 8x16 sprites
            let mut tile_num = if size == 16 {
                sprite.tile_index & 0xFE
            } else {
                sprite.tile_index
            } as usize;

            let mut line = if sprite.attributes.contains(SpriteAttributes::Y_FLIP) {
                size - current_line.wrapping_sub(sprite.y) - 1
            } else {
                current_line.wrapping_sub(sprite.y)
            };
            if line >= 8 {
                tile_num += 1;
                line -= 8;
            }
            line *= 2;
            let tile_mask = tile_num << 4;
            let data1 = self.vram[(tile_mask | line as usize) & 0x1fff];
            let data2 = self.vram[(tile_mask | (line + 1) as usize) & 0x1fff];

            for x in (0..8).rev() {
                let bit = if sprite.attributes.contains(SpriteAttributes::X_FLIP) {
                    7 - x
                } else {
                    x
                };

                let color_value = ((bit_at(data2, bit) as u8) << 1) | bit_at(data1, bit) as u8;
                let pixel = Pixel::from(color_value);
                let color = palette.colorize(pixel);
                let target_x = sprite.x.wrapping_add(7 - x);
                if target_x < SCREEN_WIDTH
                    && pixel != Pixel::Zero
                    && (!sprite.attributes.contains(SpriteAttributes::PRIORITY)
                        || !bg_prio[target_x as usize])
                {
                    self.display.write_pixel(target_x, current_line, color);
                }
            }
        }
    }

    /// Draws the current scan line to the display.
    fn draw_line(&mut self) {
        let mut bg_prio = [false; SCREEN_WIDTH as usize];

        // Render background
        if self.r.lcd_control.contains(LCDControl::BG_EN) {
            self.draw_background(&mut bg_prio);
        }

        if self.r.lcd_control.contains(LCDControl::WIN_EN) && self.r.ly >= self.r.wy {
            if self.r.ly == self.r.wy {
                self.window_line_counter = 0;
            }
            self.draw_window(&mut bg_prio);
            self.window_line_counter += 1;
        }

        // Render sprites
        if self.r.lcd_control.contains(LCDControl::OBJ_EN) {
            self.draw_sprites(&bg_prio);
        }
    }
}

impl AddressSpace for PPU {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            VRAM_BEGIN..=VRAM_END => {
                // VRAM is not accessible during Pixel Transfer mode.
                if self.r.lcd_stat.get_lcd_mode() != LCDMode::AccessVRAM {
                    self.vram[usize::from(address - VRAM_BEGIN)] = value;
                }
            }
            OAM_BEGIN..=OAM_END => {
                // OAM is not accessible during Pixel Transfer or OAM Search.
                match self.r.lcd_stat.get_lcd_mode() {
                    LCDMode::AccessOAM | LCDMode::AccessVRAM => {}
                    _ => self.oam[(address - OAM_BEGIN) as usize] = value,
                }
            }
            PPU_LCDC => self.r.lcd_control = LCDControl::from_bits_truncate(value),
            PPU_STAT => self.r.lcd_stat = LCDState::from_bits_truncate(value),
            PPU_SCY => self.r.scy = value,
            PPU_SCX => self.r.scx = value,
            PPU_LY => {} // LY is read-only
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
            VRAM_BEGIN..=VRAM_END => {
                match self.r.lcd_stat.get_lcd_mode() {
                    // In Pixel Transfer mode, the VRAM is not accessible.
                    // Return 0xFF to indicate that the value is not available.
                    LCDMode::AccessVRAM => 0xFF,
                    _ => self.vram[usize::from(address - VRAM_BEGIN)],
                }
            }
            OAM_BEGIN..=OAM_END => {
                match self.r.lcd_stat.get_lcd_mode() {
                    // In Pixel Transfer mode or during OAM Search, the OAM is not accessible.
                    // Return 0xFF to indicate that the value is not available.
                    LCDMode::AccessOAM | LCDMode::AccessVRAM => 0xFF,
                    _ => self.oam[(address - OAM_BEGIN) as usize],
                }
            }
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
