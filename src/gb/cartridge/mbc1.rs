use crate::gb::cartridge::controller::{BankController, SaveError};
use crate::gb::cartridge::{CartridgeConfig, RAM_BANK_SIZE, ROM_BANK_SIZE, bank_mask};
use crate::gb::constants::*;
use std::sync::Arc;

/// Before external RAM can be read or written,
/// it must be enabled by writing 0x0A to anywhere in this address space.
/// Any value with 0x0A in the lower 4 bits enables the RAM attached to the MBC,
/// and any other value disables the RAM.
const RAM_ENABLE_BEGIN: u16 = 0x0000;
const RAM_ENABLE_END: u16 = 0x1FFF;

/// This 5-bit register (range $01-$1F) selects the ROM bank number for the 4000–7FFF region.
/// Higher bits are discarded — writing $E1 (binary 11100001) to this register would select bank $01.
/// If this register is set to $00, it behaves as if it is set to $01.
const ROM_BANK_NUMBER_BEGIN: u16 = 0x2000;
const ROM_BANK_NUMBER_END: u16 = 0x3FFF;

/// This 2-bit register range can be used to select a RAM Bank in range from $00–$03 (32 KiB ram
/// carts only), or to specify the upper two bits (bits 5-6) of the ROM Bank number (1 MiB ROM or
/// larger carts only). If neither ROM nor RAM is large enough, setting this register does nothing.
const RAM_BANK_NUMBER_BEGIN: u16 = 0x4000;
const RAM_BANK_NUMBER_END: u16 = 0x5FFF;

/// This 1-bit register selects between the two MBC1 banking modes,
/// controlling the behaviour of the secondary 2-bit banking register (above).
/// If the cart is not large enough to use the 2-bit register (≤ 8 KiB RAM and ≤ 512 KiB ROM)
/// this mode select has no observable effect. The program may freely switch between the two
/// modes at any time.
const BANKING_MODE_SELECT_BEGIN: u16 = 0x6000;
const BANKING_MODE_SELECT_END: u16 = 0x7FFF;

/// This 1-bit register selects between the two MBC1 banking modes,
/// controlling the behaviour of the secondary 2-bit banking register (0x4000 - 0x5FFF).
/// If the cart is not large enough to use the 2-bit register (≤ 8 KiB RAM and ≤ 512 KiB ROM)
/// this mode select has no observable effect. The program may freely switch between the two modes
/// at any time.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum BankingMode {
    Simple,
    Advanced,
}

/// In its default configuration, MBC1 supports up to 512 KiB ROM with up to 32 KiB of banked RAM.
/// Some cartridges wire the MBC differently, where the 2-bit RAM banking register is wired as an
/// extension of the ROM banking register (instead of to RAM) in order to support up to 2 MiB ROM,
/// at the cost of only supporting a fixed 8 KiB of cartridge RAM. All MBC1 cartridges with 1 MiB
/// of ROM or more use this alternate wiring. Also see the note on MBC1M multi-game compilation
/// carts below.
#[derive(Clone)]
pub struct MBC1 {
    config: CartridgeConfig,
    rom: Arc<[u8]>,
    ram: Vec<u8>,
    low_rom_bank_offset: usize, // current selected ROM bank offset for 0x0000 - 0x3FFF
    high_rom_bank_offset: usize, // current selected ROM bank offset for 0x4000 - 0x7FFF
    ram_bank_offset: usize,     // current selected RAM bank offset for 0xA000 - 0xBFFF
    has_ram_access: bool,
    bank_low_bits: u8,  // lower 5 bits of the ROM bank number
    bank_high_bits: u8, // RAM bank number or upper 2 bits of the ROM bank number
    banking_mode: BankingMode,
}

impl MBC1 {
    pub fn new(config: CartridgeConfig, rom: Arc<[u8]>) -> Self {
        Self {
            ram: vec![0; config.ram_size()],
            low_rom_bank_offset: 0,
            high_rom_bank_offset: ROM_HIGH_BANK_BEGIN as usize,
            ram_bank_offset: 0,
            has_ram_access: false,
            banking_mode: BankingMode::Simple,
            bank_low_bits: 0b0000_0001,
            bank_high_bits: 0b0000_0000,
            rom,
            config,
        }
    }

    /// Updates the ROM banks offsets on the current banking mode.
    /// If the ROM is larger than 32 banks, we also use the upper 2 bits for the bank number,
    /// otherwise we only use the lower 5 bits.
    const fn update_rom_offsets(&mut self) {
        // If all ROM banks can be referenced with 5 bits
        // we don't consider the advanced banking mode
        if self.config.rom_banks < 32 {
            self.low_rom_bank_offset = 0;
            self.bank_low_bits &= bank_mask(self.config.rom_banks) as u8;
            self.high_rom_bank_offset = ROM_BANK_SIZE * self.bank_low_bits as usize;
            return;
        }

        let low_bank_nr = match self.banking_mode {
            BankingMode::Simple => 0,
            BankingMode::Advanced => self.bank_high_bits << 5,
        } as usize;
        self.low_rom_bank_offset = ROM_BANK_SIZE * low_bank_nr;

        let high_bank_nr = (self.bank_high_bits << 5) as usize | self.bank_low_bits as usize;
        self.high_rom_bank_offset = ROM_BANK_SIZE * high_bank_nr;
    }

    /// Updates the RAM bank offset based on the current banking mode.
    /// Only cartridges with 4 banks support RAM banking.
    fn update_ram_offset(&mut self) {
        // RAM banking is only available with cartridges with 4 banks
        if self.config.ram_banks == 4 && self.banking_mode == BankingMode::Advanced {
            self.ram_bank_offset = RAM_BANK_SIZE * self.bank_high_bits as usize;
        } else {
            self.ram_bank_offset = 0;
        }
    }
}

impl BankController for MBC1 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_LOW_BANK_END => {
                self.rom[self.low_rom_bank_offset + (address - ROM_LOW_BANK_BEGIN) as usize]
            }
            ROM_HIGH_BANK_BEGIN..=ROM_HIGH_BANK_END => {
                self.rom[self.high_rom_bank_offset + (address - ROM_HIGH_BANK_BEGIN) as usize]
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if self.has_ram_access && !self.ram.is_empty() {
                    self.ram[self.ram_bank_offset + (address - CRAM_BANK_BEGIN) as usize]
                } else {
                    UNDEFINED_READ
                }
            }
            _ => panic!("MBC1: Invalid address for read: {address:#06x}"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            // Any value with 0x0A in the lower 4 bits enables ram, any other value disables it.
            RAM_ENABLE_BEGIN..=RAM_ENABLE_END => {
                self.has_ram_access = value & 0b1111 == 0b1010;
            }
            // Sets the lower 5 bits of the ROM bank number.
            ROM_BANK_NUMBER_BEGIN..=ROM_BANK_NUMBER_END => {
                self.bank_low_bits = match value & 0b0001_1111 {
                    0 => 1,
                    n => n,
                };
                self.update_rom_offsets();
            }
            // Sets the upper 2 bits of the ROM bank number or the RAM bank number,
            // depending on the current banking mode.
            RAM_BANK_NUMBER_BEGIN..=RAM_BANK_NUMBER_END => {
                self.bank_high_bits = value & 0b11;
                self.update_rom_offsets();
                self.update_ram_offset();
            }
            // Selects the banking mode.
            BANKING_MODE_SELECT_BEGIN..=BANKING_MODE_SELECT_END => {
                match value & 0b1 {
                    0b0 => self.banking_mode = BankingMode::Simple,
                    0b1 => self.banking_mode = BankingMode::Advanced,
                    _ => unreachable!(),
                }
                self.update_rom_offsets();
                self.update_ram_offset();
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if self.has_ram_access && !self.ram.is_empty() {
                    self.ram[self.ram_bank_offset + (address - CRAM_BANK_BEGIN) as usize] = value;
                }
            }
            _ => {}
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
        if self.has_ram_access {
            return Err(SaveError::RAMLocked);
        }
        Ok(self.ram.clone().into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gb::cartridge::ControllerType;

    #[test]
    fn test_ram_state() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x03, 0x02).unwrap();
        let mut controller = MBC1::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

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
    fn test_rom_bank_lower_bits() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x03, 0x02).unwrap();
        let mut ctrl = MBC1::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0x01);
        assert_eq!(ctrl.bank_low_bits, 0x01);
        assert_eq!(ctrl.high_rom_bank_offset, ROM_BANK_SIZE);

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0x00);
        assert_eq!(ctrl.bank_low_bits, 0x01);
        assert_eq!(
            ctrl.high_rom_bank_offset, ROM_BANK_SIZE,
            "0x00 should be treated as 0x01"
        );

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0x02);
        assert_eq!(ctrl.bank_low_bits, 0x02);
        assert_eq!(ctrl.high_rom_bank_offset, ROM_BANK_SIZE * 2);

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0xFF);
        assert_eq!(
            ctrl.high_rom_bank_offset,
            ROM_BANK_SIZE * 0x1F,
            "Only first 5 bits should be used"
        );
    }

    #[test]
    fn test_rom_bank_upper_bits() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x03, 0x03).unwrap();
        let mut ctrl = MBC1::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 0b11);
        assert_eq!(ctrl.bank_high_bits, 0b11);
        assert_eq!(
            ctrl.ram_bank_offset, 0,
            "RAM bank should be 0, because of banking mode simple"
        );
        assert_eq!(ctrl.high_rom_bank_offset, ROM_BANK_SIZE);

        // Switch to advanced mode
        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b1);

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 0b11);
        assert_eq!(ctrl.bank_high_bits, 0b11);
        assert_eq!(ctrl.ram_bank_offset, RAM_BANK_SIZE * 3);
        assert_eq!(
            ctrl.high_rom_bank_offset, ROM_BANK_SIZE,
            "High ROM bank should not change, because the it only holds 16 banks"
        );

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 0xFF);
        assert_eq!(
            ctrl.bank_high_bits, 0b11,
            "Only first 2 bits should be used"
        );
    }

    #[test]
    fn test_change_banking_mode() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x03, 0x02).unwrap();
        let mut ctrl = MBC1::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b1);
        assert_eq!(ctrl.banking_mode, BankingMode::Advanced);

        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b1111);
        assert_eq!(ctrl.banking_mode, BankingMode::Advanced);

        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b1000);
        assert_eq!(ctrl.banking_mode, BankingMode::Simple);

        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b0000);
        assert_eq!(ctrl.banking_mode, BankingMode::Simple);
    }

    #[test]
    fn test_rom_banking_simple() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x05, 0x02).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC1::new(
            config,
            (0u8..64).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );

        assert_eq!(ctrl.read(ROM_LOW_BANK_BEGIN), 0);
        assert_eq!(ctrl.read(ROM_HIGH_BANK_BEGIN), 1);

        for i in 2..32 {
            ctrl.write(ROM_BANK_NUMBER_BEGIN, i);
            assert_eq!(
                ctrl.read(ROM_HIGH_BANK_BEGIN),
                i,
                "ROM bank {i} should be selected"
            );
        }

        // When trying to select bank 32, it should wrap around to 0 in simple mode
        // because 5 bits are not enough to address it.
        for _ in 32..34 {
            ctrl.write(ROM_BANK_NUMBER_BEGIN, 32);
            assert_eq!(
                ctrl.read(ROM_HIGH_BANK_BEGIN),
                1,
                "ROM bank 1 should be selected",
            );
        }

        for i in 34..64 {
            ctrl.write(ROM_BANK_NUMBER_BEGIN, i);
            assert_eq!(
                ctrl.read(ROM_HIGH_BANK_BEGIN),
                i - 32,
                "ROM bank {} should be selected",
                i - 32
            );
        }
    }

    #[test]
    fn test_rom_banking_masked() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x02, 0x02).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC1::new(
            config,
            (0u8..16).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );
        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0b1111_1001);
        assert_eq!(
            ctrl.bank_low_bits, 0b0000_1001,
            "The upper 4 bits should be masked"
        );
        assert_eq!(
            ctrl.read(ROM_HIGH_BANK_BEGIN),
            9,
            "ROM bank 9 should be selected"
        );
    }

    #[test]
    fn test_rom_banking_advanced() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x05, 0x02).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC1::new(
            config,
            (0u8..64).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );

        // Switch to advanced mode
        ctrl.write(BANKING_MODE_SELECT_BEGIN, 0b1);

        assert_eq!(
            ctrl.read(ROM_LOW_BANK_BEGIN),
            0,
            "Bank 0 should be selected"
        );
        assert_eq!(
            ctrl.read(ROM_HIGH_BANK_BEGIN),
            1,
            "Bank 1 should be selected"
        );

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 1);

        // Any attempt to address ROM Bank 32 will select bank 33 instead
        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0);
        assert_eq!(
            ctrl.read(ROM_HIGH_BANK_BEGIN),
            33,
            "ROM bank 33 should be selected"
        );

        for i in 33..64 {
            ctrl.write(ROM_BANK_NUMBER_BEGIN, i);
            assert_eq!(
                ctrl.read(ROM_HIGH_BANK_BEGIN),
                i,
                "ROM bank {i} should be selected"
            );
        }
    }

    #[test]
    fn test_ram_banking() {
        let config =
            CartridgeConfig::new(ControllerType::MBC1 { battery: true }, 0x00, 0x03).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC1::new(
            config,
            (0u8..64).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );

        // Switch to advanced mode and enable RAM
        ctrl.write(BANKING_MODE_SELECT_BEGIN, 1);
        ctrl.write(RAM_ENABLE_BEGIN, 0x0A);

        // Assert the banks are set correctly and the memory is initialized
        for i in 0u8..4 {
            ctrl.write(RAM_BANK_NUMBER_BEGIN, i);
            assert_eq!(ctrl.ram_bank_offset, RAM_BANK_SIZE * i as usize);
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
