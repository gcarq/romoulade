use crate::gb::cpu::instruction::Instruction::*;
use crate::gb::cpu::misc::*;

use crate::gb::AddressSpace;
use crate::gb::cpu::misc::JumpCondition::{Always, Carry, NotCarry, NotZero, Zero};
use crate::gb::cpu::misc::PairedRegister::{AF, BC, DE, HL, SP};
use crate::gb::cpu::misc::Register::{A, B, C, D, E, H, L};
use std::fmt;
use std::fmt::Formatter;

const OPCODE_PREFIX_16BIT: u8 = 0xCB;

#[derive(Copy, Clone)]
pub enum Instruction {
    ADD(ByteSource),               // Add n to target
    ADDHL(WordSource),             // Add nn to HL
    ADDSP(i8),                     // Add signed immediate 8 bit value to Stack Pointer
    ADC(ByteSource),               // Add n + Carry flag to A
    AND(ByteSource),               // Logically AND n with A, result in A
    BIT(u8, ByteTarget),           // Test bit b in register r
    INC(ByteTarget),               // Increment single byte register n
    INC2(PairedRegister),          // Increment word register n
    CALL(JumpCondition, u16), // Push address of next instruction onto stack and then  jump to address nn
    CCF,                      // Complement carry flag
    CP(ByteSource),           // Compare A with source
    CPL,                      // Flips all bits in A register, sets N and H flags
    DAA,                      // This instruction is useful when you’re using BCD value
    DI,                       // Disables interrupt handling by setting ime = false
    DEC(ByteTarget),          // Decrement single byte register n
    DEC2(PairedRegister),     // Decrement word register n
    EI,                       // Enables interrupt handling by setting ime = true
    HALT,                     // Halts and wait for interrupt
    JR(JumpCondition, i8),    // Relative jump to given address
    JP(JumpCondition, JumpTarget), // Jump to address nn
    LD(Load),                 // Put value into n
    NOP,                      // No operation
    OR(ByteSource),           // Logical OR n with register A, result in A.
    PUSH(PairedRegister),     // Push to the stack memory, data from the 16-bit register
    POP(PairedRegister),      // Pops to the 16-bit register
    RES(u8, ByteTarget),      // Reset bit b in register r
    RET(JumpCondition),       // Pop two bytes from stack & jump to that address
    RETI,                     // Unconditional return which also enables interrupts
    RL(ByteTarget),           // Rotate n left through Carry flag
    RLA,                      // Rotate `A` left through carry
    RLC(ByteTarget),          // Rotate target left
    RLCA,                     // Rotate A left. Old bit 7 to Carry flag
    RR(ByteTarget),           // Rotate n right through Carry flag
    RRA,                      // Rotate A right through Carry flag
    RRC(ByteTarget),          // Rotate n right. Old bit 0 to Carry flag
    RRCA,                     // Rotate A right. Old bit 0 to Carry flag
    RST(ResetCode),           // Push present address onto stack.  Jump to address 0x0000 + n
    SBC(ByteSource),          // Subtract n + Carry flag from A
    SCF,                      // Set carry flag
    SET(u8, ByteTarget),      // Set bit b in register r
    SLA(ByteTarget),          // Shift n left into Carry. LSB of n set to 0
    SRA(ByteTarget),          // Shift n right into Carry. MSB doesn't change
    SRL(ByteTarget),          // Shift right into Carry, MSB set to 0
    SUB(ByteSource),          // Subtract n from A
    STOP,                     // Halt CPU & LCD display until button pressed
    SWAP(ByteTarget),         // Swap upper & lower nibbles of n
    XOR(ByteSource),          // Logical exclusive OR n with register A, result in A
}

impl Instruction {
    /// Creates a new Instruction from location `address` in memory.
    /// As many bytes as needed are read from the bus.
    #[inline]
    pub fn new<T: AddressSpace>(address: u16, bus: &mut T) -> (Option<Instruction>, u16) {
        // Read next opcode from memory and check whether its prefixed
        let opcode = bus.read(address);
        if opcode == OPCODE_PREFIX_16BIT {
            // Read opcode from next address
            let instruction = Self::prefixed(bus.read(address + 1));
            (Some(instruction), address + 2)
        } else {
            Self::not_prefixed(opcode, address + 1, bus)
        }
    }

    /// Creates a new prefixed `Instruction` from the given opcode,
    /// the passed address is the next address after the opcode.
    /// Returns the parsed `Instruction` and the next address.
    fn prefixed(opcode: u8) -> Instruction {
        match opcode {
            0x00 => RLC(ByteTarget::R(B)),
            0x01 => RLC(ByteTarget::R(C)),
            0x02 => RLC(ByteTarget::R(D)),
            0x03 => RLC(ByteTarget::R(E)),
            0x04 => RLC(ByteTarget::R(H)),
            0x05 => RLC(ByteTarget::R(L)),
            0x06 => RLC(ByteTarget::HLI),
            0x07 => RLC(ByteTarget::R(A)),

            0x08 => RRC(ByteTarget::R(B)),
            0x09 => RRC(ByteTarget::R(C)),
            0x0a => RRC(ByteTarget::R(D)),
            0x0b => RRC(ByteTarget::R(E)),
            0x0c => RRC(ByteTarget::R(H)),
            0x0d => RRC(ByteTarget::R(L)),
            0x0e => RRC(ByteTarget::HLI),
            0x0f => RRC(ByteTarget::R(A)),

            0x10 => RL(ByteTarget::R(B)),
            0x11 => RL(ByteTarget::R(C)),
            0x12 => RL(ByteTarget::R(D)),
            0x13 => RL(ByteTarget::R(E)),
            0x14 => RL(ByteTarget::R(H)),
            0x15 => RL(ByteTarget::R(L)),
            0x16 => RL(ByteTarget::HLI),
            0x17 => RL(ByteTarget::R(A)),

            0x18 => RR(ByteTarget::R(B)),
            0x19 => RR(ByteTarget::R(C)),
            0x1a => RR(ByteTarget::R(D)),
            0x1b => RR(ByteTarget::R(E)),
            0x1c => RR(ByteTarget::R(H)),
            0x1d => RR(ByteTarget::R(L)),
            0x1e => RR(ByteTarget::HLI),
            0x1f => RR(ByteTarget::R(A)),

            0x20 => SLA(ByteTarget::R(B)),
            0x21 => SLA(ByteTarget::R(C)),
            0x22 => SLA(ByteTarget::R(D)),
            0x23 => SLA(ByteTarget::R(E)),
            0x24 => SLA(ByteTarget::R(H)),
            0x25 => SLA(ByteTarget::R(L)),
            0x26 => SLA(ByteTarget::HLI),
            0x27 => SLA(ByteTarget::R(A)),

            0x28 => SRA(ByteTarget::R(B)),
            0x29 => SRA(ByteTarget::R(C)),
            0x2a => SRA(ByteTarget::R(D)),
            0x2b => SRA(ByteTarget::R(E)),
            0x2c => SRA(ByteTarget::R(H)),
            0x2d => SRA(ByteTarget::R(L)),
            0x2e => SRA(ByteTarget::HLI),
            0x2f => SRA(ByteTarget::R(A)),

            0x30 => SWAP(ByteTarget::R(B)),
            0x31 => SWAP(ByteTarget::R(C)),
            0x32 => SWAP(ByteTarget::R(D)),
            0x33 => SWAP(ByteTarget::R(E)),
            0x34 => SWAP(ByteTarget::R(H)),
            0x35 => SWAP(ByteTarget::R(L)),
            0x36 => SWAP(ByteTarget::HLI),
            0x37 => SWAP(ByteTarget::R(A)),

            0x38 => SRL(ByteTarget::R(B)),
            0x39 => SRL(ByteTarget::R(C)),
            0x3a => SRL(ByteTarget::R(D)),
            0x3b => SRL(ByteTarget::R(E)),
            0x3c => SRL(ByteTarget::R(H)),
            0x3d => SRL(ByteTarget::R(L)),
            0x3e => SRL(ByteTarget::HLI),
            0x3f => SRL(ByteTarget::R(A)),

            0x40 => BIT(0, ByteTarget::R(B)),
            0x41 => BIT(0, ByteTarget::R(C)),
            0x42 => BIT(0, ByteTarget::R(D)),
            0x43 => BIT(0, ByteTarget::R(E)),
            0x44 => BIT(0, ByteTarget::R(H)),
            0x45 => BIT(0, ByteTarget::R(L)),
            0x46 => BIT(0, ByteTarget::HLI),
            0x47 => BIT(0, ByteTarget::R(A)),

            0x48 => BIT(1, ByteTarget::R(B)),
            0x49 => BIT(1, ByteTarget::R(C)),
            0x4a => BIT(1, ByteTarget::R(D)),
            0x4b => BIT(1, ByteTarget::R(E)),
            0x4c => BIT(1, ByteTarget::R(H)),
            0x4d => BIT(1, ByteTarget::R(L)),
            0x4e => BIT(1, ByteTarget::HLI),
            0x4f => BIT(1, ByteTarget::R(A)),
            0x50 => BIT(2, ByteTarget::R(B)),
            0x51 => BIT(2, ByteTarget::R(C)),
            0x52 => BIT(2, ByteTarget::R(D)),
            0x53 => BIT(2, ByteTarget::R(E)),
            0x54 => BIT(2, ByteTarget::R(H)),
            0x55 => BIT(2, ByteTarget::R(L)),
            0x56 => BIT(2, ByteTarget::HLI),
            0x57 => BIT(2, ByteTarget::R(A)),
            0x58 => BIT(3, ByteTarget::R(B)),
            0x59 => BIT(3, ByteTarget::R(C)),
            0x5a => BIT(3, ByteTarget::R(D)),
            0x5b => BIT(3, ByteTarget::R(E)),
            0x5c => BIT(3, ByteTarget::R(H)),
            0x5d => BIT(3, ByteTarget::R(L)),
            0x5e => BIT(3, ByteTarget::HLI),
            0x5f => BIT(3, ByteTarget::R(A)),
            0x60 => BIT(4, ByteTarget::R(B)),
            0x61 => BIT(4, ByteTarget::R(C)),
            0x62 => BIT(4, ByteTarget::R(D)),
            0x63 => BIT(4, ByteTarget::R(E)),
            0x64 => BIT(4, ByteTarget::R(H)),
            0x65 => BIT(4, ByteTarget::R(L)),
            0x66 => BIT(4, ByteTarget::HLI),
            0x67 => BIT(4, ByteTarget::R(A)),
            0x68 => BIT(5, ByteTarget::R(B)),
            0x69 => BIT(5, ByteTarget::R(C)),
            0x6a => BIT(5, ByteTarget::R(D)),
            0x6b => BIT(5, ByteTarget::R(E)),
            0x6c => BIT(5, ByteTarget::R(H)),
            0x6d => BIT(5, ByteTarget::R(L)),
            0x6e => BIT(5, ByteTarget::HLI),
            0x6f => BIT(5, ByteTarget::R(A)),
            0x70 => BIT(6, ByteTarget::R(B)),
            0x71 => BIT(6, ByteTarget::R(C)),
            0x72 => BIT(6, ByteTarget::R(D)),
            0x73 => BIT(6, ByteTarget::R(E)),
            0x74 => BIT(6, ByteTarget::R(H)),
            0x75 => BIT(6, ByteTarget::R(L)),
            0x76 => BIT(6, ByteTarget::HLI),
            0x77 => BIT(6, ByteTarget::R(A)),
            0x78 => BIT(7, ByteTarget::R(B)),
            0x79 => BIT(7, ByteTarget::R(C)),
            0x7a => BIT(7, ByteTarget::R(D)),
            0x7b => BIT(7, ByteTarget::R(E)),
            0x7c => BIT(7, ByteTarget::R(H)),
            0x7d => BIT(7, ByteTarget::R(L)),
            0x7e => BIT(7, ByteTarget::HLI),
            0x7f => BIT(7, ByteTarget::R(A)),
            0x80 => RES(0, ByteTarget::R(B)),
            0x81 => RES(0, ByteTarget::R(C)),
            0x82 => RES(0, ByteTarget::R(D)),
            0x83 => RES(0, ByteTarget::R(E)),
            0x84 => RES(0, ByteTarget::R(H)),
            0x85 => RES(0, ByteTarget::R(L)),
            0x86 => RES(0, ByteTarget::HLI),
            0x87 => RES(0, ByteTarget::R(A)),
            0x88 => RES(1, ByteTarget::R(B)),
            0x89 => RES(1, ByteTarget::R(C)),
            0x8a => RES(1, ByteTarget::R(D)),
            0x8b => RES(1, ByteTarget::R(E)),
            0x8c => RES(1, ByteTarget::R(H)),
            0x8d => RES(1, ByteTarget::R(L)),
            0x8e => RES(1, ByteTarget::HLI),
            0x8f => RES(1, ByteTarget::R(A)),
            0x90 => RES(2, ByteTarget::R(B)),
            0x91 => RES(2, ByteTarget::R(C)),
            0x92 => RES(2, ByteTarget::R(D)),
            0x93 => RES(2, ByteTarget::R(E)),
            0x94 => RES(2, ByteTarget::R(H)),
            0x95 => RES(2, ByteTarget::R(L)),
            0x96 => RES(2, ByteTarget::HLI),
            0x97 => RES(2, ByteTarget::R(A)),
            0x98 => RES(3, ByteTarget::R(B)),
            0x99 => RES(3, ByteTarget::R(C)),
            0x9a => RES(3, ByteTarget::R(D)),
            0x9b => RES(3, ByteTarget::R(E)),
            0x9c => RES(3, ByteTarget::R(H)),
            0x9d => RES(3, ByteTarget::R(L)),
            0x9e => RES(3, ByteTarget::HLI),
            0x9f => RES(3, ByteTarget::R(A)),
            0xa0 => RES(4, ByteTarget::R(B)),
            0xa1 => RES(4, ByteTarget::R(C)),
            0xa2 => RES(4, ByteTarget::R(D)),
            0xa3 => RES(4, ByteTarget::R(E)),
            0xa4 => RES(4, ByteTarget::R(H)),
            0xa5 => RES(4, ByteTarget::R(L)),
            0xa6 => RES(4, ByteTarget::HLI),
            0xa7 => RES(4, ByteTarget::R(A)),
            0xa8 => RES(5, ByteTarget::R(B)),
            0xa9 => RES(5, ByteTarget::R(C)),
            0xaa => RES(5, ByteTarget::R(D)),
            0xab => RES(5, ByteTarget::R(E)),
            0xac => RES(5, ByteTarget::R(H)),
            0xad => RES(5, ByteTarget::R(L)),
            0xae => RES(5, ByteTarget::HLI),
            0xaf => RES(5, ByteTarget::R(A)),
            0xb0 => RES(6, ByteTarget::R(B)),
            0xb1 => RES(6, ByteTarget::R(C)),
            0xb2 => RES(6, ByteTarget::R(D)),
            0xb3 => RES(6, ByteTarget::R(E)),
            0xb4 => RES(6, ByteTarget::R(H)),
            0xb5 => RES(6, ByteTarget::R(L)),
            0xb6 => RES(6, ByteTarget::HLI),
            0xb7 => RES(6, ByteTarget::R(A)),
            0xb8 => RES(7, ByteTarget::R(B)),
            0xb9 => RES(7, ByteTarget::R(C)),
            0xba => RES(7, ByteTarget::R(D)),
            0xbb => RES(7, ByteTarget::R(E)),
            0xbc => RES(7, ByteTarget::R(H)),
            0xbd => RES(7, ByteTarget::R(L)),
            0xbe => RES(7, ByteTarget::HLI),
            0xbf => RES(7, ByteTarget::R(A)),
            0xc0 => SET(0, ByteTarget::R(B)),
            0xc1 => SET(0, ByteTarget::R(C)),
            0xc2 => SET(0, ByteTarget::R(D)),
            0xc3 => SET(0, ByteTarget::R(E)),
            0xc4 => SET(0, ByteTarget::R(H)),
            0xc5 => SET(0, ByteTarget::R(L)),
            0xc6 => SET(0, ByteTarget::HLI),
            0xc7 => SET(0, ByteTarget::R(A)),
            0xc8 => SET(1, ByteTarget::R(B)),
            0xc9 => SET(1, ByteTarget::R(C)),
            0xca => SET(1, ByteTarget::R(D)),
            0xcb => SET(1, ByteTarget::R(E)),
            0xcc => SET(1, ByteTarget::R(H)),
            0xcd => SET(1, ByteTarget::R(L)),
            0xce => SET(1, ByteTarget::HLI),
            0xcf => SET(1, ByteTarget::R(A)),
            0xd0 => SET(2, ByteTarget::R(B)),
            0xd1 => SET(2, ByteTarget::R(C)),
            0xd2 => SET(2, ByteTarget::R(D)),
            0xd3 => SET(2, ByteTarget::R(E)),
            0xd4 => SET(2, ByteTarget::R(H)),
            0xd5 => SET(2, ByteTarget::R(L)),
            0xd6 => SET(2, ByteTarget::HLI),
            0xd7 => SET(2, ByteTarget::R(A)),
            0xd8 => SET(3, ByteTarget::R(B)),
            0xd9 => SET(3, ByteTarget::R(C)),
            0xda => SET(3, ByteTarget::R(D)),
            0xdb => SET(3, ByteTarget::R(E)),
            0xdc => SET(3, ByteTarget::R(H)),
            0xdd => SET(3, ByteTarget::R(L)),
            0xde => SET(3, ByteTarget::HLI),
            0xdf => SET(3, ByteTarget::R(A)),
            0xe0 => SET(4, ByteTarget::R(B)),
            0xe1 => SET(4, ByteTarget::R(C)),
            0xe2 => SET(4, ByteTarget::R(D)),
            0xe3 => SET(4, ByteTarget::R(E)),
            0xe4 => SET(4, ByteTarget::R(H)),
            0xe5 => SET(4, ByteTarget::R(L)),
            0xe6 => SET(4, ByteTarget::HLI),
            0xe7 => SET(4, ByteTarget::R(A)),
            0xe8 => SET(5, ByteTarget::R(B)),
            0xe9 => SET(5, ByteTarget::R(C)),
            0xea => SET(5, ByteTarget::R(D)),
            0xeb => SET(5, ByteTarget::R(E)),
            0xec => SET(5, ByteTarget::R(H)),
            0xed => SET(5, ByteTarget::R(L)),
            0xee => SET(5, ByteTarget::HLI),
            0xef => SET(5, ByteTarget::R(A)),
            0xf0 => SET(6, ByteTarget::R(B)),
            0xf1 => SET(6, ByteTarget::R(C)),
            0xf2 => SET(6, ByteTarget::R(D)),
            0xf3 => SET(6, ByteTarget::R(E)),
            0xf4 => SET(6, ByteTarget::R(H)),
            0xf5 => SET(6, ByteTarget::R(L)),
            0xf6 => SET(6, ByteTarget::HLI),
            0xf7 => SET(6, ByteTarget::R(A)),
            0xf8 => SET(7, ByteTarget::R(B)),
            0xf9 => SET(7, ByteTarget::R(C)),
            0xfa => SET(7, ByteTarget::R(D)),
            0xfb => SET(7, ByteTarget::R(E)),
            0xfc => SET(7, ByteTarget::R(H)),
            0xfd => SET(7, ByteTarget::R(L)),
            0xfe => SET(7, ByteTarget::HLI),
            0xff => SET(7, ByteTarget::R(A)),
        }
    }

    /// Creates a new `Instruction` from the given opcode,
    /// the passed address is the next address after the opcode.
    /// Returns the parsed `Instruction` and the next address.
    fn not_prefixed<T: AddressSpace>(
        opcode: u8,
        address: u16,
        bus: &mut T,
    ) -> (Option<Instruction>, u16) {
        let mut address = address;
        let instruction = match opcode {
            0x00 => NOP,
            0x01 => LD(Load::Word(
                WordTarget::R(BC),
                WordSource::D16(read_word(&mut address, bus)),
            )),
            0x02 => LD(Load::IndirectFrom(IndirectByteRef::BCI, ByteSource::R(A))),
            0x03 => INC2(BC),
            0x04 => INC(ByteTarget::R(B)),
            0x05 => DEC(ByteTarget::R(B)),
            0x06 => LD(Load::Byte(
                ByteTarget::R(B),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x07 => RLCA,
            0x08 => LD(Load::IndirectFromWord(
                WordTarget::D16I(read_word(&mut address, bus)),
                WordSource::R(SP),
            )),
            0x09 => ADDHL(WordSource::R(BC)),
            0x0a => LD(Load::Byte(ByteTarget::R(A), ByteSource::BCI)),
            0x0b => DEC2(BC),
            0x0c => INC(ByteTarget::R(C)),
            0x0d => DEC(ByteTarget::R(C)),
            0x0e => LD(Load::Byte(
                ByteTarget::R(C),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x0f => RRCA,

            0x10 => STOP,
            0x11 => LD(Load::Word(
                WordTarget::R(DE),
                WordSource::D16(read_word(&mut address, bus)),
            )),
            0x12 => LD(Load::IndirectFrom(IndirectByteRef::DEI, ByteSource::R(A))),
            0x13 => INC2(DE),
            0x14 => INC(ByteTarget::R(D)),
            0x15 => DEC(ByteTarget::R(D)),
            0x16 => LD(Load::Byte(
                ByteTarget::R(D),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x17 => RLA,
            0x18 => JR(Always, read_byte(&mut address, bus) as i8),
            0x19 => ADDHL(WordSource::R(DE)),
            0x1a => LD(Load::Byte(ByteTarget::R(A), ByteSource::DEI)),
            0x1b => DEC2(DE),
            0x1c => INC(ByteTarget::R(E)),
            0x1d => DEC(ByteTarget::R(E)),
            0x1e => LD(Load::Byte(
                ByteTarget::R(E),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x1f => RRA,

            0x20 => JR(NotZero, read_byte(&mut address, bus) as i8),
            0x21 => LD(Load::Word(
                WordTarget::R(HL),
                WordSource::D16(read_word(&mut address, bus)),
            )),
            0x22 => LD(Load::IndirectFromAInc(IndirectByteRef::HLI)),
            0x23 => INC2(HL),
            0x24 => INC(ByteTarget::R(H)),
            0x25 => DEC(ByteTarget::R(H)),
            0x26 => LD(Load::Byte(
                ByteTarget::R(H),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x27 => DAA,
            0x28 => JR(Zero, read_byte(&mut address, bus) as i8),
            0x29 => ADDHL(WordSource::R(HL)),
            0x2a => LD(Load::FromIndirectAInc(ByteSource::HLI)),
            0x2b => DEC2(HL),
            0x2c => INC(ByteTarget::R(L)),
            0x2d => DEC(ByteTarget::R(L)),
            0x2e => LD(Load::Byte(
                ByteTarget::R(L),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x2f => CPL,

            0x30 => JR(NotCarry, read_byte(&mut address, bus) as i8),
            0x31 => LD(Load::Word(
                WordTarget::R(SP),
                WordSource::D16(read_word(&mut address, bus)),
            )),
            0x32 => LD(Load::IndirectFromADec(IndirectByteRef::HLI)),
            0x33 => INC2(SP),
            0x34 => INC(ByteTarget::HLI),
            0x35 => DEC(ByteTarget::HLI),
            0x36 => LD(Load::Byte(
                ByteTarget::HLI,
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x37 => SCF,
            0x38 => JR(Carry, read_byte(&mut address, bus) as i8),
            0x39 => ADDHL(WordSource::R(SP)),
            0x3a => LD(Load::FromIndirectADec(ByteSource::HLI)),
            0x3b => DEC2(SP),
            0x3c => INC(ByteTarget::R(A)),
            0x3d => DEC(ByteTarget::R(A)),
            0x3e => LD(Load::Byte(
                ByteTarget::R(A),
                ByteSource::D8(read_byte(&mut address, bus)),
            )),
            0x3f => CCF,

            0x40 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(B))),
            0x41 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(C))),
            0x42 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(D))),
            0x43 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(E))),
            0x44 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(H))),
            0x45 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(L))),
            0x46 => LD(Load::Byte(ByteTarget::R(B), ByteSource::HLI)),
            0x47 => LD(Load::Byte(ByteTarget::R(B), ByteSource::R(A))),
            0x48 => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(B))),
            0x49 => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(C))),
            0x4a => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(D))),
            0x4b => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(E))),
            0x4c => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(H))),
            0x4d => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(L))),
            0x4e => LD(Load::Byte(ByteTarget::R(C), ByteSource::HLI)),
            0x4f => LD(Load::Byte(ByteTarget::R(C), ByteSource::R(A))),

            0x50 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(B))),
            0x51 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(C))),
            0x52 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(D))),
            0x53 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(E))),
            0x54 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(H))),
            0x55 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(L))),
            0x56 => LD(Load::Byte(ByteTarget::R(D), ByteSource::HLI)),
            0x57 => LD(Load::Byte(ByteTarget::R(D), ByteSource::R(A))),
            0x58 => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(B))),
            0x59 => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(C))),
            0x5a => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(D))),
            0x5b => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(E))),
            0x5c => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(H))),
            0x5d => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(L))),
            0x5e => LD(Load::Byte(ByteTarget::R(E), ByteSource::HLI)),
            0x5f => LD(Load::Byte(ByteTarget::R(E), ByteSource::R(A))),

            0x60 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(B))),
            0x61 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(C))),
            0x62 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(D))),
            0x63 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(E))),
            0x64 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(H))),
            0x65 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(L))),
            0x66 => LD(Load::Byte(ByteTarget::R(H), ByteSource::HLI)),
            0x67 => LD(Load::Byte(ByteTarget::R(H), ByteSource::R(A))),
            0x68 => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(B))),
            0x69 => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(C))),
            0x6a => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(D))),
            0x6b => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(E))),
            0x6c => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(H))),
            0x6d => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(L))),
            0x6e => LD(Load::Byte(ByteTarget::R(L), ByteSource::HLI)),
            0x6f => LD(Load::Byte(ByteTarget::R(L), ByteSource::R(A))),

            0x70 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(B))),
            0x71 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(C))),
            0x72 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(D))),
            0x73 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(E))),
            0x74 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(H))),
            0x75 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(L))),
            0x76 => HALT,
            0x77 => LD(Load::Byte(ByteTarget::HLI, ByteSource::R(A))),
            0x78 => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(B))),
            0x79 => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(C))),
            0x7a => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(D))),
            0x7b => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(E))),
            0x7c => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(H))),
            0x7d => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(L))),
            0x7e => LD(Load::Byte(ByteTarget::R(A), ByteSource::HLI)),
            0x7f => LD(Load::Byte(ByteTarget::R(A), ByteSource::R(A))),

            0x80 => ADD(ByteSource::R(B)),
            0x81 => ADD(ByteSource::R(C)),
            0x82 => ADD(ByteSource::R(D)),
            0x83 => ADD(ByteSource::R(E)),
            0x84 => ADD(ByteSource::R(H)),
            0x85 => ADD(ByteSource::R(L)),
            0x86 => ADD(ByteSource::HLI),
            0x87 => ADD(ByteSource::R(A)),
            0x88 => ADC(ByteSource::R(B)),
            0x89 => ADC(ByteSource::R(C)),
            0x8a => ADC(ByteSource::R(D)),
            0x8b => ADC(ByteSource::R(E)),
            0x8c => ADC(ByteSource::R(H)),
            0x8d => ADC(ByteSource::R(L)),
            0x8e => ADC(ByteSource::HLI),
            0x8f => ADC(ByteSource::R(A)),

            0x90 => SUB(ByteSource::R(B)),
            0x91 => SUB(ByteSource::R(C)),
            0x92 => SUB(ByteSource::R(D)),
            0x93 => SUB(ByteSource::R(E)),
            0x94 => SUB(ByteSource::R(H)),
            0x95 => SUB(ByteSource::R(L)),
            0x96 => SUB(ByteSource::HLI),
            0x97 => SUB(ByteSource::R(A)),
            0x98 => SBC(ByteSource::R(B)),
            0x99 => SBC(ByteSource::R(C)),
            0x9a => SBC(ByteSource::R(D)),
            0x9b => SBC(ByteSource::R(E)),
            0x9c => SBC(ByteSource::R(H)),
            0x9d => SBC(ByteSource::R(L)),
            0x9e => SBC(ByteSource::HLI),
            0x9f => SBC(ByteSource::R(A)),

            0xa0 => AND(ByteSource::R(B)),
            0xa1 => AND(ByteSource::R(C)),
            0xa2 => AND(ByteSource::R(D)),
            0xa3 => AND(ByteSource::R(E)),
            0xa4 => AND(ByteSource::R(H)),
            0xa5 => AND(ByteSource::R(L)),
            0xa6 => AND(ByteSource::HLI),
            0xa7 => AND(ByteSource::R(A)),
            0xa8 => XOR(ByteSource::R(B)),
            0xa9 => XOR(ByteSource::R(C)),
            0xaa => XOR(ByteSource::R(D)),
            0xab => XOR(ByteSource::R(E)),
            0xac => XOR(ByteSource::R(H)),
            0xad => XOR(ByteSource::R(L)),
            0xae => XOR(ByteSource::HLI),
            0xaf => XOR(ByteSource::R(A)),

            0xb0 => OR(ByteSource::R(B)),
            0xb1 => OR(ByteSource::R(C)),
            0xb2 => OR(ByteSource::R(D)),
            0xb3 => OR(ByteSource::R(E)),
            0xb4 => OR(ByteSource::R(H)),
            0xb5 => OR(ByteSource::R(L)),
            0xb6 => OR(ByteSource::HLI),
            0xb7 => OR(ByteSource::R(A)),
            0xb8 => CP(ByteSource::R(B)),
            0xb9 => CP(ByteSource::R(C)),
            0xba => CP(ByteSource::R(D)),
            0xbb => CP(ByteSource::R(E)),
            0xbc => CP(ByteSource::R(H)),
            0xbd => CP(ByteSource::R(L)),
            0xbe => CP(ByteSource::HLI),
            0xbf => CP(ByteSource::R(A)),

            0xc0 => RET(NotZero),
            0xc1 => POP(BC),
            0xc2 => JP(NotZero, JumpTarget::D16(read_word(&mut address, bus))),
            0xc3 => JP(Always, JumpTarget::D16(read_word(&mut address, bus))),
            0xc4 => CALL(NotZero, read_word(&mut address, bus)),
            0xc5 => PUSH(BC),
            0xc6 => ADD(ByteSource::D8(read_byte(&mut address, bus))),
            0xc7 => RST(ResetCode::RST00),
            0xc8 => RET(Zero),
            0xc9 => RET(Always),
            0xca => JP(Zero, JumpTarget::D16(read_word(&mut address, bus))),
            0xcb => unreachable!("CB Prefix"),
            0xcc => CALL(Zero, read_word(&mut address, bus)),
            0xcd => CALL(Always, read_word(&mut address, bus)),
            0xce => ADC(ByteSource::D8(read_byte(&mut address, bus))),
            0xcf => RST(ResetCode::RST08),

            0xd0 => RET(NotCarry),
            0xd1 => POP(DE),
            0xd2 => JP(NotCarry, JumpTarget::D16(read_word(&mut address, bus))),
            0xd3 => return (None, address),
            0xd4 => CALL(NotCarry, read_word(&mut address, bus)),
            0xd5 => PUSH(DE),
            0xd6 => SUB(ByteSource::D8(read_byte(&mut address, bus))),
            0xd7 => RST(ResetCode::RST10),
            0xd8 => RET(Carry),
            0xd9 => RETI,
            0xda => JP(Carry, JumpTarget::D16(read_word(&mut address, bus))),
            0xdb => return (None, address),
            0xdc => CALL(Carry, read_word(&mut address, bus)),
            0xdd => return (None, address),
            0xde => SBC(ByteSource::D8(read_byte(&mut address, bus))),
            0xdf => RST(ResetCode::RST18),

            0xe0 => LD(Load::IndirectFrom(
                IndirectByteRef::D8I(read_byte(&mut address, bus)),
                ByteSource::R(A),
            )),
            0xe1 => POP(HL),
            0xe2 => LD(Load::IndirectFrom(IndirectByteRef::CI, ByteSource::R(A))),
            0xe3 => return (None, address),
            0xe4 => return (None, address),
            0xe5 => PUSH(HL),
            0xe6 => AND(ByteSource::D8(read_byte(&mut address, bus))),
            0xe7 => RST(ResetCode::RST20),
            0xe8 => ADDSP(read_byte(&mut address, bus) as i8),
            0xe9 => JP(Always, JumpTarget::HL),
            0xea => LD(Load::IndirectFrom(
                IndirectByteRef::D16I(read_word(&mut address, bus)),
                ByteSource::R(A),
            )),
            0xeb => return (None, address),
            0xec => return (None, address),
            0xed => return (None, address),
            0xee => XOR(ByteSource::D8(read_byte(&mut address, bus))),
            0xef => RST(ResetCode::RST28),

            0xf0 => LD(Load::Byte(
                ByteTarget::R(A),
                ByteSource::D8I(read_byte(&mut address, bus)),
            )),
            0xf1 => POP(AF),
            0xf2 => LD(Load::Byte(ByteTarget::R(A), ByteSource::CI)),
            0xf3 => DI,
            0xf4 => return (None, address),
            0xf5 => PUSH(AF),
            0xf6 => OR(ByteSource::D8(read_byte(&mut address, bus))),
            0xf7 => RST(ResetCode::RST30),
            0xf8 => LD(Load::IndirectFromSPi8(
                WordTarget::R(HL),
                read_byte(&mut address, bus) as i8,
            )),
            0xf9 => LD(Load::Word(WordTarget::R(SP), WordSource::R(HL))),
            0xfa => LD(Load::Byte(
                ByteTarget::R(A),
                ByteSource::D16I(read_word(&mut address, bus)),
            )),
            0xfb => EI,
            0xfc => return (None, address),
            0xfd => return (None, address),
            0xfe => CP(ByteSource::D8(read_byte(&mut address, bus))),
            0xff => RST(ResetCode::RST38),
        };
        (Some(instruction), address)
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NOP => write!(f, "NOP"),
            HALT => write!(f, "HALT"),
            STOP => write!(f, "STOP"),
            DAA => write!(f, "DAA"),
            CPL => write!(f, "CPL"),
            CCF => write!(f, "CCF"),
            SCF => write!(f, "SCF"),
            RLA => write!(f, "RLA"),
            RRA => write!(f, "RRA"),
            EI => write!(f, "EI"),
            DI => write!(f, "DI"),
            RST(code) => write!(f, "RST {code}"),
            RET(test) => write!(f, "RET {test}"),
            RETI => write!(f, "RETI"),
            JP(test, target) => write!(f, "JP {test} {target}"),
            JR(test, offset) => write!(f, "JR {test}, {offset:#04x}"), // TODO: handle negative offsets
            CALL(test, address) => write!(f, "CALL {test}, {address:#06x}"),
            ADDHL(source) => write!(f, "ADD HL, {source}"),
            ADDSP(value) => write!(f, "ADD SP, {value:#04x}"),
            ADD(source) => write!(f, "ADD A, {source}"),
            ADC(source) => write!(f, "ADC A, {source}"),
            SUB(source) => write!(f, "SUB A, {source}"),
            SBC(source) => write!(f, "SBC A, {source}"),
            AND(source) => write!(f, "AND A, {source}"),
            OR(source) => write!(f, "OR A, {source}"),
            XOR(source) => write!(f, "XOR A, {source}"),
            CP(source) => write!(f, "CP A, {source}"),
            INC(target) => write!(f, "INC {target}"),
            INC2(target) => write!(f, "INC {target}"),
            DEC(target) => write!(f, "DEC {target}"),
            DEC2(target) => write!(f, "DEC {target}"),
            LD(load) => write!(f, "LD {load}"),
            BIT(bit, source) => write!(f, "BIT {bit}, {source}"),
            PUSH(target) => write!(f, "PUSH {target}"),
            POP(target) => write!(f, "POP {target}"),
            RES(bit, source) => write!(f, "RES {bit}, {source}"),
            RL(source) => write!(f, "RL {source}"),
            RLC(source) => write!(f, "RLC {source}"),
            RLCA => write!(f, "RLCA"),
            RR(source) => write!(f, "RR {source}"),
            RRC(source) => write!(f, "RRC {source}"),
            RRCA => write!(f, "RRCA"),
            SET(bit, source) => write!(f, "SET {bit}, {source}"),
            SLA(source) => write!(f, "SLA {source}"),
            SRA(source) => write!(f, "SRA {source}"),
            SRL(source) => write!(f, "SRL {source}"),
            SWAP(source) => write!(f, "SWAP {source}"),
        }
    }
}

/// Reads a byte from the bus at the given address,
/// increments the passed address and returns the read value.
#[inline]
fn read_byte<T: AddressSpace>(address: &mut u16, bus: &mut T) -> u8 {
    let value = bus.read(*address);
    *address += 1;
    value
}

/// Reads a word from the bus at the given address,
/// increments the passed address and returns the read value.
#[inline]
fn read_word<T: AddressSpace>(address: &mut u16, bus: &mut T) -> u16 {
    let lsb = read_byte(address, bus);
    let msb = read_byte(address, bus);
    u16::from(lsb) | (u16::from(msb) << 8)
}
