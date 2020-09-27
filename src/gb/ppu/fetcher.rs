use crate::gb::memory::constants::{PPU_BGP, VRAM_BEGIN};
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::Color;
use crate::gb::timer::Clock;
use crate::gb::AddressSpace;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::convert;

/// Defines a Palette to colorize a Pixel
/// Used by bgp, obp0 and obp1 registers
struct Palette {
    map: [Color; 4],
}

impl Palette {
    pub fn colorize(&self, pixel: Pixel) -> Color {
        self.map[u8::from(pixel) as usize]
    }
}

impl convert::From<u8> for Palette {
    /// Every two bits in the palette data byte represent a colour.
    /// Bits 7-6 maps to colour id 11, bits 5-4 map to colour id 10,
    /// bits 3-2 map to colour id 01 and bits 1-0 map to colour id 00
    fn from(value: u8) -> Self {
        Self {
            map: [
                Color::from(value & 0x3),
                Color::from((value >> 2) & 0x3),
                Color::from((value >> 4) & 0x3),
                Color::from((value >> 6) & 0x3),
            ],
        }
    }
}

/// Represents an non-colorized Pixel
#[derive(Copy, Clone)]
#[repr(u8)]
enum Pixel {
    Zero = 0x00,
    One = 0x01,
    Two = 0x10,
    Three = 0x11,
}

impl convert::From<Pixel> for u8 {
    fn from(value: Pixel) -> u8 {
        match value {
            Pixel::Zero => 0b00,
            Pixel::One => 0b01,
            Pixel::Two => 0b10,
            Pixel::Three => 0b11,
        }
    }
}

impl convert::From<u8> for Pixel {
    fn from(value: u8) -> Self {
        match value {
            0b00 => Pixel::Zero,
            0b01 => Pixel::One,
            0b10 => Pixel::Two,
            0b11 => Pixel::Three,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
pub enum FetcherState {
    ReadTileID,
    ReadTileData0,
    ReadTileData1,
    PushToFIFO,
}

pub struct Fetcher<'a> {
    pub fifo: VecDeque<Color>, // Pixel FIFO that the PPU will read.
    bus: &'a RefCell<MemoryBus>,
    clock: Clock,        // Clock cycle counter for timings.
    state: FetcherState, // Current state of our state machine.
    map_address: u16,    // Start address of BG/Windows map row.
    tile_line: u8,       // Y offset (in pixels) in the tile.

    tile_index: u8, // Index of the tile to read in the current row of the background map.

    tile_id: u8,           // Tile number in the tilemap.
    tile_data: [Pixel; 8], // Pixel data for one row of the fetched tile.
}

impl<'a> Fetcher<'a> {
    pub fn new(bus: &'a RefCell<MemoryBus>) -> Self {
        Self {
            fifo: VecDeque::with_capacity(8),
            bus,
            clock: Clock::new(),
            state: FetcherState::ReadTileID,
            tile_index: 0,
            map_address: 0,
            tile_line: 0,

            tile_id: 0,
            tile_data: [Pixel::Zero; 8],
        }
    }

    /// Start fetching a line of pixels starting from the given tile address in the
    /// background map. Here, tileLine indicates which row of pixels to pick from
    /// each tile we read.
    pub fn start(&mut self, map_address: u16, tile_line: u8) {
        self.tile_index = 0;
        self.map_address = map_address;
        self.tile_line = tile_line;
        self.state = FetcherState::ReadTileID;
        // Clear FIFO between calls, as it may still contain leftover tile data
        // from the very end of the previous scanline.
        self.fifo.clear();
    }

    pub fn step(&mut self) {
        // The Fetcher runs at half the speed of the PPU (every 2 clock cycles).
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
                let address = self.map_address + u16::from(self.tile_index);
                self.tile_id = self.read(address);
                self.state = FetcherState::ReadTileData0
            }
            FetcherState::ReadTileData0 => {
                self.read_tile_line(0);
                self.state = FetcherState::ReadTileData1
            }
            FetcherState::ReadTileData1 => {
                self.read_tile_line(1);
                self.state = FetcherState::PushToFIFO;
            }
            FetcherState::PushToFIFO => {
                if self.fifo.len() <= 8 {
                    let palette = Palette::from(self.read(PPU_BGP));
                    // We stored pixel bits from least significant (rightmost) to most
                    // (leftmost) in the data array, so we must push them in reverse
                    // order.
                    for i in (0..8).rev() {
                        self.fifo.push_back(palette.colorize(self.tile_data[i]));
                    }
                    // Advance to the next tile in the map's row.
                    self.tile_index += 1;
                    self.state = FetcherState::ReadTileID;
                }
            }
        }
    }

    /// ReadTileLine updates the fetcher's internal pixel buffer with tile data
    /// depending on the current state. Each pixel needs 2 bits of information,
    /// which are read in two separate steps.
    pub fn read_tile_line(&mut self, bit_plane: u8) {
        // A tile's graphical data takes 16 bytes (2 bytes per row of 8 pixels).
        // Tile data starts at address 0x8000 so we first compute an offset to
        // find out where the data for the tile we want starts.
        let offset = VRAM_BEGIN + self.tile_id as u16 * 16;

        // Then, from that starting offset, we compute the final address to read
        // by finding out which of the 8-pixel rows of the tile we want.
        let address = offset + u16::from(self.tile_line) * 2;

        // Finally, read the first or second byte of graphical data depending on
        // what state we're in.
        // In the next state, this will be address + 1
        let pixel_data = self.read(address + bit_plane as u16);
        for bit_pos in 0..8 {
            // Store the first bit of pixel color in the pixel data buffer.
            if bit_plane == 0 {
                self.tile_data[bit_pos] = Pixel::from((pixel_data >> bit_pos) & 1);
            } else {
                self.tile_data[bit_pos] = Pixel::from(
                    u8::from(self.tile_data[bit_pos]) | ((pixel_data >> bit_pos) & 1) << 1,
                );
            }
        }
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }
}
