mod joypad;
mod timer;

use crate::gb::bus::{InterruptRegister, MainBus};
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::*;
use crate::gb::cpu::{ImeState, CPU};
use crate::gb::utils::{bit_at, half_carry_u8, set_bit};
use crate::gb::{Bus, SubSystem};
use std::sync::Arc;

/// Represents a mock for `MemoryBus`
pub struct MockBus {
    interrupt_enable: InterruptRegister,
    interrupt_flags: InterruptRegister,
    data: Vec<u8>,
    pub cycles: u32,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            cycles: 0,
            interrupt_enable: InterruptRegister::empty(),
            interrupt_flags: InterruptRegister::empty(),
            data,
        }
    }
}

impl SubSystem for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn read(&mut self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

impl Bus for MockBus {
    fn cycle(&mut self) {
        self.cycles += 1;
    }

    fn has_irq(&self) -> bool {
        let enabled = self.interrupt_enable.bits() & 0b0001_1111;
        let flag = self.interrupt_flags.bits() & 0b0001_1111;
        enabled & flag != 0
    }

    fn set_ie(&mut self, r: InterruptRegister) {
        self.interrupt_enable = r;
    }

    fn get_ie(&self) -> InterruptRegister {
        self.interrupt_enable
    }

    fn set_if(&mut self, r: InterruptRegister) {
        self.interrupt_flags = r;
    }

    fn get_if(&self) -> InterruptRegister {
        self.interrupt_flags
    }
}

#[test]
fn test_boot_rom() {
    let header = vec![
        0x00, 0xc3, 0x13, 0x02, 0xce, 0xed, 0x66, 0x66, 0xcc, 0x0d, 0x00, 0x0b, 0x03, 0x73, 0x00,
        0x83, 0x00, 0x0c, 0x00, 0x0d, 0x00, 0x08, 0x11, 0x1f, 0x88, 0x89, 0x00, 0x0e, 0xdc, 0xcc,
        0x6e, 0xe6, 0xdd, 0xdd, 0xd9, 0x99, 0xbb, 0xbb, 0x67, 0x63, 0x6e, 0x0e, 0xec, 0xcc, 0xdd,
        0xdc, 0x99, 0x9f, 0xbb, 0xb9, 0x33, 0x3e, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x66, 0x4d, 0xeb,
    ];
    let mut rom = vec![0x00; 0x100];
    rom.extend(header);

    let cartridge = Cartridge::try_from(Arc::from(rom.into_boxed_slice())).unwrap();

    let mut cpu = CPU::default();
    let mut bus = MainBus::with_cartridge(cartridge, None);

    while cpu.pc < BOOT_END + 1 {
        cpu.step(&mut bus);
    }

    assert!(!bus.is_boot_rom_active, "Boot ROM is still active");
    assert_eq!(cpu.r.get_af(), 0x01B0, "AF is invalid");
    assert_eq!(cpu.r.get_bc(), 0x0013, "BC is invalid");
    assert_eq!(cpu.r.get_de(), 0x00D8, "DE is invalid");
    assert_eq!(cpu.r.get_hl(), 0x014D, "HL is invalid");
    assert_eq!(cpu.sp, 0xFFFE, "SP is invalid");
    assert_eq!(cpu.pc, 0x0100, "PC is invalid");
    assert_eq!(cpu.ime, ImeState::Disabled, "IME should be disabled");
}

#[test]
fn test_bit_at() {
    let x = 0b11110000u8;
    assert!(!bit_at(x, 3));
    assert!(bit_at(x, 4));
}

#[test]
fn test_set_bit() {
    let x = 0b11110000u8;
    assert_eq!(set_bit(x, 0, true), 0b11110001u8);
    assert_eq!(set_bit(x, 1, true), 0b11110010u8);
    assert_eq!(set_bit(x, 0, false), 0b11110000u8);
    assert_eq!(set_bit(x, 7, false), 0b01110000u8);
}

#[test]
fn test_half_carry_u8_true() {
    let x = 62;
    let y = 34;
    assert!(half_carry_u8(x, y));
}

#[test]
fn test_half_carry_u8_false() {
    let x = 34;
    let y = 34;
    assert!(!half_carry_u8(x, y));
}

#[test]
fn test_interrupt_flags_read() {
    let mut flags = InterruptRegister::all();
    assert_eq!(flags.bits(), 0b0001_1111);

    flags.remove(InterruptRegister::VBLANK);
    assert_eq!(flags.bits(), 0b0001_1110);

    flags.remove(InterruptRegister::STAT);
    assert_eq!(flags.bits(), 0b0001_1100);

    flags.remove(InterruptRegister::TIMER);
    assert_eq!(flags.bits(), 0b0001_1000);

    flags.remove(InterruptRegister::SERIAL);
    assert_eq!(flags.bits(), 0b0001_0000);

    flags.remove(InterruptRegister::JOYPAD);
    assert_eq!(flags.bits(), 0b0000_0000);
}

#[test]
fn test_interrupt_flags_write() {
    let mut flags = InterruptRegister::from_bits_retain(0b1111_1110);
    assert!(!flags.contains(InterruptRegister::VBLANK));

    flags = InterruptRegister::from_bits_retain(0b1111_1100);
    assert!(!flags.contains(InterruptRegister::STAT));

    flags = InterruptRegister::from_bits_retain(0b1111_1000);
    assert!(!flags.contains(InterruptRegister::TIMER));

    flags = InterruptRegister::from_bits_retain(0b1111_0000);
    assert!(!flags.contains(InterruptRegister::SERIAL));

    flags = InterruptRegister::from_bits_retain(0b1110_0000);
    assert!(!flags.contains(InterruptRegister::JOYPAD));
}
