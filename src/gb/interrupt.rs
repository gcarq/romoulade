use crate::gb::cpu::CPU;
use crate::gb::memory::constants::{INTERRUPT_ENABLE, INTERRUPT_FLAG};
use crate::gb::memory::MemoryBus;
use crate::gb::AddressSpace;
use crate::utils;

/// Represents an interrupt request
#[derive(Debug, Copy, Clone)]
#[repr(u8)]
pub enum IRQ {
    VBlank = 0,
    LCD = 1,
    Timer = 2,
    Joypad = 4,
}

impl From<u8> for IRQ {
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

impl From<IRQ> for u8 {
    fn from(value: IRQ) -> u8 {
        match value {
            IRQ::VBlank => 0,
            IRQ::LCD => 1,
            IRQ::Timer => 2,
            IRQ::Joypad => 4,
        }
    }
}

/// Handles pending interrupt requests
/// TODO: implement HALT instruction bug (Section 4.10): https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
pub fn handle(cpu: &mut CPU, bus: &mut MemoryBus) {
    let requests = bus.read(INTERRUPT_FLAG);
    if requests == 0 {
        return;
    }

    let enabled = bus.read(INTERRUPT_ENABLE);
    for i in 0..5 {
        if utils::bit_at(requests, i) && utils::bit_at(enabled, i) {
            // Only serve interrupt if IME is enabled
            if cpu.ime {
                service_interrupts(cpu, bus, IRQ::from(i));
            }
            // CPU should be always woken up from HALT
            cpu.is_halted = false;
        }
    }
}

fn service_interrupts(cpu: &mut CPU, bus: &mut MemoryBus, interrupt: IRQ) {
    // TODO: debug log: println!("Serving interrupt: {:?}...", interrupt);
    cpu.ime = false;

    // Clear interrupt request
    let req = utils::set_bit(bus.read(INTERRUPT_FLAG), u8::from(interrupt), false);
    bus.write(INTERRUPT_FLAG, req);

    // Save current execution address by pushing it onto the stack
    let pc = cpu.pc;
    cpu.push(pc, bus);

    match interrupt {
        IRQ::VBlank => cpu.pc = 0x40,
        IRQ::LCD => cpu.pc = 0x48,
        IRQ::Timer => cpu.pc = 0x50,
        IRQ::Joypad => cpu.pc = 0x60,
    }
}
