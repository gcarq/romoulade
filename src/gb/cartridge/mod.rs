use crate::gb::GBError;
use crate::gb::cartridge::controller::BankController;
use crate::gb::{GBResult, SubSystem};
use std::path::Path;
use std::sync::Arc;
use std::{fmt, fs};
use thiserror::Error;

pub mod controller;
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
const CARTRIDGE_HEADER_CHECKSUM: u16 = 0x014D;

/// These bytes contain a 16-bit (big-endian) checksum simply computed as the sum of all
/// the bytes of the cartridge ROM (except these two checksum bytes).
const CARTRIDGE_GLOBAL_CHECKSUM1: u16 = 0x014E;
const CARTRIDGE_GLOBAL_CHECKSUM2: u16 = 0x014F;

const ROM_BANK_SIZE: usize = 16384;
const RAM_BANK_SIZE: usize = 8192;

#[derive(Error, Debug)]
pub enum SaveError {
    #[error("The cartridge does not support saving.")]
    NoSaveSupport,
    #[error("The save file is not compatible with the current cartridge.")]
    InvalidChecksum,
    #[error("The save file is empty or malformed.")]
    MalformedFile,
    #[error("Failed to read or write the save file: {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(PartialEq, Copy, Clone, Debug)]
#[repr(u8)]
/// The controller type of the cartridge.
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html#0147--cartridge-type
/// TODO: implement remaining controller types
pub enum ControllerType {
    NoMBC { battery: bool },
    MBC1 { battery: bool },
    MBC2 { battery: bool },
    MBC3 { battery: bool },
    MBC5 { battery: bool },
    MBC7 { battery: bool },
}

impl ControllerType {
    /// Returns true if the controller type has battery-backed RAM.
    pub const fn has_battery(&self) -> bool {
        match self {
            ControllerType::NoMBC { battery } => *battery,
            ControllerType::MBC1 { battery } => *battery,
            ControllerType::MBC2 { battery } => *battery,
            ControllerType::MBC3 { battery } => *battery,
            ControllerType::MBC5 { battery } => *battery,
            ControllerType::MBC7 { battery } => *battery,
        }
    }
}

impl TryFrom<u8> for ControllerType {
    type Error = GBError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        let mode = match value {
            0x00 | 0x08 => ControllerType::NoMBC { battery: false },
            0x09 => ControllerType::NoMBC { battery: true },
            0x01 | 0x02 => ControllerType::MBC1 { battery: false },
            0x03 => ControllerType::MBC1 { battery: true },
            0x05 => ControllerType::MBC2 { battery: false },
            0x06 => ControllerType::MBC2 { battery: true },
            0x0F | 0x10 | 0x12 | 0x13 => ControllerType::MBC3 { battery: true },
            0x11 => ControllerType::MBC3 { battery: false },
            0x19 | 0x1A | 0x1C | 0x1D => ControllerType::MBC5 { battery: false },
            0x1B | 0x1E => ControllerType::MBC5 { battery: true },
            0x22 => ControllerType::MBC7 { battery: true },
            _ => return Err(format!("Cartridge type {value:#04x} not implemented").into()),
        };
        Ok(mode)
    }
}

impl fmt::Display for ControllerType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            ControllerType::NoMBC { .. } => "NoMBC",
            ControllerType::MBC1 { .. } => "MBC1",
            ControllerType::MBC2 { .. } => "MBC2",
            ControllerType::MBC3 { .. } => "MBC3",
            ControllerType::MBC5 { .. } => "MBC5",
            ControllerType::MBC7 { .. } => "MBC7",
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
    pub fn new(controller: ControllerType, rom_size: u8, ram_size: u8) -> GBResult<Self> {
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
            controller,
            rom_banks,
            ram_banks,
        })
    }

    #[inline(always)]
    pub const fn ram_size(&self) -> usize {
        self.ram_banks as usize * RAM_BANK_SIZE
    }

    /// Returns true if the cartridge has at least one battery-backed RAM bank.
    #[inline]
    pub const fn is_savable(&self) -> bool {
        self.ram_banks > 0 && self.controller.has_battery()
    }
}

/// Contains the cartridge header information.
/// See https://gbdev.io/pandocs/The_Cartridge_Header.html
#[derive(Clone)]
pub struct CartridgeHeader {
    pub title: String,
    pub config: CartridgeConfig,
    pub header_checksum: u8,
    pub global_checksum: u16,
}

impl TryFrom<&[u8]> for CartridgeHeader {
    type Error = GBError;
    fn try_from(buf: &[u8]) -> Result<Self, Self::Error> {
        let controller = ControllerType::try_from(buf[CARTRIDGE_TYPE as usize])?;
        Ok(Self {
            title: CartridgeHeader::parse_title(buf),
            config: CartridgeConfig::new(
                controller,
                buf[CARTRIDGE_ROM_SIZE as usize],
                buf[CARTRIDGE_RAM_SIZE as usize],
            )?,
            header_checksum: buf[CARTRIDGE_HEADER_CHECKSUM as usize],
            global_checksum: u16::from_le_bytes([
                buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize],
                buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize],
            ]),
        })
    }
}

impl CartridgeHeader {
    /// Returns the cartridge title from the cartridge header.
    fn parse_title(buf: &[u8]) -> String {
        debug_assert!(buf.len() >= CARTRIDGE_TITLE_END as usize);
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
        write!(f, "{}", self.title)
    }
}

/// Holds all relevant cartridge information data.
#[derive(Clone)]
pub struct Cartridge {
    pub header: CartridgeHeader,
    pub(crate) controller: Box<dyn BankController>,
}

impl Cartridge {
    /// Loads a save file from the given path.
    pub fn load_savefile(&mut self, path: &Path) -> Result<(), SaveError> {
        let mut data = fs::read(path)?;
        let checksum = match data.pop() {
            Some(checksum) => checksum,
            None => return Err(SaveError::MalformedFile),
        };
        if checksum != self.header.header_checksum {
            return Err(SaveError::InvalidChecksum);
        }
        self.controller.load_ram(data);
        Ok(())
    }

    /// Writes a save file to the given path.
    /// The save file contains a dump of the latest sane RAM state and the one byte header checksum.
    pub fn write_savefile(&self, path: &Path) -> Result<(), SaveError> {
        let ram = self.controller.save_ram()?;
        let mut data = ram.to_vec();
        data.push(self.header.header_checksum);
        Ok(fs::write(path, &data)?)
    }

    /// Returns the name of the autosave file based on the cartridge title and header checksum.
    pub fn autosave_filename(&self) -> String {
        format!(
            "{}_{:02x}_autosave.sav",
            self.header.title.replace(' ', "_"),
            self.header.header_checksum
        )
    }
}

impl TryFrom<Arc<[u8]>> for Cartridge {
    type Error = GBError;

    fn try_from(rom: Arc<[u8]>) -> Result<Self, Self::Error> {
        if rom.len() < CARTRIDGE_GLOBAL_CHECKSUM2 as usize {
            return Err("Cartridge is too small to calculate the checksum".into());
        }
        let header = CartridgeHeader::try_from(rom.as_ref())?;
        if let Err(msg) = verify_checksum(rom.as_ref(), header.global_checksum) {
            eprintln!("WARNING: {msg}");
        }
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
fn verify_checksum(buf: &[u8], checksum: u16) -> GBResult<()> {
    debug_assert!(buf.len() >= CARTRIDGE_GLOBAL_CHECKSUM2 as usize);

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
