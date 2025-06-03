use crate::gb::audio::AudioProcessor;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::*;
use crate::gb::joypad::Joypad;
use crate::gb::ppu::PPU;
use crate::gb::ppu::display::Display;
use crate::gb::serial::SerialTransfer;
use crate::gb::timer::Timer;
use crate::gb::{Bus, EmulatorConfig, SubSystem};

bitflags! {
    /// Represents interrupt registers IE at 0xFFFF and IF at 0xFF0F
    #[derive(Copy, Clone, PartialEq, Debug, Default)]
    pub struct InterruptRegister: u8 {
        const VBLANK = 0b00000001; // V-Blank Interrupt
        const STAT   = 0b00000010; // LCD STAT Interrupt
        const TIMER  = 0b00000100; // Timer Overflow Interrupt
        const SERIAL = 0b00001000; // Serial Transfer Completion Interrupt
        const JOYPAD = 0b00010000; // Joypad Input Interrupt
    }
}

impl InterruptRegister {
    /// Returns the interrupt with the highest priority.
    #[inline]
    pub fn highest_prio(&self) -> Option<InterruptRegister> {
        self.iter_names().map(|(_, irq)| irq).next()
    }
}

/// Defines a global Bus, all processing units should access memory through it.
#[derive(Clone)]
pub struct MainBus {
    pub is_boot_rom_active: bool,
    apu: AudioProcessor,
    serial: SerialTransfer,
    pub cartridge: Cartridge,
    timer: Timer,
    ppu: PPU,
    pub joypad: Joypad,
    pub interrupt_enable: InterruptRegister,
    pub interrupt_flag: InterruptRegister,
    wram: [u8; WRAM_SIZE],
    hram: [u8; HRAM_SIZE],
}

impl MainBus {
    pub fn with_cartridge(
        cartridge: Cartridge,
        config: EmulatorConfig,
        display: Option<Display>,
    ) -> Self {
        Self {
            cartridge,
            is_boot_rom_active: true,
            apu: AudioProcessor::default(),
            serial: SerialTransfer::new(config.print_serial),
            ppu: PPU::new(display),
            joypad: Joypad::default(),
            interrupt_enable: InterruptRegister::default(),
            interrupt_flag: InterruptRegister::default(),
            timer: Timer::default(),
            wram: [0u8; WRAM_SIZE],
            hram: [0u8; HRAM_SIZE],
        }
    }

    /// Updates the configuration of the bus and all connected subsystems.
    #[inline]
    pub fn update_config(&mut self, config: &EmulatorConfig) {
        self.ppu.update_config(config);
        self.serial.update_config(config);
    }

    /// Reads value from boot ROM or cartridge
    /// depending on `BOOT_ROM_OFF` register
    fn read_cartridge(&mut self, address: u16) -> u8 {
        match address {
            BOOT_BEGIN..=BOOT_END if self.is_boot_rom_active => BOOT_ROM[address as usize],
            _ => self.cartridge.read(address),
        }
    }

    /// Handles all writes to the I/O registers (0xFF00-0xFF7F)
    fn write_io(&mut self, address: u16, value: u8) {
        match address {
            JOYPAD => self.joypad.write(value, &mut self.interrupt_flag),
            SERIAL_TRANSFER_DATA => self.serial.write(address, value),
            SERIAL_TRANSFER_CTRL => self.serial.write(address, value),
            // undocumented
            0xFF03 => {}
            // Whenever a ROM writes to this register it will reset to 0
            TIMER_DIVIDER..=TIMER_CTRL => self.timer.write(address, value),
            // undocumented
            0xFF08..=0xFF0E => {}
            INTERRUPT_FLAG => self.interrupt_flag = InterruptRegister::from_bits_retain(value),
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => self.apu.write(address, value),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.write(address, value),
            0xFF4C => {}                   // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => {} // only used in GBC mode
            0xFF4E => {}                   // undocumented
            0xFF4F => {}                   // only used in GBC mode
            BOOT_ROM_OFF => {
                if value > 0 {
                    self.is_boot_rom_active = false;
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
            _ => panic!("Attempt to write to unmapped I/O register: {address:#06x}"),
        }
    }

    /// Handles all reads from the I/O registers (0xFF00-0xFF7F)
    fn read_io(&mut self, address: u16) -> u8 {
        match address {
            JOYPAD => self.joypad.read(),
            SERIAL_TRANSFER_DATA => self.serial.read(address),
            SERIAL_TRANSFER_CTRL => self.serial.read(address),
            0xFF03 => UNDEFINED_READ, // undocumented
            TIMER_DIVIDER..=TIMER_CTRL => self.timer.read(address),
            0xFF08..=0xFF0E => UNDEFINED_READ, // undocumented
            // Undocumented bits should be 1
            INTERRUPT_FLAG => self.interrupt_flag.bits() | 0b1110_0000,
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => self.apu.read(address),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),
            0xFF4C => UNDEFINED_READ, // only used in GBC mode
            CGB_PREPARE_SPEED_SWITCH => UNDEFINED_READ, // only used in GBC mode
            0xFF4E => UNDEFINED_READ, // undocumented
            0xFF4F => UNDEFINED_READ, // only used in GBC mode
            // When read, this register is always 0xFF
            BOOT_ROM_OFF => UNDEFINED_READ,
            0xFF51..=0xFF56 => UNDEFINED_READ, // only used in GBC mode
            0xFF57..=0xFF67 => UNDEFINED_READ, // undocumented
            0xFF68..=0xFF6F => UNDEFINED_READ, // only used in GBC mode
            CGB_WRAM_BANK => UNDEFINED_READ,   // only used in GBC mode
            0xFF71..=0xFF75 => UNDEFINED_READ, // undocumented
            PCM_AMPLITUDES12 => UNDEFINED_READ, // only used in GBC mode
            PCM_AMPLITUDES34 => UNDEFINED_READ, // only used in GBC mode
            0xFF78..=0xFF7F => UNDEFINED_READ, // undocumented
            _ => panic!("Attempt to read from unmapped I/O register: {address:#06X}"),
        }
    }

    /// Checks if the OAM DMA transfer is active and transfers the data to the OAM
    fn oam_transfer(&mut self) {
        if let Some(source_address) = self.ppu.r.oam_dma.transfer() {
            let value = self.read(source_address);
            let offset = source_address & 0b1111_1111;
            let target_address = OAM_BEGIN + offset;
            self.ppu.write_oam(target_address, value, true);
        }
        if let Some(source) = self.ppu.r.oam_dma.pending.take() {
            self.ppu.r.oam_dma.start(source);
        }
        if let Some(source) = self.ppu.r.oam_dma.requested.take() {
            self.ppu.r.oam_dma.pending = Some(source);
        }
    }
}

impl SubSystem for MainBus {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END => self.cartridge.write(address, value),
            VRAM_BEGIN..=VRAM_END => self.ppu.write(address, value),
            CRAM_BANK_BEGIN..=CRAM_BANK_END => self.cartridge.write(address, value),
            WRAM_BEGIN..=WRAM_END => self.wram[(address & 0x1FFF) as usize] = value,
            // Writes to Echo RAM, effectively mirroring to Working RAM
            ERAM_BEGIN..=ERAM_END => self.wram[(address & 0x1FFF) as usize] = value,
            // Writes during OAM DMA transfer are ignored
            OAM_BEGIN..=OAM_END => {
                if !self.ppu.r.oam_dma.is_running {
                    self.ppu.write(address, value);
                }
            }
            UNUSED_BEGIN..=UNUSED_END => {}
            IO_BEGIN..=IO_END => self.write_io(address, value),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END => self.read_cartridge(address),
            VRAM_BEGIN..=VRAM_END => self.ppu.read(address),
            CRAM_BANK_BEGIN..=CRAM_BANK_END => self.read_cartridge(address),
            WRAM_BEGIN..=WRAM_END => self.wram[(address & 0x1FFF) as usize],
            // Reads from Echo RAM, effectively mirroring to Working RAM
            ERAM_BEGIN..=ERAM_END => self.wram[(address & 0x1FFF) as usize],
            OAM_BEGIN..=OAM_END => match self.ppu.r.oam_dma.is_running {
                // During OAM DMA transfer the OAM is not accessible
                true => UNDEFINED_READ,
                false => self.ppu.read(address),
            },
            UNUSED_BEGIN..=UNUSED_END => UNDEFINED_READ,
            IO_BEGIN..=IO_END => self.read_io(address),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize],
            INTERRUPT_ENABLE => self.interrupt_enable.bits(),
        }
    }
}

impl Bus for MainBus {
    #[inline]
    fn cycle(&mut self) {
        self.oam_transfer();
        self.ppu.step(&mut self.interrupt_flag);
        self.timer.step(&mut self.interrupt_flag);
    }

    #[inline(always)]
    fn has_irq(&self) -> bool {
        let enabled = self.interrupt_enable.bits() & 0b0001_1111;
        let flag = self.interrupt_flag.bits() & 0b0001_1111;
        enabled & flag != 0
    }

    #[inline(always)]
    fn set_ie(&mut self, r: InterruptRegister) {
        self.interrupt_enable = r;
    }

    #[inline(always)]
    fn get_ie(&self) -> InterruptRegister {
        self.interrupt_enable
    }

    #[inline(always)]
    fn set_if(&mut self, r: InterruptRegister) {
        self.interrupt_flag = r;
    }

    #[inline(always)]
    fn get_if(&self) -> InterruptRegister {
        self.interrupt_flag
    }
}
