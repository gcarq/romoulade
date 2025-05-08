use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::constants::BOOT_END;
use crate::gb::cpu::CPU;
use crate::gb::interrupt;
use crate::gb::interrupt::InterruptRegister;
use crate::gb::joypad::{ActionInput, DPadInput, Joypad, JoypadInput};
use crate::gb::timer::{Timer, TimerControl};
use crate::gb::utils::{bit_at, half_carry_u8, set_bit};
use std::sync::Arc;

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
    let mut bus = Bus::with_cartridge(cartridge, None);

    while cpu.pc < BOOT_END + 1 {
        cpu.step(&mut bus);
        interrupt::handle(&mut cpu, &mut bus);
    }

    assert!(!bus.is_boot_rom_active, "Boot ROM is still active");
    assert_eq!(cpu.r.get_af(), 0x01B0, "AF is invalid");
    assert_eq!(cpu.r.get_bc(), 0x0013, "BC is invalid");
    assert_eq!(cpu.r.get_de(), 0x00D8, "DE is invalid");
    assert_eq!(cpu.r.get_hl(), 0x014D, "HL is invalid");
    assert_eq!(cpu.sp, 0xFFFE, "SP is invalid");
    assert_eq!(cpu.pc, 0x0100, "PC is invalid");
}

#[test]
fn test_joypad_dpad_pressed() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b1111_1111);

    // D-Pad selection mode while right has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Right)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1110_1110);

    // D-Pad selection mode while left has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Left)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1110_1101);

    // D-Pad selection mode while up has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Up)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1110_1011);

    // D-Pad selection mode while down has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Down)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1110_0111);
}

#[test]
fn test_joypad_dpad_not_pressed() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b1111_1111);

    // D-Pad selection mode while A has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::Action(ActionInput::A)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1110_1111);

    // D-Pad selection mode while B has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::Action(ActionInput::B)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1110_1111);

    // D-Pad selection mode while Select has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::Action(ActionInput::Select)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1110_1111);

    // D-Pad selection mode while Start has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::Action(ActionInput::Start)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1110_1111);
}

#[test]
fn test_joypad_actions_pressed() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b1111_1111);

    // Action selection mode while A has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::Action(ActionInput::A)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1101_1110);

    // Action selection mode while B has been pressed
    let irq = joypad.write(0b1101_1101, Some(JoypadInput::Action(ActionInput::B)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1101_1101);

    // Action selection mode while select has been pressed
    let irq = joypad.write(0b1101_1011, Some(JoypadInput::Action(ActionInput::Select)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1101_1011);

    // Action selection mode while start has been pressed
    let irq = joypad.write(0b1101_0111, Some(JoypadInput::Action(ActionInput::Start)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b1101_0111);
}

#[test]
fn test_joypad_actions_not_pressed() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b1111_1111);

    // Action selection mode while Right has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::DPad(DPadInput::Right)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1101_1111);

    // Action selection mode while Left has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::DPad(DPadInput::Left)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1101_1111);

    // Action selection mode while Up has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::DPad(DPadInput::Up)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1101_1111);

    // Action selection mode while Down has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::DPad(DPadInput::Down)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1101_1111);
}

#[test]
fn test_joypad_common() {
    let mut joypad = Joypad::default();

    // No selection mode while Right has been pressed
    let irq = joypad.write(0b0011_0000, Some(JoypadInput::DPad(DPadInput::Right)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1111_1111);

    // No selection mode while Select has been pressed
    let irq = joypad.write(0b0011_0000, Some(JoypadInput::Action(ActionInput::Select)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1111_1111);

    // Acceptance test of mooneye test suite:
    // See https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/bits/unused_hwio-GS.s
    let irq = joypad.write(0b1111_1111, None);
    assert!(!irq);
    assert_eq!(joypad.read(), 0b1111_1111);
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
fn test_timer_counter() {
    let mut int_reg = InterruptRegister::empty();
    let mut timer = Timer::default();
    timer.control = TimerControl::from_bits_truncate(0b0000_0101);
    timer.divider = 0b0001_0111;
    assert!(timer.control.is_enabled());

    timer.step(&mut int_reg);
    assert_eq!(timer.divider, 0b0001_1000);
    assert_eq!(timer.counter, 0b0000_0001);
    assert!(!int_reg.contains(InterruptRegister::TIMER));
}

#[test]
fn test_timer_counter_overflow() {
    let mut int_reg = InterruptRegister::empty();
    let mut timer = Timer::default();
    timer.control = TimerControl::from_bits_truncate(0b0000_0101);
    timer.divider = 0b0001_0011;
    timer.counter = 0b1111_1111;

    // Simulate a timer overflow, the interrupt shouldn't be fired immediately
    timer.step(&mut int_reg);
    assert_eq!(timer.counter, 0b0000_0000);
    assert!(!int_reg.contains(InterruptRegister::TIMER));

    timer.step(&mut int_reg);
    assert_eq!(timer.counter, 0b0000_0000);
    assert!(int_reg.contains(InterruptRegister::TIMER));
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
