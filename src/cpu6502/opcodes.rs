use crate::cpu6502::{Cpu6502, STACK_POINTER_BASE, IRQ_PROGRAM_COUNTER};
use crate::cpu6502::Flags6502;
use crate::cpu6502::LOOKUP;
use std::num::Wrapping;
use std::ops::{Add, Sub};

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

    /// Branch if not overflowed
    pub fn BVC(&mut self) -> bool {
        if !self.get_flag(Flags6502::V) {
            self.branch()
        }
        false
    }

    /// Branch if overflowed
    pub fn BVS(&mut self) -> bool {
        if self.get_flag(Flags6502::V) {
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
        // Store the high and low bytes of the program counter to the stack
        self.push_stack((self.pc >> 8) as u8);
        self.push_stack((self.pc & 0x00FF) as u8);

        self.set_flag(Flags6502::B, true);
        self.set_flag(Flags6502::U, true);
        // Push status register to the stack with the B flag set
        self.push_stack(self.status.bits());
        self.set_flag(Flags6502::B, false);
        self.set_flag(Flags6502::U, false);

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

        let value = wrap_sub(self.fetched, 1);
        self.write(self.addr_abs, value);

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        false
    }

    /// Decrements the X-register by 1
    pub fn DEX(&mut self) -> bool {
        self.x = wrap_sub(self.x, 1);
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);
        false
    }

    /// Decrements the Y register by 1
    pub fn DEY(&mut self) -> bool {
        self.y = wrap_sub(self.y, 1);
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

        let value = wrap_add(self.fetched, 1);
        self.write(self.addr_abs, value);

        self.set_flag(Flags6502::Z, value == 0);
        self.set_flag(Flags6502::N, (value & 0x80) > 0);

        false
    }

    /// Increments the X-register by 1
    pub fn INX(&mut self) -> bool {
        self.x = wrap_add(self.x, 1);
        self.set_flag(Flags6502::Z, self.x == 0);
        self.set_flag(Flags6502::N, (self.x & 0x80) > 0);
        false
    }

    /// Increments the Y register by 1
    pub fn INY(&mut self) -> bool {
        self.y = wrap_add(self.y, 1);
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
        self.push_stack((self.pc >> 8) as u8);
        self.push_stack((self.pc & 0x00FF) as u8);

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
        self.push_stack(self.a);
        false
    }

    /// Push processor status on stack
    pub fn PHP(&mut self) -> bool {
        // For PHP the status register is pushed to the stack along with the B and U flags
        self.push_stack((self.status | Flags6502::B | Flags6502::U).bits());
        false
    }

    // Pop off the stack into the accumulator
    pub fn PLA(&mut self) -> bool {
        self.a = self.pop_stack();
        self.set_flag(Flags6502::Z, self.a == 0);
        self.set_flag(Flags6502::N, (self.a & 0x80) > 0);
        false
    }

    /// Pull processor status from stack
    pub fn PLP(&mut self) -> bool {
        self.status = Flags6502::from_bits(self.pop_stack()).unwrap();
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
        self.status = Flags6502::from_bits(self.pop_stack()).unwrap();
        self.status &= !Flags6502::B;
        self.status &= !Flags6502::U;

        let lo = self.pop_stack() as u16;
        let hi = self.pop_stack() as u16;
        self.pc = (hi << 8) | lo;

        false
    }

    /// Return from Subroutine
    /// Returns to a saved program counter after jumping there (see JSR)
    pub fn RTS(&mut self) -> bool {
        let lo = self.pop_stack() as u16;
        let hi = self.pop_stack() as u16;
        self.pc = ((hi << 8) | lo) + 1;
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

#[inline(always)]
fn wrap_add(a: u8, b: u8) -> u8 {
    (Wrapping(a) + Wrapping(b)).0
}

#[inline(always)]
fn wrap_sub(a: u8, b: u8) -> u8 {
    (Wrapping(a) - Wrapping(b)).0
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use std::cell::RefCell;
    use std::rc::Rc;
    use crate::bus::Bus;
    use crate::cpu6502::{Cpu6502, Flags6502, IRQ_PROGRAM_COUNTER};
    use crate::ppu2C02::Ppu2C02;

    const START_PC: u16 = 0x0110;
    const START_ADDR_ABS: u16 = 0x0080;

    macro_rules! check_flag {
        ($status:ident, C, true)   => { assert_eq!($status.contains(Flags6502::C), true, "Carry flag is clear, despite overflow happening"); };
        ($status:ident, C, false)  => { assert_eq!($status.contains(Flags6502::C), false, "Carry flag is set, despite no overflow happening"); };
        ($status:ident, Z, true)   => { assert_eq!($status.contains(Flags6502::Z), true, "Zero flag is clear, despite result being zero"); };
        ($status:ident, Z, false)  => { assert_eq!($status.contains(Flags6502::Z), false, "Zero flag is set, despite result not being zero"); };
        ($status:ident, I, true)   => { assert_eq!($status.contains(Flags6502::I), true, "Interrupt Disable flag is clear"); };
        ($status:ident, I, false)  => { assert_eq!($status.contains(Flags6502::I), false, "Interrupt Disable flag is set"); };
        ($status:ident, D, true)   => { assert_eq!($status.contains(Flags6502::D), true, "Decimal mode flag is clear"); };
        ($status:ident, D, false)  => { assert_eq!($status.contains(Flags6502::D), false, "Decimal mode flag is set"); };
        ($status:ident, B, true)   => { assert_eq!($status.contains(Flags6502::B), true, "Break flag is clear"); };
        ($status:ident, B, false)  => { assert_eq!($status.contains(Flags6502::B), false, "Break flag is set"); };
        ($status:ident, U, true)   => { assert_eq!($status.contains(Flags6502::U), true, "Unused flag is clear"); };
        ($status:ident, U, false)  => { assert_eq!($status.contains(Flags6502::U), false, "Unused flag is set"); };
        ($status:ident, V, true)   => { assert_eq!($status.contains(Flags6502::V), true, "Overflow flag is clear, despite incorrect overflow happening"); };
        ($status:ident, V, false)  => { assert_eq!($status.contains(Flags6502::V), false, "Overflow flag flag is set, despite no incorrect overflow happening"); };
        ($status:ident, N, true)   => { assert_eq!($status.contains(Flags6502::N), true, "Negative flag is clear, despite result being negative"); };
        ($status:ident, N, false)  => { assert_eq!($status.contains(Flags6502::N), false, "Negative flag is set, despite result being positive"); };
    }

    /// Tests the branch instructions.
    /// Since every test is basically the same
    /// $flag is the flag to be tested before branching
    /// $instr is the instruction to execute
    /// if the boolean is true, then the instruction should branch when the flag is set
    /// if it's false, the instruction should branch when the flag is clear
    macro_rules! branch_test {
        ($flag:ident, $instr:ident, true) => {
            let bus = setup();
            let bus_ref = bus.borrow();

            bus_ref.cpu_mut().addr_rel = 0x0010;

            bus_ref.cpu_mut().status = Flags6502::empty();
            bus_ref.cpu_mut().$instr();
            assert_eq!(bus_ref.cpu().pc, START_PC, "Branched, despite {} flag being clear", stringify!($flag));

            bus_ref.cpu_mut().status = Flags6502::$flag;
            bus_ref.cpu_mut().$instr();
            assert_eq!(bus_ref.cpu().pc, START_PC + 0x0010, "Did not branch, despite {} flag being set", stringify!($flag));
        };
        ($flag:ident, $instr:ident, false) => {
            let bus = setup();
            let bus_ref = bus.borrow();

            bus_ref.cpu_mut().addr_rel = 0x0010;

            bus_ref.cpu_mut().status = Flags6502::$flag;
            bus_ref.cpu_mut().$instr();
            assert_eq!(bus_ref.cpu().pc, START_PC, "Branched, despite {} flag being set", stringify!($flag));

            bus_ref.cpu_mut().status = Flags6502::empty();
            bus_ref.cpu_mut().$instr();
            assert_eq!(bus_ref.cpu().pc, START_PC + 0x0010, "Did not branch, despite {} flag being clear", stringify!($flag));
        };
    }

    /// Tests whether the instructions that set and clear instructions work
    macro_rules! flag_set_test {
        ($flag:ident, $instr:ident, true) => {
            let bus = setup();
            let bus_ref = bus.borrow();
            bus_ref.cpu_mut().set_flag(Flags6502::$flag, false);
            bus_ref.cpu_mut().$instr();
            assert!(bus_ref.cpu().get_flag(Flags6502::$flag), "{} flag clear, but should be set");
        };
        ($flag:ident, $instr:ident, false) => {
            let bus = setup();
            let bus_ref = bus.borrow();
            bus_ref.cpu_mut().set_flag(Flags6502::$flag, true);
            bus_ref.cpu_mut().$instr();
            assert!(!bus_ref.cpu().get_flag(Flags6502::$flag), "{} flag set, but should be cleared");
        };
    }

    /// Creates a bus for usage in tests
    /// The PC is initialized to 0x0100 and the addressing mode is immediate
    /// The absolute address to be read by fetch is initialized to 0x0080
    fn setup() -> Rc<RefCell<Bus>> {
        let mut bus = {
            let cpu = Cpu6502::new();
            let ppu = Ppu2C02::new();
            Bus::new(cpu, ppu)
        };
        {
            let mut bus_ref = bus.borrow_mut();
            // So that the addressing mode of the current instruction is "immediate"
            bus_ref.cpu_mut().opcode = 0x09;
            bus_ref.cpu_mut().pc = START_PC;
            bus_ref.cpu_mut().addr_abs = START_ADDR_ABS;
        }
        bus
    }

    #[test]
    fn branch_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        bus_ref.cpu_mut().addr_rel = 0x0010;

        bus_ref.cpu_mut().branch();

        assert_eq!(bus_ref.cpu().pc, START_PC + 0x0010, "Branched to wrong address");
    }

    #[test]
    fn ADC_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().a = 30;

        bus_ref.cpu_mut().ADC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 40, "Accumulator value incorrect after add");
        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, V, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn ADC_carry_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 1);
        bus_ref.cpu_mut().a = 0xFF;

        bus_ref.cpu_mut().ADC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0, "Accumulator value incorrect after add");

        check_flag!(status, Z, true);
        check_flag!(status, C, true);
        check_flag!(status, V, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn ADC_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 1);
        bus_ref.cpu_mut().a = 0x7F;

        bus_ref.cpu_mut().ADC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x80, "Accumulator value incorrect after add");

        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, V, true);
        check_flag!(status, N, true);
    }

    #[test]
    fn AND_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 1);
        bus_ref.cpu_mut().a = 0x7F;

        bus_ref.cpu_mut().AND();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x01, "Accumulator value incorrect after and");
        check_flag!(status, Z, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn AND_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x80);
        bus_ref.cpu_mut().a = 0x7F;

        bus_ref.cpu_mut().AND();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x00, "Accumulator value incorrect after and");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn AND_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x81);
        bus_ref.cpu_mut().a = 0xF0;

        bus_ref.cpu_mut().AND();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x80, "Accumulator value incorrect after and");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn ASL_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x3F);
        bus_ref.cpu_mut().a = 0x3F;

        bus_ref.cpu_mut().ASL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x3F, "Accumulator modified, despite addressing mode not implied");
        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0x7E, "Read value incorrect after left shift");
        check_flag!(status, C, false);
        check_flag!(status, Z, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn ASL_immediate_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0xFF;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ASL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0xFE, "Accumulator value incorrect after left shift");
        check_flag!(status, C, true);
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn ASL_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0x80;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ASL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x00, "Accumulator value incorrect after left shift");
        check_flag!(status, C, true);
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn BCC_test() {
        branch_test!(C, BCC, false);
    }

    #[test]
    fn BCS_test() {
        branch_test!(C, BCS, true);
    }

    #[test]
    fn BEQ_test() {
        branch_test!(Z, BEQ, true);
    }

    #[test]
    fn BNE_test() {
        branch_test!(Z, BNE, false);
    }

    #[test]
    fn BPL_test() {
        branch_test!(N, BPL, false);
    }

    #[test]
    fn BMI_test() {
        branch_test!(N, BMI, true);
    }

    #[test]
    fn BVC_test() {
        branch_test!(V, BVC, false);
    }

    #[test]
    fn BVS_test() {
        branch_test!(V, BVS, true);
    }

    #[test]
    fn BRK_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.status = Flags6502::empty();

        cpu_ref.BRK();

        assert_eq!(cpu_ref.pc, 0x0000, "New program counter incorrectly read");

        let stored_status = Flags6502::from_bits(cpu_ref.pop_stack()).unwrap();
        check_flag!(stored_status, I, true);
        check_flag!(stored_status, B, true);
        check_flag!(stored_status, U, true);

        let old_pc = {
            let lo = cpu_ref.pop_stack();
            let hi = cpu_ref.pop_stack();
            (lo as u16) | ((hi as u16) << 8)
        };

        assert_eq!(old_pc, START_PC + 1, "Old program counter not read/saved correctly");

        let status = cpu_ref.status;
        check_flag!(status, I, true);
        check_flag!(status, B, false);
    }

    #[test]
    fn BIT_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0xF0);
        bus_ref.cpu_mut().a = 0x0F;

        bus_ref.cpu_mut().BIT();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, true);
        check_flag!(status, V, true);
        check_flag!(status, N, true);
    }

    #[test]
    fn CLC_test() {
        flag_set_test!(C, CLC, false);
    }

    #[test]
    fn CLD_test() {
        flag_set_test!(D, CLD, false);
    }

    #[test]
    fn CLI_test() {
        flag_set_test!(I, CLI, false);
    }

    #[test]
    fn CLV_test() {
        flag_set_test!(V, CLV, false);
    }

    #[test]
    fn CMP_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().a = 10;

        bus_ref.cpu_mut().CMP();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, true);
        check_flag!(status, C, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn CMP_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 30);
        bus_ref.cpu_mut().a = 10;

        bus_ref.cpu_mut().CMP();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn CPX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().x = 10;

        bus_ref.cpu_mut().CPX();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, true);
        check_flag!(status, C, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn CPX_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 30);
        bus_ref.cpu_mut().x = 10;

        bus_ref.cpu_mut().CPX();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn CPY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().y = 10;

        bus_ref.cpu_mut().CPY();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, true);
        check_flag!(status, C, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn CPY_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 30);
        bus_ref.cpu_mut().y = 10;

        bus_ref.cpu_mut().CPY();

        let status = bus_ref.cpu().status;

        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn DEC_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 1);

        bus_ref.cpu_mut().DEC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0, "Memory value not decremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn DEC_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0);

        bus_ref.cpu_mut().DEC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0xFF, "Memory value not decremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn DEX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().x = 1;

        bus_ref.cpu_mut().DEX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 0, "x register not decremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn DEX_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().x = 0;

        bus_ref.cpu_mut().DEX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 255, "x register not decremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn DEY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().y = 1;

        bus_ref.cpu_mut().DEY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 0, "y register not decremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn DEY_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().y = 0;

        bus_ref.cpu_mut().DEY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 255, "y register not decremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn EOR_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        bus_ref.cpu_write(START_ADDR_ABS, 0xF0);
        cpu_ref.a = 0x0F;

        cpu_ref.EOR();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0xFF, "Accumulator value incorrect after XOR");
        check_flag!(status, Z, false);
        check_flag!(status, N, true)
    }

    #[test]
    fn EOR_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        bus_ref.cpu_write(START_ADDR_ABS, 0x0F);
        cpu_ref.a = 0x0F;

        cpu_ref.EOR();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0x00, "Accumulator value incorrect after XOR");
        check_flag!(status, Z, true);
        check_flag!(status, N, false)
    }

    #[test]
    fn INC_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 255);

        bus_ref.cpu_mut().INC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0, "Memory value not incremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn INC_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x7F);

        bus_ref.cpu_mut().INC();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0x80, "Memory value not incremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn INX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().x = 255;

        bus_ref.cpu_mut().INX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 0, "x register not incremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn INX_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().x = 0x7F;

        bus_ref.cpu_mut().INX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 0x80, "x register not incremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn INY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().y = 255;

        bus_ref.cpu_mut().INY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 0, "y register not incremented correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn INY_negative_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().y = 0x7F;

        bus_ref.cpu_mut().INY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 0x80, "y register not incremented correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn JMP_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().JMP();
        assert_eq!(bus_ref.cpu().pc, START_ADDR_ABS, "Jump to incorrect address");
    }

    #[test]
    fn JSR_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().JSR();
        assert_eq!(bus_ref.cpu().pc, START_ADDR_ABS, "Jump to incorrect address");

        let old_addr = {
            let lo = bus_ref.cpu_mut().pop_stack();
            let hi = bus_ref.cpu_mut().pop_stack();
            (lo as u16) | ((hi as u16) << 8)
        };

        assert_eq!(old_addr, START_PC - 1, "Old address saved/read incorrectly");
    }

    #[test]
    fn LDA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x80);
        bus_ref.cpu_mut().LDA();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x80, "data not loaded correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn LDA_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x00);
        bus_ref.cpu_mut().LDA();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x00, "data not loaded correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn LDX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x80);
        bus_ref.cpu_mut().LDX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 0x80, "data not loaded correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn LDX_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x00);
        bus_ref.cpu_mut().LDX();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().x, 0x00, "data not loaded correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn LDY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x80);
        bus_ref.cpu_mut().LDY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 0x80, "data not loaded correctly");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn LDY_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x00);
        bus_ref.cpu_mut().LDY();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().y, 0x00, "data not loaded correctly");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn LSR_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x01);
        bus_ref.cpu_mut().a = 0x01;

        bus_ref.cpu_mut().LSR();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x01, "Accumulator modified, despite addressing mode not implied");
        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0x00, "Read value incorrect after right shift");
        check_flag!(status, C, true);
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn LSR_immediate_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0xFE;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().LSR();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x7F, "Accumulator value incorrect after right shift");
        check_flag!(status, C, false);
        check_flag!(status, Z, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn NOP_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().NOP();
        // huh
    }

    #[test]
    fn ORA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        bus_ref.cpu_write(START_ADDR_ABS, 0xF0);
        cpu_ref.a = 0x0F;

        cpu_ref.ORA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0xFF, "Accumulator value incorrect after OR");
        check_flag!(status, Z, false);
        check_flag!(status, N, true)
    }

    #[test]
    fn ORA_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        bus_ref.cpu_write(START_ADDR_ABS, 0x00);
        cpu_ref.a = 0x00;

        cpu_ref.ORA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0x00, "Accumulator value incorrect after OR");
        check_flag!(status, Z, true);
        check_flag!(status, N, false)
    }

    #[test]
    fn PHA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 10;
        cpu_ref.PHA();

        assert_eq!(cpu_ref.pop_stack(), 10, "Accumulator not pushed to stack");
    }

    #[test]
    fn PHP_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.status = Flags6502::from_bits(0x0F).unwrap();
        cpu_ref.PHP();
        let status = cpu_ref.pop_stack();
        assert_ne!(status, 0x0F, "B and U flag not pushed to stack");
        assert_eq!(status, 0x3F, "Status not pushed to stack");
    }

    #[test]
    fn PLA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.push_stack(0xF0);
        cpu_ref.PLA();

        let status = cpu_ref.status;
        assert_eq!(cpu_ref.a, 0xF0, "Accumulator not popped from stack");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn PLA_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.push_stack(0x00);
        cpu_ref.PLA();

        let status = cpu_ref.status;
        assert_eq!(cpu_ref.a, 0x00, "Accumulator not popped from stack");
        check_flag!(status, Z, true);
        check_flag!(status, N, false)
    }

    #[test]
    fn PLP_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.push_stack(0x0F);
        cpu_ref.PLP();

        let status = cpu_ref.status.bits();
        assert_ne!(status, 0x0F, "Unused flag not pulled from stack");
        assert_eq!(status, 0x2F, "Status not pulled from stack");
    }

    #[test]
    fn ROL_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x80);
        bus_ref.cpu_mut().a = 0x80;

        bus_ref.cpu_mut().ROL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x80, "Accumulator modified, despite addressing mode not implied");
        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0x01, "Read value incorrect after left bit rotate");
        check_flag!(status, C, true);
        check_flag!(status, Z, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn ROL_immediate_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0x40;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ROL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x80, "Accumulator value incorrect after left bit rotate");
        check_flag!(status, C, false);
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn ROL_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0x00;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ROL();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x00, "Accumulator value incorrect after left bit rotate");
        check_flag!(status, C, false);
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn ROR_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 0x01);
        bus_ref.cpu_mut().a = 0x01;

        bus_ref.cpu_mut().ROR();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x01, "Accumulator modified, despite addressing mode not implied");
        assert_eq!(bus_ref.cpu_read(START_ADDR_ABS, false), 0x80, "Read value incorrect after right bit rotate");
        check_flag!(status, C, true);
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn ROR_immediate_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0x40;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ROR();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x20, "Accumulator value incorrect after right bit rotate");
        check_flag!(status, C, false);
        check_flag!(status, Z, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn ROR_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_mut().a = 0x00;

        // instruction with implied addressing mode
        bus_ref.cpu_mut().opcode = 0x00;

        bus_ref.cpu_mut().ROR();

        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0x00, "Accumulator value incorrect after right bit rotate");
        check_flag!(status, C, false);
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn RTI_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.status = Flags6502::empty();

        assert_eq!(cpu_ref.pc, START_PC);

        cpu_ref.BRK();

        assert_eq!(cpu_ref.pc, 0x0000);

        cpu_ref.RTI();

        assert_eq!(cpu_ref.pc, START_PC + 1, "RTI did not return to connect address");

        let status = cpu_ref.status;
        check_flag!(status, B, false);
        check_flag!(status, U, false);
    }

    #[test]
    fn RTS_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        cpu_ref.push_stack(0x12);
        cpu_ref.push_stack(0x33);

        cpu_ref.RTS();

        assert_eq!(cpu_ref.pc, 0x1234, "Returned to wrong address");
    }

    #[test]
    fn SBC_no_carry_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().a = 30;

        bus_ref.cpu_mut().SBC();


        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 19, "Accumulator value incorrect after subtraction");
        check_flag!(status, Z, false);
        check_flag!(status, C, true);
        check_flag!(status, V, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn SBC_carry_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 10);
        bus_ref.cpu_mut().a = 30;
        bus_ref.cpu_mut().set_flag(Flags6502::C, true);

        bus_ref.cpu_mut().SBC();


        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 20, "Accumulator value incorrect after subtraction");
        check_flag!(status, Z, false);
        check_flag!(status, C, true);
        check_flag!(status, V, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn SBC_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 30);
        bus_ref.cpu_mut().a = 30;
        bus_ref.cpu_mut().set_flag(Flags6502::C, true);

        bus_ref.cpu_mut().SBC();


        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 0, "Accumulator value incorrect after subtraction");
        check_flag!(status, Z, true);
        check_flag!(status, C, true);
        check_flag!(status, V, false);
        check_flag!(status, N, false);
    }

    #[test]
    fn SBC_borrow_required_test() {
        let bus = setup();
        let bus_ref = bus.borrow();

        bus_ref.cpu_write(START_ADDR_ABS, 20);
        bus_ref.cpu_mut().a = 10;
        bus_ref.cpu_mut().set_flag(Flags6502::C, true);

        bus_ref.cpu_mut().SBC();


        let status = bus_ref.cpu().status;

        assert_eq!(bus_ref.cpu().a, 246, "Accumulator value incorrect after subtraction");
        check_flag!(status, Z, false);
        check_flag!(status, C, false);
        check_flag!(status, V, true);
        check_flag!(status, N, true);
    }

    #[test]
    fn SEC_test() {
        flag_set_test!(C, SEC, true);
    }

    #[test]
    fn SED_test() {
        flag_set_test!(D, SED, true);
    }

    #[test]
    fn SEI_test() {
        flag_set_test!(I, SEI, true);
    }

    #[test]
    fn STA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 20;
        cpu_ref.STA();

        assert_eq!(cpu_ref.read(START_ADDR_ABS), 20, "Accumulator not stored correctly");
    }

    #[test]
    fn STX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.x = 20;
        cpu_ref.STX();

        assert_eq!(cpu_ref.read(START_ADDR_ABS), 20, "X register not stored correctly");
    }


    #[test]
    fn STY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.y = 20;
        cpu_ref.STY();

        assert_eq!(cpu_ref.read(START_ADDR_ABS), 20, "Y register not stored correctly");
    }

    #[test]
    fn TAX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 0xF0;
        cpu_ref.TAX();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.x, 0xF0, "Accumulator not moved to X register");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn TAX_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 0x00;
        cpu_ref.TAX();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.x, 0x00, "Accumulator not moved to X register");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn TAY_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 0xF0;
        cpu_ref.TAY();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.y, 0xF0, "Accumulator not moved to Y register");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn TAY_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.a = 0x00;
        cpu_ref.TAY();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.y, 0x00, "Accumulator not moved to Y register");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn TSX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.stkp = 0xF0;
        cpu_ref.TSX();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.x, 0xF0, "Stack pointer not moved to X register");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn TSX_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.stkp = 0x00;
        cpu_ref.TSX();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.x, 0x00, "Stack pointer not moved to X register");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn TXA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.x = 0xF0;
        cpu_ref.TXA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0xF0, "X register not moved to Accumulator");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn TXA_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.x = 0x00;
        cpu_ref.TXA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0x00, "X register not moved to Accumulator");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn TXS_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.x = 0xF0;
        cpu_ref.TXS();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.stkp, 0xF0, "X register not moved to Stack Pointer");
    }

    #[test]
    fn TXS_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.x = 0x00;
        cpu_ref.TXS();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.stkp, 0x00, "X register not moved to Stack Pointer");
    }

    #[test]
    fn TYA_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.y = 0xF0;
        cpu_ref.TYA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0xF0, "Y register not moved to Accumulator");
        check_flag!(status, Z, false);
        check_flag!(status, N, true);
    }

    #[test]
    fn TYA_zero_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.y = 0x00;
        cpu_ref.TYA();

        let status = cpu_ref.status;

        assert_eq!(cpu_ref.a, 0x00, "Y register not moved to Accumulator");
        check_flag!(status, Z, true);
        check_flag!(status, N, false);
    }

    #[test]
    fn XXX_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();
        cpu_ref.XXX();
        // ok
    }

    #[test]
    fn is_implied_test() {
        let bus = setup();
        let bus_ref = bus.borrow();
        let mut cpu_ref = bus_ref.cpu_mut();

        assert!(!cpu_ref.is_implied());
        cpu_ref.opcode = 0x00;
        assert!(cpu_ref.is_implied());
    }
}

