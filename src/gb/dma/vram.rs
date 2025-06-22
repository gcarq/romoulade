use crate::gb::constants::*;

bitflags! {
    /// Represents the attributes of a sprite.
    /// The first 3 bits are only used in CGB mode.
    /// Bit 4 is used to select the palette in CGB mode.
    #[derive(Debug, Copy, Clone)]
    pub struct VRamDmaMode: u8 {
        const LENGTH_MASK = 0b0111_1111;
        // 1 = HBlank HDMA, 0 = General Purpose DMA
        const TRANSFER_MODE = 0b1000_0000;
    }
}

impl VRamDmaMode {
    /// Starts the VRAM DMA transfer.
    pub fn start(&mut self) {
        todo!()
    }

    /// Terminates an ongoing VRAM DMA transfer.
    pub fn stop(&mut self) {
        todo!()
    }

    /// Returns the length of the HDMA transfer in bytes.
    pub fn length(&self) -> u8 {
        ((self.bits() & Self::LENGTH_MASK.bits()) + 1) * 0x10
    }

    /// Returns true if the transfer mode is General Purpose DMA.
    pub fn is_gdma(&self) -> bool {
        !self.contains(Self::TRANSFER_MODE)
    }
}

impl Default for VRamDmaMode {
    fn default() -> Self {
        Self::from_bits_truncate(0xFF)
    }
}

#[derive(Copy, Clone)]
pub struct VRamDmaController {
    source_high: u8,       // HDMA1
    source_low: u8,        // HDMA2
    dest_high: u8,         // HDMA3
    dest_low: u8,          // HDMA4
    pub mode: VRamDmaMode, // HDMA5 (length, mode, start)
    active: bool,
}

impl VRamDmaController {
    /// Requests a new VRAM DMA transfer.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            CGB_HDMA1_VRAM_DMA_SRC => self.source_high = value,
            CGB_HDMA2_VRAM_DMA_SRC => self.source_low = value & 0b1111_0000,
            CGB_HDMA3_VRAM_DMA_DEST => self.dest_high = value & 0b0001_1111,
            CGB_HDMA4_VRAM_DMA_DEST => self.dest_low = value & 0b1111_0000,
            CGB_HDMA5_VRAM_DMA_MODE => self.mode = VRamDmaMode::from_bits_truncate(value),
            _ => unreachable!("Invalid VRAM DMA address: {:#06X}", address),
        }
    }

    /// Source address of the VRAM DMA transfer.
    #[inline]
    pub fn source(&self) -> u16 {
        u16::from_be_bytes([self.source_high, self.source_low])
    }

    /// Destination address of the VRAM DMA transfer.
    #[inline]
    pub fn destination(&self) -> u16 {
        u16::from_be_bytes([self.dest_high, self.dest_low])
    }
}

impl Default for VRamDmaController {
    fn default() -> Self {
        Self {
            source_high: 0xFF,
            source_low: 0xFF,
            dest_high: 0xFF,
            dest_low: 0xFF,
            mode: VRamDmaMode::default(),
            active: false,
        }
    }
}
