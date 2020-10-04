use crate::gb::instruction::{
    ByteSource, IncDecByteTarget, IncDecWordTarget, Instruction, JumpTest, Load, LoadByteTarget,
    LoadWordTarget, ResetCode, StackTarget, WordSource,
};
use std::fmt;

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Instruction::ADD(source) => write!(f, "ADD A,{}", source),
            Instruction::ADDHL(source) => write!(f, "ADD HL,{}", source),
            Instruction::ADDSP => write!(f, "ADD SP,i8"),
            Instruction::ADC(source) => write!(f, "ADC A,{}", source),
            Instruction::AND(source) => write!(f, "AND A,{}", source),
            Instruction::BIT(bit, source) => write!(f, "BIT {},{}", bit, source),
            Instruction::INC(target) => write!(f, "INC {}", target),
            Instruction::INC2(target) => write!(f, "INC {}", target),
            Instruction::CALL(test) => write!(f, "CALL {}u16", test),
            Instruction::CCF => write!(f, "CCF"),
            Instruction::CP(source) => write!(f, "CP A,{}", source),
            Instruction::CPL => write!(f, "CPL"),
            Instruction::DAA => write!(f, "DAA"),
            Instruction::DI => write!(f, "DI"),
            Instruction::DEC(target) => write!(f, "DEC {}", target),
            Instruction::DEC2(target) => write!(f, "DEC {}", target),
            Instruction::EI => write!(f, "EI"),
            Instruction::HALT => write!(f, "HALT"),
            Instruction::JR(test) => write!(f, "JR {}i8", test),
            Instruction::JP(test, source) => write!(f, "JR {}{}", test, source),
            Instruction::LD(Load::IndirectFromAInc(LoadByteTarget::HLI)) => write!(f, "(HL+),A"),
            Instruction::LD(Load::IndirectFromADec(LoadByteTarget::HLI)) => write!(f, "(HL-),A"),
            Instruction::LD(Load::FromIndirectAInc(ByteSource::HLI)) => write!(f, "A,(HL+)",),
            Instruction::LD(Load::FromIndirectADec(ByteSource::HLI)) => write!(f, "A,(HL-)",),
            Instruction::LD(load) => write!(f, "LD {}", load),
            Instruction::NOP => write!(f, "NOP"),
            Instruction::OR(source) => write!(f, "OR A,{}", source),
            Instruction::PUSH(target) => write!(f, "PUSH {}", target),
            Instruction::POP(target) => write!(f, "POP {}", target),
            Instruction::RES(bit, source) => write!(f, "RES {},{}", bit, source),
            Instruction::RET(test) => write!(f, "RET {}", test),
            Instruction::RETI => write!(f, "RETI"),
            Instruction::RL(source) => write!(f, "RL {}", source),
            Instruction::RLA => write!(f, "RLA"),
            Instruction::RLC(source) => write!(f, "RL {}", source),
            Instruction::RLCA => write!(f, "RLCA"),
            Instruction::RR(source) => write!(f, "RR {}", source),
            Instruction::RRA => write!(f, "RRA"),
            Instruction::RRC(source) => write!(f, "RRC {}", source),
            Instruction::RRCA => write!(f, "RRCA"),
            Instruction::RST(code) => write!(f, "RST {}", code),
            Instruction::SBC(source) => write!(f, "SBC A,{}", source),
            Instruction::SCF => write!(f, "SCF"),
            Instruction::SET(bit, source) => write!(f, "SET {},{}", bit, source),
            Instruction::SLA(source) => write!(f, "SLA {}", source),
            Instruction::SRA(source) => write!(f, "SRA {}", source),
            Instruction::SRL(source) => write!(f, "SRL {}", source),
            Instruction::SUB(source) => write!(f, "SUB A,{}", source),
            Instruction::STOP => write!(f, "STOP"),
            Instruction::SWAP(source) => write!(f, "SWAP {}", source),
            Instruction::XOR(source) => write!(f, "XOR A,{}", source),
        }
    }
}

impl fmt::Display for ByteSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ByteSource::A => write!(f, "A"),
            ByteSource::B => write!(f, "B"),
            ByteSource::C => write!(f, "C"),
            ByteSource::D => write!(f, "D"),
            ByteSource::E => write!(f, "E"),
            ByteSource::H => write!(f, "H"),
            ByteSource::L => write!(f, "L"),
            ByteSource::D8 => write!(f, "u8"),
            ByteSource::BCI => write!(f, "(BC)"),
            ByteSource::DEI => write!(f, "(DE)"),
            ByteSource::HLI => write!(f, "(HL)"),
            ByteSource::D16I => write!(f, "(u16)"),
            ByteSource::CIFF00 => write!(f, "(FF00+C)"),
            ByteSource::D8IFF00 => write!(f, "(FF00+u8)"),
        }
    }
}

impl fmt::Display for WordSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WordSource::BC => write!(f, "BC"),
            WordSource::DE => write!(f, "DE"),
            WordSource::HL => write!(f, "HL"),
            WordSource::SP => write!(f, "SP"),
            WordSource::D16 => write!(f, "(u16)"),
        }
    }
}

impl fmt::Display for IncDecByteTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncDecByteTarget::A => write!(f, "A"),
            IncDecByteTarget::B => write!(f, "B"),
            IncDecByteTarget::C => write!(f, "C"),
            IncDecByteTarget::D => write!(f, "D"),
            IncDecByteTarget::E => write!(f, "E"),
            IncDecByteTarget::H => write!(f, "H"),
            IncDecByteTarget::L => write!(f, "L"),
            IncDecByteTarget::HLI => write!(f, "(HL)"),
        }
    }
}

impl fmt::Display for IncDecWordTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IncDecWordTarget::BC => write!(f, "BC"),
            IncDecWordTarget::DE => write!(f, "DE"),
            IncDecWordTarget::HL => write!(f, "HL"),
            IncDecWordTarget::SP => write!(f, "SP"),
        }
    }
}

impl fmt::Display for JumpTest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JumpTest::NotZero => write!(f, "NZ,"),
            JumpTest::Zero => write!(f, "Z,"),
            JumpTest::NotCarry => write!(f, "NC,"),
            JumpTest::Carry => write!(f, "C,"),
            JumpTest::Always => write!(f, ""),
        }
    }
}

impl fmt::Display for Load {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Load::Byte(target, source) => write!(f, "{},{}", target, source),
            Load::Word(target, source) => write!(f, "{},{}", target, source),
            Load::IndirectFrom(target, source) => write!(f, "{},{}", target, source),
            Load::IndirectFromWord(target, source) => write!(f, "{},{}", target, source),
            Load::IndirectFromSPi8(target) => write!(f, "{},SP+i8", target),
            _ => panic!(),
        }
    }
}

impl fmt::Display for LoadByteTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadByteTarget::A => write!(f, "A"),
            LoadByteTarget::B => write!(f, "B"),
            LoadByteTarget::C => write!(f, "C"),
            LoadByteTarget::D => write!(f, "D"),
            LoadByteTarget::E => write!(f, "E"),
            LoadByteTarget::H => write!(f, "H"),
            LoadByteTarget::L => write!(f, "L"),
            LoadByteTarget::BCI => write!(f, "(BC)"),
            LoadByteTarget::DEI => write!(f, "(DE)"),
            LoadByteTarget::HLI => write!(f, "(HL)"),
            LoadByteTarget::D16I => write!(f, "(u16)"),
            LoadByteTarget::CIFF00 => write!(f, "(FF00+C)"),
            LoadByteTarget::D8IFF00 => write!(f, "(FF00+u8)"),
        }
    }
}

impl fmt::Display for LoadWordTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoadWordTarget::BC => write!(f, "BC"),
            LoadWordTarget::DE => write!(f, "DE"),
            LoadWordTarget::HL => write!(f, "HL"),
            LoadWordTarget::SP => write!(f, "SP"),
            LoadWordTarget::D16I => write!(f, "(u16)"),
        }
    }
}

impl fmt::Display for StackTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StackTarget::AF => write!(f, "AF"),
            StackTarget::BC => write!(f, "BC"),
            StackTarget::DE => write!(f, "DE"),
            StackTarget::HL => write!(f, "HL"),
        }
    }
}

impl fmt::Display for ResetCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResetCode::RST00 => write!(f, "00h"),
            ResetCode::RST08 => write!(f, "08h"),
            ResetCode::RST10 => write!(f, "10h"),
            ResetCode::RST18 => write!(f, "18h"),
            ResetCode::RST20 => write!(f, "20h"),
            ResetCode::RST28 => write!(f, "28h"),
            ResetCode::RST30 => write!(f, "30h"),
            ResetCode::RST38 => write!(f, "38h"),
        }
    }
}
