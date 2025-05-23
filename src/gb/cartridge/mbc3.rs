use crate::gb::cartridge::controller::BankController;
use crate::gb::cartridge::{CartridgeConfig, RAM_BANK_SIZE, ROM_BANK_SIZE, bank_mask};
use crate::gb::constants::*;
use std::sync::Arc;

/// Mostly the same as for MBC1, a value of $0A will enable reading and writing to external RAM
/// and to the RTC Registers! A value of $00 will disable either.
const RAM_RTC_ENABLE_BEGIN: u16 = 0x0000;
const RAM_RTC_ENABLE_END: u16 = 0x1FFF;

/// Same as for MBC1, except that the whole 7 bits of the ROM Bank Number are written directly to
/// this address. As for the MBC1, writing a value of 0x00 will select Bank 0x01 instead.
/// All other values 0x01-0x7F select the corresponding ROM Banks.
const ROM_BANK_NUMBER_BEGIN: u16 = 0x2000;
const ROM_BANK_NUMBER_END: u16 = 0x3FFF;

/// Controls what is mapped into memory at 0xA000 - 0xBFFF.
/// 0x00 - 0x07: RAM bank.
/// 0x08 - 0x0C: RTC register.
const RAM_BANK_NUMBER_BEGIN: u16 = 0x4000;
const RAM_BANK_NUMBER_END: u16 = 0x5FFF;

/// When writing 0x00, and then 0x01 to this register, the current time becomes latched into the RTC
/// registers. The latched data will not change until it becomes latched again,
/// by repeating the 0x00->0x01 procedure.
const LATCH_CLOCK_DATA_BEGIN: u16 = 0x6000;
const LATCH_CLOCK_DATA_END: u16 = 0x7FFF;

bitflags! {
    /// Represents the RTC Day High Register.
    #[derive(Clone, Default)]
    struct DayHighRegister: u8 {
        const DAY_COUNTER_MSB = 0b0000_0001;
        const HALT = 0b0100_0000;
        const Day_COUNTER_CARRY = 0b1000_0000;
    }
}

/// The RTC registers are used to keep track of play time.
/// See https://gbdev.io/pandocs/MBC3.html#clock-counter-registers
#[derive(Default, Clone)]
struct RTCRegisters {
    seconds: u8,
    minutes: u8,
    hours: u8,
    day_low: u8,
    day_high: DayHighRegister,
}

/// Determines the current selected RAM bank or RTC register.
#[derive(Clone, PartialEq, Debug)]
enum RAMBankSelection {
    RAMBank(u8),
    Seconds,
    Minutes,
    Hours,
    DayLow,
    DayHigh,
}

#[derive(Clone, Default, PartialEq)]
enum RTCLatchState {
    #[default]
    Undefined,
    Pending,
    Latched,
}

/// Beside for the ability to access up to 2MB ROM (128 banks), and 32KB RAM (4 banks),
/// the MBC3 also includes a built-in Real Time Clock (RTC).
/// TODO: Implement RTC handling
#[derive(Clone)]
pub struct MBC3 {
    config: CartridgeConfig,
    rom: Arc<[u8]>,
    ram: Vec<u8>,
    rtc: RTCRegisters,
    rom_bank_number: u8,      // Mapped ROM bank number for 0x4000 - 0x7FFF
    rtc_latch: RTCLatchState, // RTC Latch for 0x6000 - 0x7FFF
    ram_bank_selection: RAMBankSelection, // Mapped RAM bank number or RTC for 0xA000 - 0xBFFF
    has_ram_rtc_access: bool,
}

impl MBC3 {
    pub fn new(config: CartridgeConfig, rom: Arc<[u8]>) -> Self {
        Self {
            ram: vec![0; config.ram_size()],
            rtc: RTCRegisters::default(),
            rom_bank_number: 1,
            rtc_latch: RTCLatchState::default(),
            ram_bank_selection: RAMBankSelection::RAMBank(0),
            has_ram_rtc_access: false,
            rom,
            config,
        }
    }
}

impl BankController for MBC3 {
    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_LOW_BANK_END => {
                self.rom[(address - ROM_LOW_BANK_BEGIN) as usize]
            }
            ROM_HIGH_BANK_BEGIN..=ROM_HIGH_BANK_END => {
                let offset = self.rom_bank_number as usize * ROM_BANK_SIZE;
                self.rom[offset + (address - ROM_HIGH_BANK_BEGIN) as usize]
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if !self.has_ram_rtc_access {
                    return UNDEFINED_READ;
                }

                match self.ram_bank_selection {
                    RAMBankSelection::RAMBank(bank) => {
                        if self.ram.is_empty() {
                            return UNDEFINED_READ;
                        }
                        let offset = bank as usize * RAM_BANK_SIZE;
                        self.ram[offset + (address - CRAM_BANK_BEGIN) as usize]
                    }
                    RAMBankSelection::Seconds => self.rtc.seconds,
                    RAMBankSelection::Minutes => self.rtc.minutes,
                    RAMBankSelection::Hours => self.rtc.hours,
                    RAMBankSelection::DayLow => self.rtc.day_low,
                    RAMBankSelection::DayHigh => self.rtc.day_high.bits(),
                }
            }
            _ => panic!("MBC3: Invalid address for read: {address:#06x}"),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        match address {
            RAM_RTC_ENABLE_BEGIN..=RAM_RTC_ENABLE_END => {
                self.has_ram_rtc_access = value & 0b1111 == 0b1010;
            }
            ROM_BANK_NUMBER_BEGIN..=ROM_BANK_NUMBER_END => {
                self.rom_bank_number = if value == 0 { 1 } else { value & 0b0111_1111 };
                self.rom_bank_number &= bank_mask(self.config.rom_banks) as u8;
            }
            RAM_BANK_NUMBER_BEGIN..=RAM_BANK_NUMBER_END => {
                self.ram_bank_selection = match value {
                    0x00..=0x07 => RAMBankSelection::RAMBank(value),
                    0x08 => RAMBankSelection::Seconds,
                    0x09 => RAMBankSelection::Minutes,
                    0x0A => RAMBankSelection::Hours,
                    0x0B => RAMBankSelection::DayLow,
                    0x0C => RAMBankSelection::DayHigh,
                    _ => return,
                };
            }
            LATCH_CLOCK_DATA_BEGIN..=LATCH_CLOCK_DATA_END => {
                self.rtc_latch = match value {
                    0x00 => RTCLatchState::Pending,
                    0x01 if self.rtc_latch == RTCLatchState::Pending => {
                        // TODO: implement RTC handling
                        match self.ram_bank_selection {
                            RAMBankSelection::Seconds => {}
                            RAMBankSelection::Minutes => {}
                            RAMBankSelection::Hours => {}
                            RAMBankSelection::DayLow => {}
                            RAMBankSelection::DayHigh => {}
                            _ => {}
                        }
                        RTCLatchState::Latched
                    }
                    _ => RTCLatchState::Undefined,
                };
            }
            CRAM_BANK_BEGIN..=CRAM_BANK_END => {
                if !self.has_ram_rtc_access {
                    return;
                }
                match self.ram_bank_selection {
                    RAMBankSelection::RAMBank(bank) if !self.ram.is_empty() => {
                        let offset = bank as usize * RAM_BANK_SIZE;
                        self.ram[offset + (address - CRAM_BANK_BEGIN) as usize] = value;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gb::cartridge::ControllerType;

    #[test]
    fn test_ram_state() {
        let config = CartridgeConfig::new(ControllerType::MBC3, 0x03, 0x02).unwrap();
        let mut controller = MBC3::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        let addr = CRAM_BANK_BEGIN + 0x10;
        controller.write(addr, 0x42);
        assert_eq!(controller.read(addr), 0xFF, "RAM should be disabled");

        controller.write(RAM_RTC_ENABLE_BEGIN, 0x0A);
        assert_eq!(
            controller.read(addr),
            0x00,
            "First write should have been ignored"
        );

        controller.write(addr, 0x42);
        assert_eq!(controller.read(addr), 0x42, "RAM should be enabled");

        controller.write(RAM_RTC_ENABLE_BEGIN, 0xFF);
        assert_eq!(controller.read(addr), 0xFF, "RAM should be disabled");
    }

    #[test]
    fn test_rom_bank_bits() {
        let config = CartridgeConfig::new(ControllerType::MBC3, 0x08, 0x02).unwrap();
        let mut ctrl = MBC3::new(config, Arc::new([0; ROM_BANK_SIZE * 16]));

        ctrl.write(RAM_RTC_ENABLE_BEGIN, 0x01);
        assert_eq!(ctrl.rom_bank_number, 0x01);

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0x55);
        assert_eq!(ctrl.rom_bank_number, 0x55);

        ctrl.write(ROM_BANK_NUMBER_BEGIN, 0xFF);
        assert_eq!(
            ctrl.rom_bank_number, 0x7F,
            "Only first 7 bits should be used"
        );
    }
    #[test]
    fn test_ram_banking() {
        let config = CartridgeConfig::new(ControllerType::MBC3, 0x00, 0x03).unwrap();

        // Initialize each bank with a unique value
        let mut ctrl = MBC3::new(
            config,
            (0u8..64).flat_map(|i| vec![i; ROM_BANK_SIZE]).collect(),
        );

        ctrl.write(RAM_BANK_NUMBER_BEGIN, 1);
        ctrl.write(RAM_RTC_ENABLE_BEGIN, 0x0A);

        // Assert the banks are set correctly and the memory is initialized
        for i in 0u8..4 {
            ctrl.write(RAM_BANK_NUMBER_BEGIN, i);
            assert_eq!(ctrl.ram_bank_selection, RAMBankSelection::RAMBank(i));
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
