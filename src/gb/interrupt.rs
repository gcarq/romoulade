use crate::gb::bus::Bus;
use crate::gb::cpu::{CLOCKS_PER_CYCLE, CPU};

const VBLANK_IRQ_ADDRESS: u16 = 0x40;
const LCD_IRQ_ADDRESS: u16 = 0x48;
const TIMER_IRQ_ADDRESS: u16 = 0x50;
const SERIAL_IRQ_ADDRESS: u16 = 0x58;
const JOYPAD_IRQ_ADDRESS: u16 = 0x60;

bitflags! {
    /// Represents interrupt registers IE at 0xFFFF and IF at 0xFF0F
    #[derive(Copy, Clone, PartialEq)]
    pub struct InterruptRegister: u8 {
        const VBLANK = 0b00000001; // V-Blank Interrupt
        const STAT   = 0b00000010; // LCD STAT Interrupt
        const TIMER  = 0b00000100; // Timer Overflow Interrupt
        const SERIAL = 0b00001000; // Serial Transfer Completion Interrupt
        const JOYPAD = 0b00010000; // Joypad Input Interrupt
    }
}

/// Handles pending interrupt requests
/// TODO: implement HALT instruction bug (Section 4.10): https://github.com/AntonioND/giibiiadvance/blob/master/docs/TCAGBD.pdf
pub fn handle(cpu: &mut CPU, bus: &mut Bus) {
    if !bus.has_pending_interrupt() {
        return;
    }

    // CPU should be always woken up from HALT if there is a pending interrupt
    cpu.is_halted = false;

    if !cpu.ime {
        return;
    }

    for irq in [
        InterruptRegister::VBLANK,
        InterruptRegister::STAT,
        InterruptRegister::TIMER,
        InterruptRegister::SERIAL,
        InterruptRegister::JOYPAD,
    ] {
        if bus.interrupt_enable.contains(irq) && bus.interrupt_flag.contains(irq) {
            bus.interrupt_flag.remove(irq);
            let address = match irq {
                InterruptRegister::VBLANK => VBLANK_IRQ_ADDRESS,
                InterruptRegister::STAT => LCD_IRQ_ADDRESS,
                InterruptRegister::TIMER => TIMER_IRQ_ADDRESS,
                InterruptRegister::SERIAL => SERIAL_IRQ_ADDRESS,
                InterruptRegister::JOYPAD => JOYPAD_IRQ_ADDRESS,
                _ => unreachable!(),
            };
            interrupt(cpu, bus, address);
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
