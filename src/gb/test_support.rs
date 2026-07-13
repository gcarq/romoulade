use crate::gb::{Bus, constants::*};
use crate::gb::{SubSystem, bus::InterruptRegister};

/// Represents a mock for `MemoryBus`
pub struct MockBus {
    interrupt_enable: InterruptRegister,
    interrupt_flag: InterruptRegister,
    data: Vec<u8>,
    pub cycles: u32,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cycles: 0,
            interrupt_enable: InterruptRegister::empty(),
            interrupt_flag: InterruptRegister::empty(),
            data,
        }
    }
}

impl SubSystem for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            INTERRUPT_FLAG => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
            INTERRUPT_ENABLE => self.interrupt_enable = InterruptRegister::from_bits_retain(value),
            _ => self.data[address as usize] = value,
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            INTERRUPT_FLAG => self.interrupt_flag.bits() | 0b1110_0000,
            INTERRUPT_ENABLE => self.interrupt_enable.bits(),
            _ => self.data[address as usize],
        }
    }
}

impl Bus for MockBus {
    fn cycle(&mut self) {
        self.cycles += 1;
    }

    fn has_irq(&self) -> bool {
        let enabled = self.interrupt_enable.bits() & 0b0001_1111;
        let flag = self.interrupt_flag.bits() & 0b0001_1111;
        enabled & flag != 0
    }

    #[cfg(test)]
    fn set_ie(&mut self, r: InterruptRegister) {
        self.interrupt_enable = r;
    }

    fn get_ie(&self) -> InterruptRegister {
        self.interrupt_enable
    }

    fn set_if(&mut self, r: InterruptRegister) {
        self.interrupt_flag = r;
    }

    fn get_if(&self) -> InterruptRegister {
        self.interrupt_flag
    }
}
