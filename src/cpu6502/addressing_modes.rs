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
        self.pc += 1;
        false
    }

    /// Zero Page Addressing Mode with X-register offset.
    /// Same as `ZP0`, but the address supplied with the instruction has the content of the X-register added to it.
    pub fn ZPX(&mut self) -> bool {
        self.addr_abs = self.read(self.pc) as u16 + self.x as u16;
        self.addr_abs &= 0x00FF;
        self.pc += 1;
        false
    }

    /// Zero Page Addressing Mode with Y-register offset.
    /// Same as `ZP0`, but the address supplied with the instruction has the content of the Y-register added to it.
    pub fn ZPY(&mut self) -> bool {
        self.addr_abs = self.read(self.pc) as u16 + self.y as u16;
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
    use std::cell::RefCell;
    use crate::bus::Bus;
    use crate::cpu6502::Cpu6502;
    use crate::ppu2C02::Ppu2C02;

    fn setup() -> Rc<RefCell<Bus>> {
        let mut cpu = Cpu6502::new();
        let mut ppu = Ppu2C02::new();

        // This is so that fetch actually fetches a value from abs_addr
        // If this was 0x00 it wouldn't fetch a values as the operation denoted by 0x00 is BRK
        // which has an implied addressing mode and thus doesn't fetch a value
        // This can really be any opcode of an instruction that doesn't have an IMP addressing mode
        cpu.opcode = 0x01;


        cpu.pc = 0x1000;

        Bus::new(cpu, ppu)
    }

    #[test]
    fn IMP_test() {
        let mut bus = setup();

        bus.borrow().cpu_mut().a = 10;
        bus.borrow().cpu_mut().IMP();

        // The fetched register should now contain the value of the accumulator
        assert_eq!(bus.borrow().cpu_mut().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn IMM_test() {
        let mut bus = setup();
        bus.borrow_mut().cpu_write(0x1000, 10);

        bus.borrow().cpu_mut().IMM();
        bus.borrow().cpu_mut().fetch();

        // The cpu should have read the data stored at 0x1000
        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ZP0_test() {
        let mut bus = setup();
        // Write the address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);

        // Write the actual data
        bus.borrow_mut().cpu_write(0x0020, 10);

        bus.borrow().cpu_mut().ZP0();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ZPX_test() {
        let mut bus = setup();
        // Write the address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);

        bus.borrow().cpu_mut().x = 5;

        // Write the actual data
        bus.borrow_mut().cpu_write(0x0025, 10);

        bus.borrow().cpu_mut().ZPX();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ZPY_test() {
        let mut bus = setup();
        // Write the address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);

        bus.borrow().cpu_mut().y = 5;

        // Write the actual data
        bus.borrow_mut().cpu_write(0x0025, 10);

        bus.borrow().cpu_mut().ZPY();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ABS_test() {
        let mut bus = setup();
        // Write the lo and hi byte of the absolute address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);
        bus.borrow_mut().cpu_write(0x1001, 0x10);

        // Write data to the target address
        bus.borrow_mut().cpu_write(0x1020, 10);

        bus.borrow().cpu_mut().ABS();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ABX_test() {
        let mut bus = setup();
        // Write the lo and hi byte of the absolute address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);
        bus.borrow_mut().cpu_write(0x1001, 0x10);
        bus.borrow_mut().cpu_mut().x = 0x10;

        // Write data to the target address
        bus.borrow_mut().cpu_write(0x1030, 10);

        bus.borrow().cpu_mut().ABX();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn ABY_test() {
        let mut bus = setup();
        // Write the lo and hi byte of the absolute address of the data to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);
        bus.borrow_mut().cpu_write(0x1001, 0x10);
        bus.borrow_mut().cpu_mut().y = 0x10;

        // Write data to the target address
        bus.borrow_mut().cpu_write(0x1030, 10);

        bus.borrow().cpu_mut().ABY();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn IND_test() {
        let mut bus = setup();
        // Write the lo and hi byte of the pointer to the location of the target address to the program
        bus.borrow_mut().cpu_write(0x1000, 0x20);
        bus.borrow_mut().cpu_write(0x1001, 0x10);

        // Write target address to the pointer location
        bus.borrow_mut().cpu_write(0x1020, 0x23);
        bus.borrow_mut().cpu_write(0x1021, 0x01);

        // Write data to the location of the target address
        bus.borrow_mut().cpu_write(0x0123, 10);

        bus.borrow().cpu_mut().IND();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 10, "Fetched value incorrect");
    }

    #[test]
    fn IZX_test() {
        let mut bus = setup();
        // Write an address to the zero page
        bus.borrow().cpu_write(0x0020, 0x23);
        bus.borrow().cpu_write(0x0021, 0x01);

        // Write an offset to the program counter location
        bus.borrow().cpu_write(0x1000, 0x10);
        bus.borrow().cpu_mut().x = 0x10;

        // Write data to the target address

        bus.borrow().cpu_write(0x0123, 20);

        bus.borrow().cpu_mut().IZX();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 20, "Fetched data incorrect");
    }

    #[test]
    fn IZY_test() {
        let mut bus = setup();
        // Write an address to the zero page
        bus.borrow().cpu_write(0x0010, 0x20);
        bus.borrow().cpu_write(0x0011, 0x01);

        // Write an offset to the program counter location
        bus.borrow().cpu_write(0x1000, 0x10);
        bus.borrow().cpu_mut().y = 0x03;

        // Write data to the target address

        bus.borrow().cpu_write(0x0123, 20);

        bus.borrow().cpu_mut().IZY();
        bus.borrow().cpu_mut().fetch();

        assert_eq!(bus.borrow().cpu().fetched, 20, "Fetched data incorrect");
    }

    #[test]
    fn REL_test() {
        let mut bus = setup();

        // Write a relative address to the program counter location
        bus.borrow().cpu_write(0x1000, 0x10);

        bus.borrow().cpu_mut().REL();

        assert_eq!(bus.borrow().cpu_mut().addr_rel, 0x0010, "Address not set correctly");

        // Write a relative address to the program counter location which should count as negative
        bus.borrow().cpu_write(0x1001, 0x90);

        bus.borrow().cpu_mut().REL();

        assert_eq!(bus.borrow().cpu_mut().addr_rel, 0xFF90, "Negative relative address not handled correctly");
    }
}
