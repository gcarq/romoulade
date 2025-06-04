use crate::gb::cartridge::controller::BankController;
use crate::gb::cartridge::{CartridgeConfig, RAM_BANK_SIZE, ROM_BANK_SIZE, SaveError, bank_mask};
use crate::gb::constants::*;
use std::sync::Arc;

/// Mostly the same as for MBC1. Writing $0A will enable reading and writing to external RAM.
/// Writing $00 will disable it. Actual MBCs actually enable RAM when writing any value whose
/// bottom 4 bits equal $A (so $0A, $1A, and so on), and disable it when writing anything else.
/// Relying on this behavior is not recommended for compatibility reasons.
const RAM_ENABLE_BEGIN: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;

/// The 8 least significant bits of the ROM bank number go here.
/// Writing 0 will indeed give bank 0 on MBC5, unlike other MBCs.
const ROM_BANK_LOW_BITS_BEGIN: u16 = 0x2000;
const ROM_BANK_LOW_BITS_END: u16 = 0x2FFF;

/// The 9th bit of the ROM bank number goes here.
const ROM_BANK_HIGH_BIT_BEGIN: u16 = 0x3000;
const ROM_BANK_HIGH_BIT_END: u16 = 0x3FFF;

/// As for the MBC1s RAM Banking Mode, writing a value in the range $00-$0F maps the corresponding
/// external RAM bank (if any) into the memory area at A000-BFFF.
const RAM_BANK_NUMBER_BEGIN: u16 = 0x4000;
const RAM_BANK_NUMBER_END: u16 = 0x5FFF;

/// It can map up to 64 MBits (8 MiB) of ROM.
/// MBC5 (Memory Bank Controller 5) is the 5th generation MBC.
#[derive(Clone)]
pub struct MBC5 {
    config: CartridgeConfig,
    rom: Arc<[u8]>,
    ram: Vec<u8>,
    savable_ram: Option<Arc<[u8]>>, // Latest sane copy of the RAM for saving
    rom_bank_number: u16,           // Mapped ROM bank number for 0x4000 - 0x7FFF
    ram_bank_number: u8,            // Mapped RAM bank number for 0xA000 - 0xBFFF
    has_ram_access: bool,
}

impl MBC5 {
    pub fn new(config: CartridgeConfig, rom: Arc<[u8]>) -> Self {
        let ram = vec![0; config.ram_size()];
        Self {
            savable_ram: config.is_savable().then_some(ram.clone().into()),
            rom_bank_number: 1,
            ram_bank_number: 0,
            has_ram_access: false,
            config,
            ram,
            rom,
        }
    }
}

impl BankController for MBC5 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_LOW_BANK_END => {
                self.rom[(address - ROM_LOW_BANK_BEGIN) as usize]
            }
            ROM_HIGH_BANK_BEGIN..=ROM_HIGH_BANK_END => {
                let offset = self.rom_bank_number as usize * ROM_BANK_SIZE;
                self.rom[(offset + (address - ROM_HIGH_BANK_BEGIN) as usize) % self.rom.len()]
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if self.has_ram_access && !self.ram.is_empty() {
                    let offset = (address - CRAM_BANK_BEGIN) as usize;
                    let address = self.ram_bank_number as usize * RAM_BANK_SIZE + offset;
                    self.ram[address % self.ram.len()]
                } else {
                    UNDEFINED_READ
                }
            }
            _ => panic!("MBC5: Invalid address for read: {address:#06x}"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // Any value with 0x0A in the lower 4 bits enables ram, any other value disables it.
            RAM_ENABLE_BEGIN..=RAM_ENABLE_END => {
                self.has_ram_access = value & 0b1111 == 0b1010;
                if !self.has_ram_access {
                    self.savable_ram = Some(self.ram.clone().into());
                }
            }
            // Sets the lower 8 bits of the ROM bank number.
            ROM_BANK_LOW_BITS_BEGIN..=ROM_BANK_LOW_BITS_END => {
                self.rom_bank_number = (self.rom_bank_number & 0xFF00) | (value as u16 & 0x00FF);
                self.rom_bank_number &= bank_mask(self.config.rom_banks) as u16;
            }
            // Sets the upper 1 bit of the ROM bank number.
            ROM_BANK_HIGH_BIT_BEGIN..=ROM_BANK_HIGH_BIT_END => {
                self.rom_bank_number |= u16::from(value & 0b1) << 8;
                self.rom_bank_number &= bank_mask(self.config.rom_banks) as u16;
            }
            RAM_BANK_NUMBER_BEGIN..=RAM_BANK_NUMBER_END => {
                self.ram_bank_number = value & 0b0000_1111;
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if self.has_ram_access && !self.ram.is_empty() {
                    let offset = self.ram_bank_number as usize * RAM_BANK_SIZE;
                    self.ram[offset + (address - CRAM_BANK_BEGIN) as usize] = value;
                }
            }
            _ => {}
        }
    }

    fn load_ram(&mut self, ram: Vec<u8>) {
        debug_assert_eq!(ram.len(), self.ram.len());
        self.ram = ram;
    }

    fn save_ram(&self) -> Result<Arc<[u8]>, SaveError> {
        if !self.config.is_savable() {
            return Err(SaveError::NoSaveSupport);
        }
        Ok(self.savable_ram.clone().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gb::cartridge::ControllerType;

    #[test]
    fn test_ram_state() {
        let config =
            CartridgeConfig::new(ControllerType::MBC5 { battery: true }, 0x03, 0x02).unwrap();
        let mut controller = MBC5::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        let addr = CRAM_BANK_BEGIN + 0x10;
        controller.write(addr, 0x42);
        assert_eq!(controller.read(addr), 0xFF, "RAM should be disabled");

        controller.write(RAM_ENABLE_BEGIN, 0x0A);
        assert_eq!(
            controller.read(addr),
            0x00,
            "First write should have been ignored"
        );

        controller.write(addr, 0x42);
        assert_eq!(controller.read(addr), 0x42, "RAM should be enabled");

        controller.write(RAM_ENABLE_BEGIN, 0xFF);
        assert_eq!(controller.read(addr), 0xFF, "RAM should be disabled");
    }

    #[test]
    fn test_rom_bank_bits() {
        let config =
            CartridgeConfig::new(ControllerType::MBC5 { battery: true }, 0x08, 0x02).unwrap();
        let mut ctrl = MBC5::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        ctrl.write(ROM_BANK_LOW_BITS_BEGIN, 0x01);
        assert_eq!(ctrl.rom_bank_number, 0x01);

        ctrl.write(ROM_BANK_HIGH_BIT_BEGIN, 0x01);
        assert_eq!(ctrl.rom_bank_number, 0x101);

        ctrl.write(ROM_BANK_HIGH_BIT_BEGIN, 0xFF);
        assert_eq!(
            ctrl.rom_bank_number, 0x101,
            "Only first 9 bits should be used"
        );
    }
    #[test]
    fn test_ram_banking() {
        let config =
            CartridgeConfig::new(ControllerType::MBC5 { battery: true }, 0x00, 0x03).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC5::new(
            config,
            (0u8..64).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 1);
        ctrl.write(RAM_ENABLE_BEGIN, 0x0A);

        // Assert the banks are set correctly and the memory is initialized
        for i in 0u8..4 {
            ctrl.write(RAM_BANK_NUMBER_BEGIN, i);
            assert_eq!(ctrl.ram_bank_number, i);
            assert_eq!(
                ctrl.read(CRAM_BANK_BEGIN),
                0,
                "RAM should be initialized to 0"
            );
            ctrl.write(CRAM_BANK_BEGIN, i + 1);
            assert_eq!(
                ctrl.read(CRAM_BANK_BEGIN),
                i + 1,
                "RAM should return {}",
                i + 1
            );
        }

        // Assert the written values are correct when switching banks again
        for i in 0u8..4 {
            ctrl.write(RAM_BANK_NUMBER_BEGIN, i);
            assert_eq!(
                ctrl.read(CRAM_BANK_BEGIN),
                i + 1,
                "RAM should return {}",
                i + 1
            );
        }
    }
}
