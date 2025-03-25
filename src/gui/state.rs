use crate::gb::cartridge::Cartridge;
use crate::gui::EmulatorChannel;
use std::thread::JoinHandle;

/// Holds the state of the running GB emulator.
pub struct EmulatorState {
    thread: JoinHandle<()>,
    pub channel: EmulatorChannel,
    pub cartridge: Cartridge,
}

impl EmulatorState {
    pub fn new(thread: JoinHandle<()>, channel: EmulatorChannel, cartridge: Cartridge) -> Self {
        Self {
            thread,
            channel,
            cartridge,
        }
    }
}
