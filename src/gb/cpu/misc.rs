use crate::gb::AddressSpace;
use crate::gb::cpu::CPU;
use crate::gb::cpu::registers::FlagsRegister;
use std::fmt;
use std::fmt::Formatter;

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

impl fmt::Display for IncDecByteTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            IncDecByteTarget::A => "A",
            IncDecByteTarget::B => "B",
            IncDecByteTarget::C => "C",
            IncDecByteTarget::D => "D",
            IncDecByteTarget::E => "E",
            IncDecByteTarget::H => "H",
            IncDecByteTarget::L => "L",
            IncDecByteTarget::HLI => "HLI",
        };
        write!(f, "{ident}")
    }
}

pub enum IncDecWordTarget {
    BC,
    DE,
    HL,
    SP,
}

impl IncDecWordTarget {
    /// Resolves the referring value
    #[inline]
    pub fn read(&self, cpu: &mut CPU) -> u16 {
        match *self {
            IncDecWordTarget::BC => cpu.r.get_bc(),
            IncDecWordTarget::DE => cpu.r.get_de(),
            IncDecWordTarget::HL => cpu.r.get_hl(),
            IncDecWordTarget::SP => cpu.sp,
        }
    }

    /// Writes to the referring register
    #[inline]
    pub fn write(&self, cpu: &mut CPU, value: u16) {
        match *self {
            IncDecWordTarget::BC => cpu.r.set_bc(value),
            IncDecWordTarget::DE => cpu.r.set_de(value),
            IncDecWordTarget::HL => cpu.r.set_hl(value),
            IncDecWordTarget::SP => cpu.sp = value,
        }
    }
}

impl fmt::Display for IncDecWordTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            IncDecWordTarget::BC => "BC",
            IncDecWordTarget::DE => "DE",
            IncDecWordTarget::HL => "HL",
            IncDecWordTarget::SP => "SP",
        };
        write!(f, "{ident}")
    }
}

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

impl fmt::Display for LoadByteTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match *self {
            LoadByteTarget::A => "A",
            LoadByteTarget::B => "B",
            LoadByteTarget::C => "C",
            LoadByteTarget::D => "D",
            LoadByteTarget::E => "E",
            LoadByteTarget::H => "H",
            LoadByteTarget::L => "L",
            LoadByteTarget::BCI => "BCI",
            LoadByteTarget::DEI => "DEI",
            LoadByteTarget::HLI => "HLI",
            LoadByteTarget::D16I => "D16I",
            LoadByteTarget::CIFF00 => "CIFF00",
            LoadByteTarget::D8IFF00 => "D8IFF00",
        };
        write!(f, "{ident}")
    }
}

#[derive(PartialEq)]
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
                let offset = cpu.consume_byte(bus);
                bus.read(u16::from(offset) | 0xFF00)
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
            _ => unreachable!(),
        }
    }
}

impl fmt::Display for ByteSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            ByteSource::A => "A",
            ByteSource::B => "B",
            ByteSource::C => "C",
            ByteSource::D => "D",
            ByteSource::E => "E",
            ByteSource::H => "H",
            ByteSource::L => "L",
            ByteSource::D8 => "D8",
            ByteSource::BCI => "BCI",
            ByteSource::DEI => "DEI",
            ByteSource::HLI => "HLI",
            ByteSource::D16I => "D16I",
            ByteSource::CIFF00 => "CIFF00",
            ByteSource::D8IFF00 => "D8IFF00",
        };
        write!(f, "{ident}")
    }
}

pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    D16I, // value refers to address stored in next 16 bits
}

impl fmt::Display for LoadWordTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            LoadWordTarget::BC => "BC",
            LoadWordTarget::DE => "DE",
            LoadWordTarget::HL => "HL",
            LoadWordTarget::SP => "SP",
            LoadWordTarget::D16I => "D16I",
        };
        write!(f, "{ident}")
    }
}

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

impl fmt::Display for WordSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            WordSource::BC => "BC",
            WordSource::DE => "DE",
            WordSource::HL => "HL",
            WordSource::SP => "SP",
            WordSource::D16 => "D16",
        };
        write!(f, "{ident}")
    }
}

pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

impl fmt::Display for StackTarget {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            StackTarget::AF => "AF",
            StackTarget::BC => "BC",
            StackTarget::DE => "DE",
            StackTarget::HL => "HL",
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
        write!(f, "{:02X}", *self as u16)
    }
}

#[derive(Eq, PartialEq)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

impl JumpTest {
    /// Resolves the referring value
    #[inline]
    pub fn resolve(&self, cpu: &mut CPU) -> bool {
        match *self {
            JumpTest::NotZero => !cpu.r.f.contains(FlagsRegister::ZERO),
            JumpTest::Zero => cpu.r.f.contains(FlagsRegister::ZERO),
            JumpTest::NotCarry => !cpu.r.f.contains(FlagsRegister::CARRY),
            JumpTest::Carry => cpu.r.f.contains(FlagsRegister::CARRY),
            JumpTest::Always => true,
        }
    }
}

impl fmt::Display for JumpTest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            JumpTest::NotZero => "NZ",
            JumpTest::Zero => "Z",
            JumpTest::NotCarry => "NC",
            JumpTest::Carry => "C",
            JumpTest::Always => "AL",
        };
        write!(f, "{ident}")
    }
}

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

impl fmt::Display for Load {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let ident = match self {
            Load::Byte(target, source) => format!("{} {}", target, source),
            Load::Word(target, source) => format!("{} {}", target, source),
            Load::IndirectFrom(target, source) => format!("{} {}", target, source),
            Load::IndirectFromAInc(target) => format!("{} A+", target),
            Load::IndirectFromADec(target) => format!("{} A-", target),
            Load::FromIndirectAInc(source) => format!("A+ {}", source),
            Load::FromIndirectADec(source) => format!("A- {}", source),
            Load::IndirectFromWord(target, source) => format!("{} {}", target, source),
            Load::IndirectFromSPi8(target) => format!("{} SP+i8", target),
        };
        write!(f, "{ident}")
    }
}
