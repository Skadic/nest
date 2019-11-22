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

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use crate::cpu6502::Flags6502;
    use crate::cpu6502::{Cpu6502, IRQ_PROGRAM_COUNTER, STACK_POINTER_BASE};

    use crate::bus;
    use std::cell::RefCell;
    use std::rc::Rc;

    #[test]
    fn ADC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.a = 5;
        cpu.IMP();

        cpu.addr_abs = 0x1111;
        cpu.STA();
        cpu.ADC();
        assert_eq!(cpu.a, 10, "Addition failed");
        assert_eq!(cpu.status, Flags6502::empty(), "Status does not match");

        // Carry check
        cpu.a = 255;
        cpu.ADC();
        assert_eq!(cpu.a, 4, "Addition failed");
        assert_eq!(cpu.status, Flags6502::C, "Status does not match");

        cpu.CLC();

        // Overflow check. As 130 is out of the [-128, 127] range, it overflows into the negative. Thus the N flag should be set
        cpu.a = 125;
        cpu.ADC();
        assert_eq!(cpu.a, 130, "Addition failed");
        assert_eq!(
            cpu.status,
            Flags6502::V | Flags6502::N,
            "Status does not match"
        );
    }

    #[test]
    fn AND_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 0b0101;
        cpu.x = 0b0110;
        cpu.addr_abs = 0x1111;

        cpu.opcode = 0x0E;

        cpu.STX();
        cpu.AND();
        assert_eq!(cpu.a, 0b0100, "AND operation failed");
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn ASL_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 0b1100_0110;
        cpu.fetched = cpu.a;

        // The opcode of ASL with implied addressing
        cpu.opcode = 0x0A;
        cpu.ASL();
        assert_eq!(
            cpu.a, 0b1000_1100,
            "Left shift of accumulator failed: {:b} vs {:b}",
            cpu.a, 0b1000_1100
        );

        // The opcode of ASL with absolute addressing (although that doesn't really matter since we're setting the address manually anyway)
        // All that matters is that the addressing mode of the specified instruction is not 'implied'
        cpu.opcode = 0x0E;
        cpu.a = 0b1100_0110;
        cpu.addr_abs = 0x1111;
        cpu.STA();
        cpu.ASL();

        assert_eq!(
            bus.borrow().read(0x1111, false),
            0b1000_1100,
            "Left shift in memory failed: {:b}, vs {:b}",
            bus.borrow().read(0x1111, false),
            0b1000_1101
        );
    }

    #[test]
    fn BCC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::C, true);
        cpu.BCC();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being clear");
        cpu.set_flag(Flags6502::C, false);
        cpu.BCC();
        assert_eq!(
            cpu.pc, 300,
            "branch did not happen, despite flag being clear"
        );
    }

    #[test]
    fn BCS_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::C, false);
        cpu.BCS();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being set");
        cpu.set_flag(Flags6502::C, true);
        cpu.BCS();
        assert_eq!(cpu.pc, 300, "branch did not happen, despite flag being set");
    }

    #[test]
    fn BEQ_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::Z, false);
        cpu.BEQ();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being set");
        cpu.set_flag(Flags6502::Z, true);
        cpu.BEQ();
        assert_eq!(cpu.pc, 300, "branch did not happen, despite flag being set");
    }

    #[test]
    fn BNE_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::Z, true);
        cpu.BNE();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being clear");
        cpu.set_flag(Flags6502::Z, false);
        cpu.BNE();
        assert_eq!(
            cpu.pc, 300,
            "branch did not happen, despite flag being clear"
        );
    }

    #[test]
    fn BPL_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::N, true);
        cpu.BPL();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being clear");
        cpu.set_flag(Flags6502::N, false);
        cpu.BPL();
        assert_eq!(
            cpu.pc, 300,
            "branch did not happen, despite flag being clear"
        );
    }

    #[test]
    fn BMI_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::N, false);
        cpu.BMI();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being set");
        cpu.set_flag(Flags6502::N, true);
        cpu.BMI();
        assert_eq!(cpu.pc, 300, "branch did not happen, despite flag being set");
    }

    #[test]
    fn BVC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::V, false);
        cpu.BVC();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being set");
        cpu.set_flag(Flags6502::V, true);
        cpu.BVC();
        assert_eq!(cpu.pc, 300, "branch did not happen, despite flag being set");
    }

    #[test]
    fn BVS_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.pc = 200;
        cpu.addr_rel = 100;
        cpu.set_flag(Flags6502::V, true);
        cpu.BVS();
        assert_eq!(cpu.pc, 200, "branch happened, despite flag not being clear");
        cpu.set_flag(Flags6502::V, false);
        cpu.BVS();
        assert_eq!(
            cpu.pc, 300,
            "branch did not happen, despite flag being clear"
        );
    }

    #[test]
    fn BIT_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.addr_abs = 0x1111;
        cpu.opcode = 0x0E;
        cpu.write(cpu.addr_abs, 0xFF);
        cpu.BIT();

        assert_eq!(
            cpu.status,
            Flags6502::Z | Flags6502::N | Flags6502::V,
            "Status does not match"
        )
    }

    #[test]
    fn BRK_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        // Write lo of the jump address
        cpu.write(IRQ_PROGRAM_COUNTER, 0x20);
        // Write hi of the jump adress
        cpu.write(IRQ_PROGRAM_COUNTER + 1, 0x10);
        // The current program counter (pc + 1 will be written to memory)
        cpu.pc = 0x1233;

        cpu.status = Flags6502::N | Flags6502::Z;

        cpu.BRK();

        assert_eq!(
            Flags6502::from_bits(cpu.read(STACK_POINTER_BASE + cpu.stkp + 1)).unwrap(),
            Flags6502::N | Flags6502::Z | Flags6502::B | Flags6502::I
        );
        assert_eq!(
            cpu.read(STACK_POINTER_BASE + cpu.stkp + 2),
            0x34,
            "lo nibble of pc incorrect"
        );
        assert_eq!(
            cpu.read(STACK_POINTER_BASE + cpu.stkp + 3),
            0x12,
            "hi nibble of pc incorrect"
        );
        assert_eq!(cpu.pc, 0x1020, "new jump address incorrect");
    }

    #[test]
    fn clear_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.status = Flags6502::C | Flags6502::D | Flags6502::I | Flags6502::V;

        cpu.CLC();
        assert_eq!(cpu.status, Flags6502::D | Flags6502::I | Flags6502::V);
        cpu.CLD();
        assert_eq!(cpu.status, Flags6502::I | Flags6502::V);
        cpu.CLI();
        assert_eq!(cpu.status, Flags6502::V);
        cpu.CLV();
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn CMP_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.addr_abs = 0x1111;

        // Random opcode, that has absolute addressing
        cpu.opcode = 0x0E;

        // Test acc greater
        cpu.write(0x1111, 10);
        cpu.a = 20;
        cpu.CMP();
        assert_eq!(cpu.status, Flags6502::C);

        // Test acc equal to memory
        cpu.write(0x1111, 10);
        cpu.a = 10;
        cpu.CMP();
        assert_eq!(cpu.status, Flags6502::C | Flags6502::Z);

        // Test acc lesser than memory
        cpu.write(0x1111, 10);
        cpu.a = 0;
        cpu.CMP();
        assert_eq!(cpu.status, Flags6502::N);
    }

    #[test]
    fn CPX_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.addr_abs = 0x1111;

        // Random opcode, that has absolute addressing
        cpu.opcode = 0x0E;

        // Test x greater
        cpu.write(0x1111, 10);
        cpu.x = 20;
        cpu.CPX();
        assert_eq!(cpu.status, Flags6502::C);

        // Test x equal to memory
        cpu.write(0x1111, 10);
        cpu.x = 10;
        cpu.CPX();
        assert_eq!(cpu.status, Flags6502::C | Flags6502::Z);

        // Test acc lesser than memory
        cpu.write(0x1111, 10);
        cpu.x = 0;
        cpu.CPX();
        assert_eq!(cpu.status, Flags6502::N);
    }

    #[test]
    fn CPY_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.addr_abs = 0x1111;

        // Random opcode, that has absolute addressing
        cpu.opcode = 0x0E;

        // Test y greater
        cpu.write(0x1111, 10);
        cpu.y = 20;
        cpu.CPY();
        assert_eq!(cpu.status, Flags6502::C);

        // Test x equal to memory
        cpu.write(0x1111, 10);
        cpu.y = 10;
        cpu.CPY();
        assert_eq!(cpu.status, Flags6502::C | Flags6502::Z);

        // Test acc lesser than memory
        cpu.write(0x1111, 10);
        cpu.y = 0;
        cpu.CPY();
        assert_eq!(cpu.status, Flags6502::N);
    }

    #[test]
    fn DEC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.addr_abs = 0x1111;

        // Random opcode, that has absolute addressing
        cpu.opcode = 0x0E;

        // Write 10 to memory location 0x1111
        cpu.write(0x1111, 10);
        cpu.DEC();

        // Test if decrement worked
        assert_eq!(cpu.read(0x1111), 9);
    }

    #[test]
    fn DEX_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.x = 10;
        cpu.DEX();

        // Test if decrement worked
        assert_eq!(cpu.x, 9);
    }

    #[test]
    fn DEY_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.y = 10;
        cpu.DEY();

        // Test if decrement worked
        assert_eq!(cpu.y, 9);
    }

    #[test]
    fn EOR_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 0b0101;
        cpu.x = 0b0110;
        cpu.addr_abs = 0x1111;
        cpu.opcode = 0x0E;

        cpu.STX();
        cpu.EOR();
        assert_eq!(cpu.a, 0b0011, "XOR operation failed");
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn INC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.addr_abs = 0x1111;

        // Random opcode, that has absolute addressing
        cpu.opcode = 0x0E;

        // Write 10 to memory location 0x1111
        cpu.write(0x1111, 10);
        cpu.INC();

        // Test if increment worked
        assert_eq!(cpu.read(0x1111), 11);
    }

    #[test]
    fn INX_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.x = 10;
        cpu.INX();

        // Test if increment worked
        assert_eq!(cpu.x, 11);
    }

    #[test]
    fn INY_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.y = 10;
        cpu.INY();

        // Test if decrement worked
        assert_eq!(cpu.y, 11);
    }

    #[test]
    fn JMP_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.addr_abs = 0x1111;
        cpu.JMP();
        assert_eq!(cpu.pc, 0x1111);
    }

    #[test]
    fn JSR_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.pc = 0x1235;
        cpu.addr_abs = 0x1111;

        cpu.JSR();
        assert_eq!(cpu.read(STACK_POINTER_BASE + cpu.stkp + 1), 0x34, "Lo byte of return address incorrect");
        assert_eq!(cpu.read(STACK_POINTER_BASE + cpu.stkp + 2), 0x12, "Hi byte of return address incorrect");
        assert_eq!(cpu.pc, 0x1111, "Jumped to wrong address or did not jump at all");
    }

    #[test]
    fn LDA_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 100);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.LDA();

        assert_eq!(cpu.a, 100, "Accumulator not loaded or loaded incorrectly");
    }

    #[test]
    fn LDX_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 100);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.LDX();

        assert_eq!(cpu.x, 100, "X Register not loaded or loaded incorrectly");
    }

    #[test]
    fn LDY_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 100);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.LDY();

        assert_eq!(cpu.y, 100, "Y Register not loaded or loaded incorrectly");
    }

    #[test]
    fn LSR_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 0b0110);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.LSR();

        assert_eq!(cpu.read(0x1111), 0b0011, "Memory value not shifted correctly");


        // Random opcode with implied addressing
        cpu.opcode = 0x00;
        cpu.a = 0b0110;
        cpu.IMP();
        cpu.LSR();
        assert_eq!(cpu.a, 0b0011, "Accumulator not shifted correctly");
    }

    #[test]
    fn NOP_test() {
        Cpu6502::new().NOP();
    }

    #[test]
    fn ORA_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 0b0101;
        cpu.write(0x1111, 0b0110);
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;

        cpu.ORA();
        assert_eq!(cpu.a, 0b0111, "OR operation failed");
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn PHA_PLA_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 0b0101;
        cpu.PHA();
        assert_eq!(cpu.read(STACK_POINTER_BASE + cpu.stkp + 1), cpu.a);

        cpu.a = 0;
        cpu.PLA();
        assert_eq!(cpu.a, 0b0101);
    }

    #[test]
    fn PHP_PLP_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.status = Flags6502::C | Flags6502::N;
        cpu.PHP();
        assert_eq!(Flags6502::from_bits(cpu.read(STACK_POINTER_BASE + cpu.stkp + 1)).unwrap(), cpu.status);
        cpu.status = Flags6502::empty();
        cpu.PLP();
        assert_eq!(cpu.status, Flags6502::C | Flags6502::N | Flags6502::U);
    }

    #[test]
    fn ROL_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 0b1000_1100);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.ROL();

        assert_eq!(cpu.read(0x1111), 0b0001_1001, "Memory value not rotated correctly");

        // Random opcode with implied addressing
        cpu.opcode = 0x00;
        cpu.a = 0b1000_1100;
        cpu.IMP();
        cpu.ROL();
        assert_eq!(cpu.a, 0b0001_1001, "Accumulator not rotated correctly");
    }

    #[test]
    fn ROR_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.write(0x1111, 0b1000_1001);

        // Random opcode with absolute addressing
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.ROR();

        assert_eq!(cpu.read(0x1111), 0b1100_0100, "Memory value not rotated correctly");

        // Random opcode with implied addressing
        cpu.opcode = 0x00;
        cpu.a = 0b1000_1001;
        cpu.IMP();
        cpu.ROR();
        assert_eq!(cpu.a, 0b1100_0100, "Accumulator not rotated correctly");
    }

    #[test]
    fn RTI_test() {
        //TODO RTI test
    }

    #[test]
    fn RTS_test() {
        //TODO RTS test
    }

    #[test]
    fn SBC_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 30;
        cpu.write(0x1111, 10);
        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.set_flag(Flags6502::C, true);

        cpu.SBC();
        assert_eq!(cpu.a, 20, "Subtraction failed");
    }

    #[test]
    fn set_flags_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

        cpu.SEC();
        cpu.SED();
        cpu.SEI();
        assert_eq!(cpu.status, Flags6502::C | Flags6502::D | Flags6502::I);
    }

    #[test]
    fn store_test() {
        let cpu = Rc::new(RefCell::new(Cpu6502::new()));
        let _bus = bus::Bus::new(cpu.clone());
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();
        cpu.a = 10;
        cpu.x = 15;
        cpu.y = 20;

        cpu.opcode = 0x0E;
        cpu.addr_abs = 0x1111;
        cpu.STA();
        assert_eq!(cpu.read(0x1111), cpu.a, "Storing accumulator failed");
        cpu.STX();
        assert_eq!(cpu.read(0x1111), cpu.x, "Storing X register failed");
        cpu.STY();
        assert_eq!(cpu.read(0x1111), cpu.y, "Storing Y register failed");
    }

    #[test]
    fn TAX_test() {
        let mut cpu = Cpu6502::new();
        cpu.a = 5;
        cpu.TAX();
        assert_eq!(cpu.x, cpu.a);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn TAY_test() {
        let mut cpu = Cpu6502::new();
        cpu.a = 5;
        cpu.TAY();
        assert_eq!(cpu.y, cpu.a);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn TSX_test() {
        let mut cpu = Cpu6502::new();
        cpu.stkp = 5;
        cpu.TSX();
        assert_eq!(cpu.x as u16, cpu.stkp);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn TXA_test() {
        let mut cpu = Cpu6502::new();
        cpu.x = 5;
        cpu.TXA();
        assert_eq!(cpu.a, cpu.x);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn TXS_test() {
        let mut cpu = Cpu6502::new();
        cpu.x = 5;
        cpu.TXS();
        assert_eq!(cpu.stkp, cpu.x as u16);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn TYA_test() {
        let mut cpu = Cpu6502::new();
        cpu.y = 5;
        cpu.TYA();
        assert_eq!(cpu.a, cpu.y);
        assert_eq!(cpu.status, Flags6502::empty());
    }

    #[test]
    fn XXX_test() {
        Cpu6502::new().XXX();
    }
}
