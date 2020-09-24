pub mod cpu;
pub mod display;
mod instruction;
pub mod interrupt;
pub mod memory;
pub mod ppu;
pub mod registers;
pub mod timings;

pub const DISPLAY_REFRESH_RATE: u32 = 60; // TODO: exact refresh rate is 59.7

pub const SCREEN_WIDTH: u8 = 160;
pub const SCREEN_HEIGHT: u8 = 144;

/// This trait defines a common interface to interact with the memory bus.
trait AddressSpace {
    fn write(&mut self, address: u16, value: u8);
    fn read(&self, address: u16) -> u8;
}
