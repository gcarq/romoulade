use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::constants::*;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::debugger::AnnotatedInstr;
use crate::gb::{Bus, SubSystem};
use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct DebugBus {
    inner: MainBus,
}

impl DebugBus {
    /// Prefetch all instructions from ROM bank 0.
    /// TODO: adapt for banking
    pub fn fetch_instructions(&mut self) -> Vec<AnnotatedInstr> {
        let mut instructions = Vec::with_capacity(1000);
        instructions.extend(self.instructions_from_range(ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END));
        instructions.extend(self.instructions_from_range(CRAM_BANK_BEGIN..=CRAM_BANK_END));
        instructions.extend(self.instructions_from_range(WRAM_BEGIN..=WRAM_END));
        instructions.extend(self.instructions_from_range(HRAM_BEGIN..=HRAM_END));
        instructions
    }

    /// Fetches all instructions from the specified range.
    fn instructions_from_range(&mut self, range: RangeInclusive<u16>) -> Vec<AnnotatedInstr> {
        let mut instructions = Vec::new();
        let mut pc = *range.start();

        while pc <= *range.end() {
            let old_pc = pc;
            let (instruction, next_pc) = Instruction::from_opcode(self.read(pc), pc + 1, self);
            let bytes = (old_pc..next_pc).map(|addr| self.read(addr)).collect();
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
    fn write(&mut self, address: u16, value: u8) {
        self.inner.write(address, value);
    }

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
