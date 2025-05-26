use crate::gb::bus::InterruptRegister;
use crate::gb::cpu::{ImeState, CPU};
use crate::gb::Bus;

const VBLANK_IRQ_ADDRESS: u16 = 0x0040;
const LCD_IRQ_ADDRESS: u16 = 0x0048;
const TIMER_IRQ_ADDRESS: u16 = 0x0050;
const SERIAL_IRQ_ADDRESS: u16 = 0x0058;
const JOYPAD_IRQ_ADDRESS: u16 = 0x0060;

/// Handles pending interrupt requests.
pub fn handle<T: Bus>(cpu: &mut CPU, bus: &mut T) {
    debug_assert!(bus.has_irq());
    debug_assert_eq!(cpu.ime, ImeState::Enabled);
    debug_assert!(!cpu.is_halted);

    cpu.ime = ImeState::Disabled;
    bus.cycle();
    bus.cycle();

    // Save current execution address by pushing it onto the stack
    let [low, high] = cpu.r.pc.to_le_bytes();

    cpu.r.sp = cpu.r.sp.wrapping_sub(1);
    bus.cycle_write(cpu.r.sp, high);

    let int_flags = bus.get_if();
    // After the high byte push it's possible that the interrupt flag register has changed
    let irq = (bus.get_ie() & int_flags).highest_prio();

    // The low byte push is not fast enough to modify the pending IRQ
    cpu.r.sp = cpu.r.sp.wrapping_sub(1);
    bus.cycle_write(cpu.r.sp, low);

    // Clear the flag for the request that is being handled
    if let Some(irq) = irq {
        bus.set_if(int_flags - irq);
    }

    cpu.r.pc = match irq {
        Some(InterruptRegister::VBLANK) => VBLANK_IRQ_ADDRESS,
        Some(InterruptRegister::STAT) => LCD_IRQ_ADDRESS,
        Some(InterruptRegister::TIMER) => TIMER_IRQ_ADDRESS,
        Some(InterruptRegister::SERIAL) => SERIAL_IRQ_ADDRESS,
        Some(InterruptRegister::JOYPAD) => JOYPAD_IRQ_ADDRESS,
        _ => 0x0000,
    };
}