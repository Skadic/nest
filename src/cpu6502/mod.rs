use crate::bus::Bus;
use std::cell::RefCell;
use std::rc::Rc;

mod addressing_modes;
mod opcodes;


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
        Instruction::new("BRK", Cpu6502::BRK, Cpu6502::IMP, 7), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::IZX, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 8), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 3), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ZP0, 3), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ZP0, 5), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 5), Instruction::new("PHP", Cpu6502::PHP, Cpu6502::IMP, 3), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::IMM, 2), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 2), Instruction::new("???", Cpu6502::NOP, Cpu6502::IMP, 4), Instruction::new("ORA", Cpu6502::ORA, Cpu6502::ABS, 4), Instruction::new("ASL", Cpu6502::ASL, Cpu6502::ABS, 6), Instruction::new("???", Cpu6502::XXX, Cpu6502::IMP, 6),
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

/// The 6502's hardcoded stack pointer base location
const STACK_POINTER_BASE: u16 = 0x0100;

/// The location of the new program counter when an Interrupt Request happens
const IRQ_PROGRAM_COUNTER: u16 = 0xFFFE;

/// The location of the new program counter when a Non-Maskable Interrupt happens
const NMI_PROGRAM_COUNTER: u16 = 0xFFFA;

#[derive(Debug)]
pub struct Cpu6502 {
    bus: Option<Rc<RefCell<Bus>>>,
    a: u8,             // Accumulator Register
    x: u8,             // X Register
    y: u8,             // Y Register
    stkp: u16,         // Stack Pointer
    pc: u16,           // Program Counter
    status: Flags6502, // Status Register
    fetched: u8,       // Fetched data for executing instruction
    addr_abs: u16,     // Absolute memory address
    addr_rel: u16,     // Relative memory address
    opcode: u8,        // Opcode of current instruction
    cycles: u8,        // Number or clock cycles left for current instruction
}

#[allow(non_snake_case, unused)]
impl Cpu6502 {
    pub fn new() -> Self {
        Cpu6502 {
            bus: None,
            a: 0,
            x: 0,
            y: 0,
            stkp: STACK_POINTER_BASE,
            pc: 0,
            status: Flags6502::empty(),
            fetched: 0,
            addr_abs: 0,
            addr_rel: 0,
            opcode: 0,
            cycles: 0,
        }
    }

    pub fn connect_bus(&mut self, bus: Rc<RefCell<Bus>>) {
        self.bus = Some(bus);
    }

    fn read(&self, addr: u16) -> u8 {
        self.bus
            .as_ref()
            .expect("cpu not connected to Bus")
            .borrow()
            .read(addr, false)
    }

    fn write(&self, addr: u16, data: u8) {
        self.bus
            .as_ref()
            .expect("cpu not connected to Bus")
            .borrow_mut()
            .write(addr, data)
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

    pub fn clock(&mut self) {
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

    // Configure the CPU into a known state
    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.stkp = 0xFD;
        self.status = Flags6502::empty() | Flags6502::U;

        // Hardcoded address that contains the address the program counter should be set to, in case of a reset
        self.addr_abs = 0xFFFC;
        let lo = self.read(self.addr_abs) as u16;
        let hi = self.read(self.addr_abs + 1) as u16;

        self.pc = (hi << 8) | lo;

        self.addr_abs = 0x0000;
        self.addr_rel = 0x0000;
        self.fetched = 0x00;

        // A reset takes time
        self.cycles = 8;
    }

    /// Interrupt request signal
    fn irq(&mut self) {
        if !self.get_flag(Flags6502::I) {
            // Save the Program counter to the stack
            self.write(
                STACK_POINTER_BASE + self.stkp,
                ((self.pc >> 8) & 0x00FF) as u8,
            );
            self.stkp -= 1;
            self.write(STACK_POINTER_BASE + self.stkp, (self.pc & 0x00FF) as u8);
            self.stkp -= 1;

            // Set flags accordingly
            self.set_flag(Flags6502::B, false);
            self.set_flag(Flags6502::U, true);
            self.set_flag(Flags6502::I, true);

            // Save the status register to stack
            self.write(STACK_POINTER_BASE + self.stkp, self.status.bits());
            self.stkp -= 1;

            // The value of the new program counter sits at this hardcoded address
            self.addr_abs = IRQ_PROGRAM_COUNTER;
            let lo = self.read(self.addr_abs) as u16;
            let hi = self.read(self.addr_abs + 1) as u16;
            self.pc = (hi << 8) | lo;

            // Interrupts take time
            self.cycles = 7;
        }
    }

    /// Non-maskable interrupt request signal
    fn nmi(&mut self) {
        // Save the Program counter to the stack
        self.write(
            STACK_POINTER_BASE + self.stkp,
            ((self.pc >> 8) & 0x00FF) as u8,
        );
        self.stkp -= 1;
        self.write(STACK_POINTER_BASE + self.stkp, (self.pc & 0x00FF) as u8);
        self.stkp -= 1;

        // Set flags accordingly
        self.set_flag(Flags6502::B, false);
        self.set_flag(Flags6502::U, true);
        self.set_flag(Flags6502::I, true);

        // Save the status register to stack
        self.write(STACK_POINTER_BASE + self.stkp, self.status.bits());
        self.stkp -= 1;

        // The value of the new program counter sits at this hardcoded address
        self.addr_abs = NMI_PROGRAM_COUNTER;
        let lo = self.read(self.addr_abs) as u16;
        let hi = self.read(self.addr_abs + 1) as u16;
        self.pc = (hi << 8) | lo;

        // Interrupts take time
        self.cycles = 7;
    }

    /// Return from an interrupt
    fn rti(&mut self) -> bool {
        self.stkp += 1;
        // Read status from the stack
        self.status = Flags6502::from_bits(self.read(STACK_POINTER_BASE + self.stkp)).unwrap();
        self.status &= !Flags6502::B;
        self.status &= !Flags6502::U;

        // Read the program counter from stack
        self.stkp += 1;
        self.pc = self.read(STACK_POINTER_BASE + self.stkp) as u16;
        self.stkp += 1;
        self.pc |= (self.read(STACK_POINTER_BASE + self.stkp) as u16) << 8;
        false
    }

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


struct Instruction {
    pub name: String,
    pub operate: fn(&mut Cpu6502) -> bool,
    pub addrmode: fn(&mut Cpu6502) -> bool,
    pub cycles: u8,
}

impl Instruction {
    pub fn new(
        name: &str,
        operate: fn(&mut Cpu6502) -> bool,
        addrmode: fn(&mut Cpu6502) -> bool,
        cycles: u8,
    ) -> Self {
        Instruction {
            name: String::from(name),
            operate,
            addrmode,
            cycles,
        }
    }
}

pub fn disassemble(program_bytes: Vec<u8>) -> Vec<String> {
    fn cmp_fn(f1: fn(&mut Cpu6502) -> bool, f2: fn(&mut Cpu6502) -> bool) -> bool {
        f1 as usize == f2 as usize
    }

    let mut program: Vec<String> = Vec::new();

    let mut i = 0;
    while i < program_bytes.len() {
        let mut string_instr_tokens: Vec<String> = Vec::new();

        let instruction = &LOOKUP[program_bytes[i] as usize];
        let mode = |addr_mode: fn(&mut Cpu6502) -> bool| cmp_fn(instruction.addrmode, addr_mode);
        string_instr_tokens.push(instruction.name.clone());
        if mode(Cpu6502::IMP) {
        } else if mode(Cpu6502::IMM) {
            i += 1;
            string_instr_tokens.push(format!("#${:0>4}", hex::encode(vec![program_bytes[i]])))
        } else if mode(Cpu6502::ZP0)
            || mode(Cpu6502::ZPX)
            || mode(Cpu6502::ZPY)
            || mode(Cpu6502::REL)
        {
            i += 1;
            string_instr_tokens.push(format!("${:0>4}", hex::encode(vec![program_bytes[i]])))
        } else {
            let mut address = Vec::new();
            i += 1;
            address.push(program_bytes[i]);
            i += 1;
            address.push(program_bytes[i]);
            string_instr_tokens.push(format!("${:0>4}", hex::encode(address)));
        }

        //println!("{}", string_instr_tokens.join(" "));
        program.push(string_instr_tokens.join(" "));
        i += 1;
    }

    program
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
    fn flags_test() {
        let mut cpu = Cpu6502::new();

        cpu.set_flag(Flags6502::C, true);
        assert_eq!(cpu.status, Flags6502::C);

        cpu.set_flag(Flags6502::I, true);
        assert_eq!(cpu.status, Flags6502::C | Flags6502::I);
    }

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
        let mut cpu: &mut Cpu6502 = &mut cpu.borrow_mut();

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
