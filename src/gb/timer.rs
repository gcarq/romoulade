use crate::gb::utils::set_bit;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Frequency {
    Hz4096,
    Hz16384,
    Hz65536,
    Hz262144,
}

impl Frequency {
    /// Returns the number of CPU cycles for the given frequency.
    /// This is equal to the number of cpu cycles per second (4194304)
    /// divided by the timer frequency.
    #[inline]
    pub fn as_cycles(&self) -> u16 {
        match self {
            Frequency::Hz4096 => 1024,
            Frequency::Hz16384 => 256,
            Frequency::Hz65536 => 64,
            Frequency::Hz262144 => 16,
        }
    }
}

impl From<u8> for Frequency {
    #[inline]
    fn from(value: u8) -> Self {
        match value & 0b11 {
            0b00 => Frequency::Hz4096,
            0b01 => Frequency::Hz262144,
            0b10 => Frequency::Hz65536,
            0b11 => Frequency::Hz16384,
            _ => unreachable!(),
        }
    }
}

impl From<Frequency> for u8 {
    #[inline]
    fn from(value: Frequency) -> Self {
        match value {
            Frequency::Hz4096 => 0b00,
            Frequency::Hz262144 => 0b01,
            Frequency::Hz65536 => 0b10,
            Frequency::Hz16384 => 0b11,
        }
    }
}

pub struct Timer {
    pub frequency: Frequency,
    cycles: u16,
    pub value: u8,
    pub modulo: u8,
    pub on: bool,
}

impl Timer {
    #[inline]
    pub fn new(frequency: Frequency) -> Self {
        Self {
            frequency,
            cycles: 0,
            value: 0,
            modulo: 0,
            on: false,
        }
    }

    pub fn step(&mut self, cycles: u16) -> bool {
        if !self.on {
            return false;
        }

        let mut irq = false;

        self.cycles += cycles;
        let cycles_per_tick = self.frequency.as_cycles();
        while self.cycles >= cycles_per_tick {
            self.cycles -= cycles_per_tick;

            let (counter, overflow) = match self.value.checked_add(1) {
                Some(counter) => (counter, false),
                None => (self.modulo, true),
            };

            self.value = counter;
            if overflow {
                irq = true
            }
        }
        irq
    }

    /// Sets the frequency and the on/off state of the timer
    /// based on the given value.
    #[inline]
    pub fn write_control(&mut self, value: u8) {
        self.frequency = Frequency::from(value);
        self.on = (value & 0b100) == 0b100;
    }

    /// Returns the current control state of the timer
    /// when read from the TAC register.
    #[inline]
    pub fn read_control(&self) -> u8 {
        let state = u8::from(self.frequency);
        let state = set_bit(state, 2, self.on);
        state | 0b1111_1000 // Undocumented bits should be 1
    }
}

/// Represents the internal Clock which
/// can be used for each processing unit.
#[derive(Default)]
pub struct Clock {
    t_cycle: u16,
}

impl Clock {
    #[inline]
    pub fn advance(&mut self, cycles: u16) {
        self.t_cycle = self.t_cycle.wrapping_add(cycles);
    }

    #[inline]
    pub fn ticks(&self) -> u16 {
        self.t_cycle
    }

    #[inline]
    pub fn reset(&mut self) {
        self.t_cycle = 0;
    }
}
