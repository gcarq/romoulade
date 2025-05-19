use crate::gb::Bus;
use crate::gb::cpu::CPU;
use crate::gb::cpu::ops::Register::{A, B, C, D, E, H, L};
use crate::gb::cpu::ops::WordRegister::{AF, BC, DE, HL, SP};
use crate::gb::cpu::ops::{
    ByteRef, ByteSource, ByteTarget, JumpCondition, JumpTarget, Load, ResetCode, WordSource,
};
use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::tests::MockBus;

#[test]
fn test_register_a() {
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    assert_eq!(A.read(&cpu), 0x42);
    A.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.a, 0x24);
    assert_eq!(A.to_string(), "A");
}

#[test]
fn test_register_b() {
    let mut cpu = CPU::default();
    cpu.r.b = 0x42;
    assert_eq!(B.read(&cpu), 0x42);
    B.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.b, 0x24);
    assert_eq!(B.to_string(), "B");
}

#[test]
fn test_register_c() {
    let mut cpu = CPU::default();
    cpu.r.c = 0x42;
    assert_eq!(C.read(&cpu), 0x42);
    C.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.c, 0x24);
    assert_eq!(C.to_string(), "C");
}

#[test]
fn test_register_d() {
    let mut cpu = CPU::default();
    cpu.r.d = 0x42;
    assert_eq!(D.read(&cpu), 0x42);
    D.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.d, 0x24);
    assert_eq!(D.to_string(), "D");
}

#[test]
fn test_register_e() {
    let mut cpu = CPU::default();
    cpu.r.e = 0x42;
    assert_eq!(E.read(&cpu), 0x42);
    E.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.e, 0x24);
    assert_eq!(E.to_string(), "E");
}

#[test]
fn test_register_h() {
    let mut cpu = CPU::default();
    cpu.r.h = 0x42;
    assert_eq!(H.read(&cpu), 0x42);
    H.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.h, 0x24);
    assert_eq!(H.to_string(), "H");
}

#[test]
fn test_register_l() {
    let mut cpu = CPU::default();
    cpu.r.l = 0x42;
    assert_eq!(L.read(&cpu), 0x42);
    L.write(&mut cpu, 0x24);
    assert_eq!(cpu.r.l, 0x24);
    assert_eq!(L.to_string(), "L");
}

#[test]
fn test_word_register_af() {
    let mut cpu = CPU::default();
    cpu.r.set_af(0x1234);
    assert_eq!(AF.read(&cpu), 0x1230, "Lower 4 bits should be ignored");
    AF.write(&mut cpu, 0xABCD);
    assert_eq!(cpu.r.get_af(), 0xABC0, "Lower 4 bits should be ignored");
    assert_eq!(AF.to_string(), "AF");
}

#[test]
fn test_word_register_bc() {
    let mut cpu = CPU::default();
    cpu.r.set_bc(0x1234);
    assert_eq!(BC.read(&cpu), 0x1234);
    BC.write(&mut cpu, 0xABCD);
    assert_eq!(cpu.r.get_bc(), 0xABCD);
    assert_eq!(BC.to_string(), "BC");
}

#[test]
fn test_word_register_de() {
    let mut cpu = CPU::default();
    cpu.r.set_de(0x1234);
    assert_eq!(DE.read(&cpu), 0x1234);
    DE.write(&mut cpu, 0xABCD);
    assert_eq!(cpu.r.get_de(), 0xABCD);
    assert_eq!(DE.to_string(), "DE");
}

#[test]
fn test_word_register_hl() {
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x1234);
    assert_eq!(HL.read(&cpu), 0x1234);
    HL.write(&mut cpu, 0xABCD);
    assert_eq!(cpu.r.get_hl(), 0xABCD);
    assert_eq!(HL.to_string(), "HL");
}

#[test]
fn test_word_register_sp() {
    let mut cpu = CPU::default();
    cpu.r.sp = 0x1234;
    assert_eq!(SP.read(&cpu), 0x1234);
    SP.write(&mut cpu, 0xABCD);
    assert_eq!(cpu.r.sp, 0xABCD);
    assert_eq!(SP.to_string(), "SP");
}

#[test]
fn test_byte_target_r() {
    let mut bus = MockBus::new(vec![]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    let target = ByteTarget::R(A);
    assert_eq!(target.read(&cpu, &mut bus), 0x42);
    target.write(&mut cpu, &mut bus, 0x24);
    assert_eq!(cpu.r.a, 0x24);
    assert_eq!(target.to_string(), "A");
}

#[test]
fn test_byte_target_i() {
    let mut bus = MockBus::new(vec![0x11, 0x22]);
    let mut cpu = CPU::default();
    let target = ByteTarget::I(ByteRef::D16(0x0001));
    assert_eq!(target.read(&cpu, &mut bus), 0x22);
    target.write(&mut cpu, &mut bus, 0x33);
    assert_eq!(bus.cycle_read(0x0001), 0x33);
    assert_eq!(target.to_string(), "(0x0001)");
}

#[test]
fn test_byte_ref_r() {
    let mut cpu = CPU::default();
    cpu.r.set_bc(0xDEAD);
    let byte_ref = ByteRef::R(BC);
    assert_eq!(byte_ref.resolve(&cpu), 0xDEAD);
    assert_eq!(byte_ref.to_string(), "(BC)");
}

#[test]
fn test_byte_ref_d16() {
    let cpu = CPU::default();
    let byte_ref = ByteRef::D16(0xDEAD);
    assert_eq!(byte_ref.resolve(&cpu), 0xDEAD);
    assert_eq!(byte_ref.to_string(), "(0xdead)");
}

#[test]
fn test_byte_ref_c() {
    let mut cpu = CPU::default();
    cpu.r.c = 0x42;
    let byte_ref = ByteRef::C;
    assert_eq!(byte_ref.resolve(&cpu), 0xFF42);
    assert_eq!(byte_ref.to_string(), "(C)");
}

#[test]
fn test_byte_ref_d8() {
    let cpu = CPU::default();
    let byte_ref = ByteRef::D8(0x42);
    assert_eq!(byte_ref.resolve(&cpu), 0xFF42);
    assert_eq!(byte_ref.to_string(), "(0xff42)");
}

#[test]
fn test_byte_source_r() {
    let mut bus = MockBus::new(vec![]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    let source = ByteSource::R(A);
    assert_eq!(source.read(&cpu, &mut bus), 0x42);
    assert_eq!(source.to_string(), "A");
}

#[test]
fn test_byte_source_i() {
    let mut bus = MockBus::new(vec![0x11, 0x22]);
    let cpu = CPU::default();
    let source = ByteSource::I(ByteRef::D16(0x0001));
    assert_eq!(source.read(&cpu, &mut bus), 0x22);
    assert_eq!(source.to_string(), "(0x0001)");
}

#[test]
fn test_byte_source_d8() {
    let mut bus = MockBus::new(vec![]);
    let cpu = CPU::default();
    let source = ByteSource::D8(0x42);
    assert_eq!(source.read(&cpu, &mut bus), 0x42);
    assert_eq!(source.to_string(), "0x42");
}

#[test]
fn test_word_source_r() {
    let mut cpu = CPU::default();
    cpu.r.set_bc(0x1234);
    let source = WordSource::R(BC);
    assert_eq!(source.read(&cpu), 0x1234);
    assert_eq!(source.to_string(), "BC");
}

#[test]
fn test_word_source_d16() {
    let cpu = CPU::default();
    let source = WordSource::D16(0x1234);
    assert_eq!(source.read(&cpu), 0x1234);
    assert_eq!(source.to_string(), "0x1234");
}

#[test]
fn test_load_byte() {
    let load = Load::Byte(ByteTarget::R(A), ByteSource::R(C));
    assert_eq!(load.to_string(), "A, C");
}

#[test]
fn test_load_word() {
    let load = Load::Word(AF, WordSource::R(HL));
    assert_eq!(load.to_string(), "AF, HL");
}

#[test]
fn test_load_indirect_from() {
    let load = Load::IndirectFrom(ByteRef::D16(0x1234), ByteSource::R(A));
    assert_eq!(load.to_string(), "(0x1234), A");
}

#[test]
fn test_load_hli_from_a_inc() {
    let load = Load::HLIFromAInc;
    assert_eq!(load.to_string(), "(HL+), A");
}

#[test]
fn test_load_hli_from_a_dec() {
    let load = Load::HLIFromADec;
    assert_eq!(load.to_string(), "(HL-), A");
}

#[test]
fn test_load_hli_to_a_inc() {
    let load = Load::HLIToAInc;
    assert_eq!(load.to_string(), "A, (HL+)");
}

#[test]
fn test_load_hli_to_a_dec() {
    let load = Load::HLIToADec;
    assert_eq!(load.to_string(), "A, (HL-)");
}

#[test]
fn test_load_indirect_from_sp() {
    let load = Load::IndirectFromSP(ByteRef::D16(0x1234));
    assert_eq!(load.to_string(), "(0x1234), SP");
}

#[test]
fn test_hl_from_sp_i8() {
    let load = Load::HLFromSPi8(10);
    assert_eq!(load.to_string(), "HL, SP+0x0a");

    let load = Load::HLFromSPi8(-10);
    assert_eq!(load.to_string(), "HL, SP-0x0a");
}

#[test]
fn test_jump_condition_not_zero() {
    let mut cpu = CPU::default();
    cpu.r.f.remove(FlagsRegister::ZERO);
    let condition = JumpCondition::NotZero;
    assert!(condition.resolve(&cpu));
    cpu.r.f.insert(FlagsRegister::ZERO);
    assert!(!condition.resolve(&cpu));
    assert_eq!(condition.to_string(), "NZ");
}

#[test]
fn test_jump_condition_zero() {
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::ZERO);
    let condition = JumpCondition::Zero;
    assert!(condition.resolve(&cpu));
    cpu.r.f.remove(FlagsRegister::ZERO);
    assert!(!condition.resolve(&cpu));
    assert_eq!(condition.to_string(), "Z");
}

#[test]
fn test_jump_condition_not_carry() {
    let mut cpu = CPU::default();
    cpu.r.f.remove(FlagsRegister::CARRY);
    let condition = JumpCondition::NotCarry;
    assert!(condition.resolve(&cpu));
    cpu.r.f.insert(FlagsRegister::CARRY);
    assert!(!condition.resolve(&cpu));
    assert_eq!(condition.to_string(), "NC");
}

#[test]
fn test_jump_condition_carry() {
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::CARRY);
    let condition = JumpCondition::Carry;
    assert!(condition.resolve(&cpu));
    cpu.r.f.remove(FlagsRegister::CARRY);
    assert!(!condition.resolve(&cpu));
    assert_eq!(condition.to_string(), "C");
}

#[test]
fn test_jump_condition_always() {
    let cpu = CPU::default();
    let condition = JumpCondition::Always;
    assert!(condition.resolve(&cpu));
    assert_eq!(condition.to_string(), "");
}

#[test]
fn test_jump_target_d16() {
    let cpu = CPU::default();
    let target = JumpTarget::D16(0x1234);
    assert_eq!(target.read(&cpu), 0x1234);
    assert_eq!(target.to_string(), "0x1234");
}

#[test]
fn test_jump_target_hl() {
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x1234);
    let target = JumpTarget::HL;
    assert_eq!(target.read(&cpu), 0x1234);
    assert_eq!(target.to_string(), "HL");
}

#[test]
fn test_reset_code() {
    let reset = ResetCode::RST18;
    assert_eq!(reset.to_string(), "0x18");
}
