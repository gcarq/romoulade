use crate::gb::bus::InterruptRegister;
use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::cpu::tests::assert_flags;
use crate::gb::cpu::{ImeState, CPU};
use crate::gb::tests::MockBus;
use crate::gb::Bus;

#[test]
fn test_illegal_opcodes() {
    let mut bus = MockBus::new(vec![
        0xd3, 0xdb, 0xdd, 0xe3, 0xe4, 0xeb, 0xec, 0xed, 0xf4, 0xfc, 0xfd,
    ]);
    let mut cpu = CPU::default();
    assert_eq!(cpu.r.pc, 0);
    for i in 0..10 {
        cpu.step(&mut bus);
        assert_eq!(cpu.r.pc, i + 1, "Illegal opcodes should be ignored");
    }
}

#[test]
fn test_add_a_hli_no_overflow() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x42]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.a, 0x42);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_add_a_hli_overflow_zero() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x02]);
    let mut cpu = CPU::default();
    cpu.r.a = 0xff;
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.a, 0x01);
    assert_flags(cpu.r.f, false, false, true, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_add_hl_de_no_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x04);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_add_hl_de_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0xFFFE);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x0001);
    assert_flags(cpu.r.f, false, false, true, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_add_sp_s8_overflow_inc() {
    // ADD SP, s8
    let mut bus = MockBus::new(vec![0xe8, 0x01]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0xffff;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(cpu.r.sp, 0x0000);
    assert_flags(cpu.r.f, false, false, true, true);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_add_sp_s8_overflow_dec() {
    // ADD SP, s8
    let value = -1i8;
    let mut bus = MockBus::new(vec![0xe8, value as u8]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(cpu.r.sp, 0xffff);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_adc_a_e_non_zero() {
    // ADC A, E
    let mut bus = MockBus::new(vec![0x8b]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1111_0001;
    cpu.r.e = 0b0000_0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.a, 0b1111_0010);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_adc_a_d8_non_zero() {
    // ADC A, D8
    let mut bus = MockBus::new(vec![0xce, 0x01]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1111_0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(cpu.r.a, 0b1111_0010);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_and_a_b_non_zero() {
    // AND A
    let mut bus = MockBus::new(vec![0xa0]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.b = 0xff;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x02);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, true, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_and_a_b_zero() {
    // AND A
    let mut bus = MockBus::new(vec![0xa0]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.b = 0x04;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, true, false, true, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_bit_7_h_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::default();
    cpu.r.h = 0b01111111;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, true, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_bit_7_h_non_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::default();
    cpu.r.h = 0b11010000;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, true, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_call_a16() {
    // CALL a16
    let mut bus = MockBus::new(vec![0xcd, 0x11, 0x22, 0x33, 0x44]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0x0003;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0x00);
    assert_eq!(bus.cycle_read(0x01), 0x03);
    assert_eq!(cpu.r.pc, 0x2211);
    assert_eq!(cpu.r.sp, 0x01);
    assert_eq!(bus.cycles, 8);
}

#[test]
fn test_call_c_16_no_jump() {
    // CALL C, a16
    let mut bus = MockBus::new(vec![0xdc, 0x11, 0x22]);
    let mut cpu = CPU::default();
    cpu.r.f.remove(FlagsRegister::CARRY);
    cpu.r.sp = 0x03;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0x22);
    assert_eq!(bus.cycle_read(0x01), 0x11);
    assert_eq!(cpu.r.pc, 0x03);
    assert_eq!(cpu.r.sp, 0x03);
    assert_eq!(bus.cycles, 5);
}

#[test]
fn test_ccf_no_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_ccf_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_cp_b() {
    // CP B
    let mut bus = MockBus::new(vec![0xb8]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.b = 0x01;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, false, true, false, false);
    assert_eq!(cpu.r.a, 0x02);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_cpl() {
    // CPL
    let mut bus = MockBus::new(vec![0x2f; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b11010011;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, false, true, true, false);
    assert_eq!(cpu.r.a, 0b00101100);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_sub_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.insert(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xe4);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, true, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_sub_half_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.insert(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::HALF_CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x3e);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, true, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_non_sub_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.remove(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xa4);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_non_sub_0xd1() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b11010001;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x31, "(0b11010001 + 0x60) % 256 should be 0x31");
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_non_sub_0x0f() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b00001111;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x15, "(0b00001111 + 0x06) % 256 should be 0x15");
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_non_sub_half_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.remove(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::HALF_CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x4a);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_daa_zero() {
    // DAA
    let mut bus = MockBus::new(vec![0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_di() {
    // DI
    let mut bus = MockBus::new(vec![0xf3; 1]);
    let mut cpu = CPU {
        ime: ImeState::Enabled,
        ..Default::default()
    };
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Disabled);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_dec_b_no_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x02;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0x1);
    assert_flags(cpu.r.f, false, true, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_dec_b_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0xff);
    assert_flags(cpu.r.f, false, true, true, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_dec_b_zero() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x01;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0x00);
    assert_flags(cpu.r.f, true, true, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_dec_bc_word() {
    // DEC BC
    let mut bus = MockBus::new(vec![0x0b; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_bc(0x42);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.get_bc(), 0x41);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_ei() {
    // EI
    let mut bus = MockBus::new(vec![0xfb, 0x00]);
    let mut cpu = CPU {
        ime: ImeState::Disabled,
        ..Default::default()
    };
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Pending);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_ei_di_rapid() {
    // Rapid EI/DI should not result in any interrupts
    let mut bus = MockBus::new(vec![0xfb, 0xf3]);
    bus.set_ie(InterruptRegister::VBLANK);
    bus.set_if(InterruptRegister::VBLANK);
    let mut cpu = CPU {
        ime: ImeState::Disabled,
        ..Default::default()
    };
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Pending);
    assert_eq!(cpu.r.pc, 1);
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Disabled);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_ei_sequence() {
    // EI should be set to enabled after the next instruction
    let mut bus = MockBus::new(vec![0xfb, 0xfb]);
    let mut cpu = CPU {
        ime: ImeState::Disabled,
        ..Default::default()
    };
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Pending);
    assert_eq!(cpu.r.pc, 1);
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Enabled);
    assert_eq!(cpu.r.pc, 2);
}

#[test]
fn test_halt() {
    // HALT
    let mut bus = MockBus::new(vec![0x76; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert!(cpu.is_halted);

    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1, "HALT should not change PC");
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_halt_bug() {
    // Tests the HALT instruction bug
    // ADDR DATA     INSTRUCTIONS
    // 0000 76       halt
    // 0001 06 04    ld B,4
    //
    // The byte 0x06 is read twice, and the CPU effectively sees this stream of data:
    // ADDR DATA     INSTRUCTIONS
    // 0000 76       halt
    // 0001 06 06    ld B,6
    // 0002 04       inc B

    let mut bus = MockBus::new(vec![0x76, 0x06, 0x04]);
    bus.set_ie(InterruptRegister::VBLANK);
    bus.set_if(InterruptRegister::VBLANK);
    assert!(bus.has_irq());

    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert!(cpu.is_halted, "CPU should be halted normally");

    cpu.step(&mut bus);
    assert!(!cpu.is_halted, "CPU should be woken up due to pending IRQ");
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(cpu.r.b, 0x06, "B should be 6");

    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(cpu.r.b, 0x07, "B should be 7");
}

#[test]
fn test_inc_b_no_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0x01);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_inc_b_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b1111_1111;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0b0000_0000);
    assert_flags(cpu.r.f, true, false, true, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_inc_b_half_carry() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b0000_1111;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.b, 0b0001_0000);
    assert_flags(cpu.r.f, false, false, true, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_inc_de() {
    // INC DE
    let mut bus = MockBus::new(vec![0x13]);
    let mut cpu = CPU::default();
    cpu.r.set_de(0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.get_de(), 0x02);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_inc_hl() {
    // INC (HL)
    let mut bus = MockBus::new(vec![0x34, 0x03]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x01), 0x04);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_jr_s8_neg_offset() {
    // JR s8
    let mut bus = MockBus::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 251]);
    let mut cpu = CPU::default();
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    // At this point 7 byte have been consumed.
    // pc must be 7 - 5 (offset)
    assert_eq!(cpu.r.pc, 0x02);
    assert_eq!(bus.cycles, 8);
}

#[test]
fn test_jr_s8_pos_offset() {
    // JR s8
    let mut bus = MockBus::new(vec![0x18, 0x03]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 0x05);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_jr_nz_s8_no_jump() {
    // JR NZ, s8
    let mut bus = MockBus::new(vec![0x20, 0x03]);
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::ZERO);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_jp_a16_jump() {
    // JP a16
    let mut bus = MockBus::new(vec![0xc3, 0x01, 0x02]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 0x0201);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_jp_nc_no_jump() {
    // JP NC, a16
    let mut bus = MockBus::new(vec![0xd2, 0x01, 0x02]);
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_ld_c_a() {
    // LD C, A
    let mut bus = MockBus::new(vec![0x4f; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0x42);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_ld_bc_d16() {
    // LD BC, d16
    let mut bus = MockBus::new(vec![0x01, 0x42, 0x00]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.get_bc(), 0x0042);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_ld_a_a16() {
    // LD A, (a16)
    let mut bus = MockBus::new(vec![0xFA, 0x05, 0x00, 0x01, 0x02, 0x03]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x03);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_ld_hl_d8() {
    // LD (HL), D8
    let mut bus = MockBus::new(vec![0x36, 0x42, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0x42);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_ld_hl_plus_a() {
    // LD (HL+), A
    let mut bus = MockBus::new(vec![0x22, 0x00, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.r.a = 0x42;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0x42);
    assert_eq!(cpu.r.get_hl(), 0x03);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_ld_hl_minus_a() {
    // LD (HL-), A
    let mut bus = MockBus::new(vec![0x32, 0x00, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.r.a = 0x42;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0x42);
    assert_eq!(cpu.r.get_hl(), 0x01);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_ld_a16_a() {
    // LD (a16), A
    let mut bus = MockBus::new(vec![0xea, 0x05, 0x00, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x0005), 0x42);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(bus.cycles, 5);
}

#[test]
fn test_ld_a_hl_plus() {
    // LD A, (HL+)
    let mut bus = MockBus::new(vec![0x2a, 0x00, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x11);
    assert_eq!(cpu.r.get_hl(), 0x03);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_ld_a_hl_minus() {
    // LD A, (HL-)
    let mut bus = MockBus::new(vec![0x3a, 0x00, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x11);
    assert_eq!(cpu.r.get_hl(), 0x01);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_ld_a16_sp() {
    // LD (a16), SP
    let mut bus = MockBus::new(vec![0x08, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0xdead;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x0005), 0xad);
    assert_eq!(bus.cycle_read(0x0006), 0xde);
    assert_eq!(cpu.r.pc, 3);
    assert_eq!(bus.cycles, 7);
}

#[test]
fn test_ld_hl_sp_s8_pos() {
    // LD HL, SP+s8
    let mut bus = MockBus::new(vec![0xf8, 0x01]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0x0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.sp, 0x0001);
    assert_eq!(cpu.r.get_hl(), 0x0002);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_ld_hl_sp_s8_neg() {
    // LD HL, SP+s8
    let value = -1i8;
    let mut bus = MockBus::new(vec![0xf8, value as u8]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0x0009;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.sp, 0x0009);
    assert_eq!(cpu.r.get_hl(), 0x0008);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_nop() {
    // NOP
    let mut bus = MockBus::new(vec![0x00; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_or_a_c_non_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x01;
    cpu.r.c = 0x03;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_or_a_c_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x00;
    cpu.r.c = 0x00;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_res_4_l() {
    // RES 4, L
    let mut bus = MockBus::new(vec![0xcb, 0xa5]);
    let mut cpu = CPU::default();
    cpu.r.l = 0b1111_1111;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.l, 0b1110_1111);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_ret_z_jump() {
    // RET Z
    let mut bus = MockBus::new(vec![0xc8, 0x00, 0x22, 0x33]);
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::ZERO);
    cpu.r.sp = 0x0002;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 0x3322);
    assert_eq!(cpu.r.sp, 0x0004);
    assert_eq!(bus.cycles, 5);
}

#[test]
fn test_ret_z_no_jump() {
    // RET Z
    let mut bus = MockBus::new(vec![0xc8, 0x00, 0x22, 0x33]);
    let mut cpu = CPU::default();
    cpu.r.f.remove(FlagsRegister::ZERO);
    cpu.r.sp = 0x0002;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(cpu.r.sp, 0x0002);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rlca() {
    // RLCA
    let mut bus = MockBus::new(vec![0x07; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1011_0110;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0110_1101);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_reti() {
    // RETI
    let mut bus = MockBus::new(vec![0xd9, 0x34, 0x12]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0x0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, ImeState::Enabled);
    assert_eq!(cpu.r.pc, 0x1234);
    assert_eq!(bus.cycles, 4);
}

#[test]
fn test_rr_c_non_zero() {
    // RR C
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::default();
    cpu.r.c = 0b0110_0011;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0b1011_0001);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rr_c_zero() {
    // RR C
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::default();
    cpu.r.c = 0x00;
    cpu.r.f.remove(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0x00);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rl_c() {
    // RL C
    let mut bus = MockBus::new(vec![0xcb, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.c = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0b1100_0110);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rl_e_non_zero() {
    // RL E
    let mut bus = MockBus::new(vec![0xcb, 0x13]);
    let mut cpu = CPU::default();
    cpu.r.e = 0b0110_0011;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.e, 0b1100_0111);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rl_e_zero() {
    // RL E
    let mut bus = MockBus::new(vec![0xcb, 0x13]);
    let mut cpu = CPU::default();
    cpu.r.e = 0;
    cpu.r.f.remove(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.e, 0);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rla_non_zero() {
    // RLA
    let mut bus = MockBus::new(vec![0x17]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0110_0011;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1100_0111);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_rla_zero() {
    // RLA
    let mut bus = MockBus::new(vec![0x17]);
    let mut cpu = CPU::default();
    cpu.r.a = 0;
    cpu.r.f.remove(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_rlc_d() {
    // RLC D
    let mut bus = MockBus::new(vec![0xcb, 0x02]);
    let mut cpu = CPU::default();
    cpu.r.d = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.d, 0b1100_0110);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rra() {
    // RRA
    let mut bus = MockBus::new(vec![0x1F; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0011_0001);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_rrc_l_non_zero() {
    // RRC L
    let mut bus = MockBus::new(vec![0xcb, 0x0d]);
    let mut cpu = CPU::default();
    cpu.r.l = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.l, 0b1011_0001);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rrc_l_zero() {
    // RRC L
    let mut bus = MockBus::new(vec![0xcb, 0x0d]);
    let mut cpu = CPU::default();
    cpu.r.l = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.l, 0);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rrca() {
    // RRCA
    let mut bus = MockBus::new(vec![0x0f]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1011_0001);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_rst_00h() {
    // RST 00h

    // Expected execution
    // op: 0x00 -> NOP - 1 cycle
    // op: 0xC7 -> RST(RST00) - 5 cycles
    // op: 0x04 -> INC(B) - 1 cycle
    // op: 0xC9 -> RET(Always) 3 cycles
    // op: 0x0C -> INC(C) - 1 cycle
    let mut bus = MockBus::new(vec![0x04, 0xc9, 0x00, 0xC7, 0x0C, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();

    assert_eq!(cpu.r.sp, 0);
    cpu.r.pc = 0x02;
    cpu.r.sp = 0x07;

    for _ in 0..5 {
        cpu.step(&mut bus);
    }
    assert_eq!(cpu.r.b, 0x01);
    assert_eq!(cpu.r.c, 0x01);
    assert_eq!(cpu.r.pc, 0x05);
    assert_eq!(bus.cycles, 11);
}

#[test]
fn test_sbc_a_d8_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0000_0001;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1111_1100);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, true, true, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_sbc_a_d8_no_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0001_0000;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0000_1100);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, true, true, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_scf() {
    // SCF
    let mut bus = MockBus::new(vec![0x37; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_set_7_hl() {
    // BIT 7, (HL)
    let mut bus = MockBus::new(vec![0xcb, 0xfe, 0b00000010]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x02), 0b10000010);
    assert_eq!(cpu.r.pc, 2);
    assert_eq!(bus.cycles, 5);
}

#[test]
fn test_sla_a_non_zero() {
    // SLA A
    let mut bus = MockBus::new(vec![0xcb, 0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1100_0110);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_sla_a_zero() {
    // SLA A
    let mut bus = MockBus::new(vec![0xcb, 0x27]);
    let mut cpu = CPU::default();
    cpu.r.a = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_sra_e_non_zero() {
    // SRA E
    let mut bus = MockBus::new(vec![0xcb, 0x2b]);
    let mut cpu = CPU::default();
    cpu.r.e = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.e, 0b0011_0001);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_rsa_e_zero() {
    // SRA E
    let mut bus = MockBus::new(vec![0xcb, 0x2b]);
    let mut cpu = CPU::default();
    cpu.r.e = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.e, 0);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_srl_b_non_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.b, 0b0011_0001);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_srl_b_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.b, 0x00);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_stop() {
    // STOP
    let mut bus = MockBus::new(vec![0x10]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_sub_h_non_zero() {
    // SUB H
    let mut bus = MockBus::new(vec![0x94; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.h = 0x01;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x01);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, true, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_sub_h_zero() {
    // SUB H
    let mut bus = MockBus::new(vec![0x94; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.h = 0x02;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, true, true, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_swap_a_non_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1011_1010;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1010_1011);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_swap_a_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::default();
    cpu.r.a = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0);
    assert_eq!(cpu.r.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 2);
}

#[test]
fn test_pop_hl() {
    // POP HL
    let mut bus = MockBus::new(vec![0xe1, 0x11, 0x22]);
    let mut cpu = CPU::default();
    cpu.r.sp = 0x0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.get_hl(), 0x2211);
    assert_eq!(cpu.r.sp, 0x0003);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 3);
}

#[test]
fn test_push_af() {
    // PUSH AF
    let mut bus = MockBus::new(vec![0xf5, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.set_af(0xff);
    cpu.r.sp = 0x03;
    cpu.step(&mut bus);
    assert_eq!(bus.cycle_read(0x01), 0xf0);
    assert_eq!(bus.cycle_read(0x02), 0x00);
    assert_eq!(cpu.r.pc, 1);
    assert_eq!(bus.cycles, 6);
}

#[test]
fn test_xor_a_c_non_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    cpu.r.c = 0x90;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xd2);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(bus.cycles, 1);
}

#[test]
fn test_xor_a_c_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x90;
    cpu.r.c = 0x90;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.r.pc, 1);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(bus.cycles, 1);
}
