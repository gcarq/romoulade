use crate::gb::AddressSpace;
use crate::gb::cpu::CPU;
use crate::gb::cpu::registers::FlagsRegister;

/// Represents a mock for MemoryBus
struct MockBus {
    data: Vec<u8>,
}

impl MockBus {
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }
}

impl AddressSpace for MockBus {
    fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }

    fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
}

fn assert_flags(r: FlagsRegister, zero: bool, negative: bool, half_carry: bool, carry: bool) {
    assert_eq!(r.zero, zero, "zero");
    assert_eq!(r.negative, negative, "negative");
    assert_eq!(r.half_carry, half_carry, "half_carry");
    assert_eq!(r.carry, carry, "carry");
}

#[test]
fn test_add_no_overflow() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x42]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x42);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_add_overflow_zero() {
    // ADD A, HLI
    let mut bus = MockBus::new(vec![0x86, 0x02]);
    let mut cpu = CPU::new();
    cpu.r.a = 0xff;
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.a, 0x01);
    assert_flags(cpu.r.f, false, false, true, true);
}

#[test]
fn test_add2_no_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x01);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x04);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_add2_overflow() {
    // ADD HL, DE
    let mut bus = MockBus::new(vec![0x19; 1]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0xFFFE);
    cpu.r.set_de(0x03);
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.get_hl(), 0x0001);
    assert_flags(cpu.r.f, false, false, true, true);
}

#[test]
fn test_adc_non_zero() {
    // ADC A, D8
    let mut bus = MockBus::new(vec![0xce, 0x01]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b1111_0001;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_eq!(cpu.r.a, 0b1111_0010);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_and_non_zero() {
    // AND A, B
    let mut bus = MockBus::new(vec![0xa0; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x02;
    cpu.r.b = 0xff;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x02);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_and_zero() {
    // AND A, B
    let mut bus = MockBus::new(vec![0xa0; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x02;
    cpu.r.b = 0x04;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_bit_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::new();
    cpu.r.h = 0b01111111;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_bit_non_zero() {
    // BIT 7, H
    let mut bus = MockBus::new(vec![0xcb, 0x7c]);
    let mut cpu = CPU::new();
    cpu.r.h = 0b11010000;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_ccf_no_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::new();
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_ccf_carry() {
    // CCF
    let mut bus = MockBus::new(vec![0x3f; 1]);
    let mut cpu = CPU::new();
    cpu.r.f.carry = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_cpl() {
    // CPL
    let mut bus = MockBus::new(vec![0x2f; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b11010011;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, false, true, true, false);
    assert_eq!(cpu.r.a, 0b00101100);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_daa_negative_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x44;
    cpu.r.f.negative = true;
    cpu.r.f.carry = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xe4);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, true, false, true);
}

#[test]
fn test_daa_non_negative_carry() {
    // DAA
    let mut bus = MockBus::new(vec![0x27; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x44;
    cpu.r.f.negative = false;
    cpu.r.f.carry = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xa4);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_di() {
    // DI
    let mut bus = MockBus::new(vec![0xf3; 1]);
    let mut cpu = CPU::new();
    cpu.ime = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, false);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_dec_no_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0x02;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x1);
    assert_flags(cpu.r.f, false, true, false, false);
}

#[test]
fn test_dec_overflow() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0xff);
    assert_flags(cpu.r.f, false, true, true, false);
}

#[test]
fn test_dec_zero() {
    // DEC B
    let mut bus = MockBus::new(vec![0x05; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0x01;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x00);
    assert_flags(cpu.r.f, true, true, false, false);
}

#[test]
fn test_dec_word() {
    // DEC BC
    let mut bus = MockBus::new(vec![0x0b; 1]);
    let mut cpu = CPU::new();
    cpu.r.set_bc(0x42);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.get_bc(), 0x41);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_ei() {
    // EI
    let mut bus = MockBus::new(vec![0xfb; 1]);
    let mut cpu = CPU::new();
    cpu.ime = false;
    cpu.step(&mut bus);
    assert_eq!(cpu.ime, true);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_inc_no_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0x01);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_inc_overflow() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0b1111_1111;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0b0000_0000);
    assert_flags(cpu.r.f, true, false, true, false);
}

#[test]
fn test_inc_half_carry() {
    // INC B
    let mut bus = MockBus::new(vec![0x04; 1]);
    let mut cpu = CPU::new();
    cpu.r.b = 0b0000_1111;
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_eq!(cpu.r.b, 0b0001_0000);
    assert_flags(cpu.r.f, false, false, true, false);
}

#[test]
fn test_inc_word() {
    // INC (HL)
    let mut bus = MockBus::new(vec![0x34, 0x03]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x01);
    cpu.step(&mut bus);
    assert_eq!(bus.read(0x01), 0x04);
    assert_eq!(cpu.clock.ticks(), 12);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_jr_always_neg_offset() {
    // JR i8
    let mut bus = MockBus::new(vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 251]);
    let mut cpu = CPU::new();
    for _ in 0..6 {
        cpu.step(&mut bus);
    }
    // At this point 7 byte have been consumed.
    // pc must be 7 - 5 (offset)
    assert_eq!(cpu.clock.ticks(), 16);
    assert_eq!(cpu.pc, 0x02);
}

#[test]
fn test_jr_always_pos_offset() {
    // JR i8
    let mut bus = MockBus::new(vec![0x18, 0x03]);
    let mut cpu = CPU::new();
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 16);
    assert_eq!(cpu.pc, 0x05);
}

#[test]
fn test_jp_always() {
    // JP D16
    let mut bus = MockBus::new(vec![0xc3, 0x01, 0x02]);
    let mut cpu = CPU::new();
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 16);
    assert_eq!(cpu.pc, 0x0201);
}

#[test]
fn test_ld_byte_reg() {
    // LD C, A
    let mut bus = MockBus::new(vec![0x4f; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x42;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0x42);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_ld_byte_io() {
    // LD (HL), D8
    let mut bus = MockBus::new(vec![0x36, 0x42, 0x00]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(bus.read(0x02), 0x42);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn test_ld_byte_from_indirect_inc() {
    // LD A, (HL+)
    let mut bus = MockBus::new(vec![0x2a, 0x00, 0x11]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x11);
    assert_eq!(cpu.r.get_hl(), 0x03);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_nop() {
    // NOP
    let mut bus = MockBus::new(vec![0x00; 1]);
    let mut cpu = CPU::new();
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_or_non_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x01;
    cpu.r.c = 0x03;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, false, false, false, false);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_or_zero() {
    // OR A, C
    let mut bus = MockBus::new(vec![0xb1; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x00;
    cpu.r.c = 0x00;
    cpu.step(&mut bus);
    assert_flags(cpu.r.f, true, false, false, false);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_rlca() {
    // RLCA
    let mut bus = MockBus::new(vec![0x07; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b1011_0110;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0110_1101);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rr_non_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::new();
    cpu.r.c = 0b0110_0011;
    cpu.r.f.carry = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0b1011_0001);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rr_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x19]);
    let mut cpu = CPU::new();
    cpu.r.c = 0x00;
    cpu.r.f.carry = false;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.c, 0x00);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_rra() {
    // RRA
    let mut bus = MockBus::new(vec![0x1F; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0011_0001);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_rst() {
    // RST 00h

    // Expected execution
    // op: 0x00 -> NOP
    // op: 0xC7 -> RST(RST00)
    // op: 0x04 -> INC(B)
    // op: 0xC9 -> RET(Always)
    // op: 0x0C -> INC(C)
    let mut bus = MockBus::new(vec![0x04, 0xc9, 0x00, 0xC7, 0x0C, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::new();

    assert_eq!(cpu.sp, 0);
    cpu.pc = 0x02;
    cpu.sp = 0x07;

    let ticks: u16 = (0..5).map(|_| cpu.step(&mut bus)).sum();
    assert_eq!(cpu.r.b, 0x01);
    assert_eq!(cpu.r.c, 0x01);
    assert_eq!(cpu.pc, 0x05);
    assert_eq!(ticks, 52);
}

#[test]
fn test_sbc_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b0000_0001;
    cpu.r.f.carry = true;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1111_1100);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, true, true, true);
}

#[test]
fn test_sbc_no_carry() {
    // SBC A, D8
    let mut bus = MockBus::new(vec![0xde, 0x04]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b0001_0000;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b0000_1100);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, true, true, false);
}

#[test]
fn test_scf() {
    // SCF
    let mut bus = MockBus::new(vec![0x37; 1]);
    let mut cpu = CPU::new();
    cpu.step(&mut bus);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_set() {
    // BIT 7, (HL)
    let mut bus = MockBus::new(vec![0xcb, 0xfe, 0b00000010]);
    let mut cpu = CPU::new();
    cpu.r.set_hl(0x02);
    cpu.step(&mut bus);
    assert_eq!(bus.read(0x02), 0b10000010);
    assert_eq!(cpu.clock.ticks(), 16);
    assert_eq!(cpu.pc, 2);
}

#[test]
fn test_srl_non_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::new();
    cpu.r.b = 0b0110_0011;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.b, 0b0011_0001);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, true);
}

#[test]
fn test_srl_zero() {
    // SRL B
    let mut bus = MockBus::new(vec![0xcb, 0x38]);
    let mut cpu = CPU::new();
    cpu.r.b = 0x00;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.b, 0x00);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_swap_non_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::new();
    cpu.r.a = 0b1011_1010;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0b1010_1011);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_swap_zero() {
    // SWAP A
    let mut bus = MockBus::new(vec![0xcb, 0x37]);
    let mut cpu = CPU::new();
    cpu.r.a = 0;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0);
    assert_eq!(cpu.clock.ticks(), 8);
    assert_eq!(cpu.pc, 2);
    assert_flags(cpu.r.f, true, false, false, false);
}

#[test]
fn test_push() {
    // PUSH AF
    let mut bus = MockBus::new(vec![0xf5, 0x00, 0x00, 0x00]);
    let mut cpu = CPU::new();
    cpu.r.set_af(0xff);
    cpu.sp = 0x03;
    cpu.step(&mut bus);
    assert_eq!(bus.read(0x01), 0xf0);
    assert_eq!(bus.read(0x02), 0x00);
    assert_eq!(cpu.clock.ticks(), 16);
    assert_eq!(cpu.pc, 1);
}

#[test]
fn test_xor_non_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x42;
    cpu.r.c = 0x90;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0xd2);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, false, false, false, false);
}

#[test]
fn test_xor_zero() {
    // XOR A, C
    let mut bus = MockBus::new(vec![0xa9; 1]);
    let mut cpu = CPU::new();
    cpu.r.a = 0x90;
    cpu.r.c = 0x90;
    cpu.step(&mut bus);
    assert_eq!(cpu.r.a, 0x00);
    assert_eq!(cpu.clock.ticks(), 4);
    assert_eq!(cpu.pc, 1);
    assert_flags(cpu.r.f, true, false, false, false);
}
