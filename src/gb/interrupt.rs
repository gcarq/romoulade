use crate::gb::bus::Bus;
use crate::gb::cpu::{CLOCKS_PER_CYCLE, CPU};
use crate::gb::utils;

const VBLANK_IRQ_ADDRESS: u16 = 0x40;
const LCD_IRQ_ADDRESS: u16 = 0x48;
const TIMER_IRQ_ADDRESS: u16 = 0x50;
const SERIAL_IRQ_ADDRESS: u16 = 0x58;
const JOYPAD_IRQ_ADDRESS: u16 = 0x60;

/// Represents interrupt requests
#[derive(Debug, Copy, Clone, Default)]
pub struct InterruptFlags {
    pub vblank: bool,
    pub lcd: bool,
    pub timer: bool,
    pub serial: bool,
    pub joypad: bool,
}

impl From<u8> for InterruptFlags {
    fn from(value: u8) -> Self {
        Self {
            vblank: utils::bit_at(value, 0),
            lcd: utils::bit_at(value, 1),
            timer: utils::bit_at(value, 2),
            serial: utils::bit_at(value, 3),
            joypad: utils::bit_at(value, 4),
        }
    }
}

impl From<InterruptFlags> for u8 {
    fn from(val: InterruptFlags) -> Self {
        let mut value = 0;
        value = utils::set_bit(value, 0, val.vblank);
        value = utils::set_bit(value, 1, val.lcd);
        value = utils::set_bit(value, 2, val.timer);
        value = utils::set_bit(value, 3, val.serial);
        value = utils::set_bit(value, 4, val.joypad);
        value
    }
}

/// Handles pending interrupt requests
/// TODO: implement HALT instruction bug (Section 4.10): https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
pub fn handle(cpu: &mut CPU, bus: &mut Bus) {
    if !bus.has_interrupt() {
        return;
    }

    // CPU should be always woken up from HALT if there is a pending interrupt
    cpu.is_halted = false;

    if cpu.ime {
        if bus.interrupt_enable.vblank && bus.interrupt_flag.vblank {
            bus.interrupt_flag.vblank = false;
            interrupt(cpu, bus, VBLANK_IRQ_ADDRESS);
        }
        if bus.interrupt_enable.lcd && bus.interrupt_flag.lcd {
            bus.interrupt_flag.lcd = false;
            interrupt(cpu, bus, LCD_IRQ_ADDRESS);
        }
        if bus.interrupt_enable.timer && bus.interrupt_flag.timer {
            bus.interrupt_flag.timer = false;
            interrupt(cpu, bus, TIMER_IRQ_ADDRESS);
        }
        if bus.interrupt_enable.serial && bus.interrupt_flag.serial {
            bus.interrupt_flag.serial = false;
            interrupt(cpu, bus, SERIAL_IRQ_ADDRESS);
        }
        if bus.interrupt_enable.joypad && bus.interrupt_flag.joypad {
            bus.interrupt_flag.joypad = false;
            interrupt(cpu, bus, JOYPAD_IRQ_ADDRESS);
        }
    }
}

/// Handles interrupt request
fn interrupt(cpu: &mut CPU, bus: &mut Bus, address: u16) {
    cpu.ime = false;
    // Save current execution address by pushing it onto the stack
    cpu.push(cpu.pc, bus);
    cpu.pc = address;
    cpu.clock.advance(CLOCKS_PER_CYCLE * 5);
}
