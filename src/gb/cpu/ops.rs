use crate::gb::AddressSpace;
use crate::gb::cpu::CPU;
use crate::gb::cpu::instruction::ReallySigned;
use crate::gb::cpu::registers::FlagsRegister;
use std::fmt;
use std::fmt::Formatter;

/// Defines an operation on the 8-bit registers of the CPU.
#[derive(Copy, Clone)]
pub enum Register {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

impl Register {
    /// Reads value from the register.
    #[inline]
    pub fn read(&self, cpu: &CPU) -> u8 {
        match self {
            Register::A => cpu.r.a,
            Register::B => cpu.r.b,
            Register::C => cpu.r.c,
            Register::D => cpu.r.d,
            Register::E => cpu.r.e,
            Register::H => cpu.r.h,
            Register::L => cpu.r.l,
        }
    }

    /// Writes value to the register.
    #[inline]
    pub fn write(&self, cpu: &mut CPU, value: u8) {
        match self {
            Register::A => cpu.r.a = value,
            Register::B => cpu.r.b = value,
            Register::C => cpu.r.c = value,
            Register::D => cpu.r.d = value,
            Register::E => cpu.r.e = value,
            Register::H => cpu.r.h = value,
            Register::L => cpu.r.l = value,
        }
    }
}

impl fmt::Display for Register {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            Register::A => "A",
            Register::B => "B",
            Register::C => "C",
            Register::D => "D",
            Register::E => "E",
            Register::H => "H",
            Register::L => "L",
        };
        f.write_str(ident)
    }
}

/// Defines an operation on word registers of the CPU.
#[derive(Clone, Copy, PartialEq)]
pub enum WordRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl WordRegister {
    /// Read from register.
    #[inline]
    pub fn read(&self, cpu: &CPU) -> u16 {
        match self {
            WordRegister::AF => cpu.r.get_af(),
            WordRegister::BC => cpu.r.get_bc(),
            WordRegister::DE => cpu.r.get_de(),
            WordRegister::HL => cpu.r.get_hl(),
            WordRegister::SP => cpu.sp,
        }
    }

    /// Write value to register
    #[inline]
    pub fn write(&self, cpu: &mut CPU, value: u16) {
        match self {
            WordRegister::AF => cpu.r.set_af(value),
            WordRegister::BC => cpu.r.set_bc(value),
            WordRegister::DE => cpu.r.set_de(value),
            WordRegister::HL => cpu.r.set_hl(value),
            WordRegister::SP => cpu.sp = value,
        }
    }
}

impl fmt::Display for WordRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            WordRegister::AF => "AF",
            WordRegister::BC => "BC",
            WordRegister::DE => "DE",
            WordRegister::HL => "HL",
            WordRegister::SP => "SP",
        };
        f.write_str(ident)
    }
}

#[derive(Copy, Clone)]
pub enum ByteTarget {
    R(Register),
    I(ByteRef),
}

impl ByteTarget {
    /// Reads the referring value from the CPU or memory
    #[inline]
    pub fn read<T>(&self, cpu: &CPU, bus: &mut T) -> u8
    where
        T: AddressSpace,
    {
        match self {
            ByteTarget::R(reg) => reg.read(cpu),
            ByteTarget::I(indirect) => bus.read(indirect.resolve(cpu)),
        }
    }

    /// Writes to the referring register or memory location
    #[inline]
    pub fn write<T>(&self, cpu: &mut CPU, bus: &mut T, value: u8)
    where
        T: AddressSpace,
    {
        match self {
            ByteTarget::R(reg) => reg.write(cpu, value),
            ByteTarget::I(indirect) => bus.write(indirect.resolve(cpu), value),
        }
    }
}

impl fmt::Display for ByteTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteTarget::R(reg) => format!("{reg}"),
            ByteTarget::I(indirect) => format!("{indirect}"),
        };
        f.write_str(&ident)
    }
}

/// Defines a source which yields an address that can be used to read or write a byte value
#[derive(Copy, Clone)]
pub enum ByteRef {
    R(WordRegister), // value refers to memory at address from one of the paired registers
    D16(u16),        // value refers to memory at address from the next 16 bits
    C,               // value refers to memory at address from C register | 0xFF00
    D8(u8),          // value refers to memory at the address from the next 8 bits | 0xFF00
}

impl ByteRef {
    /// Resolves and returns the referring address.
    #[inline]
    pub fn resolve(&self, cpu: &CPU) -> u16 {
        match self {
            ByteRef::R(reg) => reg.read(cpu),
            ByteRef::D16(address) => *address,
            ByteRef::C => u16::from(cpu.r.c) | 0xFF00,
            ByteRef::D8(offset) => u16::from(*offset) | 0xFF00,
        }
    }
}

impl fmt::Display for ByteRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteRef::R(reg) => format!("({reg})"),
            ByteRef::D16(address) => format!("({address:#06x})"),
            ByteRef::C => "(C)".into(),
            ByteRef::D8(offset) => format!("({:#06x})", u16::from(*offset) | 0xFF00),
        };
        f.write_str(&ident)
    }
}

/// Defines a source we can read from to get a byte value.
#[derive(Copy, Clone)]
pub enum ByteSource {
    R(Register),
    I(ByteRef),
    D8(u8), // value comes from the next 8 bits
}

impl ByteSource {
    /// Read byte from the CPU or memory.
    pub fn read<T>(&self, cpu: &CPU, bus: &mut T) -> u8
    where
        T: AddressSpace,
    {
        match self {
            ByteSource::R(reg) => reg.read(cpu),
            ByteSource::D8(value) => *value,
            ByteSource::I(indirect) => bus.read(indirect.resolve(cpu)),
        }
    }
}

impl fmt::Display for ByteSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteSource::R(reg) => format!("{reg}"),
            ByteSource::D8(value) => format!("{value:#04x}"),
            ByteSource::I(indirect) => format!("{indirect}"),
        };
        f.write_str(&ident)
    }
}

/// Defines the source of a word value
#[derive(Copy, Clone, PartialEq)]
pub enum WordSource {
    R(WordRegister),
    D16(u16), // value comes from the next 16 bits
}

impl WordSource {
    /// Resolves the referring value
    #[inline]
    pub fn read(&self, cpu: &CPU) -> u16 {
        match self {
            WordSource::R(reg) => reg.read(cpu),
            WordSource::D16(word) => *word,
        }
    }
}

impl fmt::Display for WordSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            WordSource::R(reg) => format!("{reg}"),
            WordSource::D16(word) => format!("{word:#06x}"),
        };
        f.write_str(&ident)
    }
}

/// Defines the possible load operations
#[derive(Copy, Clone)]
pub enum Load {
    Byte(ByteTarget, ByteSource),
    Word(WordRegister, WordSource),
    // Store the contents of `ByteSource` in the memory location specified by `ByteRef`.
    IndirectFrom(ByteRef, ByteSource),
    // Store the contents of register A into the memory location specified by register pair HL,
    // and simultaneously increment the contents of HL.
    HLIFromAInc,
    // Store the contents of register A into the memory location specified by register pair HL,
    // and simultaneously decrement the contents of HL.
    HLIFromADec,
    // Load the contents of memory specified by register pair HL into register A,
    // and simultaneously increment the contents of HL.
    HLIToAInc,
    // Load the contents of memory specified by register pair HL into register A,
    // and simultaneously decrement the contents of HL.
    HLIToADec,
    // Store the lower byte of stack pointer SP at the address specified by the 16-bit immediate
    // operand a16, and store the upper byte of SP at address a16 + 1.
    IndirectFromSP(ByteRef),
    // Add the 8-bit signed operand to the stack pointer SP,
    // and store the result in register pair HL.
    HLFromSPi8(i8),
}

impl fmt::Display for Load {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            Load::Byte(target, source) => format!("{target}, {source}"),
            Load::Word(target, source) => format!("{target}, {source}"),
            Load::IndirectFrom(indirect, source) => format!("{indirect}, {source}"),
            Load::HLIFromAInc => "(HL+), A".into(),
            Load::HLIFromADec => "(HL-), A".into(),
            Load::HLIToAInc => "A, (HL+)".into(),
            Load::HLIToADec => "A, (HL-)".into(),
            Load::IndirectFromSP(target) => format!("{target}, SP"),
            Load::HLFromSPi8(value) => {
                let value = ReallySigned(*value);
                format!("HL, SP{value:+#04x}")
            }
        };
        f.write_str(&ident)
    }
}

/// Possible conditions for conditional instructions like JP, JR, CALL and RET
#[derive(Eq, PartialEq, Copy, Clone)]
pub enum JumpCondition {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpCondition {
    /// Resolves whether the condition is met
    #[inline]
    pub fn resolve(&self, cpu: &CPU) -> bool {
        match self {
            JumpCondition::NotZero => !cpu.r.f.contains(FlagsRegister::ZERO),
            JumpCondition::Zero => cpu.r.f.contains(FlagsRegister::ZERO),
            JumpCondition::NotCarry => !cpu.r.f.contains(FlagsRegister::CARRY),
            JumpCondition::Carry => cpu.r.f.contains(FlagsRegister::CARRY),
            JumpCondition::Always => true,
        }
    }
}

impl fmt::Display for JumpCondition {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            JumpCondition::NotZero => "NZ",
            JumpCondition::Zero => "Z",
            JumpCondition::NotCarry => "NC",
            JumpCondition::Carry => "C",
            JumpCondition::Always => "",
        };
        f.write_str(ident)
    }
}

/// Defines a target address we can jump to
#[derive(Copy, Clone)]
pub enum JumpTarget {
    D16(u16), // value comes from the next 16 bits
    HL,
}

impl JumpTarget {
    /// Resolves and returns the referring target address
    #[inline]
    pub fn read(&self, cpu: &CPU) -> u16 {
        match self {
            JumpTarget::D16(word) => *word,
            JumpTarget::HL => cpu.r.get_hl(),
        }
    }
}

impl fmt::Display for JumpTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            JumpTarget::D16(word) => format!("{word:#06x}"),
            JumpTarget::HL => "HL".to_string(),
        };
        write!(f, "{ident}")
    }
}

#[repr(u16)]
#[derive(Copy, Clone, Debug)]
pub enum ResetCode {
    RST00 = 0x00,
    RST08 = 0x08,
    RST10 = 0x10,
    RST18 = 0x18,
    RST20 = 0x20,
    RST28 = 0x28,
    RST30 = 0x30,
    RST38 = 0x38,
}

impl fmt::Display for ResetCode {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:#04x}", *self as u16)
    }
}
