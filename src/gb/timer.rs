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
    pub fn as_cycles(&self) -> u16 {
        match self {
            Frequency::Hz4096 => 1024,
            Frequency::Hz16384 => 256,
            Frequency::Hz65536 => 64,
            Frequency::Hz262144 => 16,
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
}

/// Represents the internal Clock which
/// can be used for each processing unit.
pub struct Clock {
    t_cycle: u16,
}

impl Clock {
    pub fn new() -> Self {
        Self { t_cycle: 0 }
    }

    pub fn advance(&mut self, cycles: u16) {
        self.t_cycle = self.t_cycle.wrapping_add(cycles);
    }

    pub fn ticks(&self) -> u16 {
        self.t_cycle
    }

    pub fn reset(&mut self) {
        self.t_cycle = 0;
    }
}
