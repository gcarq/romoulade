#[derive(Debug)]
pub enum Instruction {
    ADD(ArithmeticByteTarget, ArithmeticByteSource),
    ADD2(ArithmeticWordTarget, ArithmeticWordSource), // Same as ADD but with words
    ADC(ByteSource),
    AND(BitOperationSource),
    BIT(u8, BitOperationSource), // Test bit b in register r. TODO: only registers allowed
    INC(IncDecTarget),
    CALL(JumpTest),
    CP(ByteSource), // Compare A with source
    DAA,            // This instruction is useful when you’re using BCD value
    DI,             // Disables interrupt handling by setting ime = false
    DEC(IncDecTarget),
    EI, // Enables interrupt handling by setting ime = true
    HALT,
    JR(JumpTest), // Relative jump to given address
    JP(JumpTest),
    LD(LoadType),
    NOP,
    OR(BitOperationSource),
    RET(JumpTest),
    RLA,               // Rotate `A` left through carry
    RLC(PrefixTarget), // Rotate target left
    RRCA,
    RST(ResetCode),
    SUB(ArithmeticByteTarget, ArithmeticByteSource),
    STOP,
    PUSH(StackTarget), // Push to the stack memory, data from the 16-bit register
    POP(StackTarget),  // Pops to the 16-bit register
    XOR(BitOperationSource),
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
            0x7c => Some(Instruction::BIT(7, BitOperationSource::H)),
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
            //0x03 => Some(Instruction::INC(IncDecTarget::BC)),
            0x04 => Some(Instruction::INC(IncDecTarget::B)),
            0x05 => Some(Instruction::DEC(IncDecTarget::B)),
            0x06 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::B,
                ByteSource::D8,
            ))),
            //0x08 => Some(Instruction::LD(LoadType::IndirectFromWord(
            //    AddressSource::D16,
            //    WordSource::SP,
            //))),
            //0x0a => Some(Instruction::LD(LoadType::FromIndirect(
            //    LoadByteTarget::A,
            //    ByteSource::BC,
            //))),
            //0x0b => Some(Instruction::DEC(IncDecTarget::BC)),
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
            0x13 => Some(Instruction::INC(IncDecTarget::DE)),
            0x15 => Some(Instruction::DEC(IncDecTarget::D)),
            0x16 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::D8,
            ))),
            0x17 => Some(Instruction::RLA),
            0x18 => Some(Instruction::JR(JumpTest::Always)),
            //0x19 => Some(Instruction::ADD2(
            //    ArithmeticWordTarget::HL,
            //    ArithmeticWordSource::DE,
            //)),
            0x1a => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::DE,
            ))),
            0x1d => Some(Instruction::DEC(IncDecTarget::E)),
            0x1e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::E,
                ByteSource::D8,
            ))),
            0x20 => Some(Instruction::JR(JumpTest::NotZero)),
            0x21 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::HL,
                WordSource::D16,
            ))),
            0x22 => Some(Instruction::LD(LoadType::IndirectFromAInc(ByteSource::HLI))),
            0x23 => Some(Instruction::INC(IncDecTarget::HL)),
            0x24 => Some(Instruction::INC(IncDecTarget::H)),
            //0x25 => Some(Instruction::DEC(IncDecTarget::H)),
            //0x27 => Some(Instruction::DAA),
            0x28 => Some(Instruction::JR(JumpTest::Zero)),
            //0x29 => Some(Instruction::ADD2(
            //    ArithmeticWordTarget::HL,
            //    ArithmeticWordSource::HL,
            //)),
            //0x2a => Some(Instruction::LD(LoadType::FromIndirectAInc(ByteSource::HLI))),
            //0x2c => Some(Instruction::INC(IncDecTarget::L)),
            //0x2d => Some(Instruction::DEC(IncDecTarget::L)),
            0x2e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::L,
                ByteSource::D8,
            ))),
            //0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
            0x31 => Some(Instruction::LD(LoadType::Word(
                LoadWordTarget::SP,
                WordSource::D16,
            ))),
            0x32 => Some(Instruction::LD(LoadType::IndirectFromADec(ByteSource::HLI))),
            //0x36 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::HLI,
            //    ByteSource::D8,
            //))),
            //0x38 => Some(Instruction::JR(JumpTest::Carry)),
            //0x3b => Some(Instruction::DEC(IncDecTarget::SP)),
            //0x3c => Some(Instruction::INC(IncDecTarget::A)),
            0x3d => Some(Instruction::DEC(IncDecTarget::A)),
            0x3e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::D8,
            ))),
            //0x40 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::B,
            //))),
            //0x41 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::C,
            //))),
            //0x42 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::D,
            //))),
            //0x43 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::E,
            //))),
            //0x44 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::H,
            //))),
            //0x45 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::L,
            //))),
            //0x46 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::B,
            //    ByteSource::HLI,
            //))),
            0x4f => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::C,
                ByteSource::A,
            ))),
            //0x50 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::B,
            //))),
            //0x51 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::C,
            //))),
            //0x52 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::D,
            //))),
            //0x53 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::E,
            //))),
            //0x54 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::H,
            //))),
            //0x55 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::L,
            //))),
            //0x56 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::D,
            //    ByteSource::HLI,
            //))),
            0x57 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::D,
                ByteSource::A,
            ))),
            //0x5d => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::E,
            //    ByteSource::L,
            //))),
            //0x5e => Some(Instruction::LD(FromIndirect(
            //    LoadByteTarget::E,
            //    ByteSource::HLI,
            //))),
            //0x5f => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::E,
            //    ByteSource::A,
            //))),
            //0x60 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::B,
            //))),
            //0x61 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::C,
            //))),
            //0x62 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::D,
            //))),
            //0x63 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::E,
            //))),
            //0x64 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::H,
            //))),
            //0x65 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::H,
            //    ByteSource::L,
            //))),
            0x67 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::H,
                ByteSource::A,
            ))),
            //0x69 => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::L,
            //    ByteSource::C,
            //))),
            //0x6d => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::L,
            //    ByteSource::L,
            //))),
            //0x6f => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::L,
            //    ByteSource::A,
            //))),
            // TODO: is this the correct mapping?
            //0x72 => Some(Instruction::LD(LoadType::IndirectFrom(
            //    AddressSource::HLI,
            //    ByteSource::D,
            //))),
            //0x75 => Some(Instruction::LD(LoadType::IndirectFrom(
            //    AddressSource::HLI,
            //    ByteSource::L,
            //))),
            //0x76 => Some(Instruction::HALT),
            0x77 => Some(Instruction::LD(LoadType::IndirectFrom(
                AddressSource::HLI,
                ByteSource::A,
            ))),
            0x78 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::B,
            ))),
            //0x7a => Some(Instruction::LD(LoadType::Byte(
            //    LoadByteTarget::A,
            //    ByteSource::D,
            //))),
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
            //0x7e => Some(Instruction::LD(FromIndirect(
            //    LoadByteTarget::A,
            //    ByteSource::HLI,
            //))),
            //0x80 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::B,
            //)),
            //0x81 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::C,
            //)),
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
            0x86 => Some(Instruction::ADD(
                ArithmeticByteTarget::A,
                ArithmeticByteSource::HLI,
            )),
            //0x87 => Some(Instruction::ADD(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::A,
            //)),
            //0x8b => Some(Instruction::ADC(ByteSource::E)),
            //0x8c => Some(Instruction::ADC(ByteSource::H)),
            //0x8d => Some(Instruction::ADC(ByteSource::L)),
            //0x8e => Some(Instruction::ADC(ByteSource::HLI)),
            //0x8f => Some(Instruction::ADC(ByteSource::A)),
            0x90 => Some(Instruction::SUB(
                ArithmeticByteTarget::A,
                ArithmeticByteSource::B,
            )),
            //0x92 => Some(Instruction::SUB(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::D,
            //)),
            //0x96 => Some(Instruction::SUB(
            //    ArithmeticByteTarget::A,
            //    ArithmeticByteSource::HLI,
            //)),
            //0xa0 => Some(Instruction::AND(BitOperationSource::B)),
            //0xa1 => Some(Instruction::AND(BitOperationSource::C)),
            //0xa2 => Some(Instruction::AND(BitOperationSource::D)),
            //0xa3 => Some(Instruction::AND(BitOperationSource::E)),
            //0xa7 => Some(Instruction::AND(BitOperationSource::A)),
            0xaf => Some(Instruction::XOR(BitOperationSource::A)),
            //0xb3 => Some(Instruction::OR(BitOperationSource::E)),
            0xbe => Some(Instruction::CP(ByteSource::HLI)),
            0xc1 => Some(Instruction::POP(StackTarget::BC)),
            //0xc3 => Some(Instruction::JP(JumpTest::Always)),
            0xc9 => Some(Instruction::RET(JumpTest::Always)),
            0xcd => Some(Instruction::CALL(JumpTest::Always)),
            //0xcf => Some(Instruction::RST(ResetCode::RST08)),
            0xc5 => Some(Instruction::PUSH(StackTarget::BC)),
            //0xd5 => Some(Instruction::PUSH(StackTarget::DE)),
            //0xdf => Some(Instruction::RST(ResetCode::RST18)),
            0xe0 => Some(Instruction::LD(LoadType::IndirectFrom(
                AddressSource::D8,
                ByteSource::A,
            ))),
            //0xe1 => Some(Instruction::POP(StackTarget::HL)),
            0xe2 => Some(Instruction::LD(LoadType::IndirectFrom(
                AddressSource::C,
                ByteSource::A,
            ))),
            //// TODO: this is not correct! 0xe9 => Some(Instruction::JP(WordSource::HL)),
            0xea => Some(Instruction::LD(LoadType::IndirectFrom(
                AddressSource::D16,
                ByteSource::A,
            ))),
            //0xef => Some(Instruction::RST(ResetCode::RST28)),
            0xf0 => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::D8,
            ))),
            //0xf3 => Some(Instruction::DI),
            //0xfb => Some(Instruction::EI),
            0xfe => Some(Instruction::CP(ByteSource::D8)),
            //0xff => Some(Instruction::RST(ResetCode::RST38)),
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
pub enum ArithmeticByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    DEI,
    HLI,
    D8,
}

#[derive(Debug)]
pub enum ArithmeticWordTarget {
    HL,
    SP,
}

#[derive(Debug)]
pub enum ArithmeticWordSource {
    DE,
    HL,
}

#[derive(Debug)]
pub enum BitOperationSource {
    A,
    B,
    C,
    D,
    E,
    H,
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

#[derive(Debug)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI, // value refers to address stored in HL register
}

#[derive(Debug)]
/// TODO: move D16 to own struct?
pub enum ByteSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    D8, // direct 8 bit value
    BC,
    DE,
    HLI, // value refers to address stored in HL register
}

#[derive(Debug)]
pub enum AddressSource {
    C,
    D8,  // direct 8 bit value
    D16, // direct 16 bit value
    HLI,
}

#[derive(Debug)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum WordSource {
    HL,
    D16, // direct 16 bit value
    SP,
}

#[derive(Debug)]
pub enum LoadType {
    Byte(LoadByteTarget, ByteSource),
    Word(LoadWordTarget, WordSource), // just like the Byte type except with 16-bit values
    IndirectFrom(AddressSource, ByteSource), // load a memory location whose address is stored in AddressSource with the contents of the register ByteSource
    IndirectFromAInc(ByteSource),
    IndirectFromADec(ByteSource), // Same as IndirectFromA, source value is decremented afterwards
    FromIndirect(LoadByteTarget, ByteSource), // load the A register with the contents from a value from a memory location whose address is stored in some location
    FromIndirectAInc(ByteSource),
    IndirectFromWord(AddressSource, WordSource),
    // AFromByteAddress: Just like AFromIndirect except the memory address is some address in the very last byte of memory.
    // ByteAddressFromA: Just like IndirectFromA except the memory address is some address in the very last byte of memory.
}

#[derive(Debug)]
pub enum StackTarget {
    BC,
    DE,
    HL,
}

#[derive(Debug)]
pub enum ResetCode {
    RST08,
    RST18,
    RST28,
    RST38,
}
