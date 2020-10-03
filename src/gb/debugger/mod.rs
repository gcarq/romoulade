pub mod format;
mod utils;

use crate::gb::cpu::CPU;
use crate::gb::debugger::utils::resolve_byte_length;
use crate::gb::instruction::Instruction;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::PPU;
use crate::gb::timer::Timer;
use crate::gb::AddressSpace;
use std::cell::RefCell;

/// Starts the emulating loop in debug mode
pub fn emulate<T: AddressSpace>(
    cpu: &RefCell<CPU<T>>,
    bus: &RefCell<MemoryBus>,
    ppu: &mut PPU,
    timer: &mut Timer,
    irq_handler: &mut IRQHandler<T>,
) {
    let mut pc = 0x0000;
    for _ in 0..100 {
        let (instruction, new_pc) = step(pc, &bus);
        println!("{:#06X}: {}", pc, instruction);
        pc = new_pc;
    }
    /*'main: loop {
        let cycles = cpu.borrow_mut().step();
        timer.step(cycles);
        ppu.step(cycles);
        irq_handler.handle();
    }*/
}

/// Emulates one CPU step without executing it.
/// Returns a tuple with the instruction and the updated program counter.
fn step(pc: u16, bus: &RefCell<MemoryBus>) -> (Instruction, u16) {
    let opcode = bus.borrow().read(pc);
    // Read next opcode from memory
    let (opcode, prefixed) = match opcode == 0xCB {
        true => (bus.borrow().read(pc + 1), true),
        false => (opcode, false),
    };

    // Parse instruction from opcode and return it together with the next program counter
    match Instruction::from_byte(opcode, prefixed) {
        Some(instruction) => (
            instruction,
            pc + resolve_byte_length(opcode, prefixed) as u16,
        ),
        None => {
            let description = format!("0x{}{:02x}", if prefixed { "cb" } else { "" }, opcode);
            panic!("Unresolved instruction: {}.\nHALTED!", description);
        }
    }
}
