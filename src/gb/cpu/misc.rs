use crate::gb::AddressSpace;
use crate::gb::cpu::CPU;
use crate::gb::cpu::registers::FlagsRegister;
use std::fmt;
use std::fmt::Formatter;

/// Defines the 8-bit registers of the CPU
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
    /// Reads the referring value from the register
    #[inline]
    pub fn read(&self, cpu: &mut CPU) -> u8 {
        match *self {
            Register::A => cpu.r.a,
            Register::B => cpu.r.b,
            Register::C => cpu.r.c,
            Register::D => cpu.r.d,
            Register::E => cpu.r.e,
            Register::H => cpu.r.h,
            Register::L => cpu.r.l,
        }
    }

    /// Writes to the referring register
    #[inline]
    pub fn write(&self, cpu: &mut CPU, value: u8) {
        match *self {
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
        write!(f, "{ident}")
    }
}

/// Defines paired registers that can be used in instructions to read and write from.
#[derive(Clone, Copy)]
pub enum PairedRegister {
    AF,
    BC,
    DE,
    HL,
    SP,
}

impl PairedRegister {
    /// Read from register
    #[inline]
    pub fn read(&self, cpu: &mut CPU) -> u16 {
        match *self {
            PairedRegister::AF => cpu.r.get_af(),
            PairedRegister::BC => cpu.r.get_bc(),
            PairedRegister::DE => cpu.r.get_de(),
            PairedRegister::HL => cpu.r.get_hl(),
            PairedRegister::SP => cpu.sp,
        }
    }

    /// Write value to register
    #[inline]
    pub fn write(&self, cpu: &mut CPU, value: u16) {
        match *self {
            PairedRegister::AF => cpu.r.set_af(value),
            PairedRegister::BC => cpu.r.set_bc(value),
            PairedRegister::DE => cpu.r.set_de(value),
            PairedRegister::HL => cpu.r.set_hl(value),
            PairedRegister::SP => cpu.sp = value,
        }
    }
}

impl fmt::Display for PairedRegister {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match *self {
            PairedRegister::AF => "AF",
            PairedRegister::BC => "BC",
            PairedRegister::DE => "DE",
            PairedRegister::HL => "HL",
            PairedRegister::SP => "SP",
        };
        write!(f, "{ident}")
    }
}

#[derive(Copy, Clone)]
pub enum ByteTarget {
    R(Register),
    HLI,
}

impl ByteTarget {
    /// Reads the referring value from the CPU or memory
    #[inline]
    pub fn read<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T) -> u8 {
        match *self {
            ByteTarget::R(reg) => reg.read(cpu),
            ByteTarget::HLI => bus.read(cpu.r.get_hl()),
        }
    }

    /// Writes to the referring register or memory location
    #[inline]
    pub fn write<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T, value: u8) {
        match *self {
            ByteTarget::R(reg) => reg.write(cpu, value),
            ByteTarget::HLI => bus.write(cpu.r.get_hl(), value),
        }
    }
}

impl fmt::Display for ByteTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteTarget::R(reg) => format!("{reg}"),
            ByteTarget::HLI => "(HL)".into(),
        };
        write!(f, "{ident}")
    }
}

#[derive(Copy, Clone)]
pub enum WordTarget {
    R(PairedRegister),
    D16I(u16), // value refers to memory at address from the next 16 bits
}

impl WordTarget {
    /// Writes to the referring register
    #[inline]
    pub fn write<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T, value: u16) {
        match *self {
            WordTarget::R(reg) => reg.write(cpu, value),
            WordTarget::D16I(address) => {
                bus.write(address, value as u8);
                bus.write(address + 1, (value >> 8) as u8)
            }
        }
    }
}

impl fmt::Display for WordTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            WordTarget::R(reg) => format!("{reg}"),
            WordTarget::D16I(address) => format!("({address:#06x})"),
        };
        write!(f, "{ident}")
    }
}

/// Defines a source we can read from to get a byte value
#[derive(Copy, Clone)]
pub enum IndirectByteRef {
    BCI,       // value refers to memory at address from BC register
    DEI,       // value refers to memory at address from DE register
    HLI,       // value refers to memory at address from HL register
    D16I(u16), // value refers to memory at address from the next 16 bits
    CI,        // value refers to memory at address from C register | 0xFF00
    D8I(u8),   // value refers to memory at the address from the next 8 bits | 0xFF00
}

impl IndirectByteRef {
    /// Resolves and returns the referring address.
    #[inline]
    pub fn resolve(&self, cpu: &mut CPU) -> u16 {
        match *self {
            IndirectByteRef::BCI => cpu.r.get_bc(),
            IndirectByteRef::DEI => cpu.r.get_de(),
            IndirectByteRef::D16I(address) => address,
            IndirectByteRef::HLI => cpu.r.get_hl(),
            IndirectByteRef::CI => u16::from(cpu.r.c) | 0xFF00,
            IndirectByteRef::D8I(offset) => u16::from(offset) | 0xFF00,
        }
    }
}

impl fmt::Display for IndirectByteRef {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match *self {
            IndirectByteRef::BCI => "(BC)".into(),
            IndirectByteRef::DEI => "(DE)".into(),
            IndirectByteRef::HLI => "(HL)".into(),
            IndirectByteRef::D16I(address) => format!("({address:#06x})"),
            IndirectByteRef::CI => "(C)".into(),
            IndirectByteRef::D8I(offset) => format!("({:#06x})", u16::from(offset) | 0xFF00),
        };
        write!(f, "{ident}")
    }
}

/// Defines a source we can read from to get a byte value.
#[derive(Copy, Clone)]
pub enum ByteSource {
    R(Register),
    D8(u8),    // value comes from the next 8 bits
    BCI,       // value refers to memory at address from BC register
    DEI,       // value refers to memory at address from DE register
    HLI,       // value refers to memory at address from HL register
    D16I(u16), // value refers to memory at address from the next 16 bits
    CI,        // value refers to memory at address from C register | 0xFF00
    D8I(u8),   // value refers to memory at the address from the next 8 bits | 0xFF00
}

impl ByteSource {
    /// Read byte from the CPU or memory.
    pub fn read<T: AddressSpace>(&self, cpu: &mut CPU, bus: &mut T) -> u8 {
        match *self {
            ByteSource::R(reg) => reg.read(cpu),
            ByteSource::D8(value) => value,
            ByteSource::BCI => bus.read(cpu.r.get_bc()),
            ByteSource::DEI => bus.read(cpu.r.get_de()),
            ByteSource::HLI => bus.read(cpu.r.get_hl()),
            ByteSource::D16I(address) => bus.read(address),
            ByteSource::CI => bus.read(u16::from(cpu.r.c) | 0xFF00),
            ByteSource::D8I(offset) => bus.read(u16::from(offset) | 0xFF00),
        }
    }
}

impl fmt::Display for ByteSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteSource::R(reg) => format!("{reg}"),
            ByteSource::D8(value) => format!("{value:#04x}"),
            ByteSource::BCI => "(BC)".into(),
            ByteSource::DEI => "(DE)".into(),
            ByteSource::HLI => "(HL)".into(),
            ByteSource::D16I(address) => format!("({address:#06x})"),
            ByteSource::CI => "(C)".into(),
            ByteSource::D8I(offset) => format!("({:#06x})", u16::from(*offset) | 0xFF00),
        };
        write!(f, "{ident}")
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
        match *self {
            JumpTarget::D16(word) => word,
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

/// Defines the source of a word value
#[derive(Copy, Clone)]
pub enum WordSource {
    R(PairedRegister),
    D16(u16), // value comes from the next 16 bits
}

impl WordSource {
    /// Resolves the referring value
    #[inline]
    pub fn read(&self, cpu: &mut CPU) -> u16 {
        match *self {
            WordSource::R(reg) => reg.read(cpu),
            WordSource::D16(word) => word,
        }
    }
}

impl fmt::Display for WordSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            WordSource::R(reg) => format!("{reg}"),
            WordSource::D16(word) => format!("{word:#06x}"),
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
    pub fn resolve(&self, cpu: &mut CPU) -> bool {
        match *self {
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
        write!(f, "{ident}")
    }
}

/// Defines the possible load operations
#[derive(Copy, Clone)]
pub enum Load {
    Byte(ByteTarget, ByteSource),
    Word(WordTarget, WordSource),
    IndirectFrom(IndirectByteRef, ByteSource), // load a memory location whose address is stored in AddressSource with the contents of the register ByteSource
    IndirectFromAInc(IndirectByteRef),
    IndirectFromADec(IndirectByteRef), // Same as IndirectFromA, source value is decremented afterward
    FromIndirectAInc(ByteSource),
    FromIndirectADec(ByteSource),
    IndirectFromWord(WordTarget, WordSource),
    IndirectFromSPi8(WordTarget, i8), // Put SP plus 8 bit immediate value into target.
}

impl fmt::Display for Load {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // TODO: fix formatting for indirect inc, dec loads
        let ident = match self {
            Load::Byte(target, source) => format!("{target}, {source}"),
            Load::Word(target, source) => format!("{target}, {source}"),
            Load::IndirectFrom(target, source) => format!("{target}, {source}"),
            Load::IndirectFromAInc(target) => format!("{target}+, A",),
            Load::IndirectFromADec(target) => format!("{target}-, A",),
            Load::FromIndirectAInc(source) => format!("A, {source}+",),
            Load::FromIndirectADec(source) => format!("A, {source}-"),
            Load::IndirectFromWord(target, source) => format!("{target}, {source}"),
            Load::IndirectFromSPi8(target, value) => format!("{target}, SP + {value:#04x}"),
        };
        write!(f, "{ident}")
    }
}
