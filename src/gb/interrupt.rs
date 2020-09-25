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
    VBLANK = 0,
    LCD = 1,
    TIMER = 2,
    JOYPAD = 4,
}

impl convert::From<u8> for IRQ {
    fn from(value: u8) -> Self {
        match value {
            0 => IRQ::VBLANK,
            1 => IRQ::LCD,
            2 => IRQ::TIMER,
            4 => IRQ::JOYPAD,
            _ => unimplemented!(),
        }
    }
}

impl convert::From<IRQ> for u8 {
    fn from(value: IRQ) -> u8 {
        match value {
            IRQ::VBLANK => 0,
            IRQ::LCD => 1,
            IRQ::TIMER => 2,
            IRQ::JOYPAD => 4,
        }
    }
}

pub struct IRQHandler<'a, T: AddressSpace> {
    cpu: &'a RefCell<CPU<'a, T>>,
}

impl<'a, T: AddressSpace> IRQHandler<'a, T> {
    pub fn new(cpu: &'a RefCell<CPU<'a, T>>) -> Self {
        Self { cpu }
    }

    /// Handles pending interrupt requests
    pub fn handle(&mut self) {
        if !self.cpu.borrow().ime {
            return;
        }

        let requests = self.read(INTERRUPT_FLAG);
        if requests == 0 {
            return;
        }

        let enabled = self.read(INTERRUPT_ENABLE);
        for i in 0..5 {
            if utils::bit_at(requests, i) && utils::bit_at(enabled, i) {
                self.service_interrupts(IRQ::from(i));
            }
        }
    }

    fn service_interrupts(&mut self, interrupt: IRQ) {
        println!("Serving interrupt: {:?}...", interrupt);
        self.cpu.borrow_mut().ime = false;

        // Clear interrupt request
        let req = utils::set_bit(self.read(INTERRUPT_FLAG), u8::from(interrupt), false);
        self.write(INTERRUPT_FLAG, req);

        // Save current execution address by pushing it onto the stack
        let pc = self.cpu.borrow().pc;
        self.cpu.borrow_mut().push(pc);

        match interrupt {
            IRQ::VBLANK => self.cpu.borrow_mut().pc = 0x40,
            IRQ::LCD => self.cpu.borrow_mut().pc = 0x48,
            IRQ::TIMER => self.cpu.borrow_mut().pc = 0x50,
            IRQ::JOYPAD => self.cpu.borrow_mut().pc = 0x60,
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cpu.borrow_mut().bus.borrow_mut().write(address, value);
    }

    fn read(&self, address: u16) -> u8 {
        self.cpu.borrow_mut().bus.borrow().read(address)
    }
}
