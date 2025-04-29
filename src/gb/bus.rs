use crate::gb::audio::AudioProcessor;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::*;
use crate::gb::cpu::ImeState;
use crate::gb::interrupt::InterruptRegister;
use crate::gb::joypad::{Joypad, JoypadInput};
use crate::gb::ppu::PPU;
use crate::gb::ppu::display::Display;
use crate::gb::timer::Timer;
use crate::gb::{AddressSpace, HardwareContext};

/// Defines a global Bus, all processing units should access memory through it.
#[derive(Clone)]
pub struct Bus {
    pub is_boot_rom_active: bool,
    audio_processor: AudioProcessor,
    pub cartridge: Cartridge,
    timer: Timer,
    ppu: PPU,
    joypad: Joypad,
    pending_joypad_event: Option<JoypadInput>,
    ime: ImeState, // Interrupt Master Enable
    pub interrupt_enable: InterruptRegister,
    pub interrupt_flag: InterruptRegister,
    wram: [u8; WRAM_SIZE],
    eram: [u8; ERAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl Bus {
    pub fn with_cartridge(cartridge: Cartridge, display: Display) -> Self {
        Self {
            cartridge,
            is_boot_rom_active: true,
            audio_processor: AudioProcessor::default(),
            ppu: PPU::with_display(display),
            joypad: Joypad::default(),
            pending_joypad_event: None,
            ime: ImeState::Enabled,
            interrupt_enable: InterruptRegister::empty(),
            interrupt_flag: InterruptRegister::empty(),
            timer: Timer::default(),
            wram: [0u8; WRAM_SIZE],
            eram: [0u8; ERAM_SIZE],
            hram: [0u8; HRAM_SIZE],
        }
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
    fn read_cartridge(&mut self, address: u16) -> u8 {
        match address {
            BOOT_BEGIN..=BOOT_END if self.is_boot_rom_active => BOOT_ROM[address as usize],
            _ => self.cartridge.read(address),
        }
    }

    /// Initiate DMA transfer, the passed value specifies the upper half of the source address.
    /// See https://gbdev.io/pandocs/OAM_DMA_Transfer.html
    #[inline]
    fn dma_transfer(&mut self, value: u8) {
        self.ppu.r.dma = value;
        let address = u16::from(value) << 8;
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
                    self.interrupt_flag.insert(InterruptRegister::JOYPAD);
                    self.pending_joypad_event = None;
                }
            }
            SERIAL_TRANSFER_DATA => {} // TODO: implement me
            SERIAL_TRANSFER_CTRL => {} // TODO: implement me
            // undocumented
            0xFF03 => {}
            // Whenever a ROM writes to this register it will reset to 0
            TIMER_DIVIDER..=TIMER_CTRL => self.timer.write(address, value),
            // undocumented
            0xFF08..=0xFF0E => {}
            INTERRUPT_FLAG => self.interrupt_flag = InterruptRegister::from_bits_retain(value),
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => {
                self.audio_processor.write(address, value)
            }
            // TODO: consider moving dma_transfer to the PPU, for now it lives here because we need
            //  to access the ROM
            PPU_DMA => self.dma_transfer(value),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.write(address, value),
            0xFF4C => {}                   // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => {} // only used in GBC mode
            0xFF4E => {}                   // undocumented
            0xFF4F => {}                   // only used in GBC mode
            BOOT_ROM_OFF => {
                if value > 0 {
                    self.is_boot_rom_active = false
                }
            }
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
    fn read_io(&mut self, address: u16) -> u8 {
        match address {
            JOYPAD => self.joypad.read(),
            SERIAL_TRANSFER_DATA => 0x00,        // TODO: implement me
            SERIAL_TRANSFER_CTRL => 0b0111_1110, // TODO: implement me
            0xFF03 => 0xFF,                      // undocumented
            TIMER_DIVIDER..=TIMER_CTRL => self.timer.read(address),
            0xFF08..=0xFF0E => 0xFF, // undocumented
            // Undocumented bits should be 1
            INTERRUPT_FLAG => self.interrupt_flag.bits() | 0b1110_0000,
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => self.audio_processor.read(address),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),
            0xFF4C => 0xFF,                   // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => 0xFF, // only used in GBC mode
            0xFF4E => 0xFF,                   // undocumented
            0xFF4F => 0xFF,                   // only used in GBC mode
            // When read, this register is always 0xFF
            BOOT_ROM_OFF => 0xFF,
            0xFF51..=0xFF56 => 0xFF,  // only used in GBC mode
            0xFF57..=0xFF67 => 0xFF,  // undocumented
            0xFF68..=0xFF6F => 0xFF,  // only used in GBC mode
            CGB_WRAM_BANK => 0xFF,    // only used in GBC mode
            0xFF71..=0xFF75 => 0xFF,  // undocumented
            PCM_AMPLITUDES12 => 0xFF, // only used in GBC mode
            PCM_AMPLITUDES34 => 0xFF, // only used in GBC mode
            0xFF78..=0xFF7F => 0xFF,  // undocumented
            _ => panic!(
                "Attempt to read from unmapped I/O register: 0x{:X}",
                address
            ),
        }
    }

    /// This function is used to step all components.
    #[inline]
    fn cycle(&mut self) {
        if self.ime == ImeState::Pending {
            self.ime = ImeState::Enabled;
        }
        self.ppu.step(&mut self.interrupt_flag);
        self.timer.step(&mut self.interrupt_flag);
    }

    /// TODO: this is a hacky solution to avoid stepping the components when writing while debugging
    pub fn write_raw(&mut self, address: u16, value: u8) {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.cartridge.write(address, value),
            VRAM_BEGIN..=VRAM_END => self.ppu.write(address, value),
            CRAM_BEGIN..=CRAM_END => self.cartridge.write(address, value),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize] = value,
            ERAM_BEGIN..=ERAM_END => self.write_eram(address, value),
            OAM_BEGIN..=OAM_END => self.ppu.write(address, value),
            UNUSED_BEGIN..=UNUSED_END => {}
            IO_BEGIN..=IO_END => self.write_io(address, value),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
        }
    }

    /// TODO: this is a hacky solution to avoid stepping the components when reading while debugging
    pub fn read_raw(&mut self, address: u16) -> u8 {
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => self.read_cartridge(address),
            VRAM_BEGIN..=VRAM_END => self.ppu.read(address),
            CRAM_BEGIN..=CRAM_END => self.read_cartridge(address),
            WRAM_BEGIN..=WRAM_END => self.wram[(address - WRAM_BEGIN) as usize],
            ERAM_BEGIN..=ERAM_END => self.eram[(address - ERAM_BEGIN) as usize],
            OAM_BEGIN..=OAM_END => self.ppu.read(address),
            UNUSED_BEGIN..=UNUSED_END => 0xFF,
            IO_BEGIN..=IO_END => self.read_io(address),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize],
            INTERRUPT_ENABLE => self.interrupt_enable.bits(),
        }
    }
}

impl AddressSpace for Bus {
    fn write(&mut self, address: u16, value: u8) {
        self.cycle();
        self.write_raw(address, value);
    }

    fn read(&mut self, address: u16) -> u8 {
        self.cycle();
        self.read_raw(address)
    }
}

impl HardwareContext for Bus {
    #[inline]
    fn set_ime(&mut self, ime: ImeState) {
        self.ime = ime;
    }

    #[inline]
    fn ime(&self) -> ImeState {
        self.ime
    }

    #[inline]
    fn tick(&mut self) {
        self.cycle();
    }
}
