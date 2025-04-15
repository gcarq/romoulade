use crate::gb::constants::*;
use crate::gb::{AddressSpace, GBError};
use crate::gb::{GBResult, utils};
use std::path::Path;
use std::{fmt, fs};

/// This area of memory contains the cartridge title
const CARTRIDGE_TITLE_BEGIN: u16 = 0x0134;
const CARTRIDGE_TITLE_END: u16 = 0x0142;

/// When using any CGB registers (including those in the Video/Link chapters),
/// you must first unlock CGB features by changing byte 0143h in the cartridge header.
/// Typically, use a value of 80h for games which support both CGB and monochrome Game Boys,
/// and C0h for games which work on CGBs only. Otherwise,
/// the CGB will operate in monochrome "Non CGB" compatibility mode.
const CARTRIDGE_CGB_FLAG: u16 = 0x0143;

/// This address contains the cartridge type and what kind of hardware is present
/// 0x00     => ROM Only
/// 0x01  => MBC1
/// 0x02  => MBC1 + RAM
/// 0x03  => MBC1 + RAM + Battery
/// 0x05  => MBC2
/// ...
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
const CARTRIDGE_TYPE: u16 = 0x0147;

/// This byte indicates how much ROM is present on the cartridge.
/// In most cases, the ROM size is given by 32KiB * (1 << value).
const CARTRIDGE_ROM_SIZE: u16 = 0x0148;

/// This byte indicates how much RAM is present on the cartridge.
const CARTRIDGE_RAM_SIZE: u16 = 0x0149;

/// This byte contains an 8-bit checksum computed from the cartridge header bytes 0x0134 – 0x014C.
const CARTRIDGE_HEADER_CHECKSUM: u16 = 0x014D;

/// These bytes contain a 16-bit (big-endian) checksum simply computed as the sum of all
/// the bytes of the cartridge ROM (except these two checksum bytes).
const CARTRIDGE_GLOBAL_CHECKSUM1: u16 = 0x014E;
const CARTRIDGE_GLOBAL_CHECKSUM2: u16 = 0x014F;

#[derive(Debug, PartialEq, Copy, Clone)]
#[repr(u8)]
/// TODO: implement remaining modes including RAM banking
pub enum BankingMode {
    None,
    MBC1,
    MBC2,
}

impl TryFrom<u8> for BankingMode {
    type Error = GBError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let mode = match value {
            0x00 => BankingMode::None,
            0x01 => BankingMode::MBC1,
            0x02 => BankingMode::MBC1,
            0x03 => BankingMode::MBC1,
            0x05 => BankingMode::MBC2,
            0x06 => BankingMode::MBC2,
            _ => return Err(format!("Cartridge type {:#04X} not implemented", value).into()),
        };
        Ok(mode)
    }
}

/// Contains the cartridge header information.
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html
#[derive(Clone)]
pub struct CartridgeHeader {
    pub title: String,
    pub banking: BankingMode,
    pub cgb_flag: u8,
}

impl TryFrom<&[u8]> for CartridgeHeader {
    type Error = GBError;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        Ok(Self {
            title: CartridgeHeader::parse_title(buf),
            banking: BankingMode::try_from(buf[CARTRIDGE_TYPE as usize])?,
            cgb_flag: buf[CARTRIDGE_CGB_FLAG as usize],
        })
    }
}

impl CartridgeHeader {
    /// Returns the cartridge title from the cartridge header.
    fn parse_title(buf: &[u8]) -> String {
        buf[CARTRIDGE_TITLE_BEGIN as usize..=CARTRIDGE_TITLE_END as usize]
            .iter()
            .filter(|b| b.is_ascii_alphanumeric())
            .map(|b| char::from(*b))
            .collect()
    }
}

impl fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {} (banking: {:?}, cgb_flag: {:#04X})",
            self.title, self.banking, self.cgb_flag
        )
    }
}

/// Contains all data for a cartridge
#[derive(Clone)]
pub struct Cartridge {
    pub header: CartridgeHeader,
    rom: Vec<u8>,
    ram: [u8; CRAM_SIZE * 4],
    cur_rom_bank: u8,
    cur_ram_bank: u8,
    enable_ram: bool,
    // This variable is responsible for how to act when the game writes to
    // memory address 0x4000-0x6000
    rom_banking: bool,
}

impl Cartridge {
    /// Creates a new Cartridge from the given Path
    pub fn from_path(path: &Path) -> GBResult<Self> {
        let buffer = fs::read(path)?;
        if let Err(msg) = verify_checksum(&buffer) {
            eprintln!("WARNING: {}", msg);
        }
        Ok(Self {
            header: CartridgeHeader::try_from(buffer.as_slice())?,
            rom: buffer,
            ram: [0u8; CRAM_SIZE * 4],
            cur_rom_bank: 1,
            cur_ram_bank: 0,
            enable_ram: false,
            rom_banking: true,
        })
    }

    fn handle_banking(&mut self, address: u16, value: u8) {
        match address {
            // Do RAM enable
            ROM_BANK_0_BEGIN..=0x1FFF => {
                if self.header.banking == BankingMode::MBC1
                    || self.header.banking == BankingMode::MBC2
                {
                    self.toggle_ram_banking(address, value);
                }
            }
            // Do ROM bank change
            0x2000..=ROM_BANK_0_END => match self.header.banking {
                BankingMode::MBC1 | BankingMode::MBC2 => self.change_low_rom_bank(value),
                // ROM banking requested, but Cartridge only uses 1 ROM bank. Safe to ignore.
                BankingMode::None => {}
            },
            // Do ROM or RAM bank change
            ROM_BANK_N_BEGIN..=0x5FFF => {
                // There is no RAM bank in MBC2 so we always use RAM bank 0
                if self.header.banking != BankingMode::MBC1 {
                    return;
                }
                if self.rom_banking {
                    self.change_hi_rom_bank(value);
                    return;
                }
                self.cur_ram_bank = value & 0x03;
            }
            // Select whether we are doing ROM or RAM banking
            0x6000..=ROM_BANK_N_END => {
                if self.header.banking == BankingMode::MBC1 {
                    self.change_rom_ram_mode(value);
                }
            }
            _ => unimplemented!(),
        }
    }

    /// Enables or disables RAM banking.
    #[inline]
    fn toggle_ram_banking(&mut self, address: u16, value: u8) {
        // If MBC2 is enabled, bit 4 of the address must be zero.
        if self.header.banking == BankingMode::MBC2 && utils::bit_at(address as u8, 4) {
            todo!("implement RAM banking while MBC2 is enabled!");
        }

        // If MBC1 is enabled, the lower nibble must be equal to 0b1010 to enable cartridge RAM.
        match value & 0b1111 {
            0b1010 => self.enable_ram = true,
            0b0000 => self.enable_ram = false,
            _ => todo!("{:#04X}", value),
        }
    }

    /// Change ROM banking mode (lower 5 bits)
    #[inline]
    fn change_low_rom_bank(&mut self, value: u8) {
        if self.header.banking == BankingMode::MBC2 {
            self.cur_rom_bank = value & 0x0F;
            self.sanitize_rom_bank();
            return;
        }

        // Turn of the 5 lower bits of the current bank
        // and turn of the higher 5 bits of the passed value
        self.cur_rom_bank = (self.cur_rom_bank & 0xE0) | (value & 0x1F);
        self.sanitize_rom_bank();
    }

    /// Change ROM banking mode (bits 5 & 6)
    #[inline]
    fn change_hi_rom_bank(&mut self, value: u8) {
        // Turn of the upper 3 bits of the current bank
        // and turn of the lower 5 bits of the passed value
        self.cur_rom_bank = (self.cur_rom_bank & 0x1F) | (value & 0xE0);
        self.sanitize_rom_bank();
    }

    /// Selects either ROM or RAM banking mode
    #[inline]
    fn change_rom_ram_mode(&mut self, value: u8) {
        // The bit 0 defines whether we enable ROM banking
        self.rom_banking = (value & 0x01) == 0;
        if self.rom_banking {
            self.cur_ram_bank = 0;
        }
    }

    #[inline]
    fn sanitize_rom_bank(&mut self) {
        if self.cur_rom_bank == 0 {
            self.cur_rom_bank = 1;
        }
    }
}

impl AddressSpace for Cartridge {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=ROM_BANK_N_END => self.handle_banking(address, value),
            CRAM_BEGIN..=CRAM_END => {
                let offset = self.cur_ram_bank as usize * CRAM_SIZE;
                self.ram[(address - CRAM_BEGIN) as usize + offset] = value
            }
            _ => unimplemented!("Trying to write byte to ROM: {:#06x}", address),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.rom[(address - ROM_BANK_0_BEGIN) as usize],
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => {
                let offset = self.cur_rom_bank as usize * ROM_BANK_N_SIZE;
                self.rom[(address - ROM_BANK_N_BEGIN) as usize + offset]
            }
            CRAM_BEGIN..=CRAM_END => {
                let offset = self.cur_ram_bank as usize * CRAM_SIZE;
                self.ram[(address - CRAM_BEGIN) as usize + offset]
            }
            _ => unimplemented!("Trying to read byte from ROM: {:#06x}", address),
        }
    }
}

/// Validates the global checksum of the given buffer containing the whole cartridge.
#[inline]
fn verify_checksum(buf: &[u8]) -> GBResult<()> {
    if buf.len() < CARTRIDGE_GLOBAL_CHECKSUM2 as usize {
        return Err("Cartridge is too small to calculate the checksum".into());
    }

    let byte1 = buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize];
    let byte2 = buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize];

    let checksum = (byte1 as u16) << 8 | (byte2 as u16);
    let calculated_checksum = calculate_global_checksum(buf);

    if checksum == calculated_checksum {
        return Ok(());
    }

    let msg = format!(
        "Global checksum mismatch! Expected: {:#04X} Got: {:#04X}",
        checksum, calculated_checksum
    );
    Err(msg.into())
}

/// Calculates the global checksum by adding all bytes from the given cartridge buffer except
/// the two checksum bytes.
#[inline]
fn calculate_global_checksum(buf: &[u8]) -> u16 {
    buf.iter()
        .enumerate()
        .fold(0, |sum, (address, &byte)| match address as u16 {
            CARTRIDGE_GLOBAL_CHECKSUM1 => sum,
            CARTRIDGE_GLOBAL_CHECKSUM2 => sum,
            _ => sum.wrapping_add(byte as u16),
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_calculate_global_checksum() {
        let buf = (0..CARTRIDGE_GLOBAL_CHECKSUM2)
            .map(|i| i as u8)
            .collect::<Vec<u8>>();
        let checksum = calculate_global_checksum(&buf);
        assert_eq!(checksum, 0x8B3B);
    }

    #[test]
    fn test_verify_checksum() {
        let mut buf = (0..=CARTRIDGE_GLOBAL_CHECKSUM2)
            .map(|i| i as u8)
            .collect::<Vec<u8>>();
        buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize] = 0x8B;
        buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize] = 0x3B;
        assert!(verify_checksum(&buf).is_ok());
    }
}
