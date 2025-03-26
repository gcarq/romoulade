use crate::gb::joypad::{ActionInput, DPadInput, Joypad, JoypadInput};
use crate::gb::utils::{bit_at, half_carry_u8, set_bit};

#[test]
fn test_joypad_dpad() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b0011_1111);

    // D-Pad selection mode while right has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Right)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0010_1110);

    // D-Pad selection mode while left has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Left)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0010_1101);

    // D-Pad selection mode while up has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Up)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0010_1011);

    // D-Pad selection mode while down has been pressed
    let irq = joypad.write(0b1110_1111, Some(JoypadInput::DPad(DPadInput::Down)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0010_0111);
}

#[test]
fn test_joypad_actions() {
    let mut joypad = Joypad::default();
    assert_eq!(joypad.read(), 0b0011_1111);

    // Action selection mode while A has been pressed
    let irq = joypad.write(0b1101_1111, Some(JoypadInput::Action(ActionInput::A)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b001_1110);

    // Action selection mode while B has been pressed
    let irq = joypad.write(0b1101_1101, Some(JoypadInput::Action(ActionInput::B)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0001_1101);

    // Action selection mode while select has been pressed
    let irq = joypad.write(0b1101_1011, Some(JoypadInput::Action(ActionInput::Select)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0001_1011);

    // Action selection mode while start has been pressed
    let irq = joypad.write(0b1101_0111, Some(JoypadInput::Action(ActionInput::Start)));
    assert!(irq);
    assert_eq!(joypad.read(), 0b0001_0111);
}

#[test]
fn test_joypad_common() {
    let mut joypad = Joypad::default();

    // No selection mode while right has been pressed
    let irq = joypad.write(0b0011_0000, Some(JoypadInput::DPad(DPadInput::Right)));
    assert!(!irq);
    assert_eq!(joypad.read(), 0b0011_1111);
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
