use crate::gb::GBError;
use crate::gb::cartridge::controller::BankController;
use crate::gb::{GBResult, SubSystem};
use std::path::Path;
use std::sync::Arc;
use std::{fmt, fs};

mod controller;
mod mbc1;
mod mbc3;
mod mbc5;
mod nombc;
#[cfg(test)]
mod tests;

/// This area of memory contains the cartridge title
const CARTRIDGE_TITLE_BEGIN: u16 = 0x0134;
const CARTRIDGE_TITLE_END: u16 = 0x0142;

/// When using any CGB registers (including those in the Video/Link chapters),
/// you must first unlock CGB features by changing byte 0143h in the cartridge header.
/// Typically, use a value of 80h for games which support both CGB and monochrome Game Boys,
/// and C0h for games which work on CGBs only. Otherwise,
/// the CGB will operate in monochrome "Non CGB" compatibility mode.
const _CARTRIDGE_CGB_FLAG: u16 = 0x0143;

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
const _CARTRIDGE_HEADER_CHECKSUM: u16 = 0x014D;

/// These bytes contain a 16-bit (big-endian) checksum simply computed as the sum of all
/// the bytes of the cartridge ROM (except these two checksum bytes).
const CARTRIDGE_GLOBAL_CHECKSUM1: u16 = 0x014E;
const CARTRIDGE_GLOBAL_CHECKSUM2: u16 = 0x014F;

const ROM_BANK_SIZE: usize = 16384;
const RAM_BANK_SIZE: usize = 8192;

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
/// The controller type of the cartridge.
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
/// TODO: implement remaining modes (and specify battery support?)
pub enum ControllerType {
    NoMBC,
    MBC1,
    MBC2,
    MBC3,
    MBC5,
    MBC6,
    MBC7,
}

impl TryFrom<u8> for ControllerType {
    type Error = GBError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let mode = match value {
            0x00 | 0x08 | 0x09 => ControllerType::NoMBC,
            0x01..=0x03 => ControllerType::MBC1,
            0x05 | 0x06 => ControllerType::MBC2,
            0x0F..=0x13 => ControllerType::MBC3,
            0x19..=0x1E => ControllerType::MBC5,
            0x20 => ControllerType::MBC6,
            0x22 => ControllerType::MBC7,
            _ => return Err(format!("Cartridge type {value:#04x} not implemented").into()),
        };
        Ok(mode)
    }
}

impl fmt::Display for ControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ControllerType::NoMBC => "NoMBC",
            ControllerType::MBC1 => "MBC1",
            ControllerType::MBC2 => "MBC2",
            ControllerType::MBC3 => "MBC3",
            ControllerType::MBC5 => "MBC5",
            ControllerType::MBC6 => "MBC6",
            ControllerType::MBC7 => "MBC7",
        };
        write!(f, "{name}")
    }
}

/// Contains the configuration of the cartridge. This includes the controller type,
/// ROM size, RAM size, and the number of banks.
#[derive(Copy, Clone, Debug)]
pub struct CartridgeConfig {
    pub controller: ControllerType,
    pub rom_banks: u16,
    pub ram_banks: u16,
}

impl CartridgeConfig {
    pub fn new(banking: ControllerType, rom_size: u8, ram_size: u8) -> GBResult<Self> {
        let ram_banks = match ram_size {
            0x00 | 0x01 => 0,
            0x02 => 1,
            0x03 => 4,
            0x04 => 16,
            0x05 => 8,
            value => return Err(format!("Unsupported RAM size: {value:#04x}").into()),
        };

        // This can be expressed as 2^(value + 1) up until 512 KiB
        let rom_banks = match rom_size {
            0x00 => 2,
            0x01 => 4,
            0x02 => 8,
            0x03 => 16,
            0x04 => 32,
            0x05 => 64,
            0x06 => 128,
            0x07 => 256,
            0x08 => 512,
            value => return Err(format!("Unsupported ROM size: {value:#04x}").into()),
        };

        Ok(Self {
            controller: banking,
            rom_banks,
            ram_banks,
        })
    }

    #[inline(always)]
    pub const fn ram_size(&self) -> usize {
        self.ram_banks as usize * RAM_BANK_SIZE
    }
}

/// Contains the cartridge header information.
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html
#[derive(Clone)]
pub struct CartridgeHeader {
    pub title: String,
    pub config: CartridgeConfig,
}

impl TryFrom<&[u8]> for CartridgeHeader {
    type Error = GBError;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let controller = ControllerType::try_from(buf[CARTRIDGE_TYPE as usize])?;
        let config = CartridgeConfig::new(
            controller,
            buf[CARTRIDGE_ROM_SIZE as usize],
            buf[CARTRIDGE_RAM_SIZE as usize],
        )?;
        let title = CartridgeHeader::parse_title(buf);
        Ok(Self { title, config })
    }
}

impl CartridgeHeader {
    /// Returns the cartridge title from the cartridge header.
    fn parse_title(buf: &[u8]) -> String {
        let title = buf[CARTRIDGE_TITLE_BEGIN as usize..=CARTRIDGE_TITLE_END as usize]
            .iter()
            .filter_map(|b| b.is_ascii_alphanumeric().then_some(char::from(*b)))
            .collect::<String>();
        match title.is_empty() {
            true => "Unnamed".to_string(),
            false => title,
        }
    }
}

impl fmt::Display for CartridgeHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.title, self.config.controller)
    }
}

/// Holds all relevant cartridge information data.
#[derive(Clone)]
pub struct Cartridge {
    pub header: CartridgeHeader,
    controller: Box<dyn BankController>,
}

impl TryFrom<Arc<[u8]>> for Cartridge {
    type Error = GBError;

    fn try_from(rom: Arc<[u8]>) -> Result<Self, Self::Error> {
        if let Err(msg) = verify_checksum(rom.as_ref()) {
            eprintln!("WARNING: {msg}");
        }
        let header = CartridgeHeader::try_from(rom.as_ref())?;
        let controller = controller::new(header.config, rom);
        Ok(Self { controller, header })
    }
}

impl TryFrom<&Path> for Cartridge {
    type Error = GBError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let rom = fs::read(path)?;
        Cartridge::try_from(Arc::from(rom.into_boxed_slice()))
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.header)
    }
}

impl SubSystem for Cartridge {
    #[inline]
    fn write(&mut self, address: u16, value: u8) {
        self.controller.write(address, value);
    }

    #[inline]
    fn read(&mut self, address: u16) -> u8 {
        self.controller.read(address)
    }
}

/// Validates the global checksum of the given buffer containing the whole cartridge.
fn verify_checksum(buf: &[u8]) -> GBResult<()> {
    if buf.len() < CARTRIDGE_GLOBAL_CHECKSUM2 as usize {
        return Err("Cartridge is too small to calculate the checksum".into());
    }

    let byte1 = buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize];
    let byte2 = buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize];
    let checksum = u16::from(byte1) << 8 | u16::from(byte2);
    let calculated_checksum = calculate_global_checksum(buf);

    if checksum == calculated_checksum {
        return Ok(());
    }

    let msg = format!(
        "Global checksum mismatch! Expected: {calculated_checksum:#06x} Got: {checksum:#06x}"
    );
    Err(msg.into())
}

/// Calculates the global checksum by adding all bytes from the given cartridge buffer except
/// the two checksum bytes.
fn calculate_global_checksum(buf: &[u8]) -> u16 {
    buf.iter()
        .enumerate()
        .fold(0, |sum, (address, &byte)| match address as u16 {
            CARTRIDGE_GLOBAL_CHECKSUM1 => sum,
            CARTRIDGE_GLOBAL_CHECKSUM2 => sum,
            _ => sum.wrapping_add(byte as u16),
        })
}

/// This function masks the ROM Bank Number to the number of banks in the cartridge.
#[inline]
const fn bank_mask(rom_banks: u16) -> u32 {
    let mask = u16::BITS - rom_banks.leading_zeros();
    (1 << mask) - 1
}
