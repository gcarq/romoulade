use crate::gb::cpu::registers::FlagsRegister;
use crate::gb::cpu::{CPU, ImeState};
use crate::gb::{AddressSpace, HardwareContext};

/// Represents a mock for MemoryBus
struct MockBus {
    ime: ImeState,
    data: Vec<u8>,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            ime: ImeState::Enabled,
            data,
        }
    }
}

impl AddressSpace for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn read(&mut self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

impl HardwareContext for MockBus {
    fn set_ime(&mut self, ime: ImeState) {
        self.ime = ime;
    }

    fn ime(&self) -> ImeState {
        self.ime
    }

    fn tick(&mut self) {
        // No-op
    }
}

fn assert_flags(r: FlagsRegister, zero: bool, negative: bool, half_carry: bool, carry: bool) {
    assert_eq!(r.contains(FlagsRegister::ZERO), zero);
    assert_eq!(r.contains(FlagsRegister::SUBTRACTION), negative);
    assert_eq!(r.contains(FlagsRegister::HALF_CARRY), half_carry);
    assert_eq!(r.contains(FlagsRegister::CARRY), carry);
}

#[test]
fn test_add_a_hli_no_overflow() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x42]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x42);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_add_a_hli_overflow_zero() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x02]);
    let mut cpu = CPU::default();
    cpu.r.a = 0xff;
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x01);
    assert_flags(cpu.r.f, false, false, true, true);
}

#[test]
fn test_add_hl_de_no_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x04);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_add_hl_de_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0xFFFE);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x0001);
    assert_flags(cpu.r.f, false, false, true, true);
}

#[test]
fn test_adc_a_d8_non_zero() {
    // ADC A, D8
    let mut bus = MockBus::new(vec![0xce, 0x01]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1111_0001;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.r.a, 0b1111_0010);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_and_a_b_non_zero() {
    // AND A, B
    let mut bus = MockBus::new(vec![0xa0; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.b = 0xff;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0x02);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_and_a_b_zero() {
    // AND A, B
    let mut bus = MockBus::new(vec![0xa0; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x02;
    cpu.r.b = 0x04;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_bit_7_h_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::default();
    cpu.r.h = 0b01111111;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_bit_7_h_non_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::default();
    cpu.r.h = 0b11010000;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_ccf_no_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_ccf_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::default();
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_cpl() {
    // CPL
    let mut bus = MockBus::new(vec![0x2f; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b11010011;
    cpu.step(&mut bus).unwrap();
    assert_flags(cpu.r.f, false, true, true, false);
    assert_eq!(cpu.r.a, 0b00101100);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_daa_negative_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.insert(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0xe4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, true, false, true);
}

#[test]
fn test_daa_non_negative_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x44;
    cpu.r.f.remove(FlagsRegister::SUBTRACTION);
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0xa4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_di() {
    // DI
    let mut bus = MockBus::new(vec![0xf3; 1]);
    let mut cpu = CPU::default();
    bus.set_ime(ImeState::Enabled);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.ime(), ImeState::Disabled);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_dec_b_no_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x02;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x1);
    assert_flags(cpu.r.f, false, true, false, false);
}

#[test]
fn test_dec_b_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0xff);
    assert_flags(cpu.r.f, false, true, true, false);
}

#[test]
fn test_dec_b_zero() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x01;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x00);
    assert_flags(cpu.r.f, true, true, false, false);
}

#[test]
fn test_dec_bc_word() {
    // DEC BC
    let mut bus = MockBus::new(vec![0x0b; 1]);
    let mut cpu = CPU::default();
    cpu.r.set_bc(0x42);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.get_bc(), 0x41);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_ei() {
    // EI
    let mut bus = MockBus::new(vec![0xfb, 0x00]);
    let mut cpu = CPU::default();
    bus.set_ime(ImeState::Disabled);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.ime(), ImeState::Pending);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_rapid_ei_di() {
    // Rapid EI/DI should not result in any interrupts
    let mut bus = MockBus::new(vec![0xfb, 0xf3]);
    let mut cpu = CPU::default();
    bus.set_ime(ImeState::Disabled);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.ime(), ImeState::Pending);
    assert_eq!(cpu.pc, 1);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.ime(), ImeState::Disabled);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn test_inc_b_no_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x01);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_inc_b_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b1111_1111;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0b0000_0000);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_inc_b_half_carry() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b0000_1111;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0b0001_0000);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_inc_hl() {
    // INC (HL)
    let mut bus = MockBus::new(vec![0x34, 0x03]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read(0x01), 0x04);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_jr_s8_neg_offset() {
    // JR s8
    let mut bus = MockBus::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 251]);
    let mut cpu = CPU::default();
    for _ in 0..6 {
        cpu.step(&mut bus).unwrap();
    }
    // At this point 7 byte have been consumed.
    // pc must be 7 - 5 (offset)
    assert_eq!(cpu.pc, 0x02);
}

#[test]
fn test_jr_s8_always_pos_offset() {
    // JR s8
    let mut bus = MockBus::new(vec![0x18, 0x03]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 0x05);
}

#[test]
fn test_jp_a16() {
    // JP D16
    let mut bus = MockBus::new(vec![0xc3, 0x01, 0x02]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 0x0201);
}

#[test]
fn test_ld_c_a() {
    // LD C, A
    let mut bus = MockBus::new(vec![0x4f; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.c, 0x42);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_ld_a_a16() {
    // LD A, (a16)
    let mut bus = MockBus::new(vec![0xFA, 0x05, 0x00, 0x01, 0x02, 0x03]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0x03);
    assert_eq!(cpu.pc, 3);
}

#[test]
fn test_ld_hl_d8() {
    // LD (HL), D8
    let mut bus = MockBus::new(vec![0x36, 0x42, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read(0x02), 0x42);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn test_ld_a_hl_plus() {
    // LD A, (HL+)
    let mut bus = MockBus::new(vec![0x2a, 0x00, 0x11]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0x11);
    assert_eq!(cpu.r.get_hl(), 0x03);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_nop() {
    // NOP
    let mut bus = MockBus::new(vec![0x00; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_or_a_c_non_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x01;
    cpu.r.c = 0x03;
    cpu.step(&mut bus).unwrap();
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_or_a_c_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x00;
    cpu.r.c = 0x00;
    cpu.step(&mut bus).unwrap();
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_rlca() {
    // RLCA
    let mut bus = MockBus::new(vec![0x07; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1011_0110;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0b0110_1101);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rr_c_non_zero() {
    // RR C
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::default();
    cpu.r.c = 0b0110_0011;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.c, 0b1011_0001);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rr_c_zero() {
    // RR C
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::default();
    cpu.r.c = 0x00;
    cpu.r.f.remove(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.c, 0x00);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_rra() {
    // RRA
    let mut bus = MockBus::new(vec![0x1F; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0110_0011;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0b0011_0001);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rst_00h() {
    // RST 00h

    // Expected execution
    // op: 0x00 -> NOP
    // op: 0xC7 -> RST(RST00)
    // op: 0x04 -> INC(B)
    // op: 0xC9 -> RET(Always)
    // op: 0x0C -> INC(C)
    let mut bus = MockBus::new(vec![0x04, 0xc9, 0x00, 0xC7, 0x0C, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();

    assert_eq!(cpu.sp, 0);
    cpu.pc = 0x02;
    cpu.sp = 0x07;

    for _ in 0..5 {
        cpu.step(&mut bus).unwrap();
    }
    assert_eq!(cpu.r.b, 0x01);
    assert_eq!(cpu.r.c, 0x01);
    assert_eq!(cpu.pc, 0x05);
}

#[test]
fn test_sbc_a_d8_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0000_0001;
    cpu.r.f.insert(FlagsRegister::CARRY);
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0b1111_1100);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, true, true, true);
}

#[test]
fn test_sbc_a_d8_no_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b0001_0000;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0b0000_1100);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, true, true, false);
}

#[test]
fn test_scf() {
    // SCF
    let mut bus = MockBus::new(vec![0x37; 1]);
    let mut cpu = CPU::default();
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_set_7_hl() {
    // BIT 7, (HL)
    let mut bus = MockBus::new(vec![0xcb, 0xfe, 0b00000010]);
    let mut cpu = CPU::default();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read(0x02), 0b10000010);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn test_srl_b_non_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::default();
    cpu.r.b = 0b0110_0011;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.b, 0b0011_0001);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_srl_b_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::default();
    cpu.r.b = 0x00;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.b, 0x00);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_swap_a_non_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::default();
    cpu.r.a = 0b1011_1010;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0b1010_1011);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_swap_a_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::default();
    cpu.r.a = 0;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_push_af() {
    // PUSH AF
    let mut bus = MockBus::new(vec![0xf5, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::default();
    cpu.r.set_af(0xff);
    cpu.sp = 0x03;
    cpu.step(&mut bus).unwrap();
    assert_eq!(bus.read(0x01), 0xf0);
    assert_eq!(bus.read(0x02), 0x00);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_xor_a_c_non_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x42;
    cpu.r.c = 0x90;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0xd2);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_xor_a_c_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::default();
    cpu.r.a = 0x90;
    cpu.r.c = 0x90;
    cpu.step(&mut bus).unwrap();
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_af_register() {
    let mut cpu = CPU::default();
    cpu.r.set_af(0b1101_1111_1111_1111);
    assert_eq!(cpu.r.a, 0b1101_1111);
    assert_eq!(cpu.r.f.bits(), 0b1111_0000);
    assert_eq!(cpu.r.get_af(), 0b1101_1111_1111_0000);
}

#[test]
fn test_bc_register() {
    let mut cpu = CPU::default();
    cpu.r.set_bc(0b0110_1111_1111_1011);
    assert_eq!(cpu.r.b, 0b0110_1111);
    assert_eq!(cpu.r.c, 0b1111_1011);
    assert_eq!(cpu.r.get_bc(), 0b0110_1111_1111_1011);
}

#[test]
fn test_de_register() {
    let mut cpu = CPU::default();
    cpu.r.set_de(0b0110_1101_1101_1011);
    assert_eq!(cpu.r.d, 0b0110_1101);
    assert_eq!(cpu.r.e, 0b1101_1011);
    assert_eq!(cpu.r.get_de(), 0b0110_1101_1101_1011);
}

#[test]
fn test_hl_register() {
    let mut cpu = CPU::default();
    cpu.r.set_hl(0b0110_1110_1111_1010);
    assert_eq!(cpu.r.h, 0b0110_1110);
    assert_eq!(cpu.r.l, 0b1111_1010);
    assert_eq!(cpu.r.get_hl(), 0b0110_1110_1111_1010);
}
