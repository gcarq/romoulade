use crate::gb::cartridge::BankingMode::MBC2;
use crate::gb::memory::constants::{
    CRAM_BEGIN, CRAM_END, CRAM_SIZE, ROM_BANK_0_BEGIN, ROM_BANK_0_END, ROM_BANK_N_BEGIN,
    ROM_BANK_N_END, ROM_BANK_N_SIZE,
};
use crate::gb::AddressSpace;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::{convert, fmt, fs, io};

/// This area of memory contains ROM title
const TITLE_BEGIN: u16 = 0x0134;
const TITLE_END: u16 = 0x0143;

/// This address contains the number of ROM banks
/// 0     => No memory banking
/// 1..3  => MBC1
/// 5     => MBC2
const CARTRIDGE_ROM_BANKS: u16 = 0x0147;

/// This address contains the number of RAM banks,
/// maximum are 4 banks.
const CARTRIDGE_RAM_BANKS: u16 = 0x0148;

#[derive(Debug, PartialEq)]
#[repr(u8)]
/// TODO: implement remaining modes
pub enum BankingMode {
    None,
    MBC1,
    MBC2, // Ram Baking is not used in MBC2!
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
            banking: BankingMode::from(buf[CARTRIDGE_ROM_BANKS as usize]),
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
    pub fn from_path(path: &Path) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let metadata = fs::metadata(&path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read_exact(&mut buffer)?;
        let meta = Metadata::from_buf(&buffer);
        Ok(Self {
            meta,
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
            0x0000..=0x1FFF => {
                if self.meta.banking == BankingMode::MBC1 || self.meta.banking == BankingMode::MBC1
                {
                    self.toggle_ram_banking(address, value);
                }
            }
            // Do ROM bank change
            0x2000..=ROM_BANK_0_END => match self.meta.banking {
                BankingMode::MBC1 | BankingMode::MBC2 => self.change_low_rom_bank(value),
                // ROM banking requested, but Cartridge only uses 1 ROM bank. Safe to ignore.
                BankingMode::None => {}
            },
            // Do ROM or RAM bank change
            0x4000..=0x5FFF => {
                // There is no RAM bank in MBC2 so we always use RAM bank 0
                if self.meta.banking != BankingMode::MBC1 {
                    return;
                }
                if self.rom_banking {
                    self.change_hi_rom_bank(value);
                    return;
                }
                self.cur_ram_bank = value & 0x03;
            }
            // Select whether we are doing ROM or RAM banking
            0x6000..=0x7FFF => {
                if self.meta.banking == BankingMode::MBC1 {
                    self.change_rom_ram_mode(value);
                }
            }
            _ => unimplemented!(),
        }
    }

    /// Enables or disables RAM banking.
    fn toggle_ram_banking(&mut self, address: u16, value: u8) {
        // If MBC2 is enabled, bit 4 of the address must be zero.
        if self.meta.banking == MBC2
        /*&& utils::bit_at(address as u8, 4)*/
        {
            // TODO: implement me
            panic!("TODO: implement RAM banking while MBC2 is enabled!");
            return;
        }

        // If MBC1 is enabled, the lower nibble must be equal to 0X0A to enable cartridge RAM.
        match value & 0x0F {
            0x0A => self.enable_ram = true,
            0x00 => self.enable_ram = false,
            _ => panic!(),
        }
    }

    // TODO:
    fn change_low_rom_bank(&mut self, value: u8) {
        if self.meta.banking == BankingMode::MBC2 {
            self.cur_rom_bank = value & 0x0F;
            self.sanitize_rom_bank();
            return;
        }

        // Turn of the 5 lower bits of the current bank
        // and turn of the higher 5 bits of the passed value
        self.cur_rom_bank = (self.cur_rom_bank & 0xE0) | (value & 0x1F);
        self.sanitize_rom_bank();
    }

    // TODO:
    fn change_hi_rom_bank(&mut self, value: u8) {
        // Turn of the upper 3 bits of the current bank
        // and turn of the lower 5 bits of the passed value
        self.cur_rom_bank = (self.cur_rom_bank & 0x1F) | (value & 0xE0);
        self.sanitize_rom_bank();
    }

    /// Selects either ROM or RAM banking mode
    fn change_rom_ram_mode(&mut self, value: u8) {
        // The bit 0 defines whether we enable ROM banking
        self.rom_banking = (value & 0x01) == 0;
        if self.rom_banking {
            self.cur_ram_bank = 0;
        }
    }

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
                let address = address + self.cur_ram_bank as u16 * CRAM_SIZE as u16;
                self.ram[(address - CRAM_BEGIN) as usize] = value
            }
            _ => unimplemented!("Trying to write byte to ROM: {:#06x}", address),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.rom[address as usize],
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => {
                let address = address + self.cur_rom_bank as u16 * ROM_BANK_N_SIZE as u16;
                self.rom[(address - ROM_BANK_N_BEGIN) as usize]
            }
            CRAM_BEGIN..=CRAM_END => {
                let address = address + self.cur_ram_bank as u16 * CRAM_SIZE as u16;
                self.ram[(address - CRAM_BEGIN) as usize]
            }
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
