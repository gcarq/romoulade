use crate::gb::bus::{Bus, InterruptRegister};
use crate::gb::constants::*;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::{AddressSpace, HardwareContext};
use std::ops::RangeInclusive;

#[derive(Clone)]
pub struct DebugBus {
    inner: Bus,
}

impl DebugBus {
    /// Prefetch all instructions from ROM bank 0.
    /// TODO: adapt for banking
    pub fn fetch_instructions(&mut self) -> Vec<(u16, Instruction)> {
        let mut instructions = Vec::with_capacity(1000);
        instructions.extend(self.instructions_from_range(ROM_LOW_BANK_BEGIN..=ROM_HIGH_BANK_END));
        instructions.extend(self.instructions_from_range(CRAM_BANK_BEGIN..=CRAM_BANK_END));
        instructions.extend(self.instructions_from_range(WRAM_BEGIN..=WRAM_END));
        instructions.extend(self.instructions_from_range(HRAM_BEGIN..=HRAM_END));
        instructions
    }

    /// Fetches all instructions from the specified range.
    fn instructions_from_range(&mut self, range: RangeInclusive<u16>) -> Vec<(u16, Instruction)> {
        let mut instructions = Vec::new();
        let mut pc = *range.start();

        while pc <= *range.end() {
            let (instruction, next_pc) = Instruction::from_memory(pc, self);
            instructions.push((pc, instruction));
            pc = next_pc;
        }
        instructions
    }
}

impl From<Bus> for DebugBus {
    #[inline(always)]
    fn from(bus: Bus) -> Self {
        Self { inner: bus }
    }
}

impl HardwareContext for DebugBus {
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

    #[inline(always)]
    fn has_irq(&self) -> bool {
        self.inner.has_irq()
    }

    #[inline(always)]
    fn tick(&mut self) {
        self.inner.tick();
    }
}

impl AddressSpace for DebugBus {
    #[inline]
    fn write(&mut self, address: u16, value: u8) {
        self.inner.write_raw(address, value);
    }

    #[inline]
    fn read(&mut self, address: u16) -> u8 {
        self.inner.read_raw(address)
    }
}
