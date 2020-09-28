use crate::gb::instruction::{
    ArithmeticWordTarget, ByteSource, IncDecTarget, Instruction, JumpTest, LoadByteTarget,
    LoadType, LoadWordTarget, PrefixTarget, ResetCode, StackTarget, WordSource,
};
use crate::gb::timer::Clock;
use crate::gb::AddressSpace;
use crate::utils;
use registers::Registers;
use std::cell::RefCell;
use std::thread;
use std::time::Duration;

mod registers;
#[cfg(test)]
mod tests;

/// Implements the CPU for the GB (DMG-01),
/// the CPU is LR35902 which is a subset of i8080 & Z80.
pub struct CPU<'a, T: AddressSpace> {
    pub r: Registers,
    pub pc: u16,   // Program counter
    pub sp: u16,   // Stack Pointer
    pub ime: bool, // Interrupt Master Enable
    pub is_halted: bool,
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

    /// Makes one CPU step, this consumes one or more bytes depending on the
    /// next instruction and current CPU state (halted, stopped, etc.).
    pub fn step(&mut self) -> u32 {
        self.clock.reset();

        if self.is_halted {
            self.clock.advance(4);
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
                println!(
                    "Unresolved instruction found for: {}.\nHALTED!",
                    description
                );
                loop {
                    thread::sleep(Duration::from_millis(10));
                }
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

    /// Executes the given instruction, advances the internal clock
    /// and returns the updated program counter.
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(source) => self.handle_add(source),
            Instruction::ADD2(target, source) => self.handle_add2(target, source),
            Instruction::ADDSP => self.handle_add_sp(),
            Instruction::ADC(source) => self.handle_adc(source),
            Instruction::AND(source) => self.handle_and(source),
            Instruction::BIT(bit, source) => self.handle_bit(bit, source),
            Instruction::CALL(test) => self.handle_call(test),
            Instruction::CCF => self.handle_ccf(),
            Instruction::CP(source) => self.handle_cp(source),
            Instruction::CPL => self.handle_cpl(),
            Instruction::DAA => self.handle_daa(),
            Instruction::DI => self.handle_interrupt(false),
            Instruction::DEC(target) => self.handle_dec(target),
            Instruction::EI => self.handle_interrupt(true),
            Instruction::HALT => self.handle_halt(),
            Instruction::INC(target) => self.handle_inc(target),
            Instruction::JR(test) => self.handle_jr(test),
            Instruction::JP(test, source) => self.handle_jp(test, source),
            Instruction::LD(load_type) => self.handle_ld(load_type),
            Instruction::NOP => self.handle_nop(),
            Instruction::OR(source) => self.handle_or(source),
            Instruction::RES(bit, source) => self.handle_res(bit, source),
            Instruction::RET(test) => self.handle_ret(test),
            Instruction::RETI => self.handle_reti(),
            Instruction::RL(target) => self.handle_rl(target),
            Instruction::RLA => self.handle_rla(),
            Instruction::RLC(target) => self.handle_rlc(target),
            Instruction::RLCA => self.handle_rlca(),
            Instruction::RR(target) => self.handle_rr(target),
            Instruction::RRA => self.handle_rra(),
            Instruction::RRC(target) => self.handle_rrc(target),
            Instruction::RRCA => self.handle_rrca(),
            Instruction::RST(code) => self.handle_rst(code),
            Instruction::SBC(source) => self.handle_sbc(source),
            Instruction::SCF => self.handle_scf(),
            Instruction::SET(bit, source) => self.handle_set(bit, source),
            Instruction::SRL(source) => self.handle_srl(source),
            Instruction::STOP => self.handle_stop(),
            Instruction::SUB(source) => self.handle_sub(source),
            Instruction::SWAP(source) => self.handle_swap(source),
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
    pub fn consume_byte(&mut self) -> u8 {
        self.pc = self.pc.wrapping_add(1);
        self.read(self.pc)
    }

    /// Reads the next word and increases pc
    pub fn consume_word(&mut self) -> u16 {
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
    fn handle_add(&mut self, source: ByteSource) -> u16 {
        let source_value = source.resolve_value(self);
        let (new_value, did_overflow) = self.r.a.overflowing_add(source_value);
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.r.f.update(
            new_value == 0,
            false,
            utils::half_carry_u8(self.r.a, source_value),
            did_overflow,
        );
        self.r.a = new_value;

        match source {
            ByteSource::HLI => self.clock.advance(8),
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles ADD instructions with words
    /// TODO: refactor me
    fn handle_add2(&mut self, target: ArithmeticWordTarget, source: WordSource) -> u16 {
        let source_value = source.resolve_value(self);
        let target_value = match target {
            ArithmeticWordTarget::HL => self.r.get_hl(),
        };

        let (new_value, overflow) = target_value.overflowing_add(source_value);

        match target {
            ArithmeticWordTarget::HL => {
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u16(target_value, source_value);
                self.r.f.carry = overflow;
                self.r.set_hl(new_value);
            }
        }

        match target {
            ArithmeticWordTarget::HL => self.clock.advance(8),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles ADD SP, i8 instruction
    fn handle_add_sp(&mut self) -> u16 {
        let sp = self.sp as i32;
        let byte = self.consume_byte() as i8 as i32;
        let result = sp.wrapping_add(byte as i32);
        self.sp = result as u16;

        // Carry and half carry are for the low byte
        let half_carry = (sp ^ byte as i32 ^ result) & 0x10 != 0;
        let carry = (sp ^ byte as i32 ^ result) & 0x100 != 0;
        self.r.f.update(false, false, half_carry, carry);
        self.clock.advance(16);
        self.pc.wrapping_add(1)
    }

    /// Handles ADC instructions
    fn handle_adc(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self) as u32;

        // Check for carry using 32bit arithmetic
        // TODO: can be improved
        let a = self.r.a as u32;
        let carry = self.r.f.carry as u32;

        let result = a.wrapping_add(value).wrapping_add(carry);
        self.r.a = result as u8;

        let half_carry = (a ^ value ^ result) & 0x10 != 0;
        let carry = result & 0x100 != 0;
        self.r.f.update(result as u8 == 0, false, half_carry, carry);
        match source {
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles AND instructions
    fn handle_and(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        self.r.a &= value;
        self.r.f.update(self.r.a == 0, false, true, false);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles BIT instructions
    fn handle_bit(&mut self, bit: u8, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        self.r.f.zero = !utils::bit_at(value, bit);
        self.r.f.negative = false;
        self.r.f.half_carry = true;
        match source {
            ByteSource::HLI => self.clock.advance(12),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handle CALL instructions
    fn handle_call(&mut self, test: JumpTest) -> u16 {
        let should_jump = test.resolve_value(self);
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.clock.advance(24);
            self.push(next_pc);
            self.consume_word()
        } else {
            self.clock.advance(12);
            next_pc
        }
    }

    /// Handle CCF instruction
    fn handle_ccf(&mut self) -> u16 {
        self.r.f.negative = false;
        self.r.f.half_carry = false;
        self.r.f.carry = !self.r.f.carry;
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles CP instructions
    fn handle_cp(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
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

    /// Handles CPL instruction
    fn handle_cpl(&mut self) -> u16 {
        self.r.a = !self.r.a;
        self.r.f.negative = true;
        self.r.f.half_carry = true;
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles DAA instruction
    fn handle_daa(&mut self) -> u16 {
        let mut adjust = 0;
        // See if we had a carry/borrow for the high nibble in the last
        // operation
        if self.r.f.carry {
            adjust |= 0b0110_0000;
        }
        // See if we had a carry/borrow for the low nibble in the last
        // operation
        if self.r.f.half_carry {
            adjust |= 0b0000_0110
        }

        let result = if self.r.f.negative {
            // If the operation was a subtraction we're done since we
            // can never end up in the A-F range by subtracting
            // without generating a (half)carry.
            self.r.a.wrapping_sub(adjust)
        } else {
            // Additions are a bit more tricky because we might have
            // to adjust even if we haven't overflowed (and no carry
            // is present). For instance: 0x08 + 0x04 -> 0x0c.
            if self.r.a & 0b0000_1111 > 0x09 {
                adjust |= 0b0000_0110;
            }
            if self.r.a > 0x99 {
                adjust |= 0b0110_0000;
            }
            self.r.a.wrapping_add(adjust)
        };

        self.r.a = result;
        self.r.f.zero = result == 0;
        self.r.f.half_carry = false;
        self.r.f.carry = adjust & 0x60 != 0;
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles DEC instructions
    /// TODO: refactor me
    fn handle_dec(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::A => {
                self.r.f.half_carry = self.r.a.trailing_zeros() >= 4;
                self.r.a = self.r.a.wrapping_sub(1);
                self.r.f.zero = self.r.a == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::B => {
                self.r.f.half_carry = self.r.b.trailing_zeros() >= 4;
                self.r.b = self.r.b.wrapping_sub(1);
                self.r.f.zero = self.r.b == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::C => {
                self.r.f.half_carry = self.r.c.trailing_zeros() >= 4;
                self.r.c = self.r.c.wrapping_sub(1);

                self.r.f.zero = self.r.c == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::D => {
                self.r.f.half_carry = self.r.d.trailing_zeros() >= 4;
                self.r.d = self.r.d.wrapping_sub(1);
                self.r.f.zero = self.r.d == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::E => {
                self.r.f.half_carry = self.r.e.trailing_zeros() >= 4;
                self.r.e = self.r.e.wrapping_sub(1);
                self.r.f.zero = self.r.e == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::H => {
                self.r.f.half_carry = self.r.h.trailing_zeros() >= 4;
                self.r.h = self.r.h.wrapping_sub(1);
                self.r.f.zero = self.r.h == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::L => {
                self.r.f.half_carry = self.r.l.trailing_zeros() >= 4;
                self.r.l = self.r.l.wrapping_sub(1);
                self.r.f.zero = self.r.l == 0;
                self.r.f.negative = true;
            }
            IncDecTarget::HLI => {
                let value = self.read(self.r.get_hl());
                self.r.f.half_carry = value.trailing_zeros() >= 4;
                let value = value.wrapping_sub(1);
                self.r.f.zero = value == 0;
                self.r.f.negative = true;
                self.write(self.r.get_hl(), value);
            }
            IncDecTarget::BC => self.r.set_bc(self.r.get_bc().wrapping_sub(1)),
            IncDecTarget::DE => self.r.set_de(self.r.get_de().wrapping_sub(1)),
            IncDecTarget::HL => self.r.set_hl(self.r.get_hl().wrapping_sub(1)),
            IncDecTarget::SP => self.sp = self.sp.wrapping_sub(1),
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
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles INC instructions
    /// TODO: refactor me
    fn handle_inc(&mut self, target: IncDecTarget) -> u16 {
        match target {
            IncDecTarget::A => {
                let result = self.r.a.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.a, 1);
                self.r.a = result;
            }
            IncDecTarget::B => {
                let result = self.r.b.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.b, 1);
                self.r.b = result;
            }
            IncDecTarget::C => {
                let result = self.r.c.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.c, 1);
                self.r.c = result;
            }
            IncDecTarget::D => {
                let result = self.r.d.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.d, 1);
                self.r.d = result;
            }
            IncDecTarget::E => {
                let result = self.r.e.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.e, 1);
                self.r.e = result;
            }
            IncDecTarget::H => {
                let result = self.r.h.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.h, 1);
                self.r.h = result;
            }
            IncDecTarget::L => {
                let result = self.r.l.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(self.r.l, 1);
                self.r.l = result;
            }
            IncDecTarget::HLI => {
                let value = self.read(self.r.get_hl());
                let result = value.wrapping_add(1);
                self.r.f.zero = result == 0;
                self.r.f.negative = false;
                self.r.f.half_carry = utils::half_carry_u8(value, 1);
                self.write(self.r.get_hl(), result);
            }
            IncDecTarget::BC => self.r.set_bc(self.r.get_bc().wrapping_add(1)),
            IncDecTarget::DE => self.r.set_de(self.r.get_de().wrapping_add(1)),
            IncDecTarget::HL => self.r.set_hl(self.r.get_hl().wrapping_add(1)),
            IncDecTarget::SP => self.sp = self.sp.wrapping_add(1),
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
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles JR instructions
    fn handle_jr(&mut self, test: JumpTest) -> u16 {
        let should_jump = test.resolve_value(self);
        if !should_jump {
            self.clock.advance(8);
            return self.pc.wrapping_add(2);
        }

        self.clock.advance(12);
        // TODO: this seems incorrect. Maybe cast with `as *const i8 as i8`?
        let offset = self.consume_byte() as i8;
        let pc = (self.pc as i16).wrapping_add(offset as i16);
        // TODO: is this correct?
        (pc as u16).wrapping_add(1)
    }

    /// Handles JP instructions
    fn handle_jp(&mut self, test: JumpTest, source: WordSource) -> u16 {
        let should_jump = test.resolve_value(self);
        if should_jump {
            return match source {
                WordSource::HL => {
                    self.clock.advance(4);
                    self.r.get_hl()
                }
                WordSource::D16 => {
                    self.clock.advance(16);
                    self.consume_word()
                }
                _ => unimplemented!(),
            };
        }
        self.clock.advance(12);
        // If we don't jump we need to still move the program
        // counter forward by 3 since the jump instruction is
        // 3 bytes wide (1 byte for tag and 2 bytes for jump address)
        self.pc.wrapping_add(3)
    }

    /// Handles LD instructions
    fn handle_ld(&mut self, load_type: LoadType) -> u16 {
        match load_type {
            LoadType::Byte(target, source) => {
                let value = source.resolve_value(self);
                match target {
                    LoadByteTarget::A => self.r.a = value,
                    LoadByteTarget::B => self.r.b = value,
                    LoadByteTarget::CIFF00 => self.r.c = value,
                    LoadByteTarget::D => self.r.d = value,
                    LoadByteTarget::E => self.r.e = value,
                    LoadByteTarget::H => self.r.h = value,
                    LoadByteTarget::L => self.r.l = value,
                    LoadByteTarget::HLI => self.write(self.r.get_hl(), value),
                    _ => unimplemented!(),
                }

                // Each I/O operation takes 4 additional cycles,
                // that means if both source and target involve I/O
                // it takes 12 cycles in total.
                match (source, target) {
                    (ByteSource::D8, LoadByteTarget::HLI) => self.clock.advance(12),
                    (ByteSource::D8, _) => self.clock.advance(8),
                    (ByteSource::HLI, _) => self.clock.advance(8),
                    (_, LoadByteTarget::HLI) => self.clock.advance(8),
                    (_, _) => self.clock.advance(4),
                }

                self.pc.wrapping_add(1)
            }
            LoadType::Word(target, source) => {
                let value = source.resolve_value(self);
                match target {
                    LoadWordTarget::BC => self.r.set_bc(value),
                    LoadWordTarget::DE => self.r.set_de(value),
                    LoadWordTarget::HL => self.r.set_hl(value),
                    LoadWordTarget::SP => self.sp = value,
                    _ => unimplemented!(),
                }
                self.clock.advance(12);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFrom(target, source) => {
                let value = source.resolve_value(self);
                let addr = match target {
                    LoadByteTarget::BCI => self.r.get_bc(),
                    LoadByteTarget::DEI => self.r.get_de(),
                    LoadByteTarget::D16I => self.consume_word(),
                    LoadByteTarget::HLI => self.r.get_hl(),
                    LoadByteTarget::CIFF00 => {
                        self.clock.advance(8);
                        u16::from(self.r.c).wrapping_add(0xFF00)
                    }
                    LoadByteTarget::D8IFF00 => {
                        self.clock.advance(4);
                        u16::from(self.consume_byte()).wrapping_add(0xFF00)
                    }
                    _ => unimplemented!(),
                };
                self.write(addr, value);
                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromAInc(target) => {
                let addr = match target {
                    LoadByteTarget::HLI => self.r.get_hl(),
                    _ => unimplemented!(),
                };
                self.write(addr, self.r.a);
                match target {
                    LoadByteTarget::HLI => self.r.set_hl(addr.wrapping_add(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromADec(target) => {
                let addr = match target {
                    LoadByteTarget::HLI => self.r.get_hl(),
                    _ => unimplemented!(),
                };
                self.write(addr, self.r.a);
                match target {
                    LoadByteTarget::HLI => self.r.set_hl(addr.wrapping_sub(1)),
                    _ => unimplemented!(),
                }
                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromWord(target, source) => {
                let value = source.resolve_value(self);
                match target {
                    LoadWordTarget::SP => self.sp = value,
                    LoadWordTarget::D16I => {
                        let address = self.consume_word();
                        self.write(address, value as u8);
                        self.write(address + 1, (value >> 8) as u8);
                    }
                    _ => unimplemented!(),
                };
                match target {
                    LoadWordTarget::D16I => self.clock.advance(20),
                    _ => self.clock.advance(8),
                }
                self.pc.wrapping_add(1)
            }
            LoadType::FromIndirect(target, source) => {
                let value = source.resolve_value(self);
                match target {
                    LoadByteTarget::A => self.r.a = value,
                    LoadByteTarget::E => self.r.e = value,
                    _ => unimplemented!(),
                }
                match source {
                    ByteSource::D16I => self.clock.advance(16),
                    ByteSource::D8IFF00 => self.clock.advance(12),
                    _ => self.clock.advance(8),
                }

                self.pc.wrapping_add(1)
            }
            LoadType::FromIndirectAInc(source) => {
                let value = source.resolve_value(self);
                self.r.a = value;
                match source {
                    ByteSource::HLI => self.r.set_hl(self.r.get_hl().wrapping_add(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::FromIndirectADec(source) => {
                let value = source.resolve_value(self);
                self.r.a = value;
                match source {
                    ByteSource::HLI => self.r.set_hl(self.r.get_hl().wrapping_sub(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            LoadType::IndirectFromSPi8(target) => {
                // TODO: generalize this
                let sp = self.sp as i32;
                let n = self.consume_byte() as i8;

                let nn = n as i32;

                let result = sp.wrapping_add(nn);
                let carry = (sp ^ nn ^ result) & 0x100 != 0;
                let half_carry = (sp ^ nn ^ result) & 0x10 != 0;
                self.r.f.update(false, false, half_carry, carry);
                match target {
                    LoadWordTarget::HL => self.r.set_hl(result as u16),
                    _ => unimplemented!(),
                };
                self.clock.advance(12);
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
    fn handle_or(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        self.r.a |= value;
        self.r.f.update(self.r.a == 0, false, false, false);

        match source {
            ByteSource::D8 => self.clock.advance(8),
            ByteSource::HLI => self.clock.advance(8),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles POP instruction
    fn handle_pop(&mut self, target: StackTarget) -> u16 {
        let result = self.pop();
        match target {
            StackTarget::AF => self.r.set_af(result),
            StackTarget::BC => self.r.set_bc(result),
            StackTarget::DE => self.r.set_de(result),
            StackTarget::HL => self.r.set_hl(result),
        };
        self.clock.advance(12);
        self.pc.wrapping_add(1)
    }

    /// Handles PUSH instruction
    fn handle_push(&mut self, target: StackTarget) -> u16 {
        let value = match target {
            StackTarget::AF => self.r.get_af(),
            StackTarget::BC => self.r.get_bc(),
            StackTarget::DE => self.r.get_de(),
            StackTarget::HL => self.r.get_hl(),
        };
        self.push(value);
        self.clock.advance(16);
        self.pc.wrapping_add(1)
    }

    /// Handles RES instructions
    fn handle_res(&mut self, bit: u8, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        let result = utils::set_bit(value, bit, false);
        match source {
            ByteSource::A => self.r.a = result,
            ByteSource::B => self.r.b = result,
            ByteSource::C => self.r.c = result,
            ByteSource::D => self.r.d = result,
            ByteSource::E => self.r.e = result,
            ByteSource::H => self.r.h = result,
            ByteSource::L => self.r.l = result,
            ByteSource::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
            _ => unimplemented!(),
        }
        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles RET instruction
    fn handle_ret(&mut self, test: JumpTest) -> u16 {
        let should_jump = test.resolve_value(self);
        if should_jump {
            self.clock.advance(20);
            self.pop()
        } else {
            self.clock.advance(8);
            self.pc.wrapping_add(1)
        }
    }

    /// Handles RETI instruction
    fn handle_reti(&mut self) -> u16 {
        self.clock.advance(16);
        self.ime = true;
        self.pop()
    }

    /// Handles RL instructions
    /// Rotate n left through Carry flag.
    fn handle_rl(&mut self, target: PrefixTarget) -> u16 {
        let value = target.resolve_value(self);
        let carry = utils::bit_at(value, 7);
        let result = (value << 1) | carry as u8;
        self.r.f.update(result == 0, false, false, carry);
        match target {
            PrefixTarget::A => self.r.a = result,
            PrefixTarget::B => self.r.b = result,
            PrefixTarget::C => self.r.c = result,
            PrefixTarget::D => self.r.d = result,
            PrefixTarget::E => self.r.e = result,
            PrefixTarget::H => self.r.h = result,
            PrefixTarget::L => self.r.l = result,
            PrefixTarget::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
        }
        self.clock.advance(8);
        self.pc.wrapping_add(2)
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
        let value = target.resolve_value(self);
        let carry = utils::bit_at(value, 7);
        let result = (value << 1) | (value >> 7);
        self.r.f.update(result == 0, false, false, carry);

        match target {
            PrefixTarget::A => self.r.a = result,
            PrefixTarget::B => self.r.b = result,
            PrefixTarget::C => self.r.c = result,
            PrefixTarget::D => self.r.d = result,
            PrefixTarget::E => self.r.e = result,
            PrefixTarget::H => self.r.h = result,
            PrefixTarget::L => self.r.l = result,
            PrefixTarget::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
        }

        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles RLCA instruction
    fn handle_rlca(&mut self) -> u16 {
        let carry = self.r.a >> 7;
        self.r.a = self.r.a << 1 | carry;
        self.r.f.update(false, false, false, carry != 0);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RR instructions
    fn handle_rr(&mut self, target: PrefixTarget) -> u16 {
        let value = target.resolve_value(self);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) | ((self.r.f.carry as u8) << 7);
        match target {
            PrefixTarget::A => self.r.a = result,
            PrefixTarget::B => self.r.b = result,
            PrefixTarget::C => self.r.c = result,
            PrefixTarget::D => self.r.d = result,
            PrefixTarget::E => self.r.e = result,
            PrefixTarget::H => self.r.h = result,
            PrefixTarget::L => self.r.l = result,
            PrefixTarget::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
        }
        self.r.f.update(result == 0, false, false, carry);
        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles RRA instruction
    fn handle_rra(&mut self) -> u16 {
        let carry = self.r.a & 0x01 != 0;
        let result = (self.r.a >> 1) | ((self.r.f.carry as u8) << 7);
        self.r.a = result;
        self.r.f.update(false, false, false, carry);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RRC instructions
    fn handle_rrc(&mut self, target: PrefixTarget) -> u16 {
        let value = target.resolve_value(self);
        let carry = utils::bit_at(value, 0);
        let result = (value >> 1) | (value << 7);
        self.r.f.update(result == 0, false, false, carry);
        match target {
            PrefixTarget::A => self.r.a = result,
            PrefixTarget::B => self.r.b = result,
            PrefixTarget::C => self.r.c = result,
            PrefixTarget::D => self.r.d = result,
            PrefixTarget::E => self.r.e = result,
            PrefixTarget::H => self.r.h = result,
            PrefixTarget::L => self.r.l = result,
            PrefixTarget::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
        }

        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles RCAA instruction
    fn handle_rrca(&mut self) -> u16 {
        let bit = self.r.a & 1;
        self.r.a >>= 1;
        self.r.f.update(self.r.a == 0, false, false, bit == 1);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RST instructions
    fn handle_rst(&mut self, code: ResetCode) -> u16 {
        self.clock.advance(16);
        self.sp = self.sp.wrapping_sub(2);
        self.push(self.pc);
        match code {
            ResetCode::RST00 => 0x00,
            ResetCode::RST08 => 0x08,
            ResetCode::RST18 => 0x18,
            ResetCode::RST28 => 0x28,
            ResetCode::RST38 => 0x38,
        }
    }

    /// Handles SBC instructions
    fn handle_sbc(&mut self, source: ByteSource) -> u16 {
        let a = self.r.a as u32;
        let value = source.resolve_value(self) as u32;
        let result = a.wrapping_sub(value).wrapping_sub(self.r.f.carry as u32);
        self.r.a = result as u8;
        self.r.f.update(
            result as u8 == 0,
            true,
            (a ^ value ^ result) & 0x10 != 0,
            result & 0x100 != 0,
        );
        match source {
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles SCF instruction
    fn handle_scf(&mut self) -> u16 {
        self.r.f.negative = false;
        self.r.f.half_carry = false;
        self.r.f.carry = true;
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles SET instructions
    fn handle_set(&mut self, bit: u8, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        let result = utils::set_bit(value, bit, true);
        match source {
            ByteSource::A => self.r.a = result,
            ByteSource::B => self.r.b = result,
            ByteSource::C => self.r.c = result,
            ByteSource::D => self.r.d = result,
            ByteSource::E => self.r.e = result,
            ByteSource::H => self.r.h = result,
            ByteSource::L => self.r.l = result,
            ByteSource::HLI => {
                self.clock.advance(8);
                self.write(self.r.get_hl(), result)
            }
            _ => unimplemented!(),
        }
        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles SRL instructions
    fn handle_srl(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        let carry = value & 0x01 != 0;
        let result = value >> 1;
        match source {
            ByteSource::A => self.r.a = result,
            ByteSource::B => self.r.b = result,
            _ => unimplemented!(),
        }
        self.r.f.update(result == 0, false, false, carry);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles STOP instruction
    fn handle_stop(&mut self) -> u16 {
        // TODO: implement me
        self.clock.advance(4);
        self.pc.wrapping_add(2)
    }

    /// Handles SUB instructions
    fn handle_sub(&mut self, source: ByteSource) -> u16 {
        let source_value = source.resolve_value(self);
        let (new_value, did_overflow) = self.r.a.overflowing_sub(source_value);
        // Half Carry is set if adding the lower nibbles of the value and register A
        // together result in a value bigger than 0xF. If the result is larger than 0xF
        // than the addition caused a carry from the lower nibble to the upper nibble.
        self.r.f.update(
            new_value == 0,
            true,
            // TODO: verify me
            (self.r.a ^ source_value ^ new_value) & 0x10 != 0,
            did_overflow,
        );
        self.r.a = new_value;
        match source {
            ByteSource::HLI => self.clock.advance(8),
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles SWAP instructions
    fn handle_swap(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        let result = (value & 0x0F) << 4 | (value & 0xF0) >> 4;
        self.r.f.update(result == 0, false, false, false);

        match source {
            ByteSource::A => self.r.a = result,
            _ => unimplemented!(),
        }

        self.clock.advance(8);
        self.pc.wrapping_add(2)
    }

    /// Handles XOR instructions
    fn handle_xor(&mut self, source: ByteSource) -> u16 {
        let value = source.resolve_value(self);
        self.r.a ^= value;
        self.r.f.update(self.r.a == 0, false, false, false);
        match source {
            ByteSource::D8 => self.clock.advance(8),
            ByteSource::HLI => self.clock.advance(8),
            _ => self.clock.advance(4),
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
