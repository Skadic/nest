use crate::cpu6502::{Cpu6502, STACK_POINTER_BASE, IRQ_PROGRAM_COUNTER};
use crate::cpu6502::Flags6502;
use crate::cpu6502::LOOKUP;
use std::num::Wrapping;

// Opcodes. These return true if they *potentially* need another clock cycle. false otherwise
// They also set the flags accordingly
#[allow(non_snake_case, unused)]
impl Cpu6502 {
    /// Addition of the fetched value to the accumulator with carry bit
    /// This instruction can overflow the accumulator register if working with signed numbers and the value overflows.
    /// In that case the following truth table determines whether an overflow happened:
    /// Here, A is the accumulator register, M is the fetched value, R the result and V the Overflow flag. 0 = Positive value, 1 = Negative value
    /// Each letter (except V) refers to the most significant bit of the specified value
    ///
    /// | A | M | R | V |
    /// |---|---|---|---|
    /// | 0 | 0 | 0 | 0 |
    /// | 0 | 0 | 1 | 1 |
    /// | 0 | 1 | 0 | 0 |
    /// | 0 | 1 | 1 | 0 |
    /// | 1 | 0 | 0 | 0 |
    /// | 1 | 0 | 1 | 0 |
    /// | 1 | 1 | 0 | 1 |
    /// | 1 | 1 | 1 | 0 |
    ///
    /// As a result, the formula that fulfills this truth table is V = (A ^ R) & (M ^ R)
    pub fn ADC(&mut self) -> bool {
        self.fetch();
        // Add the accumulator, the fetched data, and the carry bit (Use Wrapping, to allow overflow)
        let temp: u16 = (Wrapping(self.a as u16)
            + Wrapping(self.fetched as u16)
            + Wrapping(self.get_flag(Flags6502::C) as u16))
        .0;
        // If the sum overflows, the 8-bit range, set the Carry bit
        self.set_flag(Flags6502::C, temp > 0xFF);
        // If the result of the sum (within 8-bit range) is Zero, set the Zero flag
        self.set_flag(Flags6502::Z, (temp & 0x00FF) == 0);
        // If the result is (potentially) negative, check the most significant bit and set the flag accordingly
        self.set_flag(Flags6502::N, (temp & 0x80) > 0);
        // Set the overflow flag according to the determined formula
        self.set_flag(
            Flags6502::V,
            ((self.a as u16 ^ temp) & (self.fetched as u16 ^ temp) & 0x0080) > 0,
        );

        self.a = (temp & 0x00FF) as u8;
        true
    }

    /// Performs a binary and between the accumulator and the fetched data
    pub fn AND(&mut self) -> bool {
        self.a &= self.fetch();
        // If the result is 0, set the Zero flag
        self.set_flag(Flags6502::Z, self.a == 0x00);
        // If the result is negative, set the Negative flag
        self.set_flag(Flags6502::N, self.a & 0x80 > 0);

        // Needs another clock cycle if page boundaries are crossed
        // As this is *potential* for operations, no conditionals are required
        true
    }

    /// Arithmetic left shift
    /// If Addressing mode is Implied, then the accumulator is shifted
    /// Otherwise, the value at the memory location is shifted and written back
    pub fn ASL(&mut self) -> bool {
        self.fetch();
        let temp = (self.fetched as u16) << 1;
        self.set_flag(Flags6502::C, (temp & 0xFF00) > 0);
        self.set_flag(Flags6502::Z, (temp & 0x00FF) == 0);
        self.set_flag(Flags6502::N, (temp & 0x80) > 0);
        if self.is_implied() {
            self.a = (temp & 0x00FF) as u8;
        } else {
            self.write(self.addr_abs, (temp & 0x00FF) as u8);
        }

        false
    }

    /// Branch if the carry flag of the status register is clear
    pub fn BCC(&mut self) -> bool {
        if !self.get_flag(Flags6502::C) {
            self.branch();
        }
        false
    }

    /// Branch if the carry flag of the status register is set
    pub fn BCS(&mut self) -> bool {
        if self.get_flag(Flags6502::C) {
            self.branch()
        }
        false
    }

    /// Branch if equal (i.e. the Zero flag is set)
    pub fn BEQ(&mut self) -> bool {
        if self.get_flag(Flags6502::Z) {
            self.branch()
        }
        false
    }

    /// Branch if not equal (i.e. the Zero flag is clear)
    pub fn BNE(&mut self) -> bool {
        if !self.get_flag(Flags6502::Z) {
            self.branch()
        }
        false
    }

    /// Branch if positive
    pub fn BPL(&mut self) -> bool {
        if !self.get_flag(Flags6502::N) {
            self.branch()
        }
        false
    }

    /// Branch if negative
    pub fn BMI(&mut self) -> bool {
        if self.get_flag(Flags6502::N) {
            self.branch()
        }
        false
    }

    /// Branch if overflowed
    pub fn BVC(&mut self) -> bool {
        if self.get_flag(Flags6502::V) {
            self.branch()
        }
        false
    }

    /// Branch if not overflowed
    pub fn BVS(&mut self) -> bool {
        if !self.get_flag(Flags6502::V) {
            self.branch()
        }
        false
    }

    /// I have no idea what this instruction is for
    pub fn BIT(&mut self) -> bool {
        self.fetch();
        let temp = self.a & self.fetched;
        self.set_flag(Flags6502::Z, (temp & 0x00FF) == 0x00);
        self.set_flag(Flags6502::N, (self.fetched & (1 << 7)) > 0);
        self.set_flag(Flags6502::V, (self.fetched & (1 << 6)) > 0);

        false
    }

    /// Force Break
    /// Program sourced interrupt
    pub fn BRK(&mut self) -> bool {
        self.pc += 1;

        self.set_flag(Flags6502::I, true);
        self.write(STACK_POINTER_BASE + self.stkp, (self.pc >> 8) as u8);
        self.stkp -= 1;
        self.write(STACK_POINTER_BASE + self.stkp, (self.pc & 0x00FF) as u8);
        self.stkp -= 1;

        self.set_flag(Flags6502::B, true);
        self.write(STACK_POINTER_BASE + self.stkp, self.status.bits());
        self.stkp -= 1;
        self.set_flag(Flags6502::B, false);

        self.pc = self.read(IRQ_PROGRAM_COUNTER) as u16
            | ((self.read(IRQ_PROGRAM_COUNTER + 1) as u16) << 8);
        false
    }

    /// Clear Carry flag
    pub fn CLC(&mut self) -> bool {
        self.set_flag(Flags6502::C, false);
        false
    }

    /// Clear Decimal Mode flag
    pub fn CLD(&mut self) -> bool {
        self.set_flag(Flags6502::D, false);
        false
    }

    /// Clear Interrupt Disable Flag
    pub fn CLI(&mut self) -> bool {
        self.set_flag(Flags6502::I, false);
        false
    }

    /// Clear Overflow Flag
    pub fn CLV(&mut self) -> bool {
        self.set_flag(Flags6502::V, false);
        false
    }

    /// Compares the accumulator to memory
    /// Operation:  
    /// C <- acc >= mem  
    /// Z <- (acc - mem) == 0  
    /// N <- (acc - mem) < 0 (as in: the sign is 1)
    pub fn CMP(&mut self) -> bool {
        self.fetch();
        // Make Rust allow overflow/underflow
        let value = (Wrapping(self.a as u16) - Wrapping(self.fetched as u16)).0;
        self.set_flag(Flags6502::C, self.a >= self.fetched);
        self.set_flag(Flags6502::Z, (value & 0x00FF) == 0);
        self.set_flag(Flags6502::N, (value & 0x0080) > 0);
        true
    }

    /// Compares the X-register to memory
    /// Operation:  
    /// C <- x >= mem  
    /// Z <- (x - mem) == 0  
    /// N <- (x - mem) < 0 (as in: the sign is 1)
    pub fn CPX(&mut self) -> bool {
        self.fetch();
        let value = (Wrapping(self.x as u16) - Wrapping(self.fetched as u16)).0;
        self.set_flag(Flags6502::C, self.x >= self.fetched);
        self.set_flag(Flags6502::Z, (value & 0x00FF) == 0);
        self.set_flag(Flags6502::N, (value & 0x0080) > 0);
        false
    }

    /// Compares the Y-register to memory
    /// Operation:  
    /// C <- y >= mem  
    /// Z <- (y - mem) == 0  
    /// N <- (y - mem) < 0 (as in: the sign is 1)
    pub fn CPY(&mut self) -> bool {
        self.fetch();
        let value = (Wrapping(self.y as u16) - Wrapping(self.fetched as u16)).0;
        self.set_flag(Flags6502::C, self.y >= self.fetched);
        self.set_flag(Flags6502::Z, (value & 0x00FF) == 0);
        self.set_flag(Flags6502::N, (value & 0x0080) > 0);
        false
    }

    /// Decrement value at memory location
    pub fn DEC(&mut self) -> bool {
        self.fetch();

        let value = self.fetched - 1;
        self.write(self.addr_abs, value);

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        false
    }

    /// Decrements the X-register by 1
    pub fn DEX(&mut self) -> bool {
        self.x -= 1;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);
        false
    }

    /// Decrements the Y register by 1
    pub fn DEY(&mut self) -> bool {
        self.y -= 1;
        self.set_flag(Flags6502::Z, self.y == 0);
        self.set_flag(Flags6502::N, (self.y & 0x80) > 0);
        false
    }

    /// Exclusive or of memory with accumulator
    pub fn EOR(&mut self) -> bool {
        self.fetch();
        self.a ^= self.fetched;
        self.set_flag(Flags6502::Z, self.a == 0);
        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);
        true
    }

    /// Increments memory location by 1
    pub fn INC(&mut self) -> bool {
        self.fetch();

        let value = self.fetched + 1;
        self.write(self.addr_abs, value);

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        false
    }

    /// Increments the X-register by 1
    pub fn INX(&mut self) -> bool {
        self.x += 1;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);
        false
    }

    /// Increments the Y register by 1
    pub fn INY(&mut self) -> bool {
        self.y += 1;
        self.set_flag(Flags6502::Z, self.y == 0);
        self.set_flag(Flags6502::N, (self.y & 0x80) > 0);
        false
    }

    /// Jump to memory location without saving return address
    pub fn JMP(&mut self) -> bool {
        self.pc = self.addr_abs;
        false
    }

    /// Jump to memory location *with* saving return address
    pub fn JSR(&mut self) -> bool {
        // Write current program counter to stack
        self.pc -= 1;
        self.write(STACK_POINTER_BASE + self.stkp, (self.pc >> 8) as u8);
        self.stkp -= 1;
        self.write(STACK_POINTER_BASE + self.stkp, (self.pc & 0x00FF) as u8);
        self.stkp -= 1;

        // Jump to new address
        self.pc = self.addr_abs;
        false
    }

    /// Load accumulator from memory
    pub fn LDA(&mut self) -> bool {
        self.fetch();
        self.a = self.fetched;
        self.set_flag(Flags6502::Z, self.a == 0);
        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);
        true
    }

    /// Load X register from memory
    pub fn LDX(&mut self) -> bool {
        self.fetch();
        self.x = self.fetched;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);
        true
    }

    /// Load Y register from memory
    pub fn LDY(&mut self) -> bool {
        self.fetch();
        self.y = self.fetched;
        self.set_flag(Flags6502::Z, self.y == 0);
        self.set_flag(Flags6502::N, (self.y & 0x80) > 0);
        true
    }

    /// Shift memory or accumulator 1 bit right
    pub fn LSR(&mut self) -> bool {
        self.fetch();

        let value = self.fetched >> 1;
        self.set_flag(Flags6502::N, false); // First bit will always be zero
        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::C, self.fetched & 1 > 0); // If 1 bit is lost by shifting right

        if self.is_implied() {
            self.a = value;
        } else {
            self.write(self.addr_abs, value);
        }

        false
    }

    /// No operation
    pub fn NOP(&mut self) -> bool {
        false
    }

    /// Or memory with accumulator
    pub fn ORA(&mut self) -> bool {
        self.fetch();
        self.a |= self.fetched;

        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);
        self.set_flag(Flags6502::Z, self.a == 0);
        true
    }

    // Push accumulator to the stack
    pub fn PHA(&mut self) -> bool {
        self.write(STACK_POINTER_BASE + self.stkp, self.a);
        self.stkp -= 1;
        false
    }

    /// Push processor status on stack
    pub fn PHP(&mut self) -> bool {
        self.write(STACK_POINTER_BASE + self.stkp, self.status.bits());
        self.stkp -= 1;
        false
    }

    // Pop off the stack into the accumulator
    pub fn PLA(&mut self) -> bool {
        self.stkp += 1;
        self.a = self.read(STACK_POINTER_BASE + self.stkp);
        self.set_flag(Flags6502::Z, self.a == 0);
        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);
        false
    }

    /// Pull processor status from stack
    pub fn PLP(&mut self) -> bool {
        self.stkp += 1;
        self.status = Flags6502::from_bits(self.read(STACK_POINTER_BASE + self.stkp)).unwrap();
        self.set_flag(Flags6502::U, true);
        false
    }

    /// Rotate 1 bit left (Memory or accumulator)
    /// E.g. 100101 -> 001011
    pub fn ROL(&mut self) -> bool {
        self.fetch();

        // Shift the fetched value to the left by 1
        let mut value = ((self.fetched as u16) << 1);
        // Add a 1 as the least significant bit, if a 1 was "shifted out of the 8-bit bounds"
        value |= ((value & 0x100) > 0) as u16;

        self.set_flag(Flags6502::C, (value & 0xFF00) > 0);

        // Bring the value back to the u8 range
        let value = (value & 0xFF) as u8;

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        if self.is_implied() {
            self.a = value;
        } else {
            self.write(self.addr_abs, value);
        }

        false
    }

    /// Rotate 1 bit right (Memory or accumulator)
    /// E.g. 100101 -> 110010
    pub fn ROR(&mut self) -> bool {
        self.fetch();

        // Shift the fetched value to the right by 1
        let mut value = (self.fetched >> 1);
        // Add a 1 as the most significant bit, if a 1 was "shifted out"
        value |= (self.fetched & 1) << 7;

        self.set_flag(Flags6502::C, (self.fetched & 1) > 0);

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        if self.is_implied() {
            self.a = value;
        } else {
            self.write(self.addr_abs, value);
        }

        false
    }

    /// Return from interrupt.
    /// Get the status register and the program counter from stack
    pub fn RTI(&mut self) -> bool {
        self.stkp += 1;
        self.status = Flags6502::from_bits(self.read(STACK_POINTER_BASE + self.stkp)).unwrap();
        self.status &= !Flags6502::B;
        self.status &= !Flags6502::U;

        self.stkp += 1;
        let lo = self.read(STACK_POINTER_BASE + self.stkp) as u16;
        self.stkp += 1;
        let hi = self.read(STACK_POINTER_BASE + self.stkp) as u16;
        self.pc = (hi << 8) | lo;

        false
    }

    /// Return from Subroutine
    /// Returns to a saved program counter after jumping there (see JSR)
    pub fn RTS(&mut self) -> bool {
        self.stkp += 1;
        let lo = self.read(STACK_POINTER_BASE + self.stkp) as u16;
        self.stkp += 1;
        let hi = self.read(STACK_POINTER_BASE + self.stkp) as u16;
        self.pc = (hi << 8) | lo;
        false
    }

    /// Subtraction of the fetched value from the accumulator with carry bit (which is a borrow bit in this case)
    /// The Operation is `A = A - M - (1 - C)`
    /// This can also be written as `A = A + -M - 1 + C`, so Addition Hardware can be reused
    ///
    /// Because -M = ~M + 1 in binary representation, A = A + -M - 1 + C = A + ~M + C
    pub fn SBC(&mut self) -> bool {
        self.fetch();

        // Invert M
        let value = Wrapping((self.fetched as u16) ^ 0x00FF);

        // Add just like in ADC
        let temp: u16 =
            (Wrapping(self.a as u16) + value + Wrapping(self.get_flag(Flags6502::C) as u16)).0;
        self.set_flag(Flags6502::C, temp > 0xFF);
        self.set_flag(Flags6502::Z, (temp & 0x00FF) == 0);
        self.set_flag(Flags6502::N, (temp & 0x80) > 0);
        self.set_flag(
            Flags6502::V,
            ((self.a as u16 ^ temp) & (self.fetched as u16 ^ temp) & 0x0080) > 0,
        );

        self.a = (temp & 0x00FF) as u8;
        true
    }

    /// Set Carry flag
    pub fn SEC(&mut self) -> bool {
        self.set_flag(Flags6502::C, true);
        false
    }

    /// Set Decimal flag
    pub fn SED(&mut self) -> bool {
        self.set_flag(Flags6502::D, true);
        false
    }

    /// Set "Disable Interrupts" flag
    pub fn SEI(&mut self) -> bool {
        self.set_flag(Flags6502::I, true);
        false
    }

    /// Store accumulator in memory
    pub fn STA(&mut self) -> bool {
        self.write(self.addr_abs, self.a);
        false
    }

    /// Store X register in memory
    pub fn STX(&mut self) -> bool {
        self.write(self.addr_abs, self.x);
        false
    }

    /// Store Y register in memory
    pub fn STY(&mut self) -> bool {
        self.write(self.addr_abs, self.y);
        false
    }

    /// Transfer the accumulator to the X register
    pub fn TAX(&mut self) -> bool {
        self.x = self.a;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);

        false
    }

    /// Transfer the accumulator to the X register
    pub fn TAY(&mut self) -> bool {
        self.y = self.a;
        self.set_flag(Flags6502::Z, self.a == 0);
        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);

        false
    }

    /// Transfer Stack Pointer to X register
    pub fn TSX(&mut self) -> bool {
        self.x = (self.stkp & 0xFF) as u8;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);

        false
    }

    /// Transfer the X register to the accumulator
    pub fn TXA(&mut self) -> bool {
        self.a = self.x;
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);

        false
    }

    /// Transfer the X register to the Stack Pointer register
    pub fn TXS(&mut self) -> bool {
        self.stkp = self.x as u16;
        false
    }

    /// Transfer the Y register to the accumulator
    pub fn TYA(&mut self) -> bool {
        self.a = self.y;
        self.set_flag(Flags6502::Z, self.y == 0);
        self.set_flag(Flags6502::N, (self.y & 0x80) > 0);

        false
    }

    // Illegal Opcode
    pub fn XXX(&mut self) -> bool {
        false
    }

    /// Branch method, because all branches *basically* work the same, just with different branch conditions
    pub fn branch(&mut self) {
        // Uses 1 more cycle for branching
        self.cycles += 1;

        // Calculate jump address
        let new_addr = self.pc + self.addr_rel;

        // If the branch requires crossing a page boundary, it requires 1 more cycle
        if (new_addr & 0xFF00) != (self.pc & 0xFF00) {
            self.cycles += 1;
        }

        self.pc = new_addr;
    }

    /// Returns true if the current addressing mode is implied (see Cpu6502::IMP())
    pub fn is_implied(&self) -> bool {
        LOOKUP[self.opcode as usize].addrmode as usize == Self::IMP as usize
    }
}