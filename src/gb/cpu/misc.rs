use crate::gb::AddressSpace;
use crate::gb::cpu::CPU;
use crate::gb::cpu::registers::FlagsRegister;

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

pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
    D16I, // value refers to address stored in next 16 bits
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

pub enum StackTarget {
    AF,
    BC,
    DE,
    HL,
}

#[repr(u16)]
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
