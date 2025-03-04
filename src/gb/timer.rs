use crate::gb::interrupt::IRQ;
use crate::gb::memory::constants::{TIMER_COUNTER, TIMER_CTRL, TIMER_DIVIDER, TIMER_MODULO};
use crate::gb::memory::MemoryBus;
use crate::gb::{AddressSpace, CPU_CLOCK_SPEED};
use std::cell::RefCell;

bitflags! {
    struct Control: u8 {
        const SPEED1   = 0b00000001;
        const SPEED2   = 0b00000010;
        const RUNNING  = 0b00000100;
    }
}

/// System Timer, counts up at configurable frequency.
pub struct Timer<'a> {
    bus: &'a RefCell<MemoryBus>,
    timer: u32,
    divider: u32,
    clock_speed: u32,
}

impl<'a> Timer<'a> {
    pub fn new(bus: &'a RefCell<MemoryBus>) -> Self {
        Self {
            bus,
            timer: 0,
            divider: 0,
            clock_speed: CPU_CLOCK_SPEED / 4096,
        }
    }

    pub fn step(&mut self, cycles: u32) {
        // Check if clock is running
        if self.read_ctrl().contains(Control::RUNNING) {
            self.timer += cycles;

            // Check if enough cpu clock cycles have happened to update the timer
            if self.timer >= self.clock_speed {
                self.timer = 0;

                let (new_counter, did_overflow) = self.read(TIMER_COUNTER).overflowing_add(1);
                if did_overflow {
                    // Reset counter to modulo and request interrupt if overflow did happen
                    self.write(TIMER_COUNTER, self.read(TIMER_MODULO));
                    self.bus.borrow_mut().irq(IRQ::Timer);
                } else {
                    self.write(TIMER_COUNTER, new_counter);
                }
            }
        }
        self.update_clock_speed();
        self.update_divider(cycles);
    }

    /// Updates clock frequency
    fn update_clock_speed(&mut self) {
        let freq = self.read(TIMER_CTRL) & 0x03;
        let clock_speed = match freq {
            0 => 1024, // 4096 Hz
            1 => 16,   // 262144 Hz
            2 => 64,   // 65536 Hz
            3 => 256,  // 16382 Hz
            _ => panic!(),
        };
        if clock_speed != self.clock_speed {
            self.timer = 0;
        }
        self.clock_speed = clock_speed;
    }

    /// Updates divider
    fn update_divider(&mut self, cycles: u32) {
        self.divider += cycles;
        if self.divider >= 256 {
            self.divider = 0;
            let divider = self.read(TIMER_DIVIDER);
            self.bus
                .borrow_mut()
                .write_unchecked(TIMER_DIVIDER, divider.wrapping_add(1));
        }
    }

    fn read_ctrl(&self) -> Control {
        Control::from_bits(self.read(TIMER_CTRL)).expect("Got invalid bits!")
    }
}

impl AddressSpace for Timer<'_> {
    fn write(&mut self, address: u16, value: u8) {
        self.bus.borrow_mut().write(address, value)
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }
}

/// Represents the internal Clock which
/// can be used for each processing unit.
pub struct Clock {
    t_cycle: u32,
}

impl Clock {
    pub fn new() -> Self {
        Self { t_cycle: 0 }
    }

    pub fn advance(&mut self, cycles: u32) {
        self.t_cycle = self.t_cycle.wrapping_add(cycles);
    }

    pub fn ticks(&self) -> u32 {
        self.t_cycle
    }

    pub fn reset(&mut self) {
        self.t_cycle = 0;
    }
}
