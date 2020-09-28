use crate::gb::cpu::CPU;
use crate::gb::AddressSpace;

#[derive(Debug)]
pub enum Instruction {
    ADD(ByteSource),                        // Add n to target
    ADD2(ArithmeticWordTarget, WordSource), // Same as ADD but with words
    ADDSP,                                  // Add signed immediate 8 bit value to Stack Pointer
    ADC(ByteSource),                        // Add n + Carry flag to A
    AND(ByteSource),                        // Logically AND n with A, result in A
    BIT(u8, ByteSource),                    // Test bit b in register r
    INC(IncDecTarget),                      // Increment register n
    CALL(JumpTest), // Push address of next instruction onto stack and then  jump to address nn
    CCF,            // Complement carry flag
    CP(ByteSource), // Compare A with source
    CPL,            // Flips all bits in A register, sets N and H flags
    DAA,            // This instruction is useful when you’re using BCD value
    DI,             // Disables interrupt handling by setting ime = false
    DEC(IncDecTarget), // Decrement register n
    EI,             // Enables interrupt handling by setting ime = true
    HALT,           // Halts and wait for interrupt
    JR(JumpTest),   // Relative jump to given address
    JP(JumpTest, WordSource), // Jump to address nn
    LD(LoadType),   // Put value into n
    NOP,            // No operation
    OR(ByteSource), // Logical OR n with register A, result in A.
    PUSH(StackTarget), // Push to the stack memory, data from the 16-bit register
    POP(StackTarget), // Pops to the 16-bit register
    RES(u8, ByteSource), // Reset bit b in register r
    RET(JumpTest),  // Pop two bytes from stack & jump to that address
    RETI,           // Unconditional return which also enables interrupts
    RL(PrefixTarget), // Rotate n left through Carry flag
    RLA,            // Rotate `A` left through carry
    RLC(PrefixTarget), // Rotate target left
    RLCA,           // Rotate A left. Old bit 7 to Carry flag
    RR(PrefixTarget), // Rotate n right through Carry flag
    RRA,            // Rotate A right through Carry flag
    RRC(PrefixTarget), // Rotate n right. Old bit 0 to Carry flag
    RRCA,           // Rotate A right. Old bit 0 to Carry flag
    RST(ResetCode), // Push present address onto stack.  Jump to address 0x0000 + n
    SBC(ByteSource), // Subtract n + Carry flag from A
    SCF,            // Set carry flag
    SET(u8, ByteSource), // Set bit b in register r
    SLA(PrefixTarget), // Shift n left into Carry. LSB of n set to 0
    SRA(PrefixTarget), // Shift n right into Carry. MSB doesn't change
    SRL(ByteSource), // Shift right into Carry, MSB set to 0
    SUB(ByteSource), // Subtract n from A
    STOP,           // Halt CPU & LCD display until button pressed
    SWAP(ByteSource), // Swap upper & lower nibbles of n
    XOR(ByteSource), // Logical exclusive OR n with register A, result in A
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
            0x00 => Some(Instruction::RLC(PrefixTarget::B)),
            0x01 => Some(Instruction::RLC(PrefixTarget::C)),
            0x02 => Some(Instruction::RLC(PrefixTarget::D)),
            0x03 => Some(Instruction::RLC(PrefixTarget::E)),
            0x04 => Some(Instruction::RLC(PrefixTarget::H)),
            0x05 => Some(Instruction::RLC(PrefixTarget::L)),
            0x06 => Some(Instruction::RLC(PrefixTarget::HLI)),
            0x07 => Some(Instruction::RLC(PrefixTarget::A)),
            0x08 => Some(Instruction::RRC(PrefixTarget::B)),
            0x09 => Some(Instruction::RRC(PrefixTarget::C)),
            0x0a => Some(Instruction::RRC(PrefixTarget::D)),
            0x0b => Some(Instruction::RRC(PrefixTarget::E)),
            0x0c => Some(Instruction::RRC(PrefixTarget::H)),
            0x0d => Some(Instruction::RRC(PrefixTarget::L)),
            0x0e => Some(Instruction::RRC(PrefixTarget::HLI)),
            0x0f => Some(Instruction::RRC(PrefixTarget::A)),

            0x10 => Some(Instruction::RL(PrefixTarget::B)),
            0x11 => Some(Instruction::RL(PrefixTarget::C)),
            0x12 => Some(Instruction::RL(PrefixTarget::D)),
            0x13 => Some(Instruction::RL(PrefixTarget::E)),
            0x14 => Some(Instruction::RL(PrefixTarget::H)),
            0x15 => Some(Instruction::RL(PrefixTarget::L)),
            0x16 => Some(Instruction::RL(PrefixTarget::HLI)),
            0x17 => Some(Instruction::RL(PrefixTarget::A)),
            0x18 => Some(Instruction::RR(PrefixTarget::B)),
            0x19 => Some(Instruction::RR(PrefixTarget::C)),
            0x1a => Some(Instruction::RR(PrefixTarget::D)),
            0x1b => Some(Instruction::RR(PrefixTarget::E)),
            0x1c => Some(Instruction::RR(PrefixTarget::H)),
            0x1d => Some(Instruction::RR(PrefixTarget::L)),
            0x1e => Some(Instruction::RR(PrefixTarget::HLI)),
            0x1f => Some(Instruction::RR(PrefixTarget::A)),

            0x20 => Some(Instruction::SLA(PrefixTarget::B)),
            0x21 => Some(Instruction::SLA(PrefixTarget::C)),
            0x22 => Some(Instruction::SLA(PrefixTarget::D)),
            0x23 => Some(Instruction::SLA(PrefixTarget::E)),
            0x24 => Some(Instruction::SLA(PrefixTarget::H)),
            0x25 => Some(Instruction::SLA(PrefixTarget::L)),
            0x26 => Some(Instruction::SLA(PrefixTarget::HLI)),
            0x27 => Some(Instruction::SLA(PrefixTarget::A)),
            0x28 => Some(Instruction::SRA(PrefixTarget::B)),
            0x29 => Some(Instruction::SRA(PrefixTarget::C)),
            0x2a => Some(Instruction::SRA(PrefixTarget::D)),
            0x2b => Some(Instruction::SRA(PrefixTarget::E)),
            0x2c => Some(Instruction::SRA(PrefixTarget::H)),
            0x2d => Some(Instruction::SRA(PrefixTarget::L)),
            0x2e => Some(Instruction::SRA(PrefixTarget::HLI)),
            0x2f => Some(Instruction::SRA(PrefixTarget::A)),

            0x30 => Some(Instruction::SWAP(ByteSource::B)),
            0x31 => Some(Instruction::SWAP(ByteSource::C)),
            0x32 => Some(Instruction::SWAP(ByteSource::D)),
            0x33 => Some(Instruction::SWAP(ByteSource::E)),
            0x34 => Some(Instruction::SWAP(ByteSource::H)),
            0x35 => Some(Instruction::SWAP(ByteSource::L)),
            0x36 => Some(Instruction::SWAP(ByteSource::HLI)),
            0x37 => Some(Instruction::SWAP(ByteSource::A)),
            0x38 => Some(Instruction::SRL(ByteSource::B)),
            0x39 => Some(Instruction::SRL(ByteSource::C)),
            0x3a => Some(Instruction::SRL(ByteSource::D)),
            0x3b => Some(Instruction::SRL(ByteSource::E)),
            0x3c => Some(Instruction::SRL(ByteSource::H)),
            0x3d => Some(Instruction::SRL(ByteSource::L)),
            0x3e => Some(Instruction::SRL(ByteSource::HLI)),
            0x3f => Some(Instruction::SRL(ByteSource::A)),

            0x40 => Some(Instruction::BIT(0, ByteSource::B)),
            0x41 => Some(Instruction::BIT(0, ByteSource::C)),
            0x42 => Some(Instruction::BIT(0, ByteSource::D)),
            0x43 => Some(Instruction::BIT(0, ByteSource::E)),
            0x44 => Some(Instruction::BIT(0, ByteSource::H)),
            0x45 => Some(Instruction::BIT(0, ByteSource::L)),
            0x46 => Some(Instruction::BIT(0, ByteSource::HLI)),
            0x47 => Some(Instruction::BIT(0, ByteSource::A)),
            0x48 => Some(Instruction::BIT(1, ByteSource::B)),
            0x49 => Some(Instruction::BIT(1, ByteSource::C)),
            0x4a => Some(Instruction::BIT(1, ByteSource::D)),
            0x4b => Some(Instruction::BIT(1, ByteSource::E)),
            0x4c => Some(Instruction::BIT(1, ByteSource::H)),
            0x4d => Some(Instruction::BIT(1, ByteSource::L)),
            0x4e => Some(Instruction::BIT(1, ByteSource::HLI)),
            0x4f => Some(Instruction::BIT(1, ByteSource::A)),

            0x50 => Some(Instruction::BIT(2, ByteSource::B)),
            0x51 => Some(Instruction::BIT(2, ByteSource::C)),
            0x52 => Some(Instruction::BIT(2, ByteSource::D)),
            0x53 => Some(Instruction::BIT(2, ByteSource::E)),
            0x54 => Some(Instruction::BIT(2, ByteSource::H)),
            0x55 => Some(Instruction::BIT(2, ByteSource::L)),
            0x56 => Some(Instruction::BIT(2, ByteSource::HLI)),
            0x57 => Some(Instruction::BIT(2, ByteSource::A)),
            0x58 => Some(Instruction::BIT(3, ByteSource::B)),
            0x59 => Some(Instruction::BIT(3, ByteSource::C)),
            0x5a => Some(Instruction::BIT(3, ByteSource::D)),
            0x5b => Some(Instruction::BIT(3, ByteSource::E)),
            0x5c => Some(Instruction::BIT(3, ByteSource::H)),
            0x5d => Some(Instruction::BIT(3, ByteSource::L)),
            0x5e => Some(Instruction::BIT(3, ByteSource::HLI)),
            0x5f => Some(Instruction::BIT(3, ByteSource::A)),

            0x60 => Some(Instruction::BIT(4, ByteSource::B)),
            0x61 => Some(Instruction::BIT(4, ByteSource::C)),
            0x62 => Some(Instruction::BIT(4, ByteSource::D)),
            0x63 => Some(Instruction::BIT(4, ByteSource::E)),
            0x64 => Some(Instruction::BIT(4, ByteSource::H)),
            0x65 => Some(Instruction::BIT(4, ByteSource::L)),
            0x66 => Some(Instruction::BIT(4, ByteSource::HLI)),
            0x67 => Some(Instruction::BIT(4, ByteSource::A)),
            0x68 => Some(Instruction::BIT(5, ByteSource::B)),
            0x69 => Some(Instruction::BIT(5, ByteSource::C)),
            0x6a => Some(Instruction::BIT(5, ByteSource::D)),
            0x6b => Some(Instruction::BIT(5, ByteSource::E)),
            0x6c => Some(Instruction::BIT(5, ByteSource::H)),
            0x6d => Some(Instruction::BIT(5, ByteSource::L)),
            0x6e => Some(Instruction::BIT(5, ByteSource::HLI)),
            0x6f => Some(Instruction::BIT(5, ByteSource::A)),

            0x70 => Some(Instruction::BIT(6, ByteSource::B)),
            0x71 => Some(Instruction::BIT(6, ByteSource::C)),
            0x72 => Some(Instruction::BIT(6, ByteSource::D)),
            0x73 => Some(Instruction::BIT(6, ByteSource::E)),
            0x74 => Some(Instruction::BIT(6, ByteSource::H)),
            0x75 => Some(Instruction::BIT(6, ByteSource::L)),
            0x76 => Some(Instruction::BIT(6, ByteSource::HLI)),
            0x77 => Some(Instruction::BIT(6, ByteSource::A)),
            0x78 => Some(Instruction::BIT(7, ByteSource::B)),
            0x79 => Some(Instruction::BIT(7, ByteSource::C)),
            0x7a => Some(Instruction::BIT(7, ByteSource::D)),
            0x7b => Some(Instruction::BIT(7, ByteSource::E)),
            0x7c => Some(Instruction::BIT(7, ByteSource::H)),
            0x7d => Some(Instruction::BIT(7, ByteSource::L)),
            0x7e => Some(Instruction::BIT(7, ByteSource::HLI)),
            0x7f => Some(Instruction::BIT(7, ByteSource::A)),

            0x80 => Some(Instruction::RES(0, ByteSource::B)),
            0x81 => Some(Instruction::RES(0, ByteSource::C)),
            0x82 => Some(Instruction::RES(0, ByteSource::D)),
            0x83 => Some(Instruction::RES(0, ByteSource::E)),
            0x84 => Some(Instruction::RES(0, ByteSource::H)),
            0x85 => Some(Instruction::RES(0, ByteSource::L)),
            0x86 => Some(Instruction::RES(0, ByteSource::HLI)),
            0x87 => Some(Instruction::RES(0, ByteSource::A)),
            0x88 => Some(Instruction::RES(1, ByteSource::B)),
            0x89 => Some(Instruction::RES(1, ByteSource::C)),
            0x8a => Some(Instruction::RES(1, ByteSource::D)),
            0x8b => Some(Instruction::RES(1, ByteSource::E)),
            0x8c => Some(Instruction::RES(1, ByteSource::H)),
            0x8d => Some(Instruction::RES(1, ByteSource::L)),
            0x8e => Some(Instruction::RES(1, ByteSource::HLI)),
            0x8f => Some(Instruction::RES(1, ByteSource::A)),

            0x90 => Some(Instruction::RES(2, ByteSource::B)),
            0x91 => Some(Instruction::RES(2, ByteSource::C)),
            0x92 => Some(Instruction::RES(2, ByteSource::D)),
            0x93 => Some(Instruction::RES(2, ByteSource::E)),
            0x94 => Some(Instruction::RES(2, ByteSource::H)),
            0x95 => Some(Instruction::RES(2, ByteSource::L)),
            0x96 => Some(Instruction::RES(2, ByteSource::HLI)),
            0x97 => Some(Instruction::RES(2, ByteSource::A)),
            0x98 => Some(Instruction::RES(3, ByteSource::B)),
            0x99 => Some(Instruction::RES(3, ByteSource::C)),
            0x9a => Some(Instruction::RES(3, ByteSource::D)),
            0x9b => Some(Instruction::RES(3, ByteSource::E)),
            0x9c => Some(Instruction::RES(3, ByteSource::H)),
            0x9d => Some(Instruction::RES(3, ByteSource::L)),
            0x9e => Some(Instruction::RES(3, ByteSource::HLI)),
            0x9f => Some(Instruction::RES(3, ByteSource::A)),

            0xa0 => Some(Instruction::RES(4, ByteSource::B)),
            0xa1 => Some(Instruction::RES(4, ByteSource::C)),
            0xa2 => Some(Instruction::RES(4, ByteSource::D)),
            0xa3 => Some(Instruction::RES(4, ByteSource::E)),
            0xa4 => Some(Instruction::RES(4, ByteSource::H)),
            0xa5 => Some(Instruction::RES(4, ByteSource::L)),
            0xa6 => Some(Instruction::RES(4, ByteSource::HLI)),
            0xa7 => Some(Instruction::RES(4, ByteSource::A)),
            0xa8 => Some(Instruction::RES(5, ByteSource::B)),
            0xa9 => Some(Instruction::RES(5, ByteSource::C)),
            0xaa => Some(Instruction::RES(5, ByteSource::D)),
            0xab => Some(Instruction::RES(5, ByteSource::E)),
            0xac => Some(Instruction::RES(5, ByteSource::H)),
            0xad => Some(Instruction::RES(5, ByteSource::L)),
            0xae => Some(Instruction::RES(5, ByteSource::HLI)),
            0xaf => Some(Instruction::RES(5, ByteSource::A)),

            0xb0 => Some(Instruction::RES(6, ByteSource::B)),
            0xb1 => Some(Instruction::RES(6, ByteSource::C)),
            0xb2 => Some(Instruction::RES(6, ByteSource::D)),
            0xb3 => Some(Instruction::RES(6, ByteSource::E)),
            0xb4 => Some(Instruction::RES(6, ByteSource::H)),
            0xb5 => Some(Instruction::RES(6, ByteSource::L)),
            0xb6 => Some(Instruction::RES(6, ByteSource::HLI)),
            0xb7 => Some(Instruction::RES(6, ByteSource::A)),
            0xb8 => Some(Instruction::RES(7, ByteSource::B)),
            0xb9 => Some(Instruction::RES(7, ByteSource::C)),
            0xba => Some(Instruction::RES(7, ByteSource::D)),
            0xbb => Some(Instruction::RES(7, ByteSource::E)),
            0xbc => Some(Instruction::RES(7, ByteSource::H)),
            0xbd => Some(Instruction::RES(7, ByteSource::L)),
            0xbe => Some(Instruction::RES(7, ByteSource::HLI)),
            0xbf => Some(Instruction::RES(7, ByteSource::A)),

            0xc0 => Some(Instruction::SET(0, ByteSource::B)),
            0xc1 => Some(Instruction::SET(0, ByteSource::C)),
            0xc2 => Some(Instruction::SET(0, ByteSource::D)),
            0xc3 => Some(Instruction::SET(0, ByteSource::E)),
            0xc4 => Some(Instruction::SET(0, ByteSource::H)),
            0xc5 => Some(Instruction::SET(0, ByteSource::L)),
            0xc6 => Some(Instruction::SET(0, ByteSource::HLI)),
            0xc7 => Some(Instruction::SET(0, ByteSource::A)),
            0xc8 => Some(Instruction::SET(1, ByteSource::B)),
            0xc9 => Some(Instruction::SET(1, ByteSource::C)),
            0xca => Some(Instruction::SET(1, ByteSource::D)),
            0xcb => Some(Instruction::SET(1, ByteSource::E)),
            0xcc => Some(Instruction::SET(1, ByteSource::H)),
            0xcd => Some(Instruction::SET(1, ByteSource::L)),
            0xce => Some(Instruction::SET(1, ByteSource::HLI)),
            0xcf => Some(Instruction::SET(1, ByteSource::A)),

            0xd0 => Some(Instruction::SET(2, ByteSource::B)),
            0xd1 => Some(Instruction::SET(2, ByteSource::C)),
            0xd2 => Some(Instruction::SET(2, ByteSource::D)),
            0xd3 => Some(Instruction::SET(2, ByteSource::E)),
            0xd4 => Some(Instruction::SET(2, ByteSource::H)),
            0xd5 => Some(Instruction::SET(2, ByteSource::L)),
            0xd6 => Some(Instruction::SET(2, ByteSource::HLI)),
            0xd7 => Some(Instruction::SET(2, ByteSource::A)),
            0xd8 => Some(Instruction::SET(3, ByteSource::B)),
            0xd9 => Some(Instruction::SET(3, ByteSource::C)),
            0xda => Some(Instruction::SET(3, ByteSource::D)),
            0xdb => Some(Instruction::SET(3, ByteSource::E)),
            0xdc => Some(Instruction::SET(3, ByteSource::H)),
            0xdd => Some(Instruction::SET(3, ByteSource::L)),
            0xde => Some(Instruction::SET(3, ByteSource::HLI)),
            0xdf => Some(Instruction::SET(3, ByteSource::A)),

            0xe0 => Some(Instruction::SET(4, ByteSource::B)),
            0xe1 => Some(Instruction::SET(4, ByteSource::C)),
            0xe2 => Some(Instruction::SET(4, ByteSource::D)),
            0xe3 => Some(Instruction::SET(4, ByteSource::E)),
            0xe4 => Some(Instruction::SET(4, ByteSource::H)),
            0xe5 => Some(Instruction::SET(4, ByteSource::L)),
            0xe6 => Some(Instruction::SET(4, ByteSource::HLI)),
            0xe7 => Some(Instruction::SET(4, ByteSource::A)),
            0xe8 => Some(Instruction::SET(5, ByteSource::B)),
            0xe9 => Some(Instruction::SET(5, ByteSource::C)),
            0xea => Some(Instruction::SET(5, ByteSource::D)),
            0xeb => Some(Instruction::SET(5, ByteSource::E)),
            0xec => Some(Instruction::SET(5, ByteSource::H)),
            0xed => Some(Instruction::SET(5, ByteSource::L)),
            0xee => Some(Instruction::SET(5, ByteSource::HLI)),
            0xef => Some(Instruction::SET(5, ByteSource::A)),

            0xf0 => Some(Instruction::SET(6, ByteSource::B)),
            0xf1 => Some(Instruction::SET(6, ByteSource::C)),
            0xf2 => Some(Instruction::SET(6, ByteSource::D)),
            0xf3 => Some(Instruction::SET(6, ByteSource::E)),
            0xf4 => Some(Instruction::SET(6, ByteSource::H)),
            0xf5 => Some(Instruction::SET(6, ByteSource::L)),
            0xf6 => Some(Instruction::SET(6, ByteSource::HLI)),
            0xf7 => Some(Instruction::SET(6, ByteSource::A)),
            0xf8 => Some(Instruction::SET(7, ByteSource::B)),
            0xf9 => Some(Instruction::SET(7, ByteSource::C)),
            0xfa => Some(Instruction::SET(7, ByteSource::D)),
            0xfb => Some(Instruction::SET(7, ByteSource::E)),
            0xfc => Some(Instruction::SET(7, ByteSource::H)),
            0xfd => Some(Instruction::SET(7, ByteSource::L)),
            0xfe => Some(Instruction::SET(7, ByteSource::HLI)),
            0xff => Some(Instruction::SET(7, ByteSource::A)),
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
            0x02 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::BCI,
                ByteSource::A,
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
            0x09 => Some(Instruction::ADD2(ArithmeticWordTarget::HL, WordSource::BC)),
            0x0a => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::BCI,
            ))),
            0x0b => Some(Instruction::DEC(IncDecTarget::BC)),
            0x0c => Some(Instruction::INC(IncDecTarget::C)),
            0x0d => Some(Instruction::DEC(IncDecTarget::C)),
            0x0e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::D8,
            ))),
            0x0f => Some(Instruction::RRCA),

            0x10 => Some(Instruction::STOP),
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
            0x1b => Some(Instruction::DEC(IncDecTarget::DE)),
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
            0x2b => Some(Instruction::DEC(IncDecTarget::HL)),
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
            0x37 => Some(Instruction::SCF),
            0x38 => Some(Instruction::JR(JumpTest::Carry)),
            0x39 => Some(Instruction::ADD2(ArithmeticWordTarget::HL, WordSource::SP)),
            0x3a => Some(Instruction::LD(LoadType::FromIndirectADec(ByteSource::HLI))),
            0x3b => Some(Instruction::DEC(IncDecTarget::SP)),
            0x3c => Some(Instruction::INC(IncDecTarget::A)),
            0x3d => Some(Instruction::DEC(IncDecTarget::A)),
            0x3e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::A,
                ByteSource::D8,
            ))),
            0x3f => Some(Instruction::CCF),

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
                LoadByteTarget::CIFF00,
                ByteSource::B,
            ))),
            0x49 => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::C,
            ))),
            0x4a => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::D,
            ))),
            0x4b => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::E,
            ))),
            0x4c => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::H,
            ))),
            0x4d => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::L,
            ))),
            0x4e => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
                ByteSource::HLI,
            ))),
            0x4f => Some(Instruction::LD(LoadType::Byte(
                LoadByteTarget::CIFF00,
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

            0x80 => Some(Instruction::ADD(ByteSource::B)),
            0x81 => Some(Instruction::ADD(ByteSource::C)),
            0x82 => Some(Instruction::ADD(ByteSource::D)),
            0x83 => Some(Instruction::ADD(ByteSource::E)),
            0x84 => Some(Instruction::ADD(ByteSource::H)),
            0x85 => Some(Instruction::ADD(ByteSource::L)),
            0x86 => Some(Instruction::ADD(ByteSource::HLI)),
            0x87 => Some(Instruction::ADD(ByteSource::A)),
            0x88 => Some(Instruction::ADC(ByteSource::B)),
            0x89 => Some(Instruction::ADC(ByteSource::C)),
            0x8a => Some(Instruction::ADC(ByteSource::D)),
            0x8b => Some(Instruction::ADC(ByteSource::E)),
            0x8c => Some(Instruction::ADC(ByteSource::H)),
            0x8d => Some(Instruction::ADC(ByteSource::L)),
            0x8e => Some(Instruction::ADC(ByteSource::HLI)),
            0x8f => Some(Instruction::ADC(ByteSource::A)),

            0x90 => Some(Instruction::SUB(ByteSource::B)),
            0x91 => Some(Instruction::SUB(ByteSource::C)),
            0x92 => Some(Instruction::SUB(ByteSource::D)),
            0x93 => Some(Instruction::SUB(ByteSource::E)),
            0x94 => Some(Instruction::SUB(ByteSource::H)),
            0x95 => Some(Instruction::SUB(ByteSource::L)),
            0x96 => Some(Instruction::SUB(ByteSource::HLI)),
            0x97 => Some(Instruction::SUB(ByteSource::A)),
            0x98 => Some(Instruction::SBC(ByteSource::B)),
            0x99 => Some(Instruction::SBC(ByteSource::C)),
            0x9a => Some(Instruction::SBC(ByteSource::D)),
            0x9b => Some(Instruction::SBC(ByteSource::E)),
            0x9c => Some(Instruction::SBC(ByteSource::H)),
            0x9d => Some(Instruction::SBC(ByteSource::L)),
            0x9e => Some(Instruction::SBC(ByteSource::HLI)),
            0x9f => Some(Instruction::SBC(ByteSource::A)),

            0xa0 => Some(Instruction::AND(ByteSource::B)),
            0xa1 => Some(Instruction::AND(ByteSource::C)),
            0xa2 => Some(Instruction::AND(ByteSource::D)),
            0xa3 => Some(Instruction::AND(ByteSource::E)),
            0xa4 => Some(Instruction::AND(ByteSource::H)),
            0xa5 => Some(Instruction::AND(ByteSource::L)),
            0xa6 => Some(Instruction::AND(ByteSource::HLI)),
            0xa7 => Some(Instruction::AND(ByteSource::A)),
            0xa8 => Some(Instruction::XOR(ByteSource::B)),
            0xa9 => Some(Instruction::XOR(ByteSource::C)),
            0xaa => Some(Instruction::XOR(ByteSource::D)),
            0xab => Some(Instruction::XOR(ByteSource::E)),
            0xac => Some(Instruction::XOR(ByteSource::H)),
            0xad => Some(Instruction::XOR(ByteSource::L)),
            0xae => Some(Instruction::XOR(ByteSource::HLI)),
            0xaf => Some(Instruction::XOR(ByteSource::A)),

            0xb0 => Some(Instruction::OR(ByteSource::B)),
            0xb1 => Some(Instruction::OR(ByteSource::C)),
            0xb2 => Some(Instruction::OR(ByteSource::D)),
            0xb3 => Some(Instruction::OR(ByteSource::E)),
            0xb4 => Some(Instruction::OR(ByteSource::H)),
            0xb5 => Some(Instruction::OR(ByteSource::L)),
            0xb6 => Some(Instruction::OR(ByteSource::HLI)),
            0xb7 => Some(Instruction::OR(ByteSource::A)),
            0xb8 => Some(Instruction::CP(ByteSource::B)),
            0xb9 => Some(Instruction::CP(ByteSource::C)),
            0xba => Some(Instruction::CP(ByteSource::D)),
            0xbb => Some(Instruction::CP(ByteSource::E)),
            0xbc => Some(Instruction::CP(ByteSource::H)),
            0xbd => Some(Instruction::CP(ByteSource::L)),
            0xbe => Some(Instruction::CP(ByteSource::HLI)),
            0xbf => Some(Instruction::CP(ByteSource::A)),

            0xc0 => Some(Instruction::RET(JumpTest::NotZero)),
            0xc1 => Some(Instruction::POP(StackTarget::BC)),
            0xc3 => Some(Instruction::JP(JumpTest::Always, WordSource::D16)),
            0xc4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xc6 => Some(Instruction::ADD(ByteSource::D8)),
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
            0xd6 => Some(Instruction::SUB(ByteSource::D8)),
            0xd8 => Some(Instruction::RET(JumpTest::Carry)),
            0xd9 => Some(Instruction::RETI),
            0xda => Some(Instruction::JP(JumpTest::Carry, WordSource::D16)),
            0xdc => Some(Instruction::CALL(JumpTest::Carry)),
            0xde => Some(Instruction::SBC(ByteSource::D8)),
            0xdf => Some(Instruction::RST(ResetCode::RST18)),

            0xe0 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::D8IFF00,
                ByteSource::A,
            ))),
            0xe1 => Some(Instruction::POP(StackTarget::HL)),
            0xe2 => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::CIFF00,
                ByteSource::A,
            ))),
            0xe5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xe6 => Some(Instruction::AND(ByteSource::D8)),
            0xe8 => Some(Instruction::ADDSP),
            0xe9 => Some(Instruction::JP(JumpTest::Always, WordSource::HL)),
            0xea => Some(Instruction::LD(LoadType::IndirectFrom(
                LoadByteTarget::D16I,
                ByteSource::A,
            ))),
            0xee => Some(Instruction::XOR(ByteSource::D8)),
            0xef => Some(Instruction::RST(ResetCode::RST28)),

            0xf0 => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::D8IFF00,
            ))),
            0xf1 => Some(Instruction::POP(StackTarget::AF)),
            0xf2 => Some(Instruction::LD(LoadType::FromIndirect(
                LoadByteTarget::A,
                ByteSource::CIFF00,
            ))),
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
pub enum ArithmeticWordTarget {
    HL,
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
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

impl PrefixTarget {
    /// Resolves the referring value
    pub fn resolve_value<T: AddressSpace>(&self, cpu: &mut CPU<T>) -> u8 {
        match *self {
            PrefixTarget::A => cpu.r.a,
            PrefixTarget::B => cpu.r.b,
            PrefixTarget::C => cpu.r.c,
            PrefixTarget::D => cpu.r.d,
            PrefixTarget::E => cpu.r.e,
            PrefixTarget::H => cpu.r.h,
            PrefixTarget::L => cpu.r.l,
            PrefixTarget::HLI => cpu.read(cpu.r.get_hl()),
        }
    }
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
    D,
    E,
    H,
    L,
    BCI,     // value refers to address stored in BC register
    DEI,     // value refers to address stored in DE register
    HLI,     // value refers to address stored in HL register
    D16I,    // value refers to address stored in next 16 bits
    CIFF00,  // value refers to C register + 0xFF00
    D8IFF00, // value refers to address stored in next 8 bits + 0xFF00
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
    D8,      // direct 8 bit value
    BCI,     // value refers to address stored in BC register
    DEI,     // value refers to address stored in DE register
    HLI,     // value refers to address stored in HL register
    D16I,    // value refers to address stored in next 16 bits
    CIFF00,  // value refers to C register + 0xFF00
    D8IFF00, // value refers to address stored in next 8 bits + 0xFF00
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
            ByteSource::BCI => cpu.read(cpu.r.get_bc()),
            ByteSource::DEI => cpu.read(cpu.r.get_de()),
            ByteSource::HLI => cpu.read(cpu.r.get_hl()),
            ByteSource::D16I => {
                let address = cpu.consume_word();
                cpu.read(address)
            }
            ByteSource::CIFF00 => cpu.read(u16::from(cpu.r.c).wrapping_add(0xFF00)),
            ByteSource::D8IFF00 => {
                let address = u16::from(cpu.consume_byte()).wrapping_add(0xFF00);
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
    BC,
    DE,
    HL,
    SP,
    D16, // direct 16 bit value
}

impl WordSource {
    /// Resolves the referring value
    pub fn resolve_value<T: AddressSpace>(&self, cpu: &mut CPU<T>) -> u16 {
        match *self {
            WordSource::BC => cpu.r.get_bc(),
            WordSource::DE => cpu.r.get_de(),
            WordSource::HL => cpu.r.get_hl(),
            WordSource::SP => cpu.sp,
            WordSource::D16 => cpu.consume_word(),
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
    FromIndirectADec(ByteSource),
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
