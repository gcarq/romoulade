/// When the Game Boy first boot's the very bottom 256 bytes
/// of memory is occupied with the boot ROM.
pub const BOOT_BEGIN: u16 = 0x0000;
pub const BOOT_END: u16 = 0x00FF;
pub const BOOT_SIZE: usize = (BOOT_END - BOOT_BEGIN + 1) as usize;

/// This area of memory always contains the first bank from the cartridge.
pub const ROM_LOW_BANK_BEGIN: u16 = 0x0000;
pub const ROM_LOW_BANK_END: u16 = 0x3FFF;

/// This area of memory contains a switchable bank from the cartridge (01..nn).
/// Writing to this area of memory changes the currently selected bank.
pub const ROM_HIGH_BANK_BEGIN: u16 = 0x4000;
pub const ROM_HIGH_BANK_END: u16 = 0x7FFF;

/// This area of memory contains data about the graphics that can be displayed to the screen.
/// The Game Boy uses a tiling system for graphics meaning that a game doesn't control
/// the specific pixels that get drawn to the screen, at least not directly.
pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = (VRAM_END - VRAM_BEGIN + 1) as usize;

/// Cartridges (being physical devices) sometimes had extra RAM on them.
/// This gave games even more memory to work with.
/// If the cartridge had this extra RAM the Game Boy
/// automatically mapped the RAM into this area of memory.
pub const CRAM_BANK_BEGIN: u16 = 0xA000;
pub const CRAM_BANK_END: u16 = 0xBFFF;

/// This is the RAM that the Game Boy allows a game to use.
/// Our idea of RAM really just being a plain old array where the game could
/// read and write bytes to really only applies to this section of memory.
pub const WRAM_BEGIN: u16 = 0xC000;
pub const WRAM_END: u16 = 0xDFFF;
pub const WRAM_SIZE: usize = (WRAM_END - WRAM_BEGIN + 1) as usize;

/// This section of memory directly mirrors the working RAM section - meaning
/// if you write into the first address of working RAM (0xC000),
/// the same value will appear in the first spot of echo RAM (0xE000).
/// Nintendo actively discouraged developers from using this area of
/// memory and as such we can just pretend it doesn't exist.
pub const ERAM_BEGIN: u16 = 0xE000;
pub const ERAM_END: u16 = 0xFDFF;
pub const ERAM_SIZE: usize = (ERAM_END - ERAM_BEGIN + 1) as usize;

/// This area of memory contains the description of graphical sprites.
/// The tiles we talked about above were used for backgrounds and levels but not for characters,
/// enemies or objects the user interacted with. These entities, known as sprites,
/// have extra capabilities. The description for how they should look lives here.
pub const OAM_BEGIN: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_SIZE: usize = (OAM_END - OAM_BEGIN + 1) as usize;

/// This area is unmapped: reading from it just returns 0xFF
pub const UNUSED_BEGIN: u16 = 0xFEA0;
pub const UNUSED_END: u16 = 0xFEFF;

/// This is one of the most dense areas of memory.
/// Practically every byte has a special meaning.
/// It's used by both the screen and the sound system to determine different settings.
/// We'll be talking a lot about this in the future.
pub const IO_BEGIN: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;

/// Pixel Processing Unit Registers area
pub const PPU_REGISTER_START: u16 = 0xFF40;
pub const PPU_REGISTER_END: u16 = 0xFF4B;

/// Audio Registers area
pub const AUDIO_REGISTERS_START: u16 = 0xFF10;
pub const AUDIO_REGISTERS_END: u16 = 0xFF3F;
pub const AUDIO_REGISTERS_SIZE: usize = (AUDIO_REGISTERS_END - AUDIO_REGISTERS_START + 1) as usize;

/// This area is also just normal RAM but is used a lot because some of the LD instructions
/// we've already seen can easily target this area in memory.
/// This area is also sometimes used for the stack,
/// but the top of working RAM is also used for this purpose.
pub const HRAM_BEGIN: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const HRAM_SIZE: usize = (HRAM_END - HRAM_BEGIN + 1) as usize;

/// Joypad Input Register
pub const JOYPAD: u16 = 0xFF00;

pub const SERIAL_TRANSFER_DATA: u16 = 0xFF01;
pub const SERIAL_TRANSFER_CTRL: u16 = 0xFF02;

// Timer Registers
/// Counts up at a fixed 16384Hz rate, resets to 0 whenever written to (DIV).
pub const TIMER_DIVIDER: u16 = 0xFF04;

/// Counts up at a specified rate. Triggers INT (0x50) when overflows (TIMA).
pub const TIMER_COUNTER: u16 = 0xFF05;

/// When counter overflows, it's reset to start at modulo (TMA).
pub const TIMER_MODULO: u16 = 0xFF06;

/// Timer Controller (TAC) which uses 3 bits,
/// bit 2 specifies whether the timer is enabled (1) or disabled (0).
pub const TIMER_CTRL: u16 = 0xFF07;

/// This register is used to prepare the Game Boy to switch
/// between CGB Double Speed Mode and Normal Speed Mode.
/// The actual speed switch is performed by executing a STOP command after Bit 0 has been set.
/// After that Bit 0 will be cleared automatically, and the Game Boy will operate at the 'other' speed.
pub const CGB_PREPARE_SPEED_SWITCH: u16 = 0xFF4D;

/// In CGB Mode 32 KBytes internal RAM are available.
/// This memory is divided into 8 banks of 4 KBytes each.
/// Bank 0 is always available in memory at C000-CFFF,
/// Bank 1-7 can be selected into the address space at D000-DFFF.
pub const CGB_WRAM_BANK: u16 = 0xFF70;

/// Boot ROM lock bit
/// 0b1 = Boot ROM is disabled and 0x0000-0x00FF works normally
/// 0b0 = Boot Rom is active and intercepts access to 0x0000-0x00FF
/// Can only transition from 0b0 to 0b1, so once 0b1 has written,
/// the boot ROM is permanently disabled until the next system reset.
pub const BOOT_ROM_OFF: u16 = 0xFF50;

/// Those two registers are read-only.
/// The low nibble is a copy of sound channel #1's PCM amplitude,
/// the high nibble a copy of sound channel #2's.
pub const PCM_AMPLITUDES12: u16 = 0xFF76;
pub const PCM_AMPLITUDES34: u16 = 0xFF77;

// Interrupt Controller Registers
pub const INTERRUPT_FLAG: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;

/// This defines the default value when reading from an undefined memory address,
/// or when reading from a memory region that is currently not readable.
pub const UNDEFINED_READ: u8 = 0xFF;

/// Contains the DMG Bootstrap ROM,
/// disassembled code is outlined here: https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM.
pub(crate) const BOOT_ROM: [u8; BOOT_SIZE] = [
    0x31, 0xFE, 0xFF, 0xAF, 0x21, 0xFF, 0x9F, 0x32, 0xCB, 0x7C, 0x20, 0xFB, 0x21, 0x26, 0xFF, 0x0E,
    0x11, 0x3E, 0x80, 0x32, 0xE2, 0x0C, 0x3E, 0xF3, 0xE2, 0x32, 0x3E, 0x77, 0x77, 0x3E, 0xFC, 0xE0,
    0x47, 0x11, 0x04, 0x01, 0x21, 0x10, 0x80, 0x1A, 0xCD, 0x95, 0x00, 0xCD, 0x96, 0x00, 0x13, 0x7B,
    0xFE, 0x34, 0x20, 0xF3, 0x11, 0xD8, 0x00, 0x06, 0x08, 0x1A, 0x13, 0x22, 0x23, 0x05, 0x20, 0xF9,
    0x3E, 0x19, 0xEA, 0x10, 0x99, 0x21, 0x2F, 0x99, 0x0E, 0x0C, 0x3D, 0x28, 0x08, 0x32, 0x0D, 0x20,
    0xF9, 0x2E, 0x0F, 0x18, 0xF3, 0x67, 0x3E, 0x64, 0x57, 0xE0, 0x42, 0x3E, 0x91, 0xE0, 0x40, 0x04,
    0x1E, 0x02, 0x0E, 0x0C, 0xF0, 0x44, 0xFE, 0x90, 0x20, 0xFA, 0x0D, 0x20, 0xF7, 0x1D, 0x20, 0xF2,
    0x0E, 0x13, 0x24, 0x7C, 0x1E, 0x83, 0xFE, 0x62, 0x28, 0x06, 0x1E, 0xC1, 0xFE, 0x64, 0x20, 0x06,
    0x7B, 0xE2, 0x0C, 0x3E, 0x87, 0xE2, 0xF0, 0x42, 0x90, 0xE0, 0x42, 0x15, 0x20, 0xD2, 0x05, 0x20,
    0x4F, 0x16, 0x20, 0x18, 0xCB, 0x4F, 0x06, 0x04, 0xC5, 0xCB, 0x11, 0x17, 0xC1, 0xCB, 0x11, 0x17,
    0x05, 0x20, 0xF5, 0x22, 0x23, 0x22, 0x23, 0xC9, 0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B,
    0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D, 0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E,
    0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99, 0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC,
    0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E, 0x3C, 0x42, 0xB9, 0xA5, 0xB9, 0xA5, 0x42, 0x3C,
    0x21, 0x04, 0x01, 0x11, 0xA8, 0x00, 0x1A, 0x13, 0xBE, 0x00, 0x00, 0x23, 0x7D, 0xFE, 0x34, 0x20,
    0xF5, 0x06, 0x19, 0x78, 0x86, 0x23, 0x05, 0x20, 0xFB, 0x86, 0x00, 0x00, 0x3E, 0x01, 0xE0, 0x50,
];
