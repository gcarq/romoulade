use crate::gb::instruction::{
    AddressSource, ArithmeticByteSource, ArithmeticByteTarget, ArithmeticWordSource,
    ArithmeticWordTarget, BitOperationSource, ByteSource, IncDecTarget, Instruction, JumpTest,
    LoadByteTarget, LoadType, LoadWordTarget, PrefixTarget, ResetCode, StackTarget, WordSource,
};
use crate::gb::timings::Clock;
use crate::gb::AddressSpace;
use crate::utils;
use registers::Registers;
use std::cell::RefCell;

mod registers;
#[cfg(test)]
mod tests;

pub struct CPU<'a, T: AddressSpace> {
    r: Registers,
    pub pc: u16,   // Program counter
    sp: u16,       // Stack Pointer
    pub ime: bool, // Interrupt Master Enable
    is_halted: bool,
    pub bus: &'a RefCell<T>, // TODO: make me private
    clock: Clock,
}

impl<'a, T: AddressSpace> CPU<'a, T> {
    pub fn new(bus: &'a RefCell<T>) -> Self {
        Self {
            r: Registers::default(),
            pc: 0,
            sp: 0,
            ime: true,
            is_halted: false,
            bus,
            clock: Clock::new(),
        }
    }

    pub fn step(&mut self) -> u32 {
        self.clock.reset();
        if self.is_halted {
            return self.clock.ticks();
        }

        let mut opcode = self.read(self.pc);
        let prefixed = opcode == 0xCB;
        if prefixed {
            opcode = self.read(self.pc + 1);
        }

        let next_pc = match Instruction::from_byte(opcode, prefixed) {
            Some(instruction) => {
                self.log(opcode, &instruction);
                self.execute(instruction)
            }
            None => {
                self.print_registers(opcode, None);
                let description = format!("0x{}{:02x}", if prefixed { "cb" } else { "" }, opcode);
                panic!("Unresolved instruction found for: {}", description)
            }
        };
        self.pc = next_pc;
        self.clock.ticks()
    }

    /// Prints the current registers, pointers with opcode and resolved instruction
    fn print_registers(&self, opcode: u8, instruction: Option<&Instruction>) {
        println!(
            "{} | pc: {:<5} sp: {:<5} | op: {:#04x} -> {}",
            self.r,
            format!("{:#06x}", self.pc),
            format!("{:#06x}", self.sp),
            opcode,
            match instruction {
                Some(i) => format!("{:?}", i),
                None => "????".to_string(),
            }
        );
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(target, source) => self.handle_add(target, source),
            Instruction::ADD2(target, source) => self.handle_add2(target, source),
            Instruction::ADC(source) => self.handle_adc(source),
            Instruction::AND(source) => self.handle_and(source),
            Instruction::BIT(bit, source) => self.handle_bit(bit, source),
            Instruction::CALL(test) => self.handle_call(test),
            Instruction::CP(source) => self.handle_cp(source),
            Instruction::DAA => self.handle_daa(),
            Instruction::DI => self.handle_interrupt(false),
            Instruction::DEC(target) => self.handle_dec(target),
            Instruction::EI => self.handle_interrupt(true),
            Instruction::HALT => self.handle_halt(),
            Instruction::INC(target) => self.handle_inc(target),
            Instruction::JR(test) => self.handle_jr(test),
            Instruction::JP(test) => self.handle_jp(test),
            Instruction::LD(load_type) => self.handle_ld(load_type),
            Instruction::NOP => self.handle_nop(),
            Instruction::OR(source) => self.handle_or(source),
            Instruction::RET(test) => self.handle_ret(test),
            Instruction::RLA => self.handle_rla(),
            Instruction::RLC(target) => self.handle_rlc(target),
            Instruction::RRCA => self.handle_rrca(),
            Instruction::RST(code) => self.handle_rst(code),
            Instruction::STOP => self.handle_stop(),
            Instruction::SUB(target, source) => self.handle_sub(target, source),
            Instruction::PUSH(target) => self.handle_push(target),
            Instruction::POP(target) => self.handle_pop(target),
            Instruction::XOR(source) => self.handle_xor(source),
        }
    }

    fn log(&mut self, opcode: u8, instruction: &Instruction) {
        match self.pc {
            0x0000 => println!("Setup Stack..."),
            0x0003 => println!("Setup VRAM..."),
            0x000c => println!("Setup Sound..."),
            0x001d => println!("Setup BG palette..."),
            0x0021 => println!("Loading logo data from cart into Video RAM..."),
            0x0040 => println!("Setup background tilemap..."),
            0x005d => println!("Turning on LCD and showing Background..."),
            0x0062..=0x00fd => {}
            0x00fe => println!("Done with processing boot ROM. Switching to Cartridge..."),
            0x0100..=0xffff => self.print_registers(opcode, Some(instruction)),
            _ => {}
        }
    }

    /// Reads the next byte and increases pc
    fn consume_byte(&mut self) -> u8 {
        self.pc += 1;
        self.read(self.pc)
    }

    /// Reads the next word and increases pc
    fn consume_word(&mut self) -> u16 {
        u16::from(self.consume_byte()) | u16::from(self.consume_byte()) << 8
    }

    /// Push a u16 value onto the stack
    pub fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        // Write the most significant byte
        self.write(self.sp, (value >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        // Write the least significant byte
        self.write(self.sp, value as u8);
    }

    /// Pop a u16 value from the stack
    fn pop(&mut self) -> u16 {
        let lsb = self.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        let msb = self.read(self.sp) as u16;
        self.sp = self.sp.wrapping_add(1);

        (msb << 8) | lsb
    }

    /// Handles ADD instructions
    fn handle_add(&mut self, target: ArithmeticByteTarget, source: ArithmeticByteSource) -> u16 {
        let source_value = match source {
            ArithmeticByteSource::A => self.r.a,
            ArithmeticByteSource::B => self.r.b,
            ArithmeticByteSource::C => self.r.c,
            ArithmeticByteSource::D => self.r.d,
            ArithmeticByteSource::E => self.r.e,
            ArithmeticByteSource::H => self.r.h,
            ArithmeticByteSource::L => self.r.l,
            ArithmeticByteSource::HLI => self.read(self.r.get_hl()),
            _ => unimplemented!(),
        };
        let target_value = match target {
            ArithmeticByteTarget::A => self.r.a,
            _ => unimplemented!(),
        };

        // TODO: is this correct?
        let (new_value, did_overflow) = target_value.overflowing_add(source_value);
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.r.f.update(
            new_value == 0,
            false,
            (target_value ^ source_value ^ new_value) & 0x10 != 0,
            did_overflow,
        );

        match target {
            ArithmeticByteTarget::A => self.r.a = new_value,
            _ => unimplemented!(),
        }

        match source {
            ArithmeticByteSource::HLI => self.clock.advance(8),
            ArithmeticByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles ADD instructions with words
    fn handle_add2(&mut self, target: ArithmeticWordTarget, source: ArithmeticWordSource) -> u16 {
        let source_value = match source {
            ArithmeticWordSource::DE => self.r.get_de(),
            ArithmeticWordSource::HL => self.r.get_hl(),
        };
        let target_value = match target {
            ArithmeticWordTarget::HL => self.r.get_hl(),
            _ => unimplemented!(),
        };

        let (new_value, did_overflow) = target_value.overflowing_add(source_value);
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.r.f.update(
            new_value == 0,
            false,
            (target_value & 0xF) + (source_value & 0xF) > 0xF,
            did_overflow,
        );
        match target {
            ArithmeticWordTarget::HL => self.r.set_hl(new_value),
            _ => unimplemented!(),
        }

        match target {
            ArithmeticWordTarget::SP => self.clock.advance(16),
            ArithmeticWordTarget::HL => self.clock.advance(8),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles ADC instructions
    /// TODO: might be buggy
    fn handle_adc(&mut self, source: ByteSource) -> u16 {
        let value = match source {
            ByteSource::A => self.r.a,
            ByteSource::E => self.r.e,
            ByteSource::H => self.r.h,
            ByteSource::L => self.r.l,
            ByteSource::HLI => self.read(self.r.get_hl()),
            _ => unimplemented!(),
        };
        self.r.a = self.r.a.wrapping_add(value);
        self.r.a = self.r.a.wrapping_add(if self.r.f.carry { 1 } else { 0 });

        self.r.f.zero = self.r.a == 0;
        self.r.f.negative = false;
        self.r.f.half_carry = (self.r.a & 0xF) + (self.r.a & 0xF) > 0xF;
        self.pc.wrapping_add(1)
    }

    /// Handles AND instructions
    fn handle_and(&mut self, source: BitOperationSource) -> u16 {
        let value = match source {
            BitOperationSource::A => self.r.a,
            BitOperationSource::B => self.r.b,
            BitOperationSource::C => self.r.c,
            BitOperationSource::D => self.r.d,
            BitOperationSource::E => self.r.e,
            _ => unimplemented!(),
        };
        self.r.a &= value;
        self.r.f.update(self.r.a == 0, false, true, false);
        self.pc.wrapping_add(1)
    }

    /// Handles BIT instructions
    fn handle_bit(&mut self, bit: u8, source: BitOperationSource) -> u16 {
        let value = match source {
            BitOperationSource::H => self.r.h,
            _ => unimplemented!(),
        };

        self.r.f.zero = !utils::bit_at(value, bit);
        self.r.f.negative = false;
        self.r.f.half_carry = true;
        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handle CALL function
    fn handle_call(&mut self, test: JumpTest) -> u16 {
        let should_jump = match test {
            JumpTest::Always => true,
            JumpTest::NotZero => !self.r.f.zero,
            _ => unimplemented!(),
        };

        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.clock.advance(24);
            self.push(next_pc);
            self.consume_word()
        } else {
            // TODO: advance clock?
            next_pc
        }
    }

    /// Handles CP instructions
    fn handle_cp(&mut self, source: ByteSource) -> u16 {
        let value = match source {
            ByteSource::D8 => self.consume_byte(),
            ByteSource::HLI => self.read(self.r.get_hl()),
            _ => unimplemented!(),
        };
        let result = u32::from(self.r.a).wrapping_sub(u32::from(value));

        self.r.f.zero = result as u8 == 0;
        self.r.f.half_carry = (self.r.a ^ value ^ result as u8) & 0x10 != 0;
        self.r.f.carry = result & 0x100 != 0;
        self.r.f.negative = true;

        match source {
            ByteSource::D8 | ByteSource::HLI => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles DAA instruction
    fn handle_daa(&mut self) -> u16 {
        // From https://forums.nesdev.com/viewtopic.php?t=15944
        if !self.r.f.negative {
            // After an addition, adjust if (half-)carry occurred or if result is out of bounds
            if self.r.f.carry || self.r.a > 0x90 {
                self.r.a = self.r.a.wrapping_add(0x60);
                self.r.f.carry = true;
            }
            if self.r.f.half_carry || (self.r.a & 0x0f) > 0x09 {
                self.r.a = self.r.a.wrapping_add(0x06);
            }
        } else {
            // after a subtraction, only adjust if (half-)carry occurred
            if self.r.f.carry {
                self.r.a = self.r.a.wrapping_sub(0x60);
            }
            if self.r.f.half_carry {
                self.r.a = self.r.a.wrapping_sub(0x06);
            }
        }
        self.r.f.zero = self.r.a == 0;
        self.r.f.half_carry = false;
        self.pc.wrapping_add(1)
    }

    /// Handles DEC instructions
    fn handle_dec(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::A => {
                self.r.f.half_carry = self.r.a & 0xF == 0;
                self.r.a = self.r.a.wrapping_sub(1);
                self.r.f.zero = self.r.a == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::B => {
                self.r.f.half_carry = self.r.b & 0xF == 0;
                self.r.b = self.r.b.wrapping_sub(1);
                self.r.f.zero = self.r.b == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::C => {
                self.r.f.half_carry = self.r.c & 0xF == 0;
                self.r.c = self.r.c.wrapping_sub(1);

                self.r.f.zero = self.r.c == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::D => {
                self.r.f.half_carry = self.r.d & 0xF == 0;
                self.r.d = self.r.d.wrapping_sub(1);
                self.r.f.zero = self.r.d == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::E => {
                self.r.f.half_carry = self.r.e & 0xF == 0;
                self.r.e = self.r.e.wrapping_sub(1);
                self.r.f.zero = self.r.e == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::H => {
                self.r.f.half_carry = self.r.h & 0xF == 0;
                self.r.h = self.r.h.wrapping_sub(1);
                self.r.f.zero = self.r.h == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::L => {
                self.r.f.half_carry = self.r.l & 0xF == 0;
                self.r.l = self.r.l.wrapping_sub(1);
                self.r.f.zero = self.r.l == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::BC => {
                self.r.set_bc(self.r.get_bc().wrapping_sub(1));
            }
            IncDecTarget::SP => {
                self.sp = self.sp.wrapping_sub(1);
            }
            _ => unimplemented!(),
        }
        match target {
            IncDecTarget::HLI => self.clock.advance(12),
            IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => {
                self.clock.advance(8)
            }
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles HALT instruction
    fn handle_halt(&mut self) -> u16 {
        self.is_halted = true;
        self.pc.wrapping_add(1)
    }

    /// Handles INC instructions
    /// TODO: refactor me
    fn handle_inc(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::A => {
                self.r.a = self.r.a.wrapping_add(1);
                self.r.f.zero = self.r.a == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = self.r.a & 0xf == 0xf;
            }
            IncDecTarget::B => {
                self.r.b = self.r.b.wrapping_add(1);
                self.r.f.zero = self.r.b == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = self.r.b & 0xf == 0xf;
            }
            IncDecTarget::C => {
                self.r.c = self.r.c.wrapping_add(1);
                self.r.f.zero = self.r.c == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = self.r.c & 0xf == 0xf;
            }
            IncDecTarget::H => {
                self.r.h = self.r.h.wrapping_add(1);
                self.r.f.zero = self.r.h == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = self.r.h & 0xf == 0xf;
            }
            IncDecTarget::L => {
                self.r.l = self.r.l.wrapping_add(1);
                self.r.f.zero = self.r.l == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = self.r.l & 0xf == 0xf;
            }

            IncDecTarget::BC => self.r.set_bc(self.r.get_bc().wrapping_add(1)),
            IncDecTarget::DE => self.r.set_de(self.r.get_de().wrapping_add(1)),
            IncDecTarget::HL => self.r.set_hl(self.r.get_hl().wrapping_add(1)),
            _ => unimplemented!(),
        }

        match target {
            IncDecTarget::HLI => self.clock.advance(12),
            IncDecTarget::BC | IncDecTarget::DE | IncDecTarget::HL | IncDecTarget::SP => {
                self.clock.advance(8)
            }
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles EI and DI instructions
    fn handle_interrupt(&mut self, enable: bool) -> u16 {
        self.ime = enable;
        self.pc.wrapping_add(1)
    }

    /// Handles JR instructions
    fn handle_jr(&mut self, test: JumpTest) -> u16 {
        let should_jump = match test {
            JumpTest::NotZero => !self.r.f.zero,
            JumpTest::NotCarry => !self.r.f.carry,
            JumpTest::Zero => self.r.f.zero,
            JumpTest::Carry => self.r.f.carry,
            JumpTest::Always => true,
        };
        if !should_jump {
            self.clock.advance(8);
            return self.pc.wrapping_add(2);
        }

        self.clock.advance(12);
        let offset = self.consume_byte() as i8;
        let pc = (self.pc as i16).wrapping_add(offset as i16);
        // TODO: is this correct?
        (pc as u16).wrapping_add(1)
    }

    /// Handles JP instructions
    fn handle_jp(&mut self, test: JumpTest) -> u16 {
        let should_jump = match test {
            JumpTest::NotZero => !self.r.f.zero,
            JumpTest::NotCarry => !self.r.f.carry,
            JumpTest::Zero => self.r.f.zero,
            JumpTest::Carry => self.r.f.carry,
            JumpTest::Always => true,
        };
        if should_jump {
            // Gameboy is little endian so read pc + 2 as most significant bit
            // and pc + 1 as least significant bit
            let least_significant_byte = self.read(self.pc + 1) as u16;
            let most_significant_byte = self.read(self.pc + 2) as u16;
            (most_significant_byte << 8) | least_significant_byte
        } else {
            // If we don't jump we need to still move the program
            // counter forward by 3 since the jump instruction is
            // 3 bytes wide (1 byte for tag and 2 bytes for jump address)
            self.pc.wrapping_add(3)
        }
    }

    /// Handles LD instructions
    fn handle_ld(&mut self, load_type: LoadType) -> u16 {
        match load_type {
            LoadType::Byte(target, source) => {
                let value = match source {
                    ByteSource::A => self.r.a,
                    ByteSource::B => self.r.b,
                    ByteSource::C => self.r.c,
                    ByteSource::D => self.r.d,
                    ByteSource::E => self.r.e,
                    ByteSource::H => self.r.h,
                    ByteSource::L => self.r.l,
                    ByteSource::D8 => self.consume_byte(),
                    ByteSource::HLI => self.read(self.r.get_hl()),
                    _ => unimplemented!(),
                };
                match target {
                    LoadByteTarget::A => self.r.a = value,
                    LoadByteTarget::B => self.r.b = value,
                    LoadByteTarget::C => self.r.c = value,
                    LoadByteTarget::D => self.r.d = value,
                    LoadByteTarget::E => self.r.e = value,
                    LoadByteTarget::H => self.r.h = value,
                    LoadByteTarget::L => self.r.l = value,
                    LoadByteTarget::HLI => self.write(self.r.get_hl(), value),
                }

                // If IO is involved it consumes 8 cycles otherwise 4.
                match (source, target) {
                    (ByteSource::D8, _) => self.clock.advance(8),
                    (ByteSource::HLI, _) => self.clock.advance(8),
                    (_, LoadByteTarget::HLI) => self.clock.advance(8),
                    (_, _) => self.clock.advance(4),
                }

                self.pc.wrapping_add(1)
            }
            LoadType::Word(target, source) => {
                let value = match source {
                    WordSource::D16 => self.consume_word(),
                    _ => unimplemented!(),
                };
                match target {
                    LoadWordTarget::BC => self.r.set_bc(value),
                    LoadWordTarget::DE => self.r.set_de(value),
                    LoadWordTarget::HL => self.r.set_hl(value),
                    LoadWordTarget::SP => self.sp = value,
                }
                self.clock.advance(12);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFrom(address, source) => {
                let addr = match address {
                    AddressSource::C => 0xFF00 | u16::from(self.r.c),
                    AddressSource::D8 => 0xFF00 | u16::from(self.consume_byte()),
                    AddressSource::D16 => self.consume_word(),
                    AddressSource::HLI => self.r.get_hl(),
                };
                let value = match source {
                    ByteSource::A => self.r.a,
                    ByteSource::D => self.r.d,
                    ByteSource::L => self.r.l,
                    _ => unimplemented!(),
                };
                self.write(addr, value);
                match address {
                    AddressSource::D16 => self.clock.advance(16),
                    AddressSource::D8 => self.clock.advance(12),
                    _ => self.clock.advance(8),
                }
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromAInc(source) => {
                let addr = match source {
                    ByteSource::HLI => self.r.get_hl(),
                    _ => unimplemented!(),
                };
                self.write(addr, self.r.a);
                match source {
                    ByteSource::HLI => self.r.set_hl(addr.wrapping_add(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromADec(source) => {
                let addr = match source {
                    ByteSource::HLI => self.r.get_hl(),
                    _ => unimplemented!(),
                };
                self.write(addr, self.r.a);
                match source {
                    ByteSource::HLI => self.r.set_hl(addr.wrapping_sub(1)),
                    _ => unimplemented!(),
                }
                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromWord(address, source) => {
                let address = match address {
                    AddressSource::D16 => self.consume_word(),
                    _ => unimplemented!(),
                };
                let value = match source {
                    WordSource::SP => self.sp,
                    _ => unimplemented!(),
                };
                self.write(address, value as u8);
                self.write(address + 1, (value >> 8) as u8);
                self.pc.wrapping_add(1)
            }
            LoadType::FromIndirect(target, source) => {
                let addr = match source {
                    ByteSource::BC => self.r.get_bc(),
                    ByteSource::DE => self.r.get_de(),
                    ByteSource::HLI => self.r.get_hl(),
                    ByteSource::D8 => 0xFF00 | self.consume_byte() as u16,
                    _ => unimplemented!(),
                };
                match target {
                    LoadByteTarget::A => self.r.a = self.read(addr),
                    LoadByteTarget::E => self.r.e = self.read(addr),
                    _ => unimplemented!(),
                }
                match source {
                    ByteSource::D8 => self.clock.advance(12),
                    _ => self.clock.advance(8),
                }

                self.pc.wrapping_add(1)
            }
            LoadType::FromIndirectAInc(source) => {
                let addr = match source {
                    ByteSource::HLI => self.r.get_hl(),
                    _ => unimplemented!(),
                };
                self.r.a = self.read(addr);
                self.r.set_hl(self.r.get_hl().wrapping_add(1));
                self.pc.wrapping_add(1)
            }
        }
    }

    /// Handles NOP instruction
    fn handle_nop(&mut self) -> u16 {
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles OR instructions
    fn handle_or(&mut self, source: BitOperationSource) -> u16 {
        let source = match source {
            BitOperationSource::E => self.r.e,
            _ => unimplemented!(),
        };
        self.r.a |= source;
        self.r.f.update(self.r.a == 0, false, false, false);
        self.pc.wrapping_add(1)
    }

    /// Handles POP instruction
    fn handle_pop(&mut self, target: StackTarget) -> u16 {
        let result = self.pop();
        match target {
            StackTarget::BC => self.r.set_bc(result),
            StackTarget::HL => self.r.set_hl(result),
            _ => unimplemented!(),
        };
        self.clock.advance(12);
        self.pc.wrapping_add(1)
    }

    /// Handles PUSH instruction
    fn handle_push(&mut self, target: StackTarget) -> u16 {
        let value = match target {
            StackTarget::BC => self.r.get_bc(),
            StackTarget::DE => self.r.get_de(),
            _ => unimplemented!(),
        };
        self.push(value);
        self.clock.advance(16);
        self.pc.wrapping_add(1)
    }

    /// Handle RET function
    fn handle_ret(&mut self, test: JumpTest) -> u16 {
        let should_jump = match test {
            JumpTest::Always => true,
            JumpTest::NotZero => !self.r.f.zero,
            _ => unimplemented!(),
        };

        if should_jump {
            self.clock.advance(20);
            self.pop()
        } else {
            self.clock.advance(8);
            self.pc.wrapping_add(1)
        }
    }

    /// Handles RLA instruction
    /// Rotate A left through carry
    fn handle_rla(&mut self) -> u16 {
        let new_carry = utils::bit_at(self.r.a, 7);
        self.r.a = (self.r.a << 1) | self.r.f.carry as u8;
        self.r.f.update(false, false, false, new_carry);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RLC instructions
    /// Rotates register to the left and updates CPU flags
    fn handle_rlc(&mut self, target: PrefixTarget) -> u16 {
        let cur_val = match target {
            PrefixTarget::B => self.r.b,
            PrefixTarget::C => self.r.c,
        };
        let carry = utils::bit_at(cur_val, 7);
        let new_val = (cur_val << 1) | carry as u8;
        self.r.f.update(new_val == 0, false, false, carry);
        match target {
            PrefixTarget::B => self.r.b = new_val,
            PrefixTarget::C => self.r.c = new_val,
        }
        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles RCAA instruction
    fn handle_rrca(&mut self) -> u16 {
        let bit = self.r.a & 1;
        self.r.a >>= 1;
        self.r.f.update(self.r.a == 0, false, false, bit == 1);
        self.pc.wrapping_add(1)
    }

    /// Handles RST instructions
    fn handle_rst(&mut self, code: ResetCode) -> u16 {
        self.sp = self.sp.wrapping_sub(2);
        self.push(self.pc);
        match code {
            ResetCode::RST08 => 8,
            ResetCode::RST18 => 18,
            ResetCode::RST28 => 28,
            ResetCode::RST38 => 38,
        }
    }

    /// Handles STOP instruction
    fn handle_stop(&mut self) -> u16 {
        // TODO: set correct flag
        self.is_halted = true;
        self.pc.wrapping_add(1)
    }

    /// Handles SUB instructions
    fn handle_sub(&mut self, target: ArithmeticByteTarget, source: ArithmeticByteSource) -> u16 {
        let source_value = match source {
            ArithmeticByteSource::B => self.r.b,
            ArithmeticByteSource::D => self.r.d,
            ArithmeticByteSource::HLI => self.read(self.r.get_hl()),
            _ => unimplemented!(),
        };
        let target_value = match target {
            ArithmeticByteTarget::A => self.r.a,
            _ => unimplemented!(),
        };

        let (new_value, did_overflow) = target_value.overflowing_sub(source_value);
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.r.f.update(
            new_value == 0,
            true,
            (target_value ^ source_value ^ new_value) & 0x10 != 0,
            did_overflow,
        );

        match target {
            ArithmeticByteTarget::A => self.r.a = new_value,
            _ => unimplemented!(),
        }

        match source {
            ArithmeticByteSource::HLI | ArithmeticByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles XOR instructions
    fn handle_xor(&mut self, source: BitOperationSource) -> u16 {
        let value = match source {
            BitOperationSource::A => self.r.a,
            _ => unimplemented!(),
        };
        self.r.a ^= value;
        self.r.f.update(self.r.a == 0, false, false, false);
        match source {
            BitOperationSource::A => self.clock.advance(4),
            // TODO: fix set clock time for (HL=8t instead of 4t)!
            _ => unimplemented!(),
        }

        self.pc.wrapping_add(1)
    }
}

impl<'a, T: AddressSpace> AddressSpace for CPU<'a, T> {
    fn write(&mut self, address: u16, value: u8) {
        self.bus.borrow_mut().write(address, value);
    }

    fn read(&self, address: u16) -> u8 {
        self.bus.borrow().read(address)
    }
}
