use crate::gb::bus::InterruptRegister;
use crate::gb::cpu::{ImeState, CPU};
use crate::gb::Bus;

const VBLANK_IRQ_ADDRESS: u16 = 0x0040;
const LCD_IRQ_ADDRESS: u16 = 0x0048;
const TIMER_IRQ_ADDRESS: u16 = 0x0050;
const SERIAL_IRQ_ADDRESS: u16 = 0x0058;
const JOYPAD_IRQ_ADDRESS: u16 = 0x0060;

/// Handles pending interrupt requests.
/// Returns true if an interrupt was handled.
/// TODO: implement HALT instruction bug (Section 4.10):
///  `<https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf>`
pub fn handle<T: Bus>(cpu: &mut CPU, bus: &mut T) {
    if !bus.has_irq() {
        return;
    }

    // CPU should be always woken up from HALT if there is a pending interrupt
    cpu.is_halted = false;

    if cpu.ime != ImeState::Enabled {
        return;
    }

    let mut int_flags = bus.get_if();
    for irq in InterruptRegister::all().iter() {
        if bus.get_ie().contains(irq) && int_flags.contains(irq) {
            int_flags.remove(irq);
            bus.set_if(int_flags);
            let address = match irq {
                InterruptRegister::VBLANK => VBLANK_IRQ_ADDRESS,
                InterruptRegister::STAT => LCD_IRQ_ADDRESS,
                InterruptRegister::TIMER => TIMER_IRQ_ADDRESS,
                InterruptRegister::SERIAL => SERIAL_IRQ_ADDRESS,
                InterruptRegister::JOYPAD => JOYPAD_IRQ_ADDRESS,
                _ => unreachable!(),
            };
            interrupt(cpu, bus, address);
            return;
        }
    }
}

/// Handles interrupt request
#[inline]
fn interrupt<T: Bus>(cpu: &mut CPU, bus: &mut T, address: u16) {
    cpu.ime = ImeState::Disabled;
    bus.cycle();
    bus.cycle();
    // Save current execution address by pushing it onto the stack
    cpu.push(cpu.pc, bus);
    cpu.pc = address;
}
