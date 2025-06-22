use crate::gb::constants::UNDEFINED_READ;

const OAM_TRANSFER_SIZE: u8 = 0xA0;

/// This controller holds all data required for the OAM DMA transfer.
/// It takes 2 machine cycles until the transfer is started.
///   M = 0: write to $FF46 happens
///   M = 1: nothing (OAM still accessible)
///   M = 2: new DMA starts, OAM reads will return $FF
///
/// Also, when OAM DMA is restarted while a previous one is running, the previous one
/// is not immediately stopped.
///
///   M = 0: write to $FF46 happens. Previous DMA is running (OAM *not* accessible)
///   M = 1: previous DMA is running (OAM *not* accessible)
///   M = 2: new DMA starts, OAM reads will return $FF
///
/// source, requested and pending hold the upper bits of the source address.
#[derive(Clone, Copy)]
pub struct OamDmaController {
    pub is_running: bool,
    pub source: u8,
    pub requested: Option<u8>,
    pub pending: Option<u8>,
    source_address: u16, // Holds the current transfer source address
}

impl Default for OamDmaController {
    fn default() -> Self {
        Self {
            is_running: false,
            source: UNDEFINED_READ,
            requested: None,
            pending: None,
            source_address: 0x0000,
        }
    }
}

impl OamDmaController {
    /// Requests a new OAM DMA transfer.
    #[inline]
    pub const fn request(&mut self, value: u8) {
        self.requested = Some(value);
    }

    /// Starts the OAM DMA transfer, the passed value specifies the transfer source address
    /// divided by 0x100.
    #[inline]
    pub fn start(&mut self, source: u8) {
        self.source = source;
        self.is_running = true;
        self.source_address = u16::from(source) << 8;
    }

    /// Returns the current source address and increments it by 1.
    #[inline]
    pub const fn transfer(&mut self) -> Option<u16> {
        if !self.is_running {
            return None;
        }
        let address = self.source_address;
        self.source_address = self.source_address.wrapping_add(1);
        // Check if the transfer is complete. It's fine to cast the source_address to u8
        // to get the current byte offset as the initial u8 value only specified the upper bits.
        if self.source_address as u8 >= OAM_TRANSFER_SIZE {
            self.is_running = false;
        }
        Some(address)
    }
}
