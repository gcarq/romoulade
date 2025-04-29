use crate::gb::bus::Bus;
use crate::gb::constants::{ROM_BANK_0_BEGIN, ROM_BANK_0_END};
use crate::gb::cpu::ImeState;
use crate::gb::cpu::instruction::Instruction;
use crate::gb::{AddressSpace, HardwareContext};

#[derive(Clone)]
pub struct DebugBus {
    inner: Bus,
}

impl DebugBus {
    /// Prefetch all instructions from ROM bank 0.
    /// TODO: adapt for banking
    pub fn fetch_instructions(&mut self) -> Vec<(u16, Option<Instruction>)> {
        let mut instructions = Vec::with_capacity(1000);
        let mut pc = ROM_BANK_0_BEGIN;

        while pc < ROM_BANK_0_END {
            let (instruction, next_pc) = Instruction::from_memory(pc, self);
            instructions.push((pc, instruction));
            pc = next_pc;
        }
        instructions
    }

    #[inline]
    pub fn inner(&self) -> &Bus {
        &self.inner
    }
}

impl From<Bus> for DebugBus {
    #[inline(always)]
    fn from(bus: Bus) -> Self {
        Self { inner: bus }
    }
}

impl HardwareContext for DebugBus {
    #[inline]
    fn set_ime(&mut self, ime: ImeState) {
        self.inner.set_ime(ime);
    }

    #[inline]
    fn ime(&self) -> ImeState {
        self.inner.ime()
    }

    #[inline]
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
