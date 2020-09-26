use crate::gb::memory::constants::TIMER_CTRL;
use crate::gb::memory::MemoryBus;
use crate::gb::AddressSpace;
use std::cell::RefCell;

bitflags! {
    struct Control: u8 {
        // Speed:
        //     00: 4096Hz
        //     01: 262144Hz
        //     10: 65536Hz
        //     11: 16384Hz
        const SPEED1   = 0b00000001;
        const SPEED2   = 0b00000010;
        const RUNNING  = 0b00000100;
    }
}

/// TODO: handle interrupt on overflow
pub struct Timer<'a> {
    bus: &'a RefCell<MemoryBus>,
}

impl<'a> Timer<'a> {
    pub fn new(bus: &'a RefCell<MemoryBus>) -> Self {
        Self { bus }
    }

    pub fn step(&mut self, cycles: u32) {
        // TODO: update DoDividerRegister with passed cycles

        // Check if clock is running
        if !self.read_ctrl().contains(Control::RUNNING) {
            return;
        }

        unimplemented!("TODO: implement timer");
    }

    fn write_ctrl(&mut self, ctrl: Control) {
        self.write(TIMER_CTRL, ctrl.bits)
    }

    fn read_ctrl(&self) -> Control {
        Control::from_bits(self.read(TIMER_CTRL)).expect("Got invalid bits!")
    }
}

impl<'a> AddressSpace for Timer<'a> {
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
