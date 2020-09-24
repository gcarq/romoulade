use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::cpu::CPU;
use crate::gb::AddressSpace;
use std::cell::RefCell;

/// Represents a mock for MemoryBus
struct MockBus {
    data: Vec<u8>,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl AddressSpace for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

fn assert_flags(r: FlagsRegister, zero: bool, negative: bool, half_carry: bool, carry: bool) {
    assert_eq!(r.zero, zero);
    assert_eq!(r.negative, negative);
    assert_eq!(r.half_carry, half_carry);
    assert_eq!(r.carry, carry);
}

#[test]
fn test_add_no_overflow() {
    // ADD A, HLI
    let bus = RefCell::new(MockBus::new([0x86, 0x42].into()));
    let mut cpu = CPU::new(&bus);
    cpu.r.set_hl(0x01);
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x42);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_add_overflow_zero() {
    // ADD A, HLI
    let bus = RefCell::new(MockBus::new([0x86, 0x02].into()));
    let mut cpu = CPU::new(&bus);
    cpu.r.a = 0xff;
    cpu.r.set_hl(0x01);
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x01);
    assert_flags(cpu.r.f, false, false, true, true);
}

#[test]
fn test_bit_zero() {
    // BIT 7,H
    let bus = RefCell::new(MockBus::new([0xcb, 0x7c].into()));
    let mut cpu = CPU::new(&bus);
    cpu.r.h = 0b01111111;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_bit_non_zero() {
    // BIT 7,H
    let bus = RefCell::new(MockBus::new([0xcb, 0x7c].into()));
    let mut cpu = CPU::new(&bus);
    cpu.r.h = 0b11010000;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_dec_no_overflow() {
    // DEC B
    let bus = RefCell::new(MockBus::new(vec![0x05; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0x02;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x1);
    assert_flags(cpu.r.f, false, true, false, false);
}

#[test]
fn test_dec_overflow() {
    // DEC B
    let bus = RefCell::new(MockBus::new(vec![0x05; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0x00;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0xff);
    assert_flags(cpu.r.f, false, true, true, false);
}

#[test]
fn test_dec_zero() {
    // DEC B
    let bus = RefCell::new(MockBus::new(vec![0x05; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0x01;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x00);
    assert_flags(cpu.r.f, true, true, false, false);
}

#[test]
fn test_inc_no_overflow() {
    // INC B
    let bus = RefCell::new(MockBus::new(vec![0x04; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0x00;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x01);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_inc_overflow() {
    // INC B
    let bus = RefCell::new(MockBus::new(vec![0x04; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0xff;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x00);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_inc_half_carry() {
    // INC B
    let bus = RefCell::new(MockBus::new(vec![0x04; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.r.b = 0x0e;
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x0f);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_nop() {
    // NOP
    let bus = RefCell::new(MockBus::new(vec![0x00; 1]));
    let mut cpu = CPU::new(&bus);
    cpu.step();
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}
