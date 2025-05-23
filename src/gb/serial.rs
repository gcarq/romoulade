use crate::gb::SubSystem;
use crate::gb::constants::*;

bitflags! {
    /// Represents the Serial transfer control register at 0xFF02
    #[derive(Copy, Clone, Default)]
    pub struct Control: u8 {
        // 0 = External clock, 1 = Internal clock.
        const CLOCK_SELECT    = 0b0000_0001;
        // If set to 1, enable high speed serial clock (~256 kHz in single-speed mode),
        // this bit is only used in CGB Mode and is masked in DMG mode.
        const CLOCK_SPEED     = 0b0000_0010;
        // If 1, a transfer is either requested or in progress.
        const TRANSFER_ENABLE = 0b1000_0000;
    }
}

#[derive(Clone, Default)]
pub struct SerialTransfer {
    /// The transfer control register.
    pub control: Control,
    /// The transfer data register.
    pub data: u8,
    /// Prints the serial data to stdout.
    print_serial: bool,
}

impl SerialTransfer {
    #[inline]
    pub fn new(print_serial: bool) -> Self {
        Self {
            print_serial,
            ..Default::default()
        }
    }

    fn set_ctrl(&mut self, value: u8) {
        self.control = Control::from_bits_truncate(value);
        if self.print_serial && self.control.contains(Control::TRANSFER_ENABLE) {
            let byte = char::from(self.data);
            if byte.is_ascii() {
                print!("{byte}");
            } else {
                print!("?");
            }
        }
    }
}

impl SubSystem for SerialTransfer {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            SERIAL_TRANSFER_DATA => self.data = value,
            SERIAL_TRANSFER_CTRL => self.set_ctrl(value),
            _ => panic!("Attempt to write to unmapped serial register: {address:#06x}"),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            SERIAL_TRANSFER_DATA => self.data,
            // Undocumented bits should be 1
            SERIAL_TRANSFER_CTRL => self.control.bits() | 0b0111_1110,
            _ => panic!("Attempt to read from unmapped serial register: {address:#06x}"),
        }
    }
}
