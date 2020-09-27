use crate::gb::cpu::CPU;
use crate::gb::AddressSpace;

#[derive(Debug)]
pub enum Instruction {
    ADD(ArithmeticByteTarget, ByteSource),
    ADD2(ArithmeticWordTarget, WordSource), // Same as ADD but with words
    ADC(ByteSource),                        // Add n + Carry flag to A
    AND(ByteSource),
    BIT(u8, ByteSource), // Test bit b in register r
    INC(IncDecTarget),
    CALL(JumpTest),
    CP(ByteSource), // Compare A with source
    CPL,            // Flips all bits in A register, sets N and H flags
    DAA,            // This instruction is useful when you’re using BCD value
    DI,             // Disables interrupt handling by setting ime = false
    DEC(IncDecTarget),
    EI,           // Enables interrupt handling by setting ime = true
    HALT,         // Halts and wait for interrupt
    JR(JumpTest), // Relative jump to given address
    JP(JumpTest, WordSource),
    LD(LoadType),
    NOP,
    OR(ByteSource),
    RET(JumpTest),
    RETI,              // Unconditional return which also enables interrupts
    RLA,               // Rotate `A` left through carry
    RLC(PrefixTarget), // Rotate target left
    RLCA,              // Rotate A left. Old bit 7 to Carry flag
    RR(ByteSource),    // Rotate n right through Carry flag
    RRA,               // Rotate A right through Carry flag
    RRCA,
    RST(ResetCode),
    SET(u8, ByteSource), // Set bit b in register r
    SRL(ByteSource),     // Shift right into Carry, MSB set to 0
    SUB(ArithmeticByteTarget, ByteSource),
    STOP,
    SWAP(ByteSource),
    PUSH(StackTarget), // Push to the stack memory, data from the 16-bit register
    POP(StackTarget),  // Pops to the 16-bit register
    XOR(ByteSource),
}

impl Instruction {
    pub fn from_byte(byte: u8, prefixed: bool) -> Option<Instruction> {
        match prefixed {
            true => Instruction::from_byte_prefixed(byte),
            false => Instruction::from_byte_not_prefixed(byte),
        }
    }

    /// Maps 0xCB prefixed opcodes to Instructions
    fn from_byte_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            //0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            0x11 => Some(Instruction::RLC(PrefixTarget::C)),
            0x19 => Some(Instruction::RR(ByteSource::C)),
            0x1a => Some(Instruction::RR(ByteSource::D)),
            0x1b => Some(Instruction::RR(ByteSource::E)),
            0x37 => Some(Instruction::SWAP(ByteSource::A)),
            0x38 => Some(Instruction::SRL(ByteSource::B)),
            0x3f => Some(Instruction::SRL(ByteSource::A)),
            0x7c => Some(Instruction::BIT(7, ByteSource::H)),
            0xfe => Some(Instruction::SET(7, ByteSource::HLI)),
            _ => None,
        }
    }

    /// Maps non-prefixed opcodes to Instructions
    fn from_byte_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x00 => Some(Instruction::NOP),
            0x01 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::BC,
                WordSource::D16,
            ))),
            0x03 => Some(Instruction::INC(IncDecTarget::BC)),
            0x04 => Some(Instruction::INC(IncDecTarget::B)),
            0x05 => Some(Instruction::DEC(IncDecTarget::B)),
            0x06 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::D8,
            ))),
            0x07 => Some(Instruction::RLCA),
            0x08 => Some(Instruction::LD(LoadType::IndirectFromWord(
                LoadWordTarget::D16I,
                WordSource::SP,
            ))),
            //0x0a => Some(Instruction::LD(LoadType::FromIndirect(
            //    LoadByteTarget::A,
            //    ByteSource::BC,
            //))),
            0x0b => Some(Instruction::DEC(IncDecTarget::BC)),
            0x0c => Some(Instruction::INC(IncDecTarget::C)),
            0x0d => Some(Instruction::DEC(IncDecTarget::C)),
            0x0e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::D8,
            ))),
            //0x0f => Some(Instruction::RRCA),
            //0x10 => Some(Instruction::STOP),
            0x11 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::DE,
                WordSource::D16,
            ))),
            0x12 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::DEI,
                ByteSource::A,
            ))),
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x14 => Some(Instruction::INC(IncDecTarget::D)),
            0x15 => Some(Instruction::DEC(IncDecTarget::D)),
            0x16 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::D8,
            ))),
            0x17 => Some(Instruction::RLA),
            0x18 => Some(Instruction::JR(JumpTest::Always)),
            0x19 => Some(Instruction::ADD2(ArithmeticWordTarget::HL, WordSource::DE)),
            0x1a => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::DEI,
            ))),
            0x1c => Some(Instruction::INC(IncDecTarget::E)),
            0x1d => Some(Instruction::DEC(IncDecTarget::E)),
            0x1e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::D8,
            ))),
            0x1f => Some(Instruction::RRA),
            0x20 => Some(Instruction::JR(JumpTest::NotZero)),
            0x21 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                WordSource::D16,
            ))),
            0x22 => Some(Instruction::LD(LoadType::IndirectFromAInc(
                LoadByteTarget::HLI,
            ))),
            0x23 => Some(Instruction::INC(IncDecTarget::HL)),
            0x24 => Some(Instruction::INC(IncDecTarget::H)),
            0x25 => Some(Instruction::DEC(IncDecTarget::H)),
            0x26 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::D8,
            ))),
            0x27 => Some(Instruction::DAA),
            0x28 => Some(Instruction::JR(JumpTest::Zero)),
            0x29 => Some(Instruction::ADD2(ArithmeticWordTarget::HL, WordSource::HL)),
            0x2a => Some(Instruction::LD(LoadType::FromIndirectAInc(ByteSource::HLI))),
            0x2c => Some(Instruction::INC(IncDecTarget::L)),
            0x2d => Some(Instruction::DEC(IncDecTarget::L)),
            0x2e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::D8,
            ))),
            0x2f => Some(Instruction::CPL),
            0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
            0x31 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                WordSource::D16,
            ))),
            0x32 => Some(Instruction::LD(LoadType::IndirectFromADec(
                LoadByteTarget::HLI,
            ))),
            0x33 => Some(Instruction::INC(IncDecTarget::SP)),
            0x34 => Some(Instruction::INC(IncDecTarget::HLI)),
            0x35 => Some(Instruction::DEC(IncDecTarget::HLI)),
            0x36 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::D8,
            ))),
            0x38 => Some(Instruction::JR(JumpTest::Carry)),
            0x39 => Some(Instruction::ADD2(ArithmeticWordTarget::HL, WordSource::SP)),
            0x3b => Some(Instruction::DEC(IncDecTarget::SP)),
            0x3c => Some(Instruction::INC(IncDecTarget::A)),
            0x3d => Some(Instruction::DEC(IncDecTarget::A)),
            0x3e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::D8,
            ))),
            0x40 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::B,
            ))),
            0x41 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::C,
            ))),
            0x42 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::D,
            ))),
            0x43 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::E,
            ))),
            0x44 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::H,
            ))),
            0x45 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::L,
            ))),
            0x46 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::HLI,
            ))),
            0x47 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::A,
            ))),
            0x48 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::B,
            ))),
            0x49 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::C,
            ))),
            0x4a => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::D,
            ))),
            0x4b => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::E,
            ))),
            0x4c => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::H,
            ))),
            0x4d => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::L,
            ))),
            0x4e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::HLI,
            ))),
            0x4f => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::A,
            ))),
            0x50 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::B,
            ))),
            0x51 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::C,
            ))),
            0x52 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::D,
            ))),
            0x53 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::E,
            ))),
            0x54 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::H,
            ))),
            0x55 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::L,
            ))),
            0x56 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::HLI,
            ))),
            0x57 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::A,
            ))),
            0x58 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::B,
            ))),
            0x59 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::C,
            ))),
            0x5a => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::D,
            ))),
            0x5b => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::E,
            ))),
            0x5c => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::H,
            ))),
            0x5d => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::L,
            ))),
            0x5e => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::E,
                ByteSource::HLI,
            ))),
            0x5f => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::A,
            ))),
            0x60 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::B,
            ))),
            0x61 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::C,
            ))),
            0x62 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::D,
            ))),
            0x63 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::E,
            ))),
            0x64 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::H,
            ))),
            0x65 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::L,
            ))),
            0x66 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::HLI,
            ))),
            0x67 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::A,
            ))),
            0x68 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::B,
            ))),
            0x69 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::C,
            ))),
            0x6a => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::D,
            ))),
            0x6b => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::E,
            ))),
            0x6c => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::H,
            ))),
            0x6d => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::L,
            ))),
            0x6e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::HLI,
            ))),
            0x6f => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::A,
            ))),
            0x70 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::B,
            ))),
            0x71 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::C,
            ))),
            0x72 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::D,
            ))),
            0x73 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::E,
            ))),
            0x74 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::H,
            ))),
            0x75 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::L,
            ))),
            0x76 => Some(Instruction::HALT),
            0x77 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::HLI,
                ByteSource::A,
            ))),
            0x78 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::B,
            ))),
            0x79 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::C,
            ))),
            0x7a => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::D,
            ))),
            0x7b => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::E,
            ))),
            0x7c => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::H,
            ))),
            0x7d => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::L,
            ))),
            0x7e => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::HLI,
            ))),
            0x7f => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::A,
            ))),
            //0x80 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::B,
            //)),
            0x81 => Some(Instruction::ADD(ArithmeticByteTarget::A, ByteSource::C)),
            //0x82 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::D,
            //)),
            //0x83 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::E,
            //)),
            //0x84 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::H,
            //)),
            //0x85 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::L,
            //)),
            0x86 => Some(Instruction::ADD(ArithmeticByteTarget::A, ByteSource::HLI)),
            0x87 => Some(Instruction::ADD(ArithmeticByteTarget::A, ByteSource::A)),
            //0x8b => Some(Instruction::ADC(ByteSource::E)),
            //0x8c => Some(Instruction::ADC(ByteSource::H)),
            //0x8d => Some(Instruction::ADC(ByteSource::L)),
            //0x8e => Some(Instruction::ADC(ByteSource::HLI)),
            //0x8f => Some(Instruction::ADC(ByteSource::A)),
            0x90 => Some(Instruction::SUB(ArithmeticByteTarget::A, ByteSource::B)),
            0x91 => Some(Instruction::SUB(ArithmeticByteTarget::A, ByteSource::C)),
            //0x92 => Some(Instruction::SUB(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::D,
            //)),
            //0x96 => Some(Instruction::SUB(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::HLI,
            //)),
            0xa0 => Some(Instruction::AND(ByteSource::B)),
            0xa1 => Some(Instruction::AND(ByteSource::C)),
            //0xa2 => Some(Instruction::AND(ByteSource::D)),
            //0xa3 => Some(Instruction::AND(ByteSource::E)),
            0xa7 => Some(Instruction::AND(ByteSource::A)),
            0xa9 => Some(Instruction::XOR(ByteSource::C)),
            0xad => Some(Instruction::XOR(ByteSource::L)),
            0xae => Some(Instruction::XOR(ByteSource::HLI)),
            0xaf => Some(Instruction::XOR(ByteSource::A)),
            0xb0 => Some(Instruction::OR(ByteSource::B)),
            0xb1 => Some(Instruction::OR(ByteSource::C)),
            //0xb3 => Some(Instruction::OR(ByteSource::E)),
            0xb6 => Some(Instruction::OR(ByteSource::HLI)),
            0xb7 => Some(Instruction::OR(ByteSource::A)),
            0xb8 => Some(Instruction::OR(ByteSource::B)),
            0xb9 => Some(Instruction::OR(ByteSource::C)),
            0xba => Some(Instruction::CP(ByteSource::D)),
            0xbb => Some(Instruction::CP(ByteSource::E)),
            0xbe => Some(Instruction::CP(ByteSource::HLI)),
            0xc0 => Some(Instruction::RET(JumpTest::NotZero)),
            0xc1 => Some(Instruction::POP(StackTarget::BC)),
            0xc3 => Some(Instruction::JP(JumpTest::Always, WordSource::D16)),
            0xc4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xc6 => Some(Instruction::ADD(ArithmeticByteTarget::A, ByteSource::D8)),
            0xc7 => Some(Instruction::RST(ResetCode::RST00)),
            0xc9 => Some(Instruction::RET(JumpTest::Always)),
            0xca => Some(Instruction::JP(JumpTest::Zero, WordSource::D16)),
            0xcc => Some(Instruction::CALL(JumpTest::Zero)),
            0xcd => Some(Instruction::CALL(JumpTest::Always)),
            0xce => Some(Instruction::ADC(ByteSource::D8)),
            0xcf => Some(Instruction::RST(ResetCode::RST08)),
            0xc2 => Some(Instruction::JP(JumpTest::NotZero, WordSource::D16)),
            0xc5 => Some(Instruction::PUSH(StackTarget::BC)),
            0xc8 => Some(Instruction::RET(JumpTest::Zero)),
            0xd0 => Some(Instruction::RET(JumpTest::NotCarry)),
            0xd1 => Some(Instruction::POP(StackTarget::DE)),
            0xd2 => Some(Instruction::JP(JumpTest::NotCarry, WordSource::D16)),
            0xd4 => Some(Instruction::CALL(JumpTest::NotCarry)),
            0xd5 => Some(Instruction::PUSH(StackTarget::DE)),
            0xd6 => Some(Instruction::SUB(ArithmeticByteTarget::A, ByteSource::D8)),
            0xd8 => Some(Instruction::RET(JumpTest::Carry)),
            0xd9 => Some(Instruction::RETI),
            0xda => Some(Instruction::JP(JumpTest::Carry, WordSource::D16)),
            0xdc => Some(Instruction::CALL(JumpTest::Carry)),
            0xdf => Some(Instruction::RST(ResetCode::RST18)),
            0xe0 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::D8I,
                ByteSource::A,
            ))),
            0xe1 => Some(Instruction::POP(StackTarget::HL)),
            0xe2 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::C,
                ByteSource::A,
            ))),
            0xe5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xe6 => Some(Instruction::AND(ByteSource::D8)),
            0xef => Some(Instruction::RST(ResetCode::RST28)),
            0xe9 => Some(Instruction::JP(JumpTest::Always, WordSource::HL)),
            0xea => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::D16I,
                ByteSource::A,
            ))),
            0xee => Some(Instruction::XOR(ByteSource::D8)),
            0xf0 => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::D8I,
            ))),
            0xf1 => Some(Instruction::POP(StackTarget::AF)),
            0xf3 => Some(Instruction::DI),
            0xf5 => Some(Instruction::PUSH(StackTarget::AF)),
            0xf6 => Some(Instruction::OR(ByteSource::D8)),
            0xf8 => Some(Instruction::LD(LoadType::IndirectFromSPi8(
                LoadWordTarget::HL,
            ))),
            0xf9 => Some(Instruction::LD(LoadType::IndirectFromWord(
                LoadWordTarget::SP,
                WordSource::HL,
            ))),
            0xfa => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::D16I,
            ))),
            0xfb => Some(Instruction::EI),
            0xfe => Some(Instruction::CP(ByteSource::D8)),
            0xff => Some(Instruction::RST(ResetCode::RST38)),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum ArithmeticByteTarget {
    A,
    HL,
}

#[derive(Debug)]
pub enum ArithmeticWordTarget {
    HL,
    SP,
}

#[derive(Debug)]
pub enum IncDecTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    BC,
    DE,
    HL,
    SP,
    HLI,
}

#[derive(Debug)]
pub enum PrefixTarget {
    B,
    C,
}

#[derive(Debug)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpTest {
    /// Resolves the referring value
    pub fn resolve_value<T: AddressSpace>(&self, cpu: &mut CPU<T>) -> bool {
        match *self {
            JumpTest::NotZero => !cpu.r.f.zero,
            JumpTest::Zero => cpu.r.f.zero,
            JumpTest::NotCarry => !cpu.r.f.carry,
            JumpTest::Carry => cpu.r.f.carry,
            JumpTest::Always => true,
        }
    }
}

#[derive(Debug)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8I,  // value refers to address stored in next 8 bits | 0xFF00
    D16I, // value refers to address stored in next 16 bits
    DEI,  // value refers to address stored in DE register
    HLI,  // value refers to address stored in HL register
}

#[derive(Debug, PartialEq)]
pub enum ByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8,   // direct 8 bit value
    D8I,  // value refers to address stored in next 8 bits | 0xFF00
    BCI,  // value refers to address stored in BC register
    DEI,  // value refers to address stored in DE register
    HLI,  // value refers to address stored in HL register
    D16I, // value refers to address stored in next 16 bits
}

impl ByteSource {
    /// Resolves the referring value
    pub fn resolve_value<T: AddressSpace>(&self, cpu: &mut CPU<T>) -> u8 {
        match *self {
            ByteSource::A => cpu.r.a,
            ByteSource::B => cpu.r.b,
            ByteSource::C => cpu.r.c,
            ByteSource::D => cpu.r.d,
            ByteSource::E => cpu.r.e,
            ByteSource::H => cpu.r.h,
            ByteSource::L => cpu.r.l,
            ByteSource::D8 => cpu.consume_byte(),
            ByteSource::D8I => {
                let address = cpu.consume_byte() as u16;
                cpu.read(0xFF00 | address)
            }
            ByteSource::BCI => cpu.read(cpu.r.get_bc()),
            ByteSource::DEI => cpu.read(cpu.r.get_de()),
            ByteSource::HLI => cpu.read(cpu.r.get_hl()),
            ByteSource::D16I => {
                let address = cpu.consume_word();
                cpu.read(address)
            }
        }
    }
}

#[derive(Debug)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    D16I, // value refers to address stored in next 16 bits
}

#[derive(Debug)]
pub enum WordSource {
    DE,
    HL,
    D16, // direct 16 bit value
    SP,
}

impl WordSource {
    /// Resolves the referring value
    pub fn resolve_value<T: AddressSpace>(&self, cpu: &mut CPU<T>) -> u16 {
        match *self {
            WordSource::DE => cpu.r.get_de(),
            WordSource::HL => cpu.r.get_hl(),
            WordSource::D16 => cpu.consume_word(),
            WordSource::SP => cpu.sp,
        }
    }
}

#[derive(Debug)]
pub enum LoadType {
    Byte(LoadByteTarget, ByteSource),
    Word(LoadWordTarget, WordSource), // just like the Byte type except with 16-bit values
    IndirectFrom(LoadByteTarget, ByteSource), // load a memory location whose address is stored in AddressSource with the contents of the register ByteSource
    IndirectFromAInc(LoadByteTarget),
    IndirectFromADec(LoadByteTarget), // Same as IndirectFromA, source value is decremented afterwards
    FromIndirect(LoadByteTarget, ByteSource), // load the A register with the contents from a value from a memory location whose address is stored in some location
    FromIndirectAInc(ByteSource),
    IndirectFromWord(LoadWordTarget, WordSource),
    IndirectFromSPi8(LoadWordTarget), // Put SP plus 8 bit immediate value into target.
}

#[derive(Debug)]
pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum ResetCode {
    RST00,
    RST08,
    RST18,
    RST28,
    RST38,
}
