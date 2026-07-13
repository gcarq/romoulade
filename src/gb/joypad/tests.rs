use super::*;
use bitflags::Flags;

#[test]
fn test_joypad_dpad_pressed() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();

    let data = [
        (
            JoypadInputState {
                right: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1110,
        ),
        (
            JoypadInputState {
                left: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1101,
        ),
        (
            JoypadInputState {
                up: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1011,
        ),
        (
            JoypadInputState {
                down: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_0111,
        ),
    ];

    for (input, initial, expected) in data {
        joypad.write(initial, &mut int_reg);
        int_reg.clear();
        joypad.handle_input(input, &mut int_reg);
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
            JoypadInputState {
                a: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputState {
                b: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputState {
                select: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
        (
            JoypadInputState {
                start: true,
                ..Default::default()
            },
            0b1110_1111,
            0b1110_1111,
        ),
    ];

    for (input, initial, expected) in data {
        joypad.write(initial, &mut int_reg);
        int_reg.clear();
        joypad.handle_input(input, &mut int_reg);
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
            JoypadInputState {
                a: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1110,
        ),
        (
            JoypadInputState {
                b: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1101,
        ),
        (
            JoypadInputState {
                select: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1011,
        ),
        (
            JoypadInputState {
                start: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_0111,
        ),
    ];

    for (input, initial, expected) in data {
        joypad.write(initial, &mut int_reg);
        int_reg.clear();
        joypad.handle_input(input, &mut int_reg);
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
            JoypadInputState {
                right: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputState {
                left: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputState {
                up: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
        (
            JoypadInputState {
                down: true,
                ..Default::default()
            },
            0b1101_1111,
            0b1101_1111,
        ),
    ];

    for (input, initial, expected) in data {
        joypad.write(initial, &mut int_reg);
        int_reg.clear();
        joypad.handle_input(input, &mut int_reg);
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
    let input = JoypadInputState {
        right: true,
        ..Default::default()
    };
    joypad.write(0b0011_0000, &mut int_reg);
    joypad.handle_input(input, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);

    // No selection mode while Select has been pressed
    let input = JoypadInputState {
        select: true,
        ..Default::default()
    };
    joypad.write(0b0011_0000, &mut int_reg);
    joypad.handle_input(input, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);

    // Acceptance test of mooneye test suite:
    // See https://github.com/Gekkio/mooneye-test-suite/blob/main/acceptance/bits/unused_hwio-GS.s
    joypad.handle_input(JoypadInputState::default(), &mut int_reg);
    joypad.write(0b1111_1111, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1111_1111);
}

#[test]
fn test_joypad_release_is_visible_without_register_write() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();
    joypad.write(0b1110_1111, &mut int_reg);

    joypad.handle_input(
        JoypadInputState {
            right: true,
            ..Default::default()
        },
        &mut int_reg,
    );
    assert!(int_reg.contains(InterruptRegister::JOYPAD));
    assert_eq!(joypad.read(), 0b1110_1110);

    int_reg.clear();
    joypad.handle_input(JoypadInputState::default(), &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1110_1111);
}

#[test]
fn test_joypad_input_survives_register_writes() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();
    joypad.write(0b1110_1111, &mut int_reg);
    let input = JoypadInputState {
        right: true,
        ..Default::default()
    };
    joypad.handle_input(input, &mut int_reg);
    assert!(int_reg.contains(InterruptRegister::JOYPAD));

    int_reg.clear();
    joypad.handle_input(input, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1110_1110);

    joypad.write(0b1110_1111, &mut int_reg);
    assert!(int_reg.is_empty());
    assert_eq!(joypad.read(), 0b1110_1110);
}

#[test]
fn test_joypad_selection_irq() {
    let mut joypad = Joypad::default();
    let mut int_reg = InterruptRegister::empty();
    joypad.write(0b1111_1111, &mut int_reg);
    joypad.handle_input(
        JoypadInputState {
            right: true,
            ..Default::default()
        },
        &mut int_reg,
    );
    assert!(int_reg.is_empty());

    joypad.write(0b1110_1111, &mut int_reg);
    assert!(int_reg.contains(InterruptRegister::JOYPAD));
    assert_eq!(joypad.read(), 0b1110_1110);
}
