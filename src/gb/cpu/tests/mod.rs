mod instruction;
mod ops;

use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::cpu::{CPU, ImeState};
use crate::gb::{AddressSpace, HardwareContext};

/// Represents a mock for MemoryBus
struct MockBus {
    ime: ImeState,
    data: Vec<u8>,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            ime: ImeState::Enabled,
            data,
        }
    }
}

impl AddressSpace for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn read(&mut self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

impl HardwareContext for MockBus {
    fn set_ime(&mut self, ime: ImeState) {
        self.ime = ime;
    }

    fn ime(&self) -> ImeState {
        self.ime
    }

    fn tick(&mut self) {
        // No-op
    }
}

fn assert_flags(r: FlagsRegister, zero: bool, negative: bool, half_carry: bool, carry: bool) {
    assert_eq!(
        r.contains(FlagsRegister::ZERO),
        zero,
        "Expected zero flag to be {}, but it was {}",
        zero,
        r.contains(FlagsRegister::ZERO)
    );
    assert_eq!(
        r.contains(FlagsRegister::SUBTRACTION),
        negative,
        "Expected negative flag to be {}, but it was {}",
        negative,
        r.contains(FlagsRegister::SUBTRACTION)
    );
    assert_eq!(
        r.contains(FlagsRegister::HALF_CARRY),
        half_carry,
        "Expected half carry flag to be {}, but it was {}",
        half_carry,
        r.contains(FlagsRegister::HALF_CARRY)
    );
    assert_eq!(
        r.contains(FlagsRegister::CARRY),
        carry,
        "Expected carry flag to be {}, but it was {}",
        carry,
        r.contains(FlagsRegister::CARRY)
    );
}

#[test]
fn test_af_register() {
    let mut cpu = CPU::default();
    cpu.r.set_af(0b1101_1111_1111_1111);
    assert_eq!(cpu.r.a, 0b1101_1111);
    assert_eq!(cpu.r.f.bits(), 0b1111_0000);
    assert_eq!(cpu.r.get_af(), 0b1101_1111_1111_0000);
}

#[test]
fn test_bc_register() {
    let mut cpu = CPU::default();
    cpu.r.set_bc(0b0110_1111_1111_1011);
    assert_eq!(cpu.r.b, 0b0110_1111);
    assert_eq!(cpu.r.c, 0b1111_1011);
    assert_eq!(cpu.r.get_bc(), 0b0110_1111_1111_1011);
}

#[test]
fn test_de_register() {
    let mut cpu = CPU::default();
    cpu.r.set_de(0b0110_1101_1101_1011);
    assert_eq!(cpu.r.d, 0b0110_1101);
    assert_eq!(cpu.r.e, 0b1101_1011);
    assert_eq!(cpu.r.get_de(), 0b0110_1101_1101_1011);
}

#[test]
fn test_hl_register() {
    let mut cpu = CPU::default();
    cpu.r.set_hl(0b0110_1110_1111_1010);
    assert_eq!(cpu.r.h, 0b0110_1110);
    assert_eq!(cpu.r.l, 0b1111_1010);
    assert_eq!(cpu.r.get_hl(), 0b0110_1110_1111_1010);
}
