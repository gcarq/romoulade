/// When the Game Boy first boot's the very bottom 256 bytes
/// of memory is occuppied with the boot ROM.
pub const BOOT_BEGIN: u16 = 0x0000;
pub const BOOT_END: u16 = 0x00FF;
pub const BOOT_SIZE: usize = (BOOT_END - BOOT_BEGIN + 1) as usize;

/// This area of memory contains data about the graphics that can be displayed to the screen.
/// The Game Boy uses a tiling system for grapics meaning that a game doesn't control
/// the specific pixels that get drawn to the screen, at least not directly.
pub const VRAM_BEGIN: u16 = 0x8000;
pub const VRAM_END: u16 = 0x9FFF;
pub const VRAM_SIZE: usize = (VRAM_END - VRAM_BEGIN + 1) as usize;

/// Cartridges (being physical devices) sometimes had extra RAM on them.
/// This gave games even more memory to work with.
/// If the cartridge had this extra RAM the Game Boy
/// automatically mapped the RAM into this area of memory.
pub const CRAM_BEGIN: u16 = 0xA000;
pub const CRAM_END: u16 = 0xBFFF;
pub const CRAM_SIZE: usize = (CRAM_END - CRAM_BEGIN + 1) as usize;

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
/// have extra capabilties. The description for how they should look lives here.
pub const OAM_BEGIN: u16 = 0xFE00;
pub const OAM_END: u16 = 0xFE9F;
pub const OAM_SIZE: usize = (OAM_END - OAM_BEGIN + 1) as usize;

/// This is one of the most dense areas of memory.
/// Practically every byte has a special meaning.
/// It's used by both the screen and the sound system to determine different settings.
/// We'll be talking a lot about this in the future.
pub const IO_BEGIN: u16 = 0xFF00;
pub const IO_END: u16 = 0xFF7F;
pub const IO_SIZE: usize = (IO_END - IO_BEGIN + 1) as usize;

/// This area is also just normal RAM but is used a lot because some of the LD instructions
/// we've already seen can easily target this area in memory.
/// This area is also sometimes used for the stack,
/// but the top of working RAM is also used for this purpose.
pub const HRAM_BEGIN: u16 = 0xFF80;
pub const HRAM_END: u16 = 0xFFFE;
pub const HRAM_SIZE: usize = (HRAM_END - HRAM_BEGIN + 1) as usize;

// Joypad Input
pub const JOYPAD: u16 = 0xFF00;

// Timer
pub const TIMER_DIVIDER: u16 = 0xFF04;
pub const TIMER_COUNTER: u16 = 0xFF05;
pub const TIMER_MODULO: u16 = 0xFF06;
pub const TIMER_CTRL: u16 = 0xFF07;

// Pixel Processing Unit
pub const PPU_LCDC: u16 = 0xFF40;
pub const PPU_STAT: u16 = 0xFF41;
pub const PPU_SCY: u16 = 0xFF42;
pub const PPU_SCX: u16 = 0xFF43;
pub const PPU_LY: u16 = 0xFF44;
pub const PPU_LYC: u16 = 0xFF45;
pub const PPU_DMA: u16 = 0xFF46;
pub const PPU_BGP: u16 = 0xFF47;
pub const PPU_OBP0: u16 = 0xFF48;
pub const PPU_OBP1: u16 = 0xFF49;
pub const PPU_WY: u16 = 0xFF4A;
pub const PPU_WX: u16 = 0xFF4B;

// Interrupt Controller
pub const INTERRUPT_FLAG: u16 = 0xFF0F;
pub const INTERRUPT_ENABLE: u16 = 0xFFFF;
