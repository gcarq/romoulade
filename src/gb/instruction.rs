use crate::gb::cpu::CPU;
use crate::gb::AddressSpace;

#[derive(Debug)]
pub enum Instruction {
    ADD(ByteSource),          // Add n to target
    ADDHL(WordSource),        // Add nn to HL
    ADDSP,                    // Add signed immediate 8 bit value to Stack Pointer
    ADC(ByteSource),          // Add n + Carry flag to A
    AND(ByteSource),          // Logically AND n with A, result in A
    BIT(u8, ByteSource),      // Test bit b in register r
    INC(IncDecByteTarget),    // Increment single byte register n
    INC2(IncDecWordTarget),   // Increment word register n
    CALL(JumpTest), // Push address of next instruction onto stack and then  jump to address nn
    CCF,            // Complement carry flag
    CP(ByteSource), // Compare A with source
    CPL,            // Flips all bits in A register, sets N and H flags
    DAA,            // This instruction is useful when you’re using BCD value
    DI,             // Disables interrupt handling by setting ime = false
    DEC(IncDecByteTarget), // Decrement single byte register n
    DEC2(IncDecWordTarget), // Decrement word register n
    EI,             // Enables interrupt handling by setting ime = true
    HALT,           // Halts and wait for interrupt
    JR(JumpTest),   // Relative jump to given address
    JP(JumpTest, WordSource), // Jump to address nn
    LD(Load),       // Put value into n
    NOP,            // No operation
    OR(ByteSource), // Logical OR n with register A, result in A.
    PUSH(StackTarget), // Push to the stack memory, data from the 16-bit register
    POP(StackTarget), // Pops to the 16-bit register
    RES(u8, ByteSource), // Reset bit b in register r
    RET(JumpTest),  // Pop two bytes from stack & jump to that address
    RETI,           // Unconditional return which also enables interrupts
    RL(ByteSource), // Rotate n left through Carry flag
    RLA,            // Rotate `A` left through carry
    RLC(ByteSource), // Rotate target left
    RLCA,           // Rotate A left. Old bit 7 to Carry flag
    RR(ByteSource), // Rotate n right through Carry flag
    RRA,            // Rotate A right through Carry flag
    RRC(ByteSource), // Rotate n right. Old bit 0 to Carry flag
    RRCA,           // Rotate A right. Old bit 0 to Carry flag
    RST(ResetCode), // Push present address onto stack.  Jump to address 0x0000 + n
    SBC(ByteSource), // Subtract n + Carry flag from A
    SCF,            // Set carry flag
    SET(u8, ByteSource), // Set bit b in register r
    SLA(ByteSource), // Shift n left into Carry. LSB of n set to 0
    SRA(ByteSource), // Shift n right into Carry. MSB doesn't change
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
            0x00 => Some(Instruction::RLC(ByteSource::B)),
            0x01 => Some(Instruction::RLC(ByteSource::C)),
            0x02 => Some(Instruction::RLC(ByteSource::D)),
            0x03 => Some(Instruction::RLC(ByteSource::E)),
            0x04 => Some(Instruction::RLC(ByteSource::H)),
            0x05 => Some(Instruction::RLC(ByteSource::L)),
            0x06 => Some(Instruction::RLC(ByteSource::HLI)),
            0x07 => Some(Instruction::RLC(ByteSource::A)),

            0x08 => Some(Instruction::RRC(ByteSource::B)),
            0x09 => Some(Instruction::RRC(ByteSource::C)),
            0x0a => Some(Instruction::RRC(ByteSource::D)),
            0x0b => Some(Instruction::RRC(ByteSource::E)),
            0x0c => Some(Instruction::RRC(ByteSource::H)),
            0x0d => Some(Instruction::RRC(ByteSource::L)),
            0x0e => Some(Instruction::RRC(ByteSource::HLI)),
            0x0f => Some(Instruction::RRC(ByteSource::A)),

            0x10 => Some(Instruction::RL(ByteSource::B)),
            0x11 => Some(Instruction::RL(ByteSource::C)),
            0x12 => Some(Instruction::RL(ByteSource::D)),
            0x13 => Some(Instruction::RL(ByteSource::E)),
            0x14 => Some(Instruction::RL(ByteSource::H)),
            0x15 => Some(Instruction::RL(ByteSource::L)),
            0x16 => Some(Instruction::RL(ByteSource::HLI)),
            0x17 => Some(Instruction::RL(ByteSource::A)),

            0x18 => Some(Instruction::RR(ByteSource::B)),
            0x19 => Some(Instruction::RR(ByteSource::C)),
            0x1a => Some(Instruction::RR(ByteSource::D)),
            0x1b => Some(Instruction::RR(ByteSource::E)),
            0x1c => Some(Instruction::RR(ByteSource::H)),
            0x1d => Some(Instruction::RR(ByteSource::L)),
            0x1e => Some(Instruction::RR(ByteSource::HLI)),
            0x1f => Some(Instruction::RR(ByteSource::A)),

            0x20 => Some(Instruction::SLA(ByteSource::B)),
            0x21 => Some(Instruction::SLA(ByteSource::C)),
            0x22 => Some(Instruction::SLA(ByteSource::D)),
            0x23 => Some(Instruction::SLA(ByteSource::E)),
            0x24 => Some(Instruction::SLA(ByteSource::H)),
            0x25 => Some(Instruction::SLA(ByteSource::L)),
            0x26 => Some(Instruction::SLA(ByteSource::HLI)),
            0x27 => Some(Instruction::SLA(ByteSource::A)),

            0x28 => Some(Instruction::SRA(ByteSource::B)),
            0x29 => Some(Instruction::SRA(ByteSource::C)),
            0x2a => Some(Instruction::SRA(ByteSource::D)),
            0x2b => Some(Instruction::SRA(ByteSource::E)),
            0x2c => Some(Instruction::SRA(ByteSource::H)),
            0x2d => Some(Instruction::SRA(ByteSource::L)),
            0x2e => Some(Instruction::SRA(ByteSource::HLI)),
            0x2f => Some(Instruction::SRA(ByteSource::A)),

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
        }
    }

    /// Maps non-prefixed opcodes to Instructions
    fn from_byte_not_prefixed(opcode: u8) -> Option<Instruction> {
        match opcode {
            0x00 => Some(Instruction::NOP),
            0x01 => Some(Instruction::LD(Load::Word(
                LoadWordTarget::BC,
                WordSource::D16,
            ))),
            0x02 => Some(Instruction::LD(Load::IndirectFrom(
                LoadByteTarget::BCI,
                ByteSource::A,
            ))),
            0x03 => Some(Instruction::INC2(IncDecWordTarget::BC)),
            0x04 => Some(Instruction::INC(IncDecByteTarget::B)),
            0x05 => Some(Instruction::DEC(IncDecByteTarget::B)),
            0x06 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::D8,
            ))),
            0x07 => Some(Instruction::RLCA),
            0x08 => Some(Instruction::LD(Load::IndirectFromWord(
                LoadWordTarget::D16I,
                WordSource::SP,
            ))),
            0x09 => Some(Instruction::ADDHL(WordSource::BC)),
            0x0a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::BCI,
            ))),
            0x0b => Some(Instruction::DEC2(IncDecWordTarget::BC)),
            0x0c => Some(Instruction::INC(IncDecByteTarget::C)),
            0x0d => Some(Instruction::DEC(IncDecByteTarget::C)),
            0x0e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::D8,
            ))),
            0x0f => Some(Instruction::RRCA),

            0x10 => Some(Instruction::STOP),
            0x11 => Some(Instruction::LD(Load::Word(
                LoadWordTarget::DE,
                WordSource::D16,
            ))),
            0x12 => Some(Instruction::LD(Load::IndirectFrom(
                LoadByteTarget::DEI,
                ByteSource::A,
            ))),
            0x13 => Some(Instruction::INC2(IncDecWordTarget::DE)),
            0x14 => Some(Instruction::INC(IncDecByteTarget::D)),
            0x15 => Some(Instruction::DEC(IncDecByteTarget::D)),
            0x16 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::D8,
            ))),
            0x17 => Some(Instruction::RLA),
            0x18 => Some(Instruction::JR(JumpTest::Always)),
            0x19 => Some(Instruction::ADDHL(WordSource::DE)),
            0x1a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::DEI,
            ))),
            0x1b => Some(Instruction::DEC2(IncDecWordTarget::DE)),
            0x1c => Some(Instruction::INC(IncDecByteTarget::E)),
            0x1d => Some(Instruction::DEC(IncDecByteTarget::E)),
            0x1e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::D8,
            ))),
            0x1f => Some(Instruction::RRA),

            0x20 => Some(Instruction::JR(JumpTest::NotZero)),
            0x21 => Some(Instruction::LD(Load::Word(
                LoadWordTarget::HL,
                WordSource::D16,
            ))),
            0x22 => Some(Instruction::LD(Load::IndirectFromAInc(LoadByteTarget::HLI))),
            0x23 => Some(Instruction::INC2(IncDecWordTarget::HL)),
            0x24 => Some(Instruction::INC(IncDecByteTarget::H)),
            0x25 => Some(Instruction::DEC(IncDecByteTarget::H)),
            0x26 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::D8,
            ))),
            0x27 => Some(Instruction::DAA),
            0x28 => Some(Instruction::JR(JumpTest::Zero)),
            0x29 => Some(Instruction::ADDHL(WordSource::HL)),
            0x2a => Some(Instruction::LD(Load::FromIndirectAInc(ByteSource::HLI))),
            0x2b => Some(Instruction::DEC2(IncDecWordTarget::HL)),
            0x2c => Some(Instruction::INC(IncDecByteTarget::L)),
            0x2d => Some(Instruction::DEC(IncDecByteTarget::L)),
            0x2e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::D8,
            ))),
            0x2f => Some(Instruction::CPL),

            0x30 => Some(Instruction::JR(JumpTest::NotCarry)),
            0x31 => Some(Instruction::LD(Load::Word(
                LoadWordTarget::SP,
                WordSource::D16,
            ))),
            0x32 => Some(Instruction::LD(Load::IndirectFromADec(LoadByteTarget::HLI))),
            0x33 => Some(Instruction::INC2(IncDecWordTarget::SP)),
            0x34 => Some(Instruction::INC(IncDecByteTarget::HLI)),
            0x35 => Some(Instruction::DEC(IncDecByteTarget::HLI)),
            0x36 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::D8,
            ))),
            0x37 => Some(Instruction::SCF),
            0x38 => Some(Instruction::JR(JumpTest::Carry)),
            0x39 => Some(Instruction::ADDHL(WordSource::SP)),
            0x3a => Some(Instruction::LD(Load::FromIndirectADec(ByteSource::HLI))),
            0x3b => Some(Instruction::DEC2(IncDecWordTarget::SP)),
            0x3c => Some(Instruction::INC(IncDecByteTarget::A)),
            0x3d => Some(Instruction::DEC(IncDecByteTarget::A)),
            0x3e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::D8,
            ))),
            0x3f => Some(Instruction::CCF),

            0x40 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::B,
            ))),
            0x41 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::C,
            ))),
            0x42 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::D,
            ))),
            0x43 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::E,
            ))),
            0x44 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::H,
            ))),
            0x45 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::L,
            ))),
            0x46 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::HLI,
            ))),
            0x47 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::B,
                ByteSource::A,
            ))),
            0x48 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::B,
            ))),
            0x49 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::C,
            ))),
            0x4a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::D,
            ))),
            0x4b => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::E,
            ))),
            0x4c => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::H,
            ))),
            0x4d => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::L,
            ))),
            0x4e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::HLI,
            ))),
            0x4f => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::C,
                ByteSource::A,
            ))),

            0x50 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::B,
            ))),
            0x51 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::C,
            ))),
            0x52 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::D,
            ))),
            0x53 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::E,
            ))),
            0x54 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::H,
            ))),
            0x55 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::L,
            ))),
            0x56 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::HLI,
            ))),
            0x57 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::D,
                ByteSource::A,
            ))),
            0x58 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::B,
            ))),
            0x59 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::C,
            ))),
            0x5a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::D,
            ))),
            0x5b => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::E,
            ))),
            0x5c => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::H,
            ))),
            0x5d => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::L,
            ))),
            0x5e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::HLI,
            ))),
            0x5f => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::E,
                ByteSource::A,
            ))),

            0x60 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::B,
            ))),
            0x61 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::C,
            ))),
            0x62 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::D,
            ))),
            0x63 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::E,
            ))),
            0x64 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::H,
            ))),
            0x65 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::L,
            ))),
            0x66 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::HLI,
            ))),
            0x67 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::H,
                ByteSource::A,
            ))),
            0x68 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::B,
            ))),
            0x69 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::C,
            ))),
            0x6a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::D,
            ))),
            0x6b => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::E,
            ))),
            0x6c => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::H,
            ))),
            0x6d => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::L,
            ))),
            0x6e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::HLI,
            ))),
            0x6f => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::L,
                ByteSource::A,
            ))),

            0x70 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::B,
            ))),
            0x71 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::C,
            ))),
            0x72 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::D,
            ))),
            0x73 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::E,
            ))),
            0x74 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::H,
            ))),
            0x75 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::L,
            ))),
            0x76 => Some(Instruction::HALT),
            0x77 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::HLI,
                ByteSource::A,
            ))),
            0x78 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::B,
            ))),
            0x79 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::C,
            ))),
            0x7a => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::D,
            ))),
            0x7b => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::E,
            ))),
            0x7c => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::H,
            ))),
            0x7d => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::L,
            ))),
            0x7e => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::HLI,
            ))),
            0x7f => Some(Instruction::LD(Load::Byte(
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
            0xc2 => Some(Instruction::JP(JumpTest::NotZero, WordSource::D16)),
            0xc3 => Some(Instruction::JP(JumpTest::Always, WordSource::D16)),
            0xc4 => Some(Instruction::CALL(JumpTest::NotZero)),
            0xc5 => Some(Instruction::PUSH(StackTarget::BC)),
            0xc6 => Some(Instruction::ADD(ByteSource::D8)),
            0xc7 => Some(Instruction::RST(ResetCode::RST00)),
            0xc8 => Some(Instruction::RET(JumpTest::Zero)),
            0xc9 => Some(Instruction::RET(JumpTest::Always)),
            0xca => Some(Instruction::JP(JumpTest::Zero, WordSource::D16)),
            0xcb => panic!("CB Prefix"),
            0xcc => Some(Instruction::CALL(JumpTest::Zero)),
            0xcd => Some(Instruction::CALL(JumpTest::Always)),
            0xce => Some(Instruction::ADC(ByteSource::D8)),
            0xcf => Some(Instruction::RST(ResetCode::RST08)),

            0xd0 => Some(Instruction::RET(JumpTest::NotCarry)),
            0xd1 => Some(Instruction::POP(StackTarget::DE)),
            0xd2 => Some(Instruction::JP(JumpTest::NotCarry, WordSource::D16)),
            0xd3 => None,
            0xd4 => Some(Instruction::CALL(JumpTest::NotCarry)),
            0xd5 => Some(Instruction::PUSH(StackTarget::DE)),
            0xd6 => Some(Instruction::SUB(ByteSource::D8)),
            0xd7 => Some(Instruction::RST(ResetCode::RST10)),
            0xd8 => Some(Instruction::RET(JumpTest::Carry)),
            0xd9 => Some(Instruction::RETI),
            0xda => Some(Instruction::JP(JumpTest::Carry, WordSource::D16)),
            0xdb => None,
            0xdc => Some(Instruction::CALL(JumpTest::Carry)),
            0xdd => None,
            0xde => Some(Instruction::SBC(ByteSource::D8)),
            0xdf => Some(Instruction::RST(ResetCode::RST18)),

            0xe0 => Some(Instruction::LD(Load::IndirectFrom(
                LoadByteTarget::D8IFF00,
                ByteSource::A,
            ))),
            0xe1 => Some(Instruction::POP(StackTarget::HL)),
            0xe2 => Some(Instruction::LD(Load::IndirectFrom(
                LoadByteTarget::CIFF00,
                ByteSource::A,
            ))),
            0xe3 => None,
            0xe4 => None,
            0xe5 => Some(Instruction::PUSH(StackTarget::HL)),
            0xe6 => Some(Instruction::AND(ByteSource::D8)),
            0xe7 => Some(Instruction::RST(ResetCode::RST20)),
            0xe8 => Some(Instruction::ADDSP),
            0xe9 => Some(Instruction::JP(JumpTest::Always, WordSource::HL)),
            0xea => Some(Instruction::LD(Load::IndirectFrom(
                LoadByteTarget::D16I,
                ByteSource::A,
            ))),
            0xeb => None,
            0xec => None,
            0xed => None,
            0xee => Some(Instruction::XOR(ByteSource::D8)),
            0xef => Some(Instruction::RST(ResetCode::RST28)),

            0xf0 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::D8IFF00,
            ))),
            0xf1 => Some(Instruction::POP(StackTarget::AF)),
            0xf2 => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::CIFF00,
            ))),
            0xf3 => Some(Instruction::DI),
            0xf4 => None,
            0xf5 => Some(Instruction::PUSH(StackTarget::AF)),
            0xf6 => Some(Instruction::OR(ByteSource::D8)),
            0xf7 => Some(Instruction::RST(ResetCode::RST30)),
            0xf8 => Some(Instruction::LD(Load::IndirectFromSPi8(LoadWordTarget::HL))),
            0xf9 => Some(Instruction::LD(Load::Word(
                LoadWordTarget::SP,
                WordSource::HL,
            ))),
            0xfa => Some(Instruction::LD(Load::Byte(
                LoadByteTarget::A,
                ByteSource::D16I,
            ))),
            0xfb => Some(Instruction::EI),
            0xfc => None,
            0xfd => None,
            0xfe => Some(Instruction::CP(ByteSource::D8)),
            0xff => Some(Instruction::RST(ResetCode::RST38)),
        }
    }
}

#[derive(Debug)]
pub enum IncDecByteTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HLI,
}

impl IncDecByteTarget {
    /// Resolves the referring value
    pub fn read<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T) -> u8 {
        match *self {
            IncDecByteTarget::A => cpu.r.a,
            IncDecByteTarget::B => cpu.r.b,
            IncDecByteTarget::C => cpu.r.c,
            IncDecByteTarget::D => cpu.r.d,
            IncDecByteTarget::E => cpu.r.e,
            IncDecByteTarget::H => cpu.r.h,
            IncDecByteTarget::L => cpu.r.l,
            IncDecByteTarget::HLI => bus.read(cpu.r.get_hl()),
        }
    }

    /// Writes to the referring register or memory location
    pub fn write<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T, value: u8) {
        match *self {
            IncDecByteTarget::A => cpu.r.a = value,
            IncDecByteTarget::B => cpu.r.b = value,
            IncDecByteTarget::C => cpu.r.c = value,
            IncDecByteTarget::D => cpu.r.d = value,
            IncDecByteTarget::E => cpu.r.e = value,
            IncDecByteTarget::H => cpu.r.h = value,
            IncDecByteTarget::L => cpu.r.l = value,
            IncDecByteTarget::HLI => bus.write(cpu.r.get_hl(), value),
        }
    }
}

#[derive(Debug)]
pub enum IncDecWordTarget {
    BC,
    DE,
    HL,
    SP,
}

impl IncDecWordTarget {
    /// Resolves the referring value
    pub fn read(&self, cpu: &mut CPU) -> u16 {
        match *self {
            IncDecWordTarget::BC => cpu.r.get_bc(),
            IncDecWordTarget::DE => cpu.r.get_de(),
            IncDecWordTarget::HL => cpu.r.get_hl(),
            IncDecWordTarget::SP => cpu.sp,
        }
    }

    /// Writes to the referring register
    pub fn write(&self, cpu: &mut CPU, value: u16) {
        match *self {
            IncDecWordTarget::BC => cpu.r.set_bc(value),
            IncDecWordTarget::DE => cpu.r.set_de(value),
            IncDecWordTarget::HL => cpu.r.set_hl(value),
            IncDecWordTarget::SP => cpu.sp = value,
        }
    }
}

#[derive(Eq, PartialEq, Debug)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpTest {
    /// Resolves the referring value
    pub fn resolve(&self, cpu: &mut CPU) -> bool {
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
    BCI,     // value refers to address stored in BC register
    DEI,     // value refers to address stored in DE register
    HLI,     // value refers to address stored in HL register
    D16I,    // value refers to address stored in next 16 bits
    CIFF00,  // value refers to C register | 0xFF00
    D8IFF00, // value refers to address stored in next 8 bits | 0xFF00
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
    CIFF00,  // value refers to C register | 0xFF00
    D8IFF00, // value refers to address stored in next 8 bits | 0xFF00
}

impl ByteSource {
    /// Resolves the referring value
    pub fn read<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T) -> u8 {
        match *self {
            ByteSource::A => cpu.r.a,
            ByteSource::B => cpu.r.b,
            ByteSource::C => cpu.r.c,
            ByteSource::D => cpu.r.d,
            ByteSource::E => cpu.r.e,
            ByteSource::H => cpu.r.h,
            ByteSource::L => cpu.r.l,
            ByteSource::D8 => cpu.consume_byte(bus),
            ByteSource::BCI => bus.read(cpu.r.get_bc()),
            ByteSource::DEI => bus.read(cpu.r.get_de()),
            ByteSource::HLI => bus.read(cpu.r.get_hl()),
            ByteSource::D16I => {
                let address = cpu.consume_word(bus);
                bus.read(address)
            }
            ByteSource::CIFF00 => bus.read(u16::from(cpu.r.c) | 0xFF00),
            ByteSource::D8IFF00 => {
                let address = u16::from(cpu.consume_byte(bus)) | 0xFF00;
                bus.read(address)
            }
        }
    }

    /// Writes value to a register or an address referred by a register
    pub fn write<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T, value: u8) {
        match *self {
            ByteSource::A => cpu.r.a = value,
            ByteSource::B => cpu.r.b = value,
            ByteSource::C => cpu.r.c = value,
            ByteSource::D => cpu.r.d = value,
            ByteSource::E => cpu.r.e = value,
            ByteSource::H => cpu.r.h = value,
            ByteSource::L => cpu.r.l = value,
            ByteSource::BCI => bus.write(cpu.r.get_bc(), value),
            ByteSource::DEI => bus.write(cpu.r.get_de(), value),
            ByteSource::HLI => bus.write(cpu.r.get_hl(), value),
            _ => unimplemented!(),
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
    pub fn read<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T) -> u16 {
        match *self {
            WordSource::BC => cpu.r.get_bc(),
            WordSource::DE => cpu.r.get_de(),
            WordSource::HL => cpu.r.get_hl(),
            WordSource::SP => cpu.sp,
            WordSource::D16 => cpu.consume_word(bus),
        }
    }
}

#[derive(Debug)]
pub enum Load {
    Byte(LoadByteTarget, ByteSource),
    Word(LoadWordTarget, WordSource), // just like the Byte type except with 16-bit values
    IndirectFrom(LoadByteTarget, ByteSource), // load a memory location whose address is stored in AddressSource with the contents of the register ByteSource
    IndirectFromAInc(LoadByteTarget),
    IndirectFromADec(LoadByteTarget), // Same as IndirectFromA, source value is decremented afterwards
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
    RST10,
    RST18,
    RST20,
    RST28,
    RST30,
    RST38,
}
