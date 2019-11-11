use std::rc::Rc;
use std::cell::RefCell;
use crate::bus::Bus;

bitflags! {
    pub struct Flags6502: u8 {
        const C = 0x01; // Carry Bit
        const Z = 0x02; // Zero
        const I = 0x04; // Disable Interrupts
        const D = 0x08; // Decimal Mode (unused in this implementation)
        const B = 0x10; // Break
        const U = 0x20; // Unused
        const V = 0x40; // Overflow
        const N = 0x80; // Negative
    }
}
lazy_static! {
    static ref LOOKUP: [Instruction; 16 * 16] = [
        Instruction::new("BRK", Cpu6502::BRK, Cpu6502::IMM, 7), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 3), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ZP0, 3), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("PHP", Cpu6502::PHP, Cpu6502::IMP, 3), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::IMM, 2), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ABS, 4), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BPL", Cpu6502::BPL, Cpu6502::REL, 2), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ZPX, 4), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("CLC", Cpu6502::CLC, Cpu6502::IMP, 2), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ABY, 4), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ABX, 4), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
        Instruction::new("JSR", Cpu6502::JSR, Cpu6502::ABS, 6), Instruction::new("AND", Cpu6502::AND, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("BIT", Cpu6502::BIT, Cpu6502::ZP0, 3), Instruction::new("AND", Cpu6502::AND, Cpu6502::ZP0, 3), Instruction::new("ROL", Cpu6502::ROL, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("PLP", Cpu6502::PLP, Cpu6502::IMP, 4), Instruction::new("AND", Cpu6502::AND, Cpu6502::IMM, 2), Instruction::new("ROL", Cpu6502::ROL, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("BIT", Cpu6502::BIT, Cpu6502::ABS, 4), Instruction::new("AND", Cpu6502::AND, Cpu6502::ABS, 4), Instruction::new("ROL", Cpu6502::ROL, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BMI", Cpu6502::BMI, Cpu6502::REL, 2), Instruction::new("AND", Cpu6502::AND, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("AND", Cpu6502::AND, Cpu6502::ZPX, 4), Instruction::new("ROL", Cpu6502::ROL, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("SEC", Cpu6502::SEC, Cpu6502::IMP, 2), Instruction::new("AND", Cpu6502::AND, Cpu6502::ABY, 4), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("AND", Cpu6502::AND, Cpu6502::ABX, 4), Instruction::new("ROL", Cpu6502::ROL, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
        Instruction::new("RTI", Cpu6502::RTI, Cpu6502::IMP, 6), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 3), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::ZP0, 3), Instruction::new("LSR", Cpu6502::LSR, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("PHA", Cpu6502::PHA, Cpu6502::IMP, 3), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::IMM, 2), Instruction::new("LSR", Cpu6502::LSR, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("JMP", Cpu6502::JMP, Cpu6502::ABS, 3), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::ABS, 4), Instruction::new("LSR", Cpu6502::LSR, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BVC", Cpu6502::BVC, Cpu6502::REL, 2), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::ZPX, 4), Instruction::new("LSR", Cpu6502::LSR, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("CLI", Cpu6502::CLI, Cpu6502::IMP, 2), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::ABY, 4), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("EOR", Cpu6502::EOR, Cpu6502::ABX, 4), Instruction::new("LSR", Cpu6502::LSR, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
        Instruction::new("RTS", Cpu6502::RTS, Cpu6502::IMP, 6), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 3), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::ZP0, 3), Instruction::new("ROR", Cpu6502::ROR, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("PLA", Cpu6502::PLA, Cpu6502::IMP, 4), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::IMM, 2), Instruction::new("ROR", Cpu6502::ROR, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("JMP", Cpu6502::JMP, Cpu6502::IND, 5), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::ABS, 4), Instruction::new("ROR", Cpu6502::ROR, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BVS", Cpu6502::BVS, Cpu6502::REL, 2), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::ZPX, 4), Instruction::new("ROR", Cpu6502::ROR, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("SEI", Cpu6502::SEI, Cpu6502::IMP, 2), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::ABY, 4), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ADC", Cpu6502::ADC, Cpu6502::ABX, 4), Instruction::new("ROR", Cpu6502::ROR, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
        Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("STA", Cpu6502::STA, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("STY", Cpu6502::STY, Cpu6502::ZP0, 3), Instruction::new("STA", Cpu6502::STA, Cpu6502::ZP0, 3), Instruction::new("STX", Cpu6502::STX, Cpu6502::ZP0, 3), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 3), Instruction::new("DEY", Cpu6502::DEY, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("TXA", Cpu6502::TXA, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("STY", Cpu6502::STY, Cpu6502::ABS, 4), Instruction::new("STA", Cpu6502::STA, Cpu6502::ABS, 4), Instruction::new("STX", Cpu6502::STX, Cpu6502::ABS, 4), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4),
        Instruction::new("BCC", Cpu6502::BCC, Cpu6502::REL, 2), Instruction::new("STA", Cpu6502::STA, Cpu6502::IZY, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("STY", Cpu6502::STY, Cpu6502::ZPX, 4), Instruction::new("STA", Cpu6502::STA, Cpu6502::ZPX, 4), Instruction::new("STX", Cpu6502::STX, Cpu6502::ZPY, 4), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4), Instruction::new("TYA", Cpu6502::TYA, Cpu6502::IMP, 2), Instruction::new("STA", Cpu6502::STA, Cpu6502::ABY, 5), Instruction::new("TXS", Cpu6502::TXS, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 5), Instruction::new("STA", Cpu6502::STA, Cpu6502::ABX, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5),
        Instruction::new("LDY", Cpu6502::LDY, Cpu6502::IMM, 2), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::IZX, 6), Instruction::new("LDX", Cpu6502::LDX, Cpu6502::IMM, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("LDY", Cpu6502::LDY, Cpu6502::ZP0, 3), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::ZP0, 3), Instruction::new("LDX", Cpu6502::LDX, Cpu6502::ZP0, 3), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 3), Instruction::new("TAY", Cpu6502::TAY, Cpu6502::IMP, 2), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::IMM, 2), Instruction::new("TAX", Cpu6502::TAX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("LDY", Cpu6502::LDY, Cpu6502::ABS, 4), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::ABS, 4), Instruction::new("LDX", Cpu6502::LDX, Cpu6502::ABS, 4), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4),
        Instruction::new("BCS", Cpu6502::BCS, Cpu6502::REL, 2), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("LDY", Cpu6502::LDY, Cpu6502::ZPX, 4), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::ZPX, 4), Instruction::new("LDX", Cpu6502::LDX, Cpu6502::ZPY, 4), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4), Instruction::new("CLV", Cpu6502::CLV, Cpu6502::IMP, 2), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::ABY, 4), Instruction::new("TSX", Cpu6502::TSX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4), Instruction::new("LDY", Cpu6502::LDY, Cpu6502::ABX, 4), Instruction::new("LDA", Cpu6502::LDA, Cpu6502::ABX, 4), Instruction::new("LDX", Cpu6502::LDX, Cpu6502::ABY, 4), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 4),
        Instruction::new("CPY", Cpu6502::CPY, Cpu6502::IMM, 2), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("CPY", Cpu6502::CPY, Cpu6502::ZP0, 3), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::ZP0, 3), Instruction::new("DEC", Cpu6502::DEC, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("INY", Cpu6502::INY, Cpu6502::IMP, 2), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::IMM, 2), Instruction::new("DEX", Cpu6502::DEX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("CPY", Cpu6502::CPY, Cpu6502::ABS, 4), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::ABS, 4), Instruction::new("DEC", Cpu6502::DEC, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BNE", Cpu6502::BNE, Cpu6502::REL, 2), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::ZPX, 4), Instruction::new("DEC", Cpu6502::DEC, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("CLD", Cpu6502::CLD, Cpu6502::IMP, 2), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::ABY, 4), Instruction::new("NOP", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("CMP", Cpu6502::CMP, Cpu6502::ABX, 4), Instruction::new("DEC", Cpu6502::DEC, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
        Instruction::new("CPX", Cpu6502::CPX, Cpu6502::IMM, 2), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("CPX", Cpu6502::CPX, Cpu6502::ZP0, 3), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::ZP0, 3), Instruction::new("INC", Cpu6502::INC, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("INX", Cpu6502::INX, Cpu6502::IMP, 2), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::IMM, 2), Instruction::new("NOP", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::SBC, Cpu6502::IMP, 2), Instruction::new("CPX", Cpu6502::CPX, Cpu6502::ABS, 4), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::ABS, 4), Instruction::new("INC", Cpu6502::INC, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
        Instruction::new("BEQ", Cpu6502::BEQ, Cpu6502::REL, 2), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::IZY, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::ZPX, 4), Instruction::new("INC", Cpu6502::INC, Cpu6502::ZPX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6), Instruction::new("SED", Cpu6502::SED, Cpu6502::IMP, 2), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::ABY, 4), Instruction::new("NOP", Cpu6502::NOP, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("SBC", Cpu6502::SBC, Cpu6502::ABX, 4), Instruction::new("INC", Cpu6502::INC, Cpu6502::ABX, 7), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 7),
    ];
}


impl Flags6502 {
    pub fn none() -> Self {
        Flags6502::C ^ Flags6502::C
    }
}


pub struct Cpu6502 {
    bus: Option<Rc<RefCell<Bus>>>,
    a: u8, // Accumulator Register
    x: u8, // X Register
    y: u8, // Y Register
    stkp: u8, // Stack Pointer
    pc: u16, // Program Counter
    status: Flags6502, // Status Register
    fetched: u8, // Fetched data for executing instruction
    addr_abs: u16, // Absolute memory address
    addr_rel: u16, // Relative memory address
    opcode: u8, // Opcode of current instruction
    cycles: u8, // Number or clock cycles left for current instruction

}

#[allow(non_snake_case, unused)]
impl Cpu6502 {

    pub fn new() -> Self {
        Cpu6502 {
            bus: None,
            a: 0,
            x: 0,
            y: 0,
            stkp: 0,
            pc: 0,
            status: Flags6502::none(),
            fetched: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,
        }
    }

    pub fn connect_bus(&mut self, bus: Rc<RefCell<Bus>> ) {
        self.bus = Some(bus);
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus.as_ref().expect("cpu not connected to Bus").borrow().read(addr, false)
    }

    fn write(&self, addr: u16, data: u8) {
        self.bus.as_ref().expect("cpu not connected to Bus").borrow_mut().write(addr, data)
    }

    pub fn get_flag(&self, flag: Flags6502) -> bool {
        !(self.status & flag).is_empty()
    }

    pub fn set_flag(&mut self, flag: Flags6502, set: bool) {
        self.status = if set {
            self.status | flag
        } else {
            self.status & !flag
        };
    }

    fn clock(&mut self) {
        if self.cycles == 0 {

            // Read the next opcode from the memory at the program counter
            self.opcode = self.read(self.pc);
            self.pc += 1;

            // Get the instruction specified by the next opcode
            let instruction = &LOOKUP[self.opcode as usize];

            // Get starting number of cycles
            self.cycles = instruction.cycles;

            // Set the addressing mode specified by the instruction
            let additional_cycle_addrmode = (instruction.addrmode)(self);

            // Call the actual functionality of the Instruction
            let additional_cycle_operate = (instruction.operate)(self);

            // If both addrmode and operate need another clock cycle, increase the required cycles by 1
            if additional_cycle_addrmode && additional_cycle_operate {
                self.cycles += 1
            };
        }

        self.cycles -= 1;
    }

    fn reset(&self) {}

    /// Interrupt request signal
    fn irq(&self) {}

    /// Non-maskable interrupt request signal
    fn nmi(&self) {}

    /// Fetches data from the given address
    fn fetch(&mut self) -> u8 {
        // If the addressing mode is 'implied', then there is no data to fetch
        // In this case, the fetched data is the data in the accumulator (see the IMP addressing mode)
        if LOOKUP[self.opcode as usize].addrmode as usize != Self::IMP as usize {
            self.fetched = self.read(self.addr_abs);
        }
        self.fetched
    }
}


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
    /// The memory address is an absolute value (so the inscruction is a 3-byte instruction)
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
        if ptr_lo == 0x00FF { // Simulate page boundary hardware bug
            self.addr_abs = ((self.read(0xFF00 & ptr) as u16) << 8) | self.read(ptr + 0) as u16
        } else { // Behave normally
            // This reads the high byte and low byte of the actual address
            self.addr_abs = ((self.read(ptr + 1) as u16) << 8) | self.read(ptr + 0) as u16;
        }

        false
    }

    /// Indirect Addressing of the Zero Page with X-register offset.
    /// This reads an address from the Page 0 (see ZP0) at the supplied offset byte with an additional offset of the value in the X-register
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

// Opcodes. These return true if they *potentially* need another clock cycle. false otherwise
// They also set the flags accordingly
#[allow(non_snake_case, unused)]
impl Cpu6502 {

    fn ADC(&mut self) -> bool {
        false
    }

    /// Performs a binary and between the accumulator and the fetched data
    fn AND(&mut self) -> bool {
        self.a &= self.fetch();
        // If the result is 0, set the Zero flag
        self.set_flag(Flags6502::Z, self.a == 0x00);
        // If the result is negative, set the Negative flag
        self.set_flag(Flags6502::N, self.a & 0x80 > 0);

        // Needs another clock cycle if page boundaries are crossed
        // As this is *potential* for operations, no conditionals are required
        true
    }


    fn ASL(&mut self) -> bool { false }

    /// Branch if the carry flag of the status register is clear
    fn BCC(&mut self) -> bool {
        if !self.get_flag(Flags6502::C) {
            self.branch();
        }
        false
    }

    /// Branch if the carry flag of the status register is set
    fn BCS(&mut self) -> bool {
        if self.get_flag(Flags6502::C) {
            self.branch()
        }
        false
    }

    /// Branch if equal (i.e. the Zero flag is set)
    fn BEQ(&mut self) -> bool {
        if self.get_flag(Flags6502::Z) {
            self.branch()
        }
        false
    }

    /// Branch if not equal (i.e. the Zero flag is clear)
    fn BNE(&mut self) -> bool {
        if !self.get_flag(Flags6502::Z) {
            self.branch()
        }
        false
    }

    /// Branch if positive
    fn BPL(&mut self) -> bool {
        if !self.get_flag(Flags6502::N) {
            self.branch()
        }
        false
    }

    /// Branch if negative
    fn BMI(&mut self) -> bool {
        if self.get_flag(Flags6502::N) {
            self.branch()
        }
        false
    }

    /// Branch if overflowed
    fn BVC(&mut self) -> bool {
        if self.get_flag(Flags6502::V) {
            self.branch()
        }
        false
    }

    /// Branch if not overflowed
    fn BVS(&mut self) -> bool {
        if !self.get_flag(Flags6502::V) {
            self.branch()
        }
        false
    }


    fn BIT(&mut self) -> bool { false }
    fn BRK(&mut self) -> bool { false }

    /// Clear Carry flag
    fn CLC(&mut self) -> bool {
        self.set_flag(Flags6502::C, false);
        false
    }

    /// Clear Decimal Mode flag
    fn CLD(&mut self) -> bool {
        self.set_flag(Flags6502::D, false);
        false
    }

    /// Clear Interrupt Disable Flag
    fn CLI(&mut self) -> bool {
        self.set_flag(Flags6502::I, false);
        false
    }

    /// Clear Overflow Flag
    fn CLV(&mut self) -> bool {
        self.set_flag(Flags6502::V, false);
        false
    }

    fn CMP(&mut self) -> bool { false }
    fn CPX(&mut self) -> bool { false }
    fn CPY(&mut self) -> bool { false }
    fn DEC(&mut self) -> bool { false }

    /// Decrements the X-register by 1
    fn DEX(&mut self) -> bool {
        self.x -= 1;
        false
    }

    /// Decrements the Y register by 1
    fn DEY(&mut self) -> bool {
        self.y -= 1;
        false
    }
    fn EOR(&mut self) -> bool { false }
    fn INC(&mut self) -> bool { false }
    fn INX(&mut self) -> bool { false }
    fn INY(&mut self) -> bool { false }
    fn JMP(&mut self) -> bool { false }
    fn JSR(&mut self) -> bool { false }
    fn LDA(&mut self) -> bool { false }
    fn LDX(&mut self) -> bool { false }
    fn LDY(&mut self) -> bool { false }
    fn LSR(&mut self) -> bool { false }
    fn NOP(&mut self) -> bool { false }
    fn ORA(&mut self) -> bool { false }
    fn PHA(&mut self) -> bool { false }
    fn PHP(&mut self) -> bool { false }
    fn PLA(&mut self) -> bool { false }
    fn PLP(&mut self) -> bool { false }
    fn ROL(&mut self) -> bool { false }
    fn ROR(&mut self) -> bool { false }
    fn RTI(&mut self) -> bool { false }
    fn RTS(&mut self) -> bool { false }
    fn SBC(&mut self) -> bool { false }
    fn SEC(&mut self) -> bool { false }
    fn SED(&mut self) -> bool { false }
    fn SEI(&mut self) -> bool { false }
    fn STA(&mut self) -> bool { false }
    fn STX(&mut self) -> bool { false }
    fn STY(&mut self) -> bool { false }
    fn TAX(&mut self) -> bool { false }
    fn TAY(&mut self) -> bool { false }
    fn TSX(&mut self) -> bool { false }
    fn TXA(&mut self) -> bool { false }
    fn TXS(&mut self) -> bool { false }
    fn TYA(&mut self) -> bool { false }

    // Illegal Opcode
    fn XXX(&mut self) -> bool { false }

    /// Branch method, because all branches *basically* work the same, just with different branch conditions
    fn branch(&mut self) {
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

}

struct Instruction{
    pub name: String,
    pub operate: fn(&mut Cpu6502) -> bool,
    pub addrmode: fn(&mut Cpu6502) -> bool,
    pub cycles: u8
}

impl Instruction {
    pub fn new(name: &str, operate: fn(&mut Cpu6502) -> bool, addrmode: fn(&mut Cpu6502) -> bool, cycles: u8) -> Self {
        Instruction {
            name: String::from(name),
            operate,
            addrmode,
            cycles
        }
    }
}



#[cfg(test)]
mod test {
    use crate::cpu6502::Cpu6502;
    use crate::cpu6502::Flags6502;

    #[test]
    fn flags_test() {
        let mut cpu = Cpu6502::new();

        cpu.set_flag(Flags6502::C, true);
        assert_eq!(cpu.status, Flags6502::C);

        cpu.set_flag(Flags6502::I, true);
        assert_eq!(cpu.status, Flags6502::C | Flags6502::I);
    }
}