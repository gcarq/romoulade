use crate::gb::bus::InterruptRegister;
use crate::gb::cpu::{interrupt, ImeState, CPU};
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
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.sp, 0);
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
            ime: ImeState::Enabled,
            ..Default::default()
        };
        cpu.r.sp = 0x0002;
        cpu.r.pc = 0x1234;
        let mut bus = MockBus::new(vec![0x00; 100]);
        bus.set_ie(irq);
        bus.set_if(irq);

        interrupt::handle(&mut cpu, &mut bus);

        assert_eq!(cpu.r.pc, address, "PC should be set to {address:#06x}");
        assert_eq!(
            bus.get_if(),
            InterruptRegister::empty(),
            "IF should be cleared"
        );
        assert_eq!(bus.get_ie(), irq, "IE should remain unchanged");
        assert_eq!(cpu.ime, ImeState::Disabled, "IME should be disabled");
        assert_eq!(bus.read(0x0000), 0x34, "Should contain low bits of old PC");
        assert_eq!(bus.read(0x0001), 0x12, "Should contain high bits of old PC");
        assert_eq!(cpu.r.sp, 0x0000, "SP should be decremented by 2");
    }
}

#[test]
fn test_interrupt_ie_push_high_bits() {
    let mut cpu = CPU {
        ime: ImeState::Enabled,
        ..Default::default()
    };
    cpu.r.sp = 0x0000;
    cpu.r.pc = 0x1234;
    let mut bus = MockBus::new(vec![0x00; 65536]);
    bus.set_ie(InterruptRegister::VBLANK);
    bus.set_if(InterruptRegister::VBLANK);

    interrupt::handle(&mut cpu, &mut bus);
    assert_eq!(bus.read(0xFFFF), 0x12, "IE should contain high bits of old PC");
    assert_eq!(bus.read(0xFFFE), 0x34, "Address should contain low bits of old PC");
    assert_eq!(cpu.r.sp, 0xFFFE, "SP should be decremented by 2");
    assert_eq!(cpu.r.pc, 0x0000, "PC should be set to 0x000");
}

#[test]
fn test_interrupt_ie_push_low_bits() {
    let mut cpu = CPU {
        ime: ImeState::Enabled,
        ..Default::default()
    };
    cpu.r.sp = 0x0001;
    cpu.r.pc = 0x1234;
    let mut bus = MockBus::new(vec![0x00; 65536]);
    bus.set_ie(InterruptRegister::VBLANK);
    bus.set_if(InterruptRegister::VBLANK);

    interrupt::handle(&mut cpu, &mut bus);
    assert_eq!(bus.read(0x0000), 0x12, "IE should contain high bits of old PC");
    assert_eq!(bus.read(0xFFFF), 0x34, "Address should contain low bits of old PC");
    assert_eq!(cpu.r.sp, 0xFFFF, "SP should be decremented by 2");
    assert_eq!(cpu.r.pc, 0x0040, "VBLANK IRQ should still be handled normally");
}

#[test]
fn test_int_reg_highest_prio() {
    let mut flags = InterruptRegister::all();
    flags.insert(InterruptRegister::VBLANK);
    assert_eq!(flags.highest_prio(), Some(InterruptRegister::VBLANK));

    flags.remove(InterruptRegister::VBLANK);
    assert_eq!(flags.highest_prio(), Some(InterruptRegister::STAT));

    flags.remove(InterruptRegister::STAT);
    assert_eq!(flags.highest_prio(), Some(InterruptRegister::TIMER));

    flags.remove(InterruptRegister::TIMER);
    assert_eq!(flags.highest_prio(), Some(InterruptRegister::SERIAL));

    flags.remove(InterruptRegister::SERIAL);
    assert_eq!(flags.highest_prio(), Some(InterruptRegister::JOYPAD));
    flags.remove(InterruptRegister::JOYPAD);

    assert_eq!(flags.highest_prio(), None);
}