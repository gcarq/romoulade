use crate::gb::instruction::*;
use crate::gb::memory::constants::BOOT_END;
use crate::gb::timer::Clock;
use crate::gb::AddressSpace;
use crate::utils;
use registers::Registers;
use std::cell::RefCell;

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
    bus: &'a RefCell<T>,
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

        self.sanity_check(self.pc);
        // Read next opcode from memory
        let opcode = self.read(self.pc);
        let (opcode, prefixed) = match opcode == 0xCB {
            true => (self.read(self.pc + 1), true),
            false => (opcode, false),
        };

        // Parse instruction from opcode, execute it and update program counter
        self.pc = match Instruction::from_byte(opcode, prefixed) {
            Some(instruction) => self.execute(instruction),
            None => {
                let description = format!("0x{}{:02x}", if prefixed { "cb" } else { "" }, opcode);
                panic!("Unresolved instruction: {}.\nHALTED!", description);
            }
        };
        self.clock.ticks()
    }

    /*/// Prints the current registers, pointers with opcode and resolved instruction
    fn print_ctx(&self, opcode: u8, instruction: &Instruction) {
        println!(
            "{} | pc: {:<5} sp: {:<5} | op: {:#04X} -> {}",
            self.r,
            format!("{:#06X}", self.pc),
            format!("{:#06X}", self.sp),
            opcode,
            instruction
        );
    }*/

    /// Executes the given instruction, advances the internal clock
    /// and returns the updated program counter.
    fn execute(&mut self, instruction: Instruction) -> u16 {
        match instruction {
            Instruction::ADD(source) => self.handle_add(source),
            Instruction::ADDHL(source) => self.handle_add_hl(source),
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
            Instruction::DEC(target) => self.handle_dec_byte(target),
            Instruction::DEC2(target) => self.handle_dec_word(target),
            Instruction::EI => self.handle_interrupt(true),
            Instruction::HALT => self.handle_halt(),
            Instruction::INC(target) => self.handle_inc_byte(target),
            Instruction::INC2(target) => self.handle_inc_word(target),
            Instruction::JR(test) => self.handle_jr(test),
            Instruction::JP(test, source) => self.handle_jp(test, source),
            Instruction::LD(load_type) => self.handle_ld(load_type),
            Instruction::NOP => self.handle_nop(),
            Instruction::OR(source) => self.handle_or(source),
            Instruction::RES(bit, source) => self.handle_res(bit, source),
            Instruction::RET(test) => self.handle_ret(test),
            Instruction::RETI => self.handle_reti(),
            Instruction::RL(source) => self.handle_rl(source),
            Instruction::RLA => self.handle_rla(),
            Instruction::RLC(source) => self.handle_rlc(source),
            Instruction::RLCA => self.handle_rlca(),
            Instruction::RR(source) => self.handle_rr(source),
            Instruction::RRA => self.handle_rra(),
            Instruction::RRC(source) => self.handle_rrc(source),
            Instruction::RRCA => self.handle_rrca(),
            Instruction::RST(code) => self.handle_rst(code),
            Instruction::SBC(source) => self.handle_sbc(source),
            Instruction::SCF => self.handle_scf(),
            Instruction::SET(bit, source) => self.handle_set(bit, source),
            Instruction::SLA(source) => self.handle_sla(source),
            Instruction::SRA(source) => self.handle_sra(source),
            Instruction::SRL(source) => self.handle_srl(source),
            Instruction::STOP => self.handle_stop(),
            Instruction::SUB(source) => self.handle_sub(source),
            Instruction::SWAP(source) => self.handle_swap(source),
            Instruction::PUSH(target) => self.handle_push(target),
            Instruction::POP(target) => self.handle_pop(target),
            Instruction::XOR(source) => self.handle_xor(source),
        }
    }

    /// Sanity check to verify boot ROM executed successfully
    fn sanity_check(&self, address: u16) {
        // TODO: make check more robust, might be possible that we read from that address later on as well.
        if address == BOOT_END + 1 {
            assert_eq!(self.r.get_af(), 0x01B0, "AF is invalid, boot ROM failure!");
            assert_eq!(self.r.get_bc(), 0x0013, "BC is invalid, boot ROM failure!");
            assert_eq!(self.r.get_de(), 0x00D8, "DE is invalid, boot ROM failure!");
            assert_eq!(self.r.get_hl(), 0x014D, "HL is invalid, boot ROM failure!");
            assert_eq!(self.sp, 0xFFFE, "SP is invalid, boot ROM failure!");
            //TODO: debug log: println!("Done with processing boot ROM. Switching to Cartridge...");
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
        let source_value = source.read(self);
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

    /// Handles ADD HL, nn instructions
    fn handle_add_hl(&mut self, source: WordSource) -> u16 {
        let value = source.read(self);
        let hl = self.r.get_hl();
        let (result, overflow) = hl.overflowing_add(value);

        let half_carry = (hl ^ value ^ result) & 0x1000 != 0;
        self.r.f.negative = false;
        self.r.f.half_carry = half_carry;
        self.r.f.carry = overflow;
        self.r.set_hl(result);

        self.clock.advance(8);
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
        let value = source.read(self);
        let half_carry = ((self.r.a & 0x0F) + (value & 0x0F) + self.r.f.carry as u8) > 0x0F;

        let (result, overflow) = self.r.a.overflowing_add(value);
        let mut carry = overflow;
        let (result, overflow) = result.overflowing_add(self.r.f.carry as u8);
        carry |= overflow;
        self.r.f.update(result == 0, false, half_carry, carry);
        self.r.a = result;

        match source {
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles AND instructions
    fn handle_and(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        self.r.a &= value;
        self.r.f.update(self.r.a == 0, false, true, false);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles BIT instructions
    fn handle_bit(&mut self, bit: u8, source: ByteSource) -> u16 {
        let value = source.read(self);
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
        let should_jump = test.resolve(self);
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
        let value = source.read(self);
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
        if self.r.f.negative {
            if self.r.f.carry {
                self.r.a = self.r.a.wrapping_sub(0x60);
            }
            if self.r.f.half_carry {
                self.r.a = self.r.a.wrapping_sub(0x06);
            }
        } else {
            if self.r.f.carry || self.r.a > 0x99 {
                self.r.a = self.r.a.wrapping_add(0x60);
                self.r.f.carry = true;
            }
            if self.r.f.half_carry || (self.r.a & 0x0F) > 0x09 {
                self.r.a = self.r.a.wrapping_add(0x06);
            }
        }
        self.r.f.zero = self.r.a == 0;
        self.r.f.half_carry = false;

        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles DEC instructions for bytes
    fn handle_dec_byte(&mut self, target: IncDecByteTarget) -> u16 {
        let value = target.read(self);
        let result = value.wrapping_sub(1);
        target.write(self, result);
        self.r.f.half_carry = value.trailing_zeros() >= 4;
        self.r.f.zero = result == 0;
        self.r.f.negative = true;
        match target {
            IncDecByteTarget::HLI => self.clock.advance(12),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles DEC instructions for words
    fn handle_dec_word(&mut self, target: IncDecWordTarget) -> u16 {
        let value = target.read(self);
        let result = value.wrapping_sub(1);
        target.write(self, result);
        self.clock.advance(8);
        self.pc.wrapping_add(1)
    }

    /// Handles HALT instruction
    fn handle_halt(&mut self) -> u16 {
        self.is_halted = true;
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles INC instructions for bytes
    fn handle_inc_byte(&mut self, target: IncDecByteTarget) -> u16 {
        let value = target.read(self);
        let result = value.wrapping_add(1);
        target.write(self, result);
        self.r.f.half_carry = value & 0x0F == 0x0F;
        self.r.f.zero = result == 0;
        self.r.f.negative = false;
        match target {
            IncDecByteTarget::HLI => self.clock.advance(12),
            _ => self.clock.advance(4),
        }
        self.pc.wrapping_add(1)
    }

    /// Handles INC instructions for words
    fn handle_inc_word(&mut self, target: IncDecWordTarget) -> u16 {
        let value = target.read(self);
        let result = value.wrapping_add(1);
        target.write(self, result);
        self.clock.advance(8);
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
        let should_jump = test.resolve(self);

        if should_jump {
            self.clock.advance(12);
            let offset = self.consume_byte() as i8;
            (self.pc as i16).wrapping_add(1).wrapping_add(offset as i16) as u16
        } else {
            self.clock.advance(8);
            // If we don't jump we need to still move the program
            // counter forward by 2 since the rel jump instruction is
            // 2 bytes wide (1 byte for tag and 1 bytes for jump address)
            self.pc.wrapping_add(2)
        }
    }

    /// Handles JP instructions
    fn handle_jp(&mut self, test: JumpTest, source: WordSource) -> u16 {
        let should_jump = test.resolve(self);

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
    fn handle_ld(&mut self, load_type: Load) -> u16 {
        match load_type {
            Load::Byte(target, source) => {
                let value = source.read(self);
                match target {
                    LoadByteTarget::A => self.r.a = value,
                    LoadByteTarget::B => self.r.b = value,
                    LoadByteTarget::C => self.r.c = value,
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
                match source {
                    ByteSource::CIFF00 => self.clock.advance(4),
                    ByteSource::D8 => self.clock.advance(4),
                    ByteSource::D8IFF00 => self.clock.advance(8),
                    ByteSource::HLI => self.clock.advance(4),
                    ByteSource::D16I => self.clock.advance(12),
                    _ => {}
                }
                if let LoadByteTarget::HLI = target {
                    self.clock.advance(4)
                }
                self.clock.advance(4);
                self.pc.wrapping_add(1)
            }
            Load::Word(target, source) => {
                let value = source.read(self);
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
            Load::IndirectFrom(target, source) => {
                let value = source.read(self);
                let addr = match target {
                    LoadByteTarget::BCI => self.r.get_bc(),
                    LoadByteTarget::DEI => self.r.get_de(),
                    LoadByteTarget::D16I => self.consume_word(),
                    LoadByteTarget::HLI => self.r.get_hl(),
                    LoadByteTarget::CIFF00 => {
                        self.clock.advance(8);
                        u16::from(self.r.c) | 0xFF00
                    }
                    LoadByteTarget::D8IFF00 => {
                        self.clock.advance(4);
                        u16::from(self.consume_byte()) | 0xFF00
                    }
                    _ => unimplemented!(),
                };
                self.write(addr, value);
                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            Load::IndirectFromAInc(target) => {
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
            Load::IndirectFromADec(target) => {
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
            Load::IndirectFromWord(target, source) => {
                let value = source.read(self);
                match target {
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
            Load::FromIndirectAInc(source) => {
                self.r.a = source.read(self);
                match source {
                    ByteSource::HLI => self.r.set_hl(self.r.get_hl().wrapping_add(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            Load::FromIndirectADec(source) => {
                self.r.a = source.read(self);
                match source {
                    ByteSource::HLI => self.r.set_hl(self.r.get_hl().wrapping_sub(1)),
                    _ => unimplemented!(),
                }

                self.clock.advance(8);
                self.pc.wrapping_add(1)
            }
            Load::IndirectFromSPi8(target) => {
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
        let value = source.read(self);
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
        let value = source.read(self);
        let result = utils::set_bit(value, bit, false);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles RET instruction
    fn handle_ret(&mut self, test: JumpTest) -> u16 {
        let should_jump = test.resolve(self);
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
    fn handle_rl(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x80 != 0;
        let result = (value << 1) | self.r.f.carry as u8;
        self.r.f.update(result == 0, false, false, carry);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles RLA instruction
    /// Rotate A left through carry
    fn handle_rla(&mut self) -> u16 {
        let new_carry = (self.r.a >> 7) != 0;
        self.r.a = (self.r.a << 1) | self.r.f.carry as u8;
        self.r.f.update(false, false, false, new_carry);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RLC instructions
    /// Rotates register to the left and updates CPU flags
    fn handle_rlc(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x80 != 0;
        let result = (value << 1) | (value >> 7);
        self.r.f.update(result == 0, false, false, carry);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles RLCA instruction
    fn handle_rlca(&mut self) -> u16 {
        let carry = self.r.a & 0x80 != 0;
        self.r.a = (self.r.a << 1) | carry as u8;
        self.r.f.update(false, false, false, carry);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RR instructions
    fn handle_rr(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) | (u8::from(self.r.f.carry) << 7);
        source.write(self, result);
        self.r.f.update(result == 0, false, false, carry);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles RRA instruction
    fn handle_rra(&mut self) -> u16 {
        let carry = self.r.a & 0x01 != 0;
        self.r.a = (self.r.a >> 1) | (u8::from(self.r.f.carry) << 7);
        self.r.f.update(false, false, false, carry);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RRC instructions
    fn handle_rrc(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) | (value << 7);

        self.r.f.update(result == 0, false, false, carry);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles RCAA instruction
    fn handle_rrca(&mut self) -> u16 {
        let carry = self.r.a & 0x01;
        self.r.a = (self.r.a >> 1) | (carry << 7);
        self.r.f.update(false, false, false, carry != 0);
        self.clock.advance(4);
        self.pc.wrapping_add(1)
    }

    /// Handles RST instructions
    fn handle_rst(&mut self, code: ResetCode) -> u16 {
        self.clock.advance(16);
        self.push(self.pc.wrapping_add(1));
        match code {
            ResetCode::RST00 => 0x00,
            ResetCode::RST10 => 0x10,
            ResetCode::RST08 => 0x08,
            ResetCode::RST18 => 0x18,
            ResetCode::RST20 => 0x20,
            ResetCode::RST28 => 0x28,
            ResetCode::RST30 => 0x30,
            ResetCode::RST38 => 0x38,
        }
    }

    /// Handles SBC instructions
    fn handle_sbc(&mut self, source: ByteSource) -> u16 {
        let a = self.r.a as u32;
        let value = source.read(self) as u32;
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
        let value = source.read(self);
        let result = utils::set_bit(value, bit, true);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles SLA instructions
    fn handle_sla(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x80 != 0;
        let result = value << 1;
        self.r.f.update(result == 0, false, false, carry);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles SRA instructions
    fn handle_sra(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x01 != 0;
        let result = (value >> 1) | (value & 0x80);
        self.r.f.update(result == 0, false, false, carry);
        source.write(self, result);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles SRL instructions
    fn handle_srl(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        let carry = value & 0x01 != 0;
        let result = value >> 1;
        source.write(self, result);
        self.r.f.update(result == 0, false, false, carry);

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles STOP instruction
    fn handle_stop(&mut self) -> u16 {
        panic!("STOP is not implemented");
        self.clock.advance(4);
        self.pc.wrapping_add(2)
    }

    /// Handles SUB instructions
    fn handle_sub(&mut self, source: ByteSource) -> u16 {
        let a = u16::from(self.r.a);
        let value = u16::from(source.read(self));
        let result = a.wrapping_sub(value);

        let carry_bits = a ^ value ^ result;
        let half_carry = carry_bits & 0x10 != 0;
        let carry = carry_bits & 0x100 != 0;
        self.r.f.update(result == 0, true, half_carry, carry);
        self.r.a = result as u8;
        match source {
            ByteSource::HLI => self.clock.advance(8),
            ByteSource::D8 => self.clock.advance(8),
            _ => self.clock.advance(4),
        }

        self.pc.wrapping_add(1)
    }

    /// Handles SWAP instructions
    fn handle_swap(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
        self.r.f.update(value == 0, false, false, false);
        source.write(self, (value << 4) | (value >> 4));

        match source {
            ByteSource::HLI => self.clock.advance(16),
            _ => self.clock.advance(8),
        }
        self.pc.wrapping_add(2)
    }

    /// Handles XOR instructions
    fn handle_xor(&mut self, source: ByteSource) -> u16 {
        let value = source.read(self);
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
