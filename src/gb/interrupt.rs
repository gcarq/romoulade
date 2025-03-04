use crate::gb::cpu::CPU;
use crate::gb::memory::constants::{INTERRUPT_ENABLE, INTERRUPT_FLAG};
use crate::gb::AddressSpace;
use crate::utils;
use bitflags::_core::cell::RefCell;
use std::convert;

/// Represents an interrupt request
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum IRQ {
    VBlank = 0,
    LCD = 1,
    Timer = 2,
    Joypad = 4,
}

impl convert::From<u8> for IRQ {
    fn from(value: u8) -> Self {
        match value {
            0 => IRQ::VBlank,
            1 => IRQ::LCD,
            2 => IRQ::Timer,
            4 => IRQ::Joypad,
            _ => panic!(),
        }
    }
}

impl convert::From<IRQ> for u8 {
    fn from(value: IRQ) -> u8 {
        match value {
            IRQ::VBlank => 0,
            IRQ::LCD => 1,
            IRQ::Timer => 2,
            IRQ::Joypad => 4,
        }
    }
}

pub struct IRQHandler<'a, T: AddressSpace> {
    cpu: &'a RefCell<CPU<'a, T>>,
    bus: &'a RefCell<T>,
}

impl<'a, T: AddressSpace> IRQHandler<'a, T> {
    pub fn new(cpu: &'a RefCell<CPU<'a, T>>, bus: &'a RefCell<T>) -> Self {
        Self { cpu, bus }
    }

    /// Handles pending interrupt requests
    /// TODO: implement HALT instruction bug (Section 4.10): https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
    pub fn handle(&mut self) {
        let requests = self.read(INTERRUPT_FLAG);
        if requests == 0 {
            return;
        }

        let enabled = self.read(INTERRUPT_ENABLE);
        for i in 0..5 {
            if utils::bit_at(requests, i) && utils::bit_at(enabled, i) {
                // Only serve interrupt if IME is enabled
                if self.cpu.borrow().ime {
                    self.service_interrupts(IRQ::from(i));
                }
                // CPU should be always woken up from HALT
                self.cpu.borrow_mut().is_halted = false;
            }
        }
    }

    fn service_interrupts(&mut self, interrupt: IRQ) {
        // TODO: debug log: println!("Serving interrupt: {:?}...", interrupt);
        self.cpu.borrow_mut().ime = false;

        // Clear interrupt request
        let req = utils::set_bit(self.read(INTERRUPT_FLAG), u8::from(interrupt), false);
        self.write(INTERRUPT_FLAG, req);

        // Save current execution address by pushing it onto the stack
        let pc = self.cpu.borrow().pc;
        self.cpu.borrow_mut().push(pc);

        match interrupt {
            IRQ::VBlank => self.cpu.borrow_mut().pc = 0x40,
            IRQ::LCD => self.cpu.borrow_mut().pc = 0x48,
            IRQ::Timer => self.cpu.borrow_mut().pc = 0x50,
            IRQ::Joypad => self.cpu.borrow_mut().pc = 0x60,
        }
    }
}

impl<T: AddressSpace> AddressSpace for IRQHandler<'_, T> {
    fn write(&mut self, address: u16, value: u8) {
        self.bus.borrow_mut().write(address, value);
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }
}
