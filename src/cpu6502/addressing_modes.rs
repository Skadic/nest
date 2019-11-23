use crate::cpu6502::Cpu6502;



// Addressing Modes. These return true if they need another clock cycle. false otherwise
#[allow(non_snake_case, unused)]
impl Cpu6502 {
    /// Implied Addressing Mode.
    /// This means either that there is no additional data is part of the instruction,
    /// or the instruction operates on the accumulator, in which case the data in the accumulator is the fetched data.
    pub fn IMP(&mut self) -> bool {
        self.fetched = self.a;
        false
    }

    /// Immediate addressing mode. The data is supplied as part of the instruction. The address data will be the next byte after the instruction.
    pub fn IMM(&mut self) -> bool {
        self.addr_abs = self.pc;
        self.pc += 1;
        false
    }

    /// Zero Page Addressing Mode.
    /// One can think of the 16-bit memory address (0xXXXX) as the high byte addressing the memory page and the low byte addressing the offset into that page.
    /// The memory would then be 256 pages of 256 bytes each.
    /// Zero Page Addressing means that the page in this case is 0, and the data to read is in that page. This means that the high byte of the 16-bit addess is zero.
    pub fn ZP0(&mut self) -> bool {
        self.addr_abs = self.read(self.pc) as u16;
        self.addr_abs &= 0x00FF;
        self.pc += 1;
        false
    }

    /// Zero Page Addressing Mode with X-register offset.
    /// Same as `ZP0`, but the address supplied with the instruction has the content of the X-register added to it.
    pub fn ZPX(&mut self) -> bool {
        self.addr_abs = (self.read(self.pc) + self.x) as u16;
        self.addr_abs &= 0x00FF;
        self.pc += 1;
        false
    }

    /// Zero Page Addressing Mode with Y-register offset.
    /// Same as `ZP0`, but the address supplied with the instruction has the content of the Y-register added to it.
    pub fn ZPY(&mut self) -> bool {
        self.addr_abs = (self.read(self.pc) + self.y) as u16;
        self.addr_abs &= 0x00FF;
        self.pc += 1;
        false
    }

    /// Absolute Addressing Mode.
    /// The memory address is an absolute value (so the instruction is a 3-byte instruction)
    pub fn ABS(&mut self) -> bool {
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        false
    }

    /// Absolute Addressing Mode with X-register offset.
    /// Same as ABS, but the supplied address has the content of the X-register added to it.
    /// This instruction needs an additional clock cycle, if after adding the X value to the address, the address changes another page
    /// This is checked by comparing the high byte before and after adding X. If it changed, then the page addressed changed.
    pub fn ABX(&mut self) -> bool {
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.x as u16;

        self.addr_abs & 0xFF00 != hi << 8
    }

    /// Absolute Addressing Mode with Y-register offset.
    /// Same as ABS, but the supplied address has the content of the Y-register added to it.
    /// This instruction needs an additional clock cycle, if after adding the Y value to the address, the address changes another page
    /// This is checked by comparing the high byte before and after adding Y. If it changed, then the page addressed changed.
    pub fn ABY(&mut self) -> bool {
        let lo = self.read(self.pc) as u16;
        self.pc += 1;
        let hi = self.read(self.pc) as u16;
        self.pc += 1;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y as u16;

        self.addr_abs & 0xFF00 != hi << 8
    }

    /// Indirect Addressing Mode.
    /// This reads an address from memory at the address supplied by the instruction.
    /// The address that has now been read from memory is the place where the data resides.
    pub fn IND(&mut self) -> bool {
        let ptr_lo = self.read(self.pc) as u16;
        self.pc += 1;
        let ptr_hi = self.read(self.pc) as u16;
        self.pc += 1;

        // Address to read the new address from
        let ptr = (ptr_hi << 8) | ptr_lo;

        // Interestingly the hardware of the NES had a bug, in which, if the supplied address was equal to xxFF (where xx are any numbers),
        // then the most significant byte of the actual address will be fetched from xx00 instead of page XX+1.
        // So, the lower byte overflowed and reset to zero.
        // This bug is simulated here
        if ptr_lo == 0x00FF {
            // Simulate page boundary hardware bug
            self.addr_abs = ((self.read(0xFF00 & ptr) as u16) << 8) | self.read(ptr) as u16;
        } else {
            // Behave normally
            // This reads the high byte and low byte of the actual address
            self.addr_abs = ((self.read(ptr + 1) as u16) << 8) | self.read(ptr) as u16;
        }

        false
    }

    /// Indirect Addressing of the Zero Page with X-register offset.
    /// This reads an address from the Page 0 (see ZP0) at the supplied offset byte
    /// with an additional offset of the value in the X-register
    pub fn IZX(&mut self) -> bool {
        let offset = self.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.read((offset + self.x as u16) & 0x00FF) as u16;
        let hi = self.read((offset + self.x as u16 + 1) & 0x00FF) as u16;

        self.addr_abs = (hi << 8) | lo;

        false
    }

    /// Indirect Addressing of the Zero Page with Y-register offset.
    /// This reads an address from the Page 0 (see ZP0) at the supplied offset byte
    /// The resulting address is then offset by the value in the Y register
    /// Note, that confusingly, unlike IZX, the actual absolute address is offset and not the supplied address
    pub fn IZY(&mut self) -> bool {
        let offset = self.read(self.pc) as u16;
        self.pc += 1;

        let lo = self.read(offset & 0x00FF) as u16;
        let hi = self.read((offset + 1) & 0x00FF) as u16;

        self.addr_abs = (hi << 8) | lo;
        self.addr_abs += self.y as u16;

        // As we could cross a page boundary by offsetting the absolute address,
        // the instruction could take another clock cycle to complete
        // This is the same check as in ABX and ABY
        (self.addr_abs & 0xFF00) != hi << 8
    }

    /// Relative Addressing Mode.
    /// This is only used for branch instructions
    /// Branch instructions can not jump to just any everywhere in the program. They can not jump any further than at most 127 memory locations
    pub fn REL(&mut self) -> bool {
        self.addr_rel = self.read(self.pc) as u16;
        self.pc += 1;

        // As this address is relative and can be used to jump backwards, here the (technically) 8-bit relative address' 1st bit is checked
        // This is done to determine whether the number is supposed to be negative. In that case the high byte of the address is set to all 1s,
        // so that integer arithmetic can do its thing
        if self.addr_rel & 0x80 > 0 {
            self.addr_rel |= 0xFF00;
        }

        false
    }
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use std::rc::Rc;
    use std::cell::{RefCell, RefMut};
    use crate::cpu6502::Cpu6502;
    use crate::bus::Bus;
    use crate::ppu2C02::Ppu2C02;

    fn setup() -> (Rc<RefCell<Cpu6502>>, Rc<RefCell<Bus>>) {
        let cpu : Rc<RefCell<Cpu6502>> = Rc::new(
            RefCell::new(Cpu6502::new()));
        let ppu: Rc<RefCell<Ppu2C02>> = Rc::new(
            RefCell::new(Ppu2C02::new())
        );
        let bus: Rc<RefCell<Bus>> = Bus::new(cpu.clone(), ppu.clone());

        (cpu, bus)
    }

    #[test]
    fn IMP_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Data should be fetched from the accumulator, so add some random data
        cpu.a = 0x12;

        // Random opcode with Implied Addressing
        cpu.opcode = 0x00;

        // This should write the accumulator into the "fetched" attribute of the CPU;
        cpu.IMP();

        assert_eq!(cpu.fetch(), 0x12, "Data not fetched correctly")
    }

    #[test]
    fn IMM_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Pick an address, the program counter is on, in immediate addressing, the data should be read from here
        cpu.pc = 0x500;

        // Write some data to that address
        cpu.write(0x500, 0x20);

        // Random opcode with immediate addressing
        cpu.opcode = 0xA0;

        // This should set the absolute address to be read to the program counter,
        // and increase the program counter by 1
        cpu.IMM();

        // The address to be read from should be equal to what the program counter was
        assert_eq!(cpu.addr_abs, 0x500, "Address read incorrectly");

        // The program counter should be incremented
        assert_eq!(cpu.pc, 0x501, "Program counter not incremented");

        // Fetch the data
        cpu.fetch();

        // The fetch operation should read the "0x20" from the address 0x1000
        assert_eq!(cpu.fetched, 0x20, "Wrong data fetched")
    }

    #[test]
    fn ZP0_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ZP0 Addressing.
        cpu.opcode = 0x05;

        // Write some data to the Zero Page
        cpu.write(0x0010, 0x20);

        cpu.pc = 0x500;

        // Write the offset to be read to the program counter
        // (which it is going to be read from)
        cpu.write(0x500, 0x10);

        cpu.ZP0();

        // The program counter should be incremented by one
        assert_eq!(cpu.pc, 0x501);

        // The address to be read should be 0x0000 + the data that was at the program counter: 0x0010
        assert_eq!(cpu.addr_abs, 0x0010);

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched")
    }

    #[test]
    fn ZPX_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ZPX Addressing.
        cpu.opcode = 0x15;

        // Write some data to the Zero Page
        cpu.write(0x0010, 0x20);

        cpu.pc = 0x500;

        // Write the offset to be read to the program counter
        // (which it is going to be read from)
        cpu.write(0x500, 0x08);

        // Write an additional offset to x
        // So in the end the address should be 0x08 + 0x08 = 0x10
        cpu.x = 0x08;

        cpu.ZPX();

        // The program counter should be incremented by one
        assert_eq!(cpu.pc, 0x501);

        // The address to be read should be 0x0000 + the data that was at the program counter + the value of the x register: 0x0010
        assert_eq!(cpu.addr_abs, 0x0010);

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched")
    }

    #[test]
    fn ZPY_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ZPY Addressing.
        cpu.opcode = 0x96;

        // Write some data to the Zero Page
        cpu.write(0x0010, 0x20);

        cpu.pc = 0x500;

        // Write the offset to be read to the program counter
        // (which it is going to be read from)
        cpu.write(0x500, 0x08);

        // Write an additional offset to y
        // So in the end the address should be 0x08 + 0x08 = 0x10
        cpu.y = 0x08;

        cpu.ZPY();

        // The program counter should be incremented by one
        assert_eq!(cpu.pc, 0x501);

        // The address to be read should be 0x0000 + the data that was at the program counter + the value of the y register: 0x0010
        assert_eq!(cpu.addr_abs, 0x0010);

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched")
    }

    #[test]
    fn ABS_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ABS Addressing.
        cpu.opcode = 0x20;

        // Write some data to memory
        cpu.write(0x123, 0x20);

        cpu.pc = 0x500;
        // Write the lo byte to memory
        cpu.write(0x500, 0x23);
        // Write the hi byte to memory
        cpu.write(0x501, 0x01);

        cpu.ABS();

        assert_eq!(cpu.pc, 0x502, "The program counter should have been incremented by 2");
        assert_eq!(cpu.addr_abs, 0x123, "The address has been read incorrectly");

        cpu.fetch();
        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched from memory");
    }

    #[test]
    fn ABX_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ABX Addressing.
        cpu.opcode = 0x1D;

        // Write some data to memory
        cpu.write(0x127, 0x20);

        cpu.pc = 0x500;
        // Write the lo byte to memory
        cpu.write(0x500, 0x23);
        // Write the hi byte to memory
        cpu.write(0x501, 0x01);

        // Additional x offset
        cpu.x = 0x04;

        cpu.ABX();

        assert_eq!(cpu.pc, 0x502, "The program counter should have been incremented by 2");
        assert_eq!(cpu.addr_abs, 0x127, "The address has been read incorrectly");

        cpu.fetch();
        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched from memory");
    }

    #[test]
    fn ABY_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with ABX Addressing.
        cpu.opcode = 0x1D;

        // Write some data to memory
        cpu.write(0x127, 0x20);

        cpu.pc = 0x500;
        // Write the lo byte to memory
        cpu.write(0x500, 0x23);
        // Write the hi byte to memory
        cpu.write(0x501, 0x01);

        // Additional y offset
        cpu.y = 0x04;

        cpu.ABY();

        assert_eq!(cpu.pc, 0x502, "The program counter should have been incremented by 2");
        assert_eq!(cpu.addr_abs, 0x127, "The address has been read incorrectly");

        cpu.fetch();
        assert_eq!(cpu.fetched, 0x20, "Wrong data was fetched from memory");
    }

    #[test]
    fn IND_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with indirect addressing
        cpu.opcode = 0x6C;

        // Write some data to memory
        cpu.write(0x123, 0x20);

        cpu.pc = 0x500;

        // Write the lo byte of the address to memory
        cpu.write(0x500, 0x40);
        // Write the hi byte of the address to memory
        cpu.write(0x501, 0x02);


        // Write the lo byte of the target address to memory
        cpu.write(0x0240, 0x23);
        // Write the hi byte of the target address to memory
        cpu.write(0x0241, 0x01);

        cpu.IND();

        assert_eq!(cpu.pc, 0x502, "Program counter not increased by 2");
        assert_eq!(cpu.addr_abs, 0x123, "Target address read incorrectly");

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data fetched");
    }

    #[test]
    fn IND_bug_handling_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with indirect addressing
        cpu.opcode = 0x6C;

        // Write some data to memory
        cpu.write(0x123, 0x20);

        cpu.pc = 0x500;

        // Write the lo byte of the address to memory
        // This being 0xFF triggers the bug
        cpu.write(0x500, 0xFF);
        // Write the hi byte of the address to memory
        cpu.write(0x501, 0x01);


        // Write the lo byte of the target address to memory
        cpu.write(0x01FF, 0x23);
        // Write the hi byte of the target address to memory
        // Note, that this is 0x0100 and not 0x0200,
        // because the simulated bug overflowed the 0xFF of the lo byte
        // without updating the hi byte
        cpu.write(0x0100, 0x01);

        cpu.IND();

        assert_eq!(cpu.pc, 0x502, "Program counter not increased by 2");
        assert_eq!(cpu.addr_abs, 0x123, "Target address read incorrectly");

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data fetched");
    }

    #[test]
    fn IZX_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with indirect X-offset addressing
        cpu.opcode = 0x01;

        // Write some data to memory
        cpu.write(0x123, 0x20);

        cpu.pc = 0x500;

        // Write the offset in the zero page to the location of the program counter
        cpu.write(0x500, 0x40);

        // Additional offset in the X register
        cpu.x = 0x08;

        // As a result, the target address must be at 0x0048 and 0x0049
        // Write the lo byte of the target address to memory
        cpu.write(0x0048, 0x23);
        // Write the hi byte of the target address to memory
        cpu.write(0x0049, 0x01);

        cpu.IZX();

        assert_eq!(cpu.pc, 0x501, "Program counter not increased by 1");
        assert_eq!(cpu.addr_abs, 0x123, "Target address read incorrectly");

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data fetched");
    }

    #[test]
    fn IZY_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        // Random opcode with indirect X-offset addressing
        cpu.opcode = 0x01;

        // Write some data to memory
        cpu.write(0x125, 0x20);

        cpu.pc = 0x500;

        // Additional offset of the *absolute address* in the Y register
        cpu.y = 0x02;

        // Write the offset in the zero page to the location of the program counter
        cpu.write(0x500, 0x40);

        // As a result, the target address must be at 0x0040 and 0x0041
        // The resulting target address will then be offset by the value in the Y register
        // Write the lo byte of the target address to memory
        cpu.write(0x0040, 0x23);
        // Write the hi byte of the target address to memory
        cpu.write(0x0041, 0x01);

        cpu.IZY();

        assert_eq!(cpu.pc, 0x501, "Program counter not increased by 1");
        assert_eq!(cpu.addr_abs, 0x125, "Target address read incorrectly");

        cpu.fetch();

        assert_eq!(cpu.fetched, 0x20, "Wrong data fetched");
    }

    #[test]
    fn REL_test() {
        let (mut cpu, _) = setup();
        let mut cpu = cpu.borrow_mut();

        cpu.pc = 0x500;
        cpu.write(0x500, 0x000F);

        cpu.REL();

        assert_eq!(cpu.addr_rel, 0x000F, "Relative address not set");
        assert_eq!(cpu.pc, 0x501, "Program counter not incremented");

        cpu.pc = 0x500;
        cpu.write(0x500, 0x00FF);

        cpu.REL();

        assert_eq!(cpu.addr_rel, 0xFFFF, "Relative address not converted to negative");
        assert_eq!(cpu.pc, 0x501, "Program counter not incremented");

    }
}