use crate::gb::bus::InterruptRegister;
use crate::gb::cpu::{CPU, ImeState, interrupt};
use crate::gb::tests::MockBus;
use crate::gb::{Bus, SubSystem};

#[test]
fn test_interrupt_ime_disabled() {
    let mut cpu = CPU {
        is_halted: true,
        ime: ImeState::Disabled,
        ..Default::default()
    };
    let mut bus = MockBus::new(vec![0x00]);
    bus.set_ie(InterruptRegister::VBLANK);
    bus.set_if(InterruptRegister::VBLANK);
    cpu.step(&mut bus);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.sp, 0);
    assert!(
        !cpu.is_halted,
        "CPU should always wake up from HALT if an interrupt is pending"
    );
}

#[test]
fn test_interrupt_ime_enabled() {
    let data = [
        (InterruptRegister::VBLANK, 0x0040),
        (InterruptRegister::STAT, 0x0048),
        (InterruptRegister::TIMER, 0x0050),
        (InterruptRegister::SERIAL, 0x0058),
        (InterruptRegister::JOYPAD, 0x0060),
    ];

    for (irq, address) in data {
        let mut cpu = CPU {
            sp: 0x0002,
            pc: 0x1234,
            ..Default::default()
        };
        let mut bus = MockBus::new(vec![0x00; 100]);
        bus.set_ie(irq);
        bus.set_if(irq);

        interrupt::handle(&mut cpu, &mut bus);

        assert_eq!(cpu.pc, address, "PC should be set to {address:#06x}");
        assert_eq!(
            bus.get_if(),
            InterruptRegister::empty(),
            "IF should be cleared"
        );
        assert_eq!(bus.get_ie(), irq, "IE should remain unchanged");
        assert_eq!(cpu.ime, ImeState::Disabled, "IME should be disabled");
        assert_eq!(bus.read(0x0000), 0x34, "Should contain old PC (lower bits)");
        assert_eq!(bus.read(0x0001), 0x12, "Should contain old PC (upper bits)");
        assert_eq!(cpu.sp, 0x0000, "SP should be decremented by 2");
    }
}
