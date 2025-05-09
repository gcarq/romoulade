use crate::gb::joypad::{ActionInput, DPadInput, Joypad, JoypadInput};

#[test]
fn test_joypad_dpad_pressed() {
    let mut joypad = Joypad::default();

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
    assert_eq!(joypad.read(), 0b1111_1111);

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
