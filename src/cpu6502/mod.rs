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
            .cpu_read(addr, false)
    }

    fn write(&self, addr: u16, data: u8) {
        self.bus
            .as_ref()
            .expect("cpu not connected to Bus")
            .borrow_mut()
            .cpu_write(addr, data)
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
    pub(crate) fn reset(&mut self) {
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

    /// Determines if 2 Functions are the same. Used for instructions
    fn cmp_fn(f1: fn(&mut Cpu6502) -> bool, f2: fn(&mut Cpu6502) -> bool) -> bool {
        f1 as usize == f2 as usize
    }

    fn addressing_mode_name(f: fn(&mut Cpu6502) -> bool) -> &'static str {
        let cmp = |addr_mode: fn(&mut Cpu6502) -> bool| cmp_fn(f, addr_mode);

        macro_rules! gen_if {
            ( $($x:ident), *) => {
                $(
                    if cmp(Cpu6502::$x) { return stringify!($x)}
                )*
            }
        }

        gen_if! {
            IMP, IMM, ZP0, ZPX, ZPY, ABS, ABX, ABY, IND, IZX, IZY, REL
        }

        "XXX"
    }

    let mut program: Vec<String> = Vec::new();

    let mut i = 0;
    while i < program_bytes.len() {
        // The buffer to hold the tokens that make up an instruction string
        let mut string_instr_tokens: Vec<String> = Vec::new();

        // Get the instruction from the lookup table that is identified by the current byte read
        let instruction : &Instruction = &LOOKUP[program_bytes[i] as usize];
        // A function that determines if the given addressing mode is equal to the addressing mode of the current instruction
        let mode = |addr_mode: fn(&mut Cpu6502) -> bool| cmp_fn(instruction.addrmode, addr_mode);
        // Adds the name of the current instruction to the tokens
        string_instr_tokens.push(instruction.name.clone());


        if mode(Cpu6502::IMP) {
            // If the addressing mode is implied there is nothing else to do. This if is just here,
            // so it's clear from reading the code that all addressing modes are handled
        } else if mode(Cpu6502::IMM) {
            // For immediate addressing, the additional data is 1 additional byte of data, so increase i by 1
            // And add the data formatted as a hexadecimal number to the tokens
            i += 1;
            string_instr_tokens.push(format!("#${:0>4}", hex::encode(vec![program_bytes[i]])))
        } else if mode(Cpu6502::ZP0)
            || mode(Cpu6502::ZPX)
            || mode(Cpu6502::ZPY)
            || mode(Cpu6502::REL)
        {
            // The same as with immediate addressing, but the formatting is a little different
            i += 1;
            string_instr_tokens.push(format!("${:0>4}", hex::encode(vec![program_bytes[i]])))
        } else {
            // For all other address modes, the supplied data consists of 2 bytes.
            // Gather them in a vector and convert them to a hexadecimal number
            let mut address = Vec::new();
            i += 1;
            address.push(program_bytes[i]);
            i += 1;
            address.push(program_bytes[i]);
            string_instr_tokens.push(format!("${:0>4}", hex::encode(address)));
        }

        // Add the address mode to the tokens
        string_instr_tokens.push(format!("({})", addressing_mode_name(instruction.addrmode)));

        program.push(string_instr_tokens.join(" "));
        i += 1;
    }

    program
}

#[allow(non_snake_case)]
#[cfg(test)]
mod test {
    use crate::cpu6502::Flags6502;
    use crate::cpu6502::{Cpu6502};

    #[test]
    fn flags_test() {
        let mut cpu = Cpu6502::new();

        cpu.set_flag(Flags6502::C, true);
        assert_eq!(cpu.status, Flags6502::C);

        cpu.set_flag(Flags6502::I, true);
        assert_eq!(cpu.status, Flags6502::C | Flags6502::I);
    }
}
