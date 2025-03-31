use crate::gb::AddressSpace;
use crate::gb::audio::AudioProcessor;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::*;
use crate::gb::interrupt::InterruptRegister;
use crate::gb::joypad::{Joypad, JoypadInput};
use crate::gb::ppu::PPU;
use crate::gb::ppu::display::Display;
use crate::gb::timer::{Frequency, Timer};

/// Defines a global Bus, all processing units should access memory through it.
pub struct Bus {
    boot_rom_off: u8,
    audio_processor: AudioProcessor,
    cartridge: Cartridge,
    timer: Timer,
    divider: Timer,
    ppu: PPU,
    joypad: Joypad,
    pending_joypad_event: Option<JoypadInput>,
    pub interrupt_enable: InterruptRegister,
    pub interrupt_flag: InterruptRegister,
    wram: [u8; WRAM_SIZE],
    eram: [u8; ERAM_SIZE],
    oam: [u8; OAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl Bus {
    pub fn new(cartridge: Cartridge, display: Display) -> Self {
        let mut divider = Timer::new(Frequency::Hz16384);
        divider.on = true;
        divider.value = 0xAB;
        Self {
            boot_rom_off: 0,
            audio_processor: AudioProcessor::default(),
            cartridge,
            divider,
            ppu: PPU::new(display),
            joypad: Joypad::default(),
            pending_joypad_event: None,
            interrupt_enable: InterruptRegister::empty(),
            interrupt_flag: InterruptRegister::empty(),
            timer: Timer::new(Frequency::Hz4096),
            wram: [0u8; WRAM_SIZE],
            eram: [0u8; ERAM_SIZE],
            oam: [0u8; OAM_SIZE],
            hram: [0u8; HRAM_SIZE],
        }
    }

    pub fn step(&mut self, cycles: u16) {
        if self.ppu.step(cycles) {
            self.interrupt_flag |= InterruptRegister::LCD;
        }
        if self.timer.step(cycles) {
            self.interrupt_flag |= InterruptRegister::TIMER;
        }
        self.divider.step(cycles);
    }

    /// Indicates whether an interrupt is pending
    #[inline]
    pub fn has_pending_interrupt(&self) -> bool {
        let enabled = self.interrupt_enable.bits() & 0b0001_1111;
        let flag = self.interrupt_flag.bits() & 0b0001_1111;
        enabled & flag != 0
    }

    #[inline]
    pub fn handle_joypad_event(&mut self, input: JoypadInput) {
        self.pending_joypad_event = Some(input);
    }

    /// Reads value from boot ROM or cartridge
    /// depending on BOOT_ROM_OFF register
    fn read_cartridge(&self, address: u16) -> u8 {
        match address {
            BOOT_BEGIN..=BOOT_END if self.boot_rom_off == 0 => BOOT_ROM[address as usize],
            _ => self.cartridge.read(address),
        }
    }

    /// Initiate DMA transfer
    /// See https://gbdev.io/pandocs/OAM_DMA_Transfer.html
    #[inline]
    fn dma_transfer(&mut self, value: u8) {
        let address = u16::from(value) * 100;
        for offset in 0..0xA0 {
            let byte = self.read(address + offset);
            self.write(OAM_BEGIN + offset, byte);
        }
    }

    /// Writes to Echo RAM, effectively mirroring to Working RAM
    #[inline]
    fn write_eram(&mut self, address: u16, value: u8) {
        self.eram[(address - ERAM_BEGIN) as usize] = value;
        self.wram[(address - ERAM_SIZE as u16 - WRAM_BEGIN) as usize] = value;
    }

    /// Handles all writes to the I/O registers (0xFF00-0xFF7F)
    fn write_io(&mut self, address: u16, value: u8) {
        match address {
            // Whenever a ROM writes to this register we will handle the pending input events
            JOYPAD => {
                if self.joypad.write(value, self.pending_joypad_event) {
                    self.interrupt_flag |= InterruptRegister::JOYPAD;
                    self.pending_joypad_event = None;
                }
            }
            SERIAL_TRANSFER_DATA => {} // TODO: implement me
            SERIAL_TRANSFER_CTRL => {} // TODO: implement me
            0xFF03 => {}               // undocumented
            // Whenever a ROM writes to this register it will reset to 0
            TIMER_DIVIDER => self.divider.value = 0x00,
            TIMER_COUNTER => self.timer.value = value,
            TIMER_MODULO => self.timer.modulo = value,
            // Only the lower 3 bits are R/W
            TIMER_CTRL => self.timer.write_control(value),
            0xFF08..=0xFF0E => {} // undocumented
            INTERRUPT_FLAG => self.interrupt_flag = InterruptRegister::from_bits_retain(value),
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => {
                self.audio_processor.write(address, value)
            }
            PPU_DMA => self.dma_transfer(value), // TODO: move to PPU?
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.write(address, value),
            0xFF4C => {}                   // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => {} // only used in GBC mode
            0xFF4E => {}                   // undocumented
            0xFF4F => {}                   // only used in GBC mode
            BOOT_ROM_OFF => self.boot_rom_off = value,
            0xFF51..=0xFF56 => {}  // only used in GBC mode
            0xFF57..=0xFF67 => {}  // undocumented
            0xFF68..=0xFF6F => {}  // only used in GBC mode
            CGB_WRAM_BANK => {}    // only used in GBC mode
            0xFF71..=0xFF75 => {}  // undocumented
            PCM_AMPLITUDES12 => {} // only used in GBC mode
            PCM_AMPLITUDES34 => {} // only used in GBC mode
            0xFF78..=0xFF7F => {}  // undocumented
            _ => panic!("Attempt to write to unmapped I/O register: 0x{:X}", address),
        }
    }

    /// Handles all reads from the I/O registers (0xFF00-0xFF7F)
    fn read_io(&self, address: u16) -> u8 {
        match address {
            JOYPAD => self.joypad.read(),
            SERIAL_TRANSFER_DATA => 0x00,        // TODO: implement me
            SERIAL_TRANSFER_CTRL => 0b0111_1110, // TODO: implement me
            0xFF03 => 0xFF,                      // undocumented
            TIMER_DIVIDER => self.divider.value,
            TIMER_COUNTER => self.timer.value,
            TIMER_MODULO => self.timer.modulo,
            TIMER_CTRL => self.timer.read_control(),
            0xFF08..=0xFF0E => 0xFF, // undocumented
            INTERRUPT_FLAG => self.interrupt_flag.bits() | 0b1110_0000, // Undocumented bits should be 1
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => self.audio_processor.read(address),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),
            0xFF4C => 0xFF,                   // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => 0xFF, // only used in GBC mode
            0xFF4E => 0xFF,                   // undocumented
            0xFF4F => 0xFF,                   // only used in GBC mode
            BOOT_ROM_OFF => 0xFF,             // When read, this register is always 0xFF
            0xFF51..=0xFF56 => 0xFF,          // only used in GBC mode
            0xFF57..=0xFF67 => 0xFF,          // undocumented
            0xFF68..=0xFF6F => 0xFF,          // only used in GBC mode
            CGB_WRAM_BANK => 0xFF,            // only used in GBC mode
            0xFF71..=0xFF75 => 0xFF,          // undocumented
            PCM_AMPLITUDES12 => 0xFF,         // only used in GBC mode
            PCM_AMPLITUDES34 => 0xFF,         // only used in GBC mode
            0xFF78..=0xFF7F => 0xFF,          // undocumented
            _ => panic!(
                "Attempt to read from unmapped I/O register: 0x{:X}",
                address
            ),
        }
    }
}

impl AddressSpace for Bus {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.cartridge.write(address, value),
            VRAM_BEGIN..=VRAM_END => self.ppu.write(address, value),
            CRAM_BEGIN..=CRAM_END => self.cartridge.write(address, value),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize] = value,
            ERAM_BEGIN..=ERAM_END => self.write_eram(address, value),
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize] = value,
            UNUSED_BEGIN..=UNUSED_END => {}
            IO_BEGIN..=IO_END => self.write_io(address, value),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
        }
    }

    fn read(&self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.read_cartridge(address),
            VRAM_BEGIN..=VRAM_END => self.ppu.read(address),
            CRAM_BEGIN..=CRAM_END => self.read_cartridge(address),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize],
            ERAM_BEGIN..=ERAM_END => self.eram[(address - ERAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.oam[(address - OAM_BEGIN) as usize],
            UNUSED_BEGIN..=UNUSED_END => 0xFF,
            IO_BEGIN..=IO_END => self.read_io(address),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize],
            INTERRUPT_ENABLE => self.interrupt_enable.bits(),
        }
    }
}
