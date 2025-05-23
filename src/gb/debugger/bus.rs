use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::constants::*;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::AnnotatedInstr;
use crate::gb::{Bus, SubSystem};

#[derive(Clone)]
pub struct DebugBus {
    inner: MainBus,
}

impl DebugBus {
    /// Prefetch instruction from all sections that could contain code.
    pub fn fetch_instructions(&mut self) -> Vec<AnnotatedInstr> {
        let mut instructions = Vec::with_capacity(1000);
        instructions.extend(self.instructions_from_range(ROM_LOW_BANK_BEGIN, ROM_HIGH_BANK_END));
        instructions.extend(self.instructions_from_range(CRAM_BANK_BEGIN, CRAM_BANK_END));
        instructions.extend(self.instructions_from_range(WRAM_BEGIN, WRAM_END));
        instructions.extend(self.instructions_from_range(HRAM_BEGIN, HRAM_END));
        instructions
    }

    /// Fetches all instructions from the specified range (inclusive).
    fn instructions_from_range(&mut self, start: u16, end: u16) -> Vec<AnnotatedInstr> {
        let mut instructions = Vec::new();
        let mut pc = start;

        while pc <= end {
            let opcode = self.read(pc);
            // Check if resolving the current opcode would lead to an out-of-bounds read
            if u32::from(pc) + u32::from(instruction_len(opcode)) - 1 > u32::from(end) {
                break;
            }
            let (instruction, next_pc) = Instruction::from_opcode(opcode, pc + 1, self);
            let bytes = (pc..next_pc).map(|addr| self.read(addr)).collect();
            instructions.push(AnnotatedInstr::new(pc, bytes, instruction));
            pc = next_pc;
        }
        instructions
    }
}

impl From<MainBus> for DebugBus {
    #[inline(always)]
    fn from(bus: MainBus) -> Self {
        Self { inner: bus }
    }
}

impl SubSystem for DebugBus {
    #[inline(always)]
    fn write(&mut self, address: u16, value: u8) {
        self.inner.write(address, value);
    }

    #[inline(always)]
    fn read(&mut self, address: u16) -> u8 {
        self.inner.read(address)
    }
}

impl Bus for DebugBus {
    #[inline(always)]
    fn cycle_write(&mut self, address: u16, value: u8) {
        // We don't want cycle for debugger writes
        self.write(address, value);
    }

    #[inline(always)]
    fn cycle_read(&mut self, address: u16) -> u8 {
        // We don't want cycle for debugger reads
        self.read(address)
    }
    #[inline(always)]
    fn cycle(&mut self) {
        self.inner.cycle();
    }

    #[inline(always)]
    fn has_irq(&self) -> bool {
        self.inner.has_irq()
    }

    #[inline(always)]
    fn set_ie(&mut self, r: InterruptRegister) {
        self.inner.set_ie(r);
    }

    #[inline(always)]
    fn get_ie(&self) -> InterruptRegister {
        self.inner.get_ie()
    }

    #[inline(always)]
    fn set_if(&mut self, r: InterruptRegister) {
        self.inner.set_if(r);
    }

    #[inline(always)]
    fn get_if(&self) -> InterruptRegister {
        self.inner.get_if()
    }
}

/// Returns the byte length of the instruction based on the opcode.
const fn instruction_len(opcode: u8) -> u8 {
    match opcode {
        // Prefixed opcodes always have a length of 2
        0xCB => 2,
        0x06 | 0x0E => 2,
        0x10 | 0x16 | 0x18 | 0x1E => 2,
        0x20 | 0x26 | 0x28 | 0x2E => 2,
        0x30 | 0x36 | 0x38 | 0x3E => 2,
        0xC6 | 0xCE => 2,
        0xD6 | 0xDE => 2,
        0xE0 | 0xE6 | 0xE8 | 0xEE => 2,
        0xF0 | 0xF6 | 0xF8 | 0xFE => 2,
        0x01 | 0x08 => 3,
        0x11 | 0x21 | 0x31 => 3,
        0xC2 | 0xC3 | 0xC4 | 0xCA | 0xCC | 0xCD => 3,
        0xD2 | 0xD4 | 0xDA | 0xDC => 3,
        0xEA | 0xFA => 3,
        _ => 1,
    }
}
