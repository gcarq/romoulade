use crate::gb::interrupt::InterruptFlags;
use crate::gb::joypad::{ActionInput, DPadInput, Joypad, JoypadInput};
use crate::gb::timer::{Frequency, Timer};
use crate::gb::utils::{bit_at, half_carry_u8, set_bit};

#[test]
fn test_joypad_dpad() {
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
fn test_joypad_actions() {
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
fn test_joypad_common() {
    let mut joypad = Joypad::default();

    // No selection mode while right has been pressed
    let irq = joypad.write(0b0011_0000, Some(JoypadInput::DPad(DPadInput::Right)));
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
fn test_timer_ctrl_read() {
    let mut timer = Timer::new(Frequency::Hz4096);
    assert_eq!(timer.read_control(), 0b1111_1000);

    timer.on = true;
    timer.frequency = Frequency::Hz16384;
    assert_eq!(timer.read_control(), 0b1111_1111);

    timer.frequency = Frequency::Hz65536;
    assert_eq!(timer.read_control(), 0b1111_1110);

    timer.frequency = Frequency::Hz262144;
    assert_eq!(timer.read_control(), 0b1111_1101);
}

#[test]
fn test_timer_ctrl_write() {
    let mut timer = Timer::new(Frequency::Hz4096);
    timer.write_control(0b0000_0000);
    assert_eq!(timer.on, false);
    assert_eq!(timer.frequency, Frequency::Hz4096);

    timer.write_control(0b0000_0101);
    assert_eq!(timer.on, true);
    assert_eq!(timer.frequency, Frequency::Hz262144);

    timer.write_control(0b0000_0110);
    assert_eq!(timer.on, true);
    assert_eq!(timer.frequency, Frequency::Hz65536);

    timer.write_control(0b0000_0011);
    assert_eq!(timer.on, false);
    assert_eq!(timer.frequency, Frequency::Hz16384);
}

#[test]
fn test_interrupt_flags_read() {
    let mut flags = InterruptFlags {
        vblank: true,
        lcd: true,
        timer: true,
        serial: true,
        joypad: true,
    };
    assert_eq!(u8::from(flags), 0b1111_1111);

    flags.vblank = false;
    assert_eq!(u8::from(flags), 0b1111_1110);

    flags.lcd = false;
    assert_eq!(u8::from(flags), 0b1111_1100);

    flags.timer = false;
    assert_eq!(u8::from(flags), 0b1111_1000);

    flags.serial = false;
    assert_eq!(u8::from(flags), 0b1111_0000);

    flags.joypad = false;
    assert_eq!(u8::from(flags), 0b1110_0000);
}

#[test]
fn test_interrupt_flags_write() {
    let mut flags = InterruptFlags::from(0b1111_1111);
    assert_eq!(flags.vblank, true);
    assert_eq!(flags.lcd, true);
    assert_eq!(flags.timer, true);
    assert_eq!(flags.serial, true);
    assert_eq!(flags.joypad, true);

    flags = InterruptFlags::from(0b1111_1110);
    assert_eq!(flags.vblank, false);

    flags = InterruptFlags::from(0b1111_1100);
    assert_eq!(flags.lcd, false);

    flags = InterruptFlags::from(0b1111_1000);
    assert_eq!(flags.timer, false);

    flags = InterruptFlags::from(0b1111_0000);
    assert_eq!(flags.serial, false);

    flags = InterruptFlags::from(0b1110_0000);
    assert_eq!(flags.joypad, false);
}
