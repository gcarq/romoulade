use crate::gb::memory::constants::{
    ROM_BANK_0_BEGIN, ROM_BANK_0_END, ROM_BANK_N_BEGIN, ROM_BANK_N_END,
};
use crate::gb::AddressSpace;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{convert, fmt, fs, io};

/// This area of memory contains ROM title
const TITLE_BEGIN: u16 = 0x0134;
const TITLE_END: u16 = 0x0143;

/// This address contains the ROM banking mode
/// 0     => No memory banking
/// 1..3  => MBC1
/// 5     => MBC2
const CARTRIDGE_BANKING_MODE: u16 = 0x0147;

#[derive(Debug)]
#[repr(u8)]
/// TODO: implement remaining modes
pub enum BankingMode {
    None,
    MBC1,
    MBC2,
}

impl convert::From<u8> for BankingMode {
    fn from(value: u8) -> Self {
        match value {
            0 => BankingMode::None,
            1..=3 => BankingMode::MBC1,
            5..=6 => BankingMode::MBC2,
            _ => unimplemented!(),
        }
    }
}

/// Contains parsed metadata of Cartridge
pub struct Metadata {
    pub title: String,
    pub banking: BankingMode,
}

impl Metadata {
    pub fn from_buf(buf: &[u8]) -> Self {
        Self {
            title: Metadata::parse_title(buf),
            banking: BankingMode::from(buf[CARTRIDGE_BANKING_MODE as usize]),
        }
    }

    /// Returns title from metadata
    /// TODO: can it contain utf8 data?
    fn parse_title(buf: &[u8]) -> String {
        buf[TITLE_BEGIN as usize..TITLE_END as usize]
            .iter()
            .filter(|b| b.is_ascii_alphanumeric())
            .map(|b| char::from(*b))
            .collect()
    }
}

/// Contains all data for a cartridge
pub struct Cartridge {
    pub meta: Metadata,
    rom: Vec<u8>,
    cur_rom_bank: u16,
}

impl Cartridge {
    /// Creates a new Cartridge from the given Path
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let metadata = fs::metadata(&path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read_exact(&mut buffer)?;
        let meta = Metadata::from_buf(&buffer);
        Ok(Self {
            meta,
            rom: buffer,
            cur_rom_bank: 1,
        })
    }
}

impl AddressSpace for Cartridge {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => unimplemented!("TODO: handle RAM banking: {:#06x}", address),
            0x2000..=ROM_BANK_0_END => match self.meta.banking {
                BankingMode::MBC1 | BankingMode::MBC2 => {
                    unimplemented!("TODO: handle ROM banking: {:#06x}", address)
                }
                BankingMode::None => {
                    // ROM banking requested, but Cartridge only uses 1 ROM bank. Safe to ignore.
                }
            },
            0x4000..=0x6000 => unimplemented!("TODO: handle RAM banking: {:#06x}", address),
            _ => unimplemented!("Trying to write byte to ROM: {:#06x}", address),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.rom[address as usize],
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => {
                let address = address - ROM_BANK_N_BEGIN + self.cur_rom_bank * ROM_BANK_N_BEGIN;
                self.rom[address as usize]
            }
            // TODO: take car of RAM banking
            _ => unimplemented!("Trying to read byte from ROM: {:#06x}", address),
        }
    }
}

impl fmt::Display for Cartridge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Title: {} (banking: {:?})",
            self.meta.title, self.meta.banking
        )
    }
}
