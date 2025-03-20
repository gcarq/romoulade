pub mod constants;

use crate::gb::bus::constants::*;
use crate::gb::cartridge::Cartridge;
use crate::gb::interrupt::InterruptFlags;
use crate::gb::timer::{Frequency, Timer};
use crate::gb::AddressSpace;

/// Defines a global Bus, all processing units should access memory through it.
pub struct Bus {
    cartridge: Cartridge,
    timer: Timer,
    divider: Timer,
    pub interrupt_enable: InterruptFlags,
    pub interrupt_flag: InterruptFlags,
    vram: [u8; VRAM_SIZE],
    wram: [u8; WRAM_SIZE],
    eram: [u8; ERAM_SIZE],
    oam: [u8; OAM_SIZE],
    io: [u8; IO_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl Bus {
    pub fn new(cartridge: Cartridge) -> Self {
        let mut divider = Timer::new(Frequency::Hz16384);
        divider.on = true;
        divider.value = 0xAB;
        Self {
            cartridge,
            divider,
            interrupt_enable: InterruptFlags::default(),
            interrupt_flag: InterruptFlags::default(),
            timer: Timer::new(Frequency::Hz4096),
            vram: [0u8; VRAM_SIZE],
            wram: [0u8; WRAM_SIZE],
            eram: [0u8; ERAM_SIZE],
            oam: [0u8; OAM_SIZE],
            io: [0u8; IO_SIZE],
            hram: [0u8; HRAM_SIZE],
        }
    }

    pub fn step(&mut self, cycles: u16) {
        if self.timer.step(cycles) {
            self.interrupt_flag.timer = true;
        }
        self.divider.step(cycles);
    }

    /// Indicates whether an interrupt is pending
    #[inline]
    pub fn has_interrupt(&self) -> bool {
        (u8::from(self.interrupt_enable) & u8::from(self.interrupt_flag)) != 0
    }

    /// Reads value from boot ROM or cartridge
    /// depending on BOOT_ROM_OFF register
    fn read_cartridge(&self, address: u16) -> u8 {
        match address {
            BOOT_BEGIN..=BOOT_END if self.read(BOOT_ROM_OFF) == 0 => BOOT_ROM[address as usize],
            _ => self.cartridge.read(address),
        }
    }

    /// Initiate DMA transfer
    fn dma_transfer(&mut self, value: u8) {
        let address = u16::from(value) * 100;
        for offset in 0..0xA0 {
            let byte = self.read(address + offset);
            self.write(OAM_BEGIN + offset, byte);
        }
    }

    fn write_io(&mut self, address: u16, value: u8) {
        //println!("write IO: {:#06x}: {:#04x}", address, value);
        match address {
            // Whenever a ROM writes tries to write to it will reset to 0
            TIMER_DIVIDER => self.divider.value = 0,
            TIMER_COUNTER => self.timer.value = value,
            TIMER_MODULO => self.timer.modulo = value,
            // Only the lower 3 bits are R/W
            TIMER_CTRL => {
                self.timer.frequency = match value & 0b11 {
                    0b00 => Frequency::Hz4096,
                    0b01 => Frequency::Hz262144,
                    0b10 => Frequency::Hz65536,
                    0b11 => Frequency::Hz16384,
                    _ => unreachable!(),
                };
                self.timer.on = (value & 0b100) == 0b100;
            }
            INTERRUPT_FLAG => self.interrupt_flag = InterruptFlags::from(value),
            PPU_DMA => self.dma_transfer(value),
            _ => self.io[(address - IO_BEGIN) as usize] = value,
        }
    }

    /// Writes to Echo RAM, effectively mirroring to Working RAM
    fn write_eram(&mut self, address: u16, value: u8) {
        self.eram[(address - ERAM_BEGIN) as usize] = value;
        self.wram[(address - ERAM_SIZE as u16 - WRAM_BEGIN) as usize] = value;
    }

    /// TODO: document unmapped I/O registers
    /// https://gbdev.gg8.se/wiki/articles/CGB_Registers#FF6C_-_Bit_0_.28Read.2FWrite.29_-_CGB_Mode_Only
    fn read_io(&self, address: u16) -> u8 {
        match address {
            0xFF03 => 0xFF,
            TIMER_DIVIDER => self.divider.value,
            TIMER_COUNTER => self.timer.value,
            TIMER_MODULO => self.timer.modulo,
            // Only the lower 3 bits are R/W
            TIMER_CTRL => (self.timer.frequency.as_cycles() & 0b111) as u8,
            0xFF08..=0xFF0E => 0xFF,
            INTERRUPT_FLAG => u8::from(self.interrupt_flag),
            0xFF15 => 0xFF,
            0xFF1F => 0xFF,
            0xFF27..=0xFF2F => 0xFF,
            CGB_PREPARE_SPEED_SWITCH => 0xFF,
            0xFF4E => 0xFF,
            0xFF57..=0xFF5F => 0xFF,
            0xFF6C..=0xFF6F => 0xFF,
            CGB_WRAM_BANK => 0xFF,
            0xFF71..=0xFF75 => 0xFF,
            PCM_AMPLITUDES12 => 0xFF, //TODO: implement me
            PCM_AMPLITUDES34 => 0xFF, //TODO: implement me
            0xFF78..=0xFF7F => 0xFF,
            _ => self.io[(address - IO_BEGIN) as usize],
        }
    }
}

impl AddressSpace for Bus {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.cartridge.write(address, value),
            VRAM_BEGIN..=VRAM_END => self.vram[(address - VRAM_BEGIN) as usize] = value,
            CRAM_BEGIN..=CRAM_END => self.cartridge.write(address, value),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize] = value,
            ERAM_BEGIN..=ERAM_END => self.write_eram(address, value),
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize] = value,
            UNUSED_BEGIN..=UNUSED_END => {}
            IO_BEGIN..=IO_END => self.write_io(address, value),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptFlags::from(value),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.read_cartridge(address),
            VRAM_BEGIN..=VRAM_END => self.vram[(address - VRAM_BEGIN) as usize],
            CRAM_BEGIN..=CRAM_END => self.read_cartridge(address),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize],
            ERAM_BEGIN..=ERAM_END => self.eram[(address - ERAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize],
            UNUSED_BEGIN..=UNUSED_END => 0xFF,
            IO_BEGIN..=IO_END => self.read_io(address),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize],
            INTERRUPT_ENABLE => u8::from(self.interrupt_enable),
        }
    }
}
