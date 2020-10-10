pub mod cartridge;
pub mod cpu;
pub mod debugger;
pub mod display;
mod instruction;
pub mod interrupt;
pub mod memory;
pub mod ppu;
pub mod timer;

pub const DISPLAY_REFRESH_RATE: u32 = 60; // TODO: exact refresh rate is 59.7

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;
pub const VERTICAL_BLANK_SCAN_LINE_MAX: u8 = 153;

pub const CPU_CLOCK_SPEED: u32 = 1024 * 1024 * 4;

/// This trait defines a common interface to interact with the memory bus.
pub trait AddressSpace {
    fn write(&mut self, address: u16, value: u8);
    fn read(&self, address: u16) -> u8;
}
