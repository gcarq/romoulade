use crate::gb::constants::*;
use crate::gb::AddressSpace;

bitflags! {
    /// Represents the Serial transfer control register at 0xFF02
    #[derive(Copy, Clone)]
    pub struct SerialTransferControl: u8 {
        // 0 = External clock, 1 = Internal clock.
        const CLOCK_SELECT    = 0b0000_0001;
        // If set to 1, enable high speed serial clock (~256 kHz in single-speed mode)
        const CLOCK_SPEED     = 0b0000_0010;
        // If 1, a transfer is either requested or in progress.
        const TRANSFER_ENABLE = 0b1000_0000;
    }
}

#[derive(Clone)]
pub struct SerialTransfer {
    /// The transfer control register.
    pub control: SerialTransferControl,
    /// The transfer data register.
    pub data: u8,
}

impl Default for SerialTransfer {
    fn default() -> Self {
        Self {
            control: SerialTransferControl::empty(),
            data: 0,
        }
    }
}

impl AddressSpace for SerialTransfer {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            SERIAL_TRANSFER_DATA => self.data = value,
            SERIAL_TRANSFER_CTRL => self.control = SerialTransferControl::from_bits_truncate(value),
            _ => panic!("Attempt to write to unmapped serial register: 0x{:X}", address),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            SERIAL_TRANSFER_DATA => self.data,
            // Undocumented bits should be 1
            SERIAL_TRANSFER_CTRL => self.control.bits() | 0b0111_1110,
            _ => panic!("Attempt to read from unmapped serial register: 0x{:X}", address),
        }
    }
}