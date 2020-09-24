use crate::gb::cpu::CPU;
use crate::gb::memory::constants::{INTERRUPT_ENABLE, INTERRUPT_FLAG};
use crate::gb::AddressSpace;
use crate::utils;
use bitflags::_core::cell::RefCell;

pub const IRQ_VBLANK: u8 = 0b00000001;
pub const IRQ_LCD: u8 = 0b00000010;
pub const IRQ_TIMER: u8 = 0b00000100;
pub const IRQ_JOYPAD: u8 = 0b00010000;

pub struct IRQHandler<'a> {
    cpu: &'a RefCell<CPU<'a>>,
}

impl<'a> IRQHandler<'a> {
    pub fn new(cpu: &'a RefCell<CPU<'a>>) -> Self {
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
                self.service_interrupts(i);
            }
        }
    }

    fn service_interrupts(&mut self, interrupt: u8) {
        println!("Serving interrupt: {}", interrupt);
        self.cpu.borrow_mut().ime = false;

        // Clear interrupt request
        let req = utils::set_bit(self.read(INTERRUPT_FLAG), interrupt, false);
        self.write(INTERRUPT_FLAG, req);

        // Save current execution address by pushing it onto the stack
        self.cpu.borrow_mut().push(self.cpu.borrow_mut().pc);

        match interrupt {
            0 => self.cpu.borrow_mut().pc = 0x40, // V-Blank
            1 => self.cpu.borrow_mut().pc = 0x48, // LCD
            2 => self.cpu.borrow_mut().pc = 0x50, // Timer
            4 => self.cpu.borrow_mut().pc = 0x60, // Joypad
            _ => unimplemented!(),
        }
    }

    fn write(&mut self, address: u16, value: u8) {
        self.cpu.borrow_mut().bus.borrow_mut().write(address, value);
    }

    fn read(&self, address: u16) -> u8 {
        self.cpu.borrow_mut().bus.borrow().read(address)
    }
}
