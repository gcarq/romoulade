use crate::gb::bus::constants::VRAM_BEGIN;
use crate::gb::ppu::misc::{ColoredPixel, Pixel};
use crate::gb::ppu::registers::{LCDControl, Registers};
use crate::gb::timer::Clock;
use std::collections::VecDeque;

#[derive(Default)]
pub enum FetcherState {
    #[default]
    ReadTileID,
    ReadTileData0,
    ReadTileData1,
    PushToFIFO,
}

/// Implements the PixelPipeline Fetcher outlined in "Ultimate Gamboy Talk",
/// it runs at half the speed of the PPU (every 2 clock cycles).
#[derive(Default)]
pub struct Fetcher {
    pub fifo: VecDeque<ColoredPixel>, // Pixel FIFO that the PPU will read.
    clock: Clock,                     // Clock cycle counter for timings.
    state: FetcherState,              // Current state of our state machine.
    map_address: u16,                 // Start address of BG/Windows map row.
    tile_address: u16,                // Memory address to look for tile data.
    tile_line: u8,                    // Y offset (in pixels) in the tile.
    tile_index: u8, // Index of the tile to read in the current row of the background map.
    tile_id: i16,   // Tile number in the tilemap.
    tile_data: [Pixel; 8], // Pixel data for one row of the fetched tile.
}

impl Fetcher {
    /// Start fetching a line of pixels starting from the given tile address in the
    /// background map. Here, tileLine indicates which row of pixels to pick from
    /// each tile we read.
    pub fn start(&mut self, r: &Registers, map_address: u16, tile_line: u8) {
        self.tile_index = 0;
        self.map_address = map_address;
        self.tile_line = tile_line;
        self.state = FetcherState::ReadTileID;
        self.tile_address = match r.lcd_control.contains(LCDControl::TILE_SEL) {
            true => 0x8000,
            false => 0x8800,
        };

        // Clear FIFO between calls, as it may still contain leftover tile data
        // from the very end of the previous scanline.
        self.fifo.clear();
    }

    pub fn step(&mut self, r: &Registers, vram: &[u8]) {
        self.clock.advance(1);
        if self.clock.ticks() < 2 {
            return;
        }
        self.clock.reset();

        match self.state {
            FetcherState::ReadTileID => {
                // Read the tile's number from the background map. This will be used
                // in the next states to find the address where the tile's actual pixel
                // data is stored in memory.
                let address = self.map_address as usize + self.tile_index as usize;
                // The double casts are very important, because depending on the
                // memory address we read from the values can be u8 or i8!
                self.tile_id = match self.tile_address {
                    0x8000 => vram[address - VRAM_BEGIN as usize] as i16,
                    0x8800 => vram[address - VRAM_BEGIN as usize] as i8 as i16,
                    _ => unimplemented!(),
                };
                self.state = FetcherState::ReadTileData0
            }
            FetcherState::ReadTileData0 => {
                self.read_tile_line(vram, 0);
                self.state = FetcherState::ReadTileData1
            }
            FetcherState::ReadTileData1 => {
                self.read_tile_line(vram, 1);
                self.state = FetcherState::PushToFIFO;
            }
            FetcherState::PushToFIFO => {
                if self.fifo.len() > 8 {
                    return;
                }
                // We stored pixel bits from least significant (rightmost) to most
                // (leftmost) in the data array, so we must push them in reverse
                // order.
                for i in (0..8).rev() {
                    self.fifo
                        .push_back(r.bg_palette.colorize(self.tile_data[i]));
                }
                // Advance to the next tile in the map's row.
                self.tile_index += 1;
                self.state = FetcherState::ReadTileID;
            }
        }
    }

    /// ReadTileLine updates the fetcher's internal pixel buffer with tile data
    /// depending on the current state. Each pixel needs 2 bits of information,
    /// which are read in two separate steps.
    pub fn read_tile_line(&mut self, vram: &[u8], bit_plane: u8) {
        // A tile's graphical data takes 16 bytes (2 bytes per row of 8 pixels).
        let offset = match self.tile_address {
            0x8000 => self.tile_address + self.tile_id as u16 * 16,
            0x8800 => self.tile_address + (self.tile_id + 128) as u16 * 16,
            _ => unimplemented!(),
        };

        // Then, from that starting offset, we compute the final address to read
        // by finding out which of the 8-pixel rows of the tile we want.
        let address = offset as usize + self.tile_line as usize * 2;

        // Finally, read the first or second byte of graphical data depending on
        // what state we're in.
        // In the next state, this will be address + 1
        let pixel_data = vram[address - VRAM_BEGIN as usize + bit_plane as usize];
        for bit_pos in 0..8 {
            // Store the first bit of pixel color in the pixel data buffer.
            let value = (pixel_data >> bit_pos) & 1;
            if bit_plane == 0 {
                self.tile_data[bit_pos] = Pixel::from(value);
            } else {
                self.tile_data[bit_pos] =
                    Pixel::from(u8::from(self.tile_data[bit_pos]) | (value << 1));
            }
        }
    }
}
