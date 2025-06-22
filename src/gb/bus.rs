use crate::gb::audio::AudioProcessor;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::*;
use crate::gb::joypad::Joypad;
use crate::gb::ppu::PPU;
use crate::gb::ppu::display::Display;
use crate::gb::registers::CpuMode;
use crate::gb::serial::SerialTransfer;
use crate::gb::timer::Timer;
use crate::gb::{Bus, EmulatorConfig, HardwareMode, SubSystem};

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
    pub hwmode: HardwareMode,
    pub is_booting: bool,
    apu: AudioProcessor,
    serial: SerialTransfer,
    pub cartridge: Cartridge,
    timer: Timer,
    ppu: PPU,
    pub joypad: Joypad,
    pub interrupt_enable: InterruptRegister,
    pub interrupt_flag: InterruptRegister,
    wram: Vec<u8>,
    hram: [u8; HRAM_SIZE],

    // CGB specific registers
    pub key0: CpuMode,
    pub wram_bank: u8,
}

impl MainBus {
    pub fn with_cartridge(
        cartridge: Cartridge,
        config: EmulatorConfig,
        display: Option<Display>,
    ) -> Self {
        let hwmode = HardwareMode::from(&cartridge.header);
        Self {
            is_booting: true,
            apu: AudioProcessor::default(),
            serial: SerialTransfer::new(config.print_serial),
            ppu: PPU::new(display, hwmode),
            joypad: Joypad::default(),
            interrupt_enable: InterruptRegister::default(),
            interrupt_flag: InterruptRegister::default(),
            timer: Timer::default(),
            hram: [0u8; HRAM_SIZE],
            wram: match hwmode {
                HardwareMode::DMG => vec![0u8; WRAM_BANK_SIZE * 2],
                HardwareMode::CGB => vec![0u8; WRAM_BANK_SIZE * 8],
            },
            key0: CpuMode::default(),
            wram_bank: 1,
            hwmode,
            cartridge,
        }
    }

    /// Updates the configuration of the bus and all connected subsystems.
    #[inline]
    pub fn update_config(&mut self, config: &EmulatorConfig) {
        self.ppu.update_config(config);
        self.serial.update_config(config);
    }

    /// Checks if the OAM DMA transfer is active and transfers the data to the OAM
    fn oam_transfer(&mut self) {
        if let Some(source_address) = self.ppu.r.oam_dma.transfer() {
            let value = self.read(source_address);
            let offset = source_address & 0b1111_1111;
            let target_address = OAM_BEGIN + offset;
            self.ppu.write_oam_dma(target_address, value);
        }
        if let Some(source) = self.ppu.r.oam_dma.pending.take() {
            self.ppu.r.oam_dma.start(source);
        }
        if let Some(source) = self.ppu.r.oam_dma.requested.take() {
            self.ppu.r.oam_dma.pending = Some(source);
        }
    }

    /// Reads value from the boot ROM or cartridge depending on `BOOT_ROM_OFF` register
    /// and hardware mode.
    fn read_cartridge(&mut self, address: u16) -> u8 {
        match address {
            DMG_BOOT_ROM_BEGIN..=DMG_BOOT_ROM_END if self.is_booting && !self.hwmode.is_cgb() => {
                DMG_BOOT_ROM[address as usize]
            }
            CGB_BOOT_ROM_BEGIN_R1..=CGB_BOOT_ROM_END_R1
                if self.is_booting && self.hwmode.is_cgb() =>
            {
                CGB_BOOT_ROM[address as usize]
            }
            CGB_BOOT_ROM_BEGIN_R2..=CGB_BOOT_ROM_END_R2
                if self.is_booting && self.hwmode.is_cgb() =>
            {
                CGB_BOOT_ROM[address as usize]
            }
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
            // This register is only writeable in the CGB boot ROM.
            CGB_KEY0 if self.is_booting && self.hwmode.is_cgb() => {
                self.key0 = CpuMode::from_bits_truncate(value);
            }
            CGB_KEY1_SPEED_SWITCH if self.hwmode.is_cgb() => {
                todo!("implement key1")
            }
            0xFF4E => {} // undocumented
            CGB_VRAM_BANK if self.hwmode.is_cgb() => self.ppu.vram_bank = value & 0b0000_0001,
            BOOT_ROM_OFF if value > 0 => self.is_booting = false,
            CGB_HDMA1_VRAM_DMA_SRC..=CGB_HDMA5_VRAM_DMA_MODE if self.hwmode.is_cgb() => {
                self.ppu.r.vram_dma.write(address, value);
            }
            0xFF56 => {}          // only used in GBC mode
            0xFF57..=0xFF67 => {} // undocumented
            CGB_BCPS if self.hwmode.is_cgb() => self.ppu.write_bg_spec(value),
            CGB_BCPD if self.hwmode.is_cgb() => self.ppu.write_bg_palette(value),
            CGB_OBPS if self.hwmode.is_cgb() => self.ppu.write_obj_spec(value),
            CGB_OBPD if self.hwmode.is_cgb() => self.ppu.write_obj_palette(value),
            0xFF6C..=0xFF6F => {} // only used in GBC mode
            CGB_WRAM_BANK if self.hwmode.is_cgb() => {
                self.wram_bank = match value & 0b0000_0111 {
                    0 => 1,
                    n => n,
                }
            }
            0xFF71..=0xFF75 => {}  // undocumented
            PCM_AMPLITUDES12 => {} // only used in GBC mode
            PCM_AMPLITUDES34 => {} // only used in GBC mode
            0xFF78..=0xFF7F => {}  // undocumented
            _ => {}
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
            INTERRUPT_FLAG => self.interrupt_flag.bits() | 0b1110_0000,
            AUDIO_REGISTERS_START..=AUDIO_REGISTERS_END => self.apu.read(address),
            PPU_REGISTER_START..=PPU_REGISTER_END => self.ppu.read(address),
            CGB_KEY0 if self.hwmode.is_cgb() => self.key0.bits() | 0b1111_1011,
            CGB_KEY1_SPEED_SWITCH if self.hwmode.is_cgb() => todo!("CGB_KEY1 not implemented"),
            0xFF4E => UNDEFINED_READ, // undocumented
            CGB_VRAM_BANK if self.hwmode.is_cgb() => self.ppu.vram_bank | 0b1111_1110,
            BOOT_ROM_OFF => UNDEFINED_READ, // Write-only register
            // Write-only registers
            CGB_HDMA1_VRAM_DMA_SRC..=CGB_HDMA4_VRAM_DMA_DEST => UNDEFINED_READ,
            CGB_HDMA5_VRAM_DMA_MODE if self.hwmode.is_cgb() => self.ppu.r.vram_dma.mode.bits(),
            0xFF56 => UNDEFINED_READ,          // only used in GBC mode
            0xFF57..=0xFF67 => UNDEFINED_READ, // undocumented
            CGB_BCPS if self.hwmode.is_cgb() => self.ppu.bcps.bits() | 0b0100_0000,
            CGB_BCPD if self.hwmode.is_cgb() => self.ppu.read_bg_palette(),
            CGB_OBPS if self.hwmode.is_cgb() => self.ppu.ocps.bits() | 0b0100_0000,
            CGB_OBPD if self.hwmode.is_cgb() => self.ppu.read_obj_palette(),
            0xFF69..=0xFF6F => UNDEFINED_READ, // only used in GBC mode
            CGB_WRAM_BANK if self.hwmode.is_cgb() => self.wram_bank | 0b1111_1000,
            0xFF71..=0xFF75 => UNDEFINED_READ,  // undocumented
            PCM_AMPLITUDES12 => UNDEFINED_READ, // only used in GBC mode
            PCM_AMPLITUDES34 => UNDEFINED_READ, // only used in GBC mode
            0xFF78..=0xFF7F => UNDEFINED_READ,  // undocumented
            _ => UNDEFINED_READ,
        }
    }
}

impl SubSystem for MainBus {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END => self.cartridge.write(address, value),
            VRAM_BEGIN..=VRAM_END => self.ppu.write_vram(address, value),
            CRAM_BANK_BEGIN..=CRAM_BANK_END => self.cartridge.write(address, value),
            WRAM_LOW_BEGIN..=WRAM_LOW_END => self.wram[(address & 0x0FFF) as usize] = value,
            WRAM_HIGH_BEGIN..=WRAM_HIGH_END => {
                let offset = self.wram_bank as usize * WRAM_BANK_SIZE;
                self.wram[offset + (address & 0x0FFF) as usize] = value;
            }
            // Writes to Echo RAM, effectively mirroring to Working RAM
            ERAM_LOW_BEGIN..=ERAM_LOW_END => self.wram[(address & 0x0FFF) as usize] = value,
            ERAM_HIGH_BEGIN..=ERAM_HIGH_END => {
                let offset = self.wram_bank as usize * WRAM_BANK_SIZE;
                self.wram[offset + (address & 0x0DFF) as usize] = value;
            }
            OAM_BEGIN..=OAM_END => self.ppu.write_oam(address, value),
            UNUSED_BEGIN..=UNUSED_END => {}
            IO_BEGIN..=IO_END => self.write_io(address, value),
            HRAM_BEGIN..=HRAM_END => self.hram[(address - HRAM_BEGIN) as usize] = value,
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END => self.read_cartridge(address),
            VRAM_BEGIN..=VRAM_END => self.ppu.read_vram(address),
            CRAM_BANK_BEGIN..=CRAM_BANK_END => self.read_cartridge(address),
            WRAM_LOW_BEGIN..=WRAM_LOW_END => self.wram[(address & 0x0FFF) as usize],
            WRAM_HIGH_BEGIN..=WRAM_HIGH_END => {
                let offset = self.wram_bank as usize * WRAM_BANK_SIZE;
                self.wram[offset + (address & 0x0FFF) as usize]
            }
            // Reads from Echo RAM, effectively mirroring to Working RAM
            ERAM_LOW_BEGIN..=ERAM_LOW_END => self.wram[(address & 0x0FFF) as usize],
            ERAM_HIGH_BEGIN..=ERAM_HIGH_END => {
                let offset = self.wram_bank as usize * WRAM_BANK_SIZE;
                self.wram[offset + (address & 0x0DFF) as usize]
            }
            OAM_BEGIN..=OAM_END => self.ppu.read_oam(address),
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
        self.ppu.cycle(&mut self.interrupt_flag);
        self.timer.cycle(&mut self.interrupt_flag);
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
