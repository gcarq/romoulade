use crate::gb::bus::InterruptRegister;
use crate::gb::joypad::{Joypad, JoypadInputEvent};
use bitflags::Flags;

#[test]
fn test_joypad_dpad_pressed() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();

    let data = [
        (
            JoypadInputEvent {
                right: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1110,
        ),
        (
            JoypadInputEvent {
                left: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1101,
        ),
        (
            JoypadInputEvent {
                up: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1011,
        ),
        (
            JoypadInputEvent {
                down: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_0111,
        ),
    ];

    for (input, initial, expected) in data {
        int_reg.clear();
        joypad.handle_input(input);
        joypad.write(initial, &mut int_reg);
        assert!(int_reg.contains(InterruptRegister::JOYPAD));
        assert_eq!(joypad.read(), expected);
    }
}

#[test]
fn test_joypad_dpad_not_pressed() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();

    let data = [
        (
            JoypadInputEvent {
                a: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputEvent {
                b: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputEvent {
                select: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputEvent {
                start: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
    ];

    for (input, initial, expected) in data {
        int_reg.clear();
        joypad.handle_input(input);
        joypad.write(initial, &mut int_reg);
        assert!(int_reg.is_empty());
        assert_eq!(joypad.read(), expected);
    }
}

#[test]
fn test_joypad_actions_pressed() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();

    let data = [
        (
            JoypadInputEvent {
                a: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1110,
        ),
        (
            JoypadInputEvent {
                b: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1101,
        ),
        (
            JoypadInputEvent {
                select: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1011,
        ),
        (
            JoypadInputEvent {
                start: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_0111,
        ),
    ];

    for (input, initial, expected) in data {
        int_reg.clear();
        joypad.handle_input(input);
        joypad.write(initial, &mut int_reg);
        assert!(int_reg.contains(InterruptRegister::JOYPAD));
        assert_eq!(joypad.read(), expected);
    }
}

#[test]
fn test_joypad_actions_not_pressed() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();

    let data = [
        (
            JoypadInputEvent {
                right: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputEvent {
                left: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputEvent {
                up: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputEvent {
                down: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
    ];

    for (input, initial, expected) in data {
        int_reg.clear();
        joypad.handle_input(input);
        joypad.write(initial, &mut int_reg);
        assert!(int_reg.is_empty());
        assert_eq!(joypad.read(), expected);
    }
}

#[test]
fn test_joypad_common() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();
    assert_eq!(joypad.read(), 0b1100_1111, "Initial state should be 0xCF");

    // No selection mode while Right has been pressed
    let input = JoypadInputEvent {
        right: true,
        ..Default::default()
    };
    joypad.handle_input(input);
    joypad.write(0b0011_0000, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);

    // No selection mode while Select has been pressed
    let input = JoypadInputEvent {
        select: true,
        ..Default::default()
    };
    joypad.handle_input(input);
    joypad.write(0b0011_0000, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);

    // Acceptance test of mooneye test suite:
    // See https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/bits/unused_hwio-GS.s
    joypad.handle_input(JoypadInputEvent::default());
    joypad.write(0b1111_1111, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);
}

#[test]
fn test_joypad_input_is_pressed() {
    let input = JoypadInputEvent::default();
    assert!(!input.is_pressed());
    let input = JoypadInputEvent { a: true, ..input };
    assert!(input.is_pressed());
    let input = JoypadInputEvent { b: true, ..input };
    assert!(input.is_pressed());
    let input = JoypadInputEvent {
        select: true,
        ..input
    };
    assert!(input.is_pressed());
    let input = JoypadInputEvent {
        start: true,
        ..input
    };
    assert!(input.is_pressed());
    let input = JoypadInputEvent { up: true, ..input };
    assert!(input.is_pressed());
    let input = JoypadInputEvent {
        down: true,
        ..input
    };
    assert!(input.is_pressed());
    let input = JoypadInputEvent {
        left: true,
        ..input
    };
    assert!(input.is_pressed());
    let input = JoypadInputEvent {
        right: true,
        ..input
    };
    assert!(input.is_pressed());
}
