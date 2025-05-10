use crate::gb::SubSystem;
use crate::gb::bus::InterruptRegister;
use crate::gb::constants::*;
use crate::gb::timer::{Timer, TimerControl};

#[test]
fn test_timer_counter_no_overflow() {
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
fn test_timer_read() {
    let mut timer = Timer::default();
    timer.divider = 0b1111_1111;
    timer.counter = 0b1010_1010;
    timer.modulo = 0b1011_1011;
    timer.control = TimerControl::from_bits_truncate(0b0000_0100);

    assert_eq!(timer.read(TIMER_DIVIDER), 0b0000_0011);
    assert_eq!(timer.read(TIMER_COUNTER), 0b1010_1010);
    assert_eq!(timer.read(TIMER_MODULO), 0b1011_1011);
    assert_eq!(
        timer.read(TIMER_CTRL),
        0b11111100,
        "Undocumented bits should be 1"
    );
}
