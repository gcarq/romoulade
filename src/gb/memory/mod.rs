pub mod constants;

use crate::gb::cartridge::Cartridge;
use crate::gb::memory::constants::*;
use crate::gb::AddressSpace;
use crate::utils;

/// Defines a global MemoryBus, all processing units should access memory through this bus.
pub struct MemoryBus {
    cartridge: Cartridge,
    vram: [u8; VRAM_SIZE],
    cram: [u8; CRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    eram: [u8; ERAM_SIZE],
    oam: [u8; OAM_SIZE],
    io: [u8; IO_SIZE],
    hram: [u8; HRAM_SIZE],
    ie: u8,
}

impl MemoryBus {
    pub fn new(cartridge: Cartridge) -> Self {
        Self {
            cartridge,
            vram: [0u8; VRAM_SIZE],
            cram: [0u8; CRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            eram: [0u8; ERAM_SIZE],
            oam: [0u8; OAM_SIZE],
            io: [0u8; IO_SIZE],
            hram: [0u8; HRAM_SIZE],
            ie: 0,
        }
    }

    /// Requests an interrupt for the given id
    /// TODO: create enum? See interrupt.rs
    pub fn irq(&mut self, id: u8) {
        let req = utils::set_bit(self.read(INTERRUPT_FLAG), id, true);
        self.write(INTERRUPT_FLAG, req);
    }

    fn read_rom_bank(&self, address: u16) -> u8 {
        match address {
            BOOT_BEGIN..=BOOT_END => match self.read(BOOT_ROM_OFF) {
                0 => BOOT_ROM[address as usize],
                1 => self.cartridge.read(address),
                _ => unimplemented!(),
            },
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.cartridge.read(address),
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => {
                unimplemented!("TODO: Implement ROM bank handling")
            }
            _ => panic!("Out of bound read from cartridge: {:#06x}", address),
        }
    }
}

impl AddressSpace for MemoryBus {
    fn write(&mut self, address: u16, value: u8) {
        // TODO: Implement DMA Transfer if address == 0xFF46
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => {
                // TODO: don't panic, but emit a warning
                panic!("Trying to write byte to ROM: {:#06x}", address)
            }
            VRAM_BEGIN..=VRAM_END => self.vram[(address - VRAM_BEGIN) as usize] = value,
            CRAM_BEGIN..=CRAM_END => self.cram[(address - CRAM_BEGIN) as usize] = value,
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize] = value,
            ERAM_BEGIN..=ERAM_END => {
                // Mirrors Working RAM
                self.eram[(address - ERAM_BEGIN) as usize] = value;
                self.wram[(address - WRAM_BEGIN) as usize] = value;
            }
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize] = value,
            0xFEA0..=0xFEFF => {} // This area is unmapped, writing to it does nothing.
            IO_BEGIN..=IO_END => self.io[(address - IO_BEGIN) as usize] = value,
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.ie = value,
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.read_rom_bank(address),
            VRAM_BEGIN..=VRAM_END => self.vram[(address - VRAM_BEGIN) as usize],
            CRAM_BEGIN..=CRAM_END => self.cram[(address - CRAM_BEGIN) as usize],
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize],
            ERAM_BEGIN..=ERAM_END => self.eram[(address - ERAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize],
            0xFEA0..=0xFEFF => 0, // This area is unmapped, reading from it should return 0.
            IO_BEGIN..=IO_END => self.io[(address - IO_BEGIN) as usize],
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize],
            INTERRUPT_ENABLE => self.ie,
        }
    }
}
