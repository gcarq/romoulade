use crate::gb::SubSystem;
use crate::gb::bus::InterruptRegister;
use crate::gb::constants::{TIMER_COUNTER, TIMER_CTRL, TIMER_DIVIDER, TIMER_MODULO};

bitflags! {
    /// Represents the control register TAC at 0xFF07
    #[derive(Copy, Clone, Default, PartialEq)]
    pub struct TimerControl: u8 {
        const TIMER_FREQ   = 0b0000_0011; // Frequency select
        const TIMER_ENABLE = 0b0000_0100; // Enable timer
    }
}

impl TimerControl {
    /// Returns the counter mask used for edge detection.
    #[inline]
    pub fn divider_mask(self) -> u16 {
        match self.bits() & Self::TIMER_FREQ.bits() {
            0b00 => 1 << 7, // 4096 Hz
            0b01 => 1 << 1, // 262144 Hz
            0b10 => 1 << 3, // 65536 Hz
            0b11 => 1 << 5, // 16384 Hz
            _ => unreachable!(),
        }
    }

    /// Returns whether the timer is enabled.
    #[inline]
    pub const fn is_enabled(self) -> bool {
        self.contains(Self::TIMER_ENABLE)
    }
}

/// This struct holds all timer related registers.
/// See https://gbdev.io/pandocs/Timer_and_Divider_Registers.html
#[derive(Clone, Default)]
pub struct Timer {
    // DIV. The upper bits of the divider are mapped to memory but the two extra bits
    // at the top are not visible. Hence, we need to right shift the divider by 6 bits.
    pub divider: u16,
    pub counter: u8,           // TIMA
    pub modulo: u8,            // TMA
    pub control: TimerControl, // TAC
    counter_overflow: bool,    // Indicates whether the counter overflowed during the last cycle
}

impl Timer {
    /// Steps the `Timer` by one cycle.
    pub fn cycle(&mut self, int_flag: &mut InterruptRegister) {
        let prev_divider = self.divider;
        self.divider = self.divider.wrapping_add(1);

        // Handle TIMA overflow
        if self.counter_overflow {
            self.counter = self.modulo;
            self.counter_overflow = false;
            int_flag.insert(InterruptRegister::TIMER);
        } else if self.control.is_enabled() {
            // Detect falling edge
            if self.edge_bit(prev_divider) && !self.edge_bit(self.divider) {
                self.increment_counter();
            }
        }
    }

    /// Returns the current state of the divider bit for the selected frequency.
    #[inline]
    fn edge_bit(&self, divider: u16) -> bool {
        (divider & self.control.divider_mask()) != 0
    }

    /// Increments the counter and handles overflow.
    #[inline]
    const fn increment_counter(&mut self) {
        let (counter, overflow) = self.counter.overflowing_add(1);
        self.counter = counter;
        self.counter_overflow = overflow;
    }

    /// Writes a value to the divider (DIV).
    #[inline]
    fn write_divider(&mut self) {
        if self.edge_bit(self.divider) {
            self.increment_counter();
        }
        self.divider = 0;
    }

    /// Writes a value to the counter (TIMA).
    #[inline]
    const fn write_counter(&mut self, value: u8) {
        if !self.counter_overflow {
            self.counter = value;
        }
    }

    /// Writes a value to the modulo (TMA).
    #[inline]
    const fn write_modulo(&mut self, value: u8) {
        self.modulo = value;
        if self.counter_overflow {
            self.counter = value;
        }
    }

    /// Writes a value to the control register (TAC).
    fn write_control(&mut self, value: u8) {
        let old_bit = self.control.is_enabled() && self.edge_bit(self.divider);
        self.control = TimerControl::from_bits_truncate(value);
        let new_bit = self.control.is_enabled() && self.edge_bit(self.divider);
        if old_bit && !new_bit {
            self.increment_counter();
        }
    }
}

impl SubSystem for Timer {
    fn write(&mut self, address: u16, value: u8) {
        match address {
            TIMER_DIVIDER => self.write_divider(),
            TIMER_COUNTER => self.write_counter(value),
            TIMER_MODULO => self.write_modulo(value),
            TIMER_CTRL => self.write_control(value),
            _ => unreachable!(),
        }
    }

    fn read(&mut self, address: u16) -> u8 {
        match address {
            TIMER_DIVIDER => (self.divider >> 6) as u8,
            TIMER_COUNTER => self.counter,
            TIMER_MODULO => self.modulo,
            TIMER_CTRL => self.control.bits() | 0b1111_1000, // Undocumented bits should be 1,
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
#[allow(clippy::field_reassign_with_default)]
mod tests {
    use super::*;

    #[test]
    fn test_timer_counter_no_overflow() {
        let mut int_reg = InterruptRegister::empty();
        let mut timer = Timer::default();
        timer.control = TimerControl::from_bits_truncate(0b0000_0101);
        timer.divider = 0b0001_0111;
        assert!(timer.control.is_enabled());

        timer.cycle(&mut int_reg);
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
        timer.cycle(&mut int_reg);
        assert_eq!(timer.counter, 0b0000_0000);
        assert!(!int_reg.contains(InterruptRegister::TIMER));

        timer.cycle(&mut int_reg);
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
}
