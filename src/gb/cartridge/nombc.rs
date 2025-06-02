use crate::gb::cartridge::CartridgeConfig;
use crate::gb::cartridge::controller::{BankController, SaveError};
use crate::gb::constants::*;
use std::sync::Arc;

/// Small games of not more than 32 KiB ROM do not require a MBC chip for ROM banking.
/// The ROM is directly mapped to memory at 0x0000 - 0x7FFF.
/// Optionally up to 8 KiB of RAM could be connected at 0xA000 - 0xBFFF,
/// using a discrete logic decoder in place of a full MBC chip.
#[derive(Clone)]
pub struct NoMBC {
    config: CartridgeConfig,
    rom: Arc<[u8]>,
    ram: Vec<u8>,
}

impl NoMBC {
    #[inline]
    pub fn new(config: CartridgeConfig, rom: Arc<[u8]>) -> Self {
        Self {
            ram: vec![0; config.ram_size()],
            rom,
            config,
        }
    }
}

impl BankController for NoMBC {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_LOW_BANK_END => self.rom[address as usize],
            ROM_HIGH_BANK_BEGIN..=ROM_HIGH_BANK_END => self.rom[address as usize],
            CRAM_BANK_BEGIN..=CRAM_BANK_END => match self.ram.is_empty() {
                true => UNDEFINED_READ,
                false => self.ram[(address - CRAM_BANK_BEGIN) as usize],
            },
            _ => panic!("NoMBC: Invalid address for read: {address:#06x}"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        if let CRAM_BANK_BEGIN..=CRAM_BANK_END = address {
            if !self.ram.is_empty() {
                self.ram[(address - CRAM_BANK_BEGIN) as usize] = value;
            }
        }
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        debug_assert_eq!(
            ram.len(),
            self.ram.len(),
            "Given RAM size does not match the expected size",
        );
        self.ram = ram;
    }

    fn save_ram(&self) -> Result<Arc<[u8]>, SaveError> {
        if self.ram.is_empty() || !self.config.controller.has_battery() {
            return Err(SaveError::NoSaveSupport);
        }
        Ok(self.ram.clone().into())
    }
}
