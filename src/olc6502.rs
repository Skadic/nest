use std::rc::Rc;
use std::cell::RefCell;
use crate::bus::Bus;
use std::fs::read;

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
        Instruction::new("BRK", Olc6502::BRK, Olc6502::IMM, 7), Instruction::new("ORA", Olc6502::ORA, Olc6502::IZX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 3), Instruction::new("ORA", Olc6502::ORA, Olc6502::ZP0, 3), Instruction::new("ASL", Olc6502::ASL, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("PHP", Olc6502::PHP, Olc6502::IMP, 3), Instruction::new("ORA", Olc6502::ORA, Olc6502::IMM, 2), Instruction::new("ASL", Olc6502::ASL, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("ORA", Olc6502::ORA, Olc6502::ABS, 4), Instruction::new("ASL", Olc6502::ASL, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BPL", Olc6502::BPL, Olc6502::REL, 2), Instruction::new("ORA", Olc6502::ORA, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("ORA", Olc6502::ORA, Olc6502::ZPX, 4), Instruction::new("ASL", Olc6502::ASL, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("CLC", Olc6502::CLC, Olc6502::IMP, 2), Instruction::new("ORA", Olc6502::ORA, Olc6502::ABY, 4), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("ORA", Olc6502::ORA, Olc6502::ABX, 4), Instruction::new("ASL", Olc6502::ASL, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
        Instruction::new("JSR", Olc6502::JSR, Olc6502::ABS, 6), Instruction::new("AND", Olc6502::AND, Olc6502::IZX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("BIT", Olc6502::BIT, Olc6502::ZP0, 3), Instruction::new("AND", Olc6502::AND, Olc6502::ZP0, 3), Instruction::new("ROL", Olc6502::ROL, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("PLP", Olc6502::PLP, Olc6502::IMP, 4), Instruction::new("AND", Olc6502::AND, Olc6502::IMM, 2), Instruction::new("ROL", Olc6502::ROL, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("BIT", Olc6502::BIT, Olc6502::ABS, 4), Instruction::new("AND", Olc6502::AND, Olc6502::ABS, 4), Instruction::new("ROL", Olc6502::ROL, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BMI", Olc6502::BMI, Olc6502::REL, 2), Instruction::new("AND", Olc6502::AND, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("AND", Olc6502::AND, Olc6502::ZPX, 4), Instruction::new("ROL", Olc6502::ROL, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("SEC", Olc6502::SEC, Olc6502::IMP, 2), Instruction::new("AND", Olc6502::AND, Olc6502::ABY, 4), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("AND", Olc6502::AND, Olc6502::ABX, 4), Instruction::new("ROL", Olc6502::ROL, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
        Instruction::new("RTI", Olc6502::RTI, Olc6502::IMP, 6), Instruction::new("EOR", Olc6502::EOR, Olc6502::IZX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 3), Instruction::new("EOR", Olc6502::EOR, Olc6502::ZP0, 3), Instruction::new("LSR", Olc6502::LSR, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("PHA", Olc6502::PHA, Olc6502::IMP, 3), Instruction::new("EOR", Olc6502::EOR, Olc6502::IMM, 2), Instruction::new("LSR", Olc6502::LSR, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("JMP", Olc6502::JMP, Olc6502::ABS, 3), Instruction::new("EOR", Olc6502::EOR, Olc6502::ABS, 4), Instruction::new("LSR", Olc6502::LSR, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BVC", Olc6502::BVC, Olc6502::REL, 2), Instruction::new("EOR", Olc6502::EOR, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("EOR", Olc6502::EOR, Olc6502::ZPX, 4), Instruction::new("LSR", Olc6502::LSR, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("CLI", Olc6502::CLI, Olc6502::IMP, 2), Instruction::new("EOR", Olc6502::EOR, Olc6502::ABY, 4), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("EOR", Olc6502::EOR, Olc6502::ABX, 4), Instruction::new("LSR", Olc6502::LSR, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
        Instruction::new("RTS", Olc6502::RTS, Olc6502::IMP, 6), Instruction::new("ADC", Olc6502::ADC, Olc6502::IZX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 3), Instruction::new("ADC", Olc6502::ADC, Olc6502::ZP0, 3), Instruction::new("ROR", Olc6502::ROR, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("PLA", Olc6502::PLA, Olc6502::IMP, 4), Instruction::new("ADC", Olc6502::ADC, Olc6502::IMM, 2), Instruction::new("ROR", Olc6502::ROR, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("JMP", Olc6502::JMP, Olc6502::IND, 5), Instruction::new("ADC", Olc6502::ADC, Olc6502::ABS, 4), Instruction::new("ROR", Olc6502::ROR, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BVS", Olc6502::BVS, Olc6502::REL, 2), Instruction::new("ADC", Olc6502::ADC, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("ADC", Olc6502::ADC, Olc6502::ZPX, 4), Instruction::new("ROR", Olc6502::ROR, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("SEI", Olc6502::SEI, Olc6502::IMP, 2), Instruction::new("ADC", Olc6502::ADC, Olc6502::ABY, 4), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("ADC", Olc6502::ADC, Olc6502::ABX, 4), Instruction::new("ROR", Olc6502::ROR, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
        Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("STA", Olc6502::STA, Olc6502::IZX, 6), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("STY", Olc6502::STY, Olc6502::ZP0, 3), Instruction::new("STA", Olc6502::STA, Olc6502::ZP0, 3), Instruction::new("STX", Olc6502::STX, Olc6502::ZP0, 3), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 3), Instruction::new("DEY", Olc6502::DEY, Olc6502::IMP, 2), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("TXA", Olc6502::TXA, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("STY", Olc6502::STY, Olc6502::ABS, 4), Instruction::new("STA", Olc6502::STA, Olc6502::ABS, 4), Instruction::new("STX", Olc6502::STX, Olc6502::ABS, 4), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4),
        Instruction::new("BCC", Olc6502::BCC, Olc6502::REL, 2), Instruction::new("STA", Olc6502::STA, Olc6502::IZY, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("STY", Olc6502::STY, Olc6502::ZPX, 4), Instruction::new("STA", Olc6502::STA, Olc6502::ZPX, 4), Instruction::new("STX", Olc6502::STX, Olc6502::ZPY, 4), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4), Instruction::new("TYA", Olc6502::TYA, Olc6502::IMP, 2), Instruction::new("STA", Olc6502::STA, Olc6502::ABY, 5), Instruction::new("TXS", Olc6502::TXS, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 5), Instruction::new("STA", Olc6502::STA, Olc6502::ABX, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5),
        Instruction::new("LDY", Olc6502::LDY, Olc6502::IMM, 2), Instruction::new("LDA", Olc6502::LDA, Olc6502::IZX, 6), Instruction::new("LDX", Olc6502::LDX, Olc6502::IMM, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("LDY", Olc6502::LDY, Olc6502::ZP0, 3), Instruction::new("LDA", Olc6502::LDA, Olc6502::ZP0, 3), Instruction::new("LDX", Olc6502::LDX, Olc6502::ZP0, 3), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 3), Instruction::new("TAY", Olc6502::TAY, Olc6502::IMP, 2), Instruction::new("LDA", Olc6502::LDA, Olc6502::IMM, 2), Instruction::new("TAX", Olc6502::TAX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("LDY", Olc6502::LDY, Olc6502::ABS, 4), Instruction::new("LDA", Olc6502::LDA, Olc6502::ABS, 4), Instruction::new("LDX", Olc6502::LDX, Olc6502::ABS, 4), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4),
        Instruction::new("BCS", Olc6502::BCS, Olc6502::REL, 2), Instruction::new("LDA", Olc6502::LDA, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("LDY", Olc6502::LDY, Olc6502::ZPX, 4), Instruction::new("LDA", Olc6502::LDA, Olc6502::ZPX, 4), Instruction::new("LDX", Olc6502::LDX, Olc6502::ZPY, 4), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4), Instruction::new("CLV", Olc6502::CLV, Olc6502::IMP, 2), Instruction::new("LDA", Olc6502::LDA, Olc6502::ABY, 4), Instruction::new("TSX", Olc6502::TSX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4), Instruction::new("LDY", Olc6502::LDY, Olc6502::ABX, 4), Instruction::new("LDA", Olc6502::LDA, Olc6502::ABX, 4), Instruction::new("LDX", Olc6502::LDX, Olc6502::ABY, 4), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 4),
        Instruction::new("CPY", Olc6502::CPY, Olc6502::IMM, 2), Instruction::new("CMP", Olc6502::CMP, Olc6502::IZX, 6), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("CPY", Olc6502::CPY, Olc6502::ZP0, 3), Instruction::new("CMP", Olc6502::CMP, Olc6502::ZP0, 3), Instruction::new("DEC", Olc6502::DEC, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("INY", Olc6502::INY, Olc6502::IMP, 2), Instruction::new("CMP", Olc6502::CMP, Olc6502::IMM, 2), Instruction::new("DEX", Olc6502::DEX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("CPY", Olc6502::CPY, Olc6502::ABS, 4), Instruction::new("CMP", Olc6502::CMP, Olc6502::ABS, 4), Instruction::new("DEC", Olc6502::DEC, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BNE", Olc6502::BNE, Olc6502::REL, 2), Instruction::new("CMP", Olc6502::CMP, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("CMP", Olc6502::CMP, Olc6502::ZPX, 4), Instruction::new("DEC", Olc6502::DEC, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("CLD", Olc6502::CLD, Olc6502::IMP, 2), Instruction::new("CMP", Olc6502::CMP, Olc6502::ABY, 4), Instruction::new("NOP", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("CMP", Olc6502::CMP, Olc6502::ABX, 4), Instruction::new("DEC", Olc6502::DEC, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
        Instruction::new("CPX", Olc6502::CPX, Olc6502::IMM, 2), Instruction::new("SBC", Olc6502::SBC, Olc6502::IZX, 6), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("CPX", Olc6502::CPX, Olc6502::ZP0, 3), Instruction::new("SBC", Olc6502::SBC, Olc6502::ZP0, 3), Instruction::new("INC", Olc6502::INC, Olc6502::ZP0, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 5), Instruction::new("INX", Olc6502::INX, Olc6502::IMP, 2), Instruction::new("SBC", Olc6502::SBC, Olc6502::IMM, 2), Instruction::new("NOP", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::SBC, Olc6502::IMP, 2), Instruction::new("CPX", Olc6502::CPX, Olc6502::ABS, 4), Instruction::new("SBC", Olc6502::SBC, Olc6502::ABS, 4), Instruction::new("INC", Olc6502::INC, Olc6502::ABS, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6),
        Instruction::new("BEQ", Olc6502::BEQ, Olc6502::REL, 2), Instruction::new("SBC", Olc6502::SBC, Olc6502::IZY, 5), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 8), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("SBC", Olc6502::SBC, Olc6502::ZPX, 4), Instruction::new("INC", Olc6502::INC, Olc6502::ZPX, 6), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 6), Instruction::new("SED", Olc6502::SED, Olc6502::IMP, 2), Instruction::new("SBC", Olc6502::SBC, Olc6502::ABY, 4), Instruction::new("NOP", Olc6502::NOP, Olc6502::IMP, 2), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7), Instruction::new("???", Olc6502::NOP, Olc6502::IMP, 4), Instruction::new("SBC", Olc6502::SBC, Olc6502::ABX, 4), Instruction::new("INC", Olc6502::INC, Olc6502::ABX, 7), Instruction::new("???", Olc6502::XXX, Olc6502::IMP, 7),
    ];
}


impl Flags6502 {
    pub fn none() -> Self {
        Flags6502::C ^ Flags6502::C
    }
}


pub struct Olc6502 {
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
impl Olc6502 {

    pub fn new() -> Self {
        let mut cpu = Olc6502 {
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
        };

        cpu
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

    // Addressing Modes. These return true if they need another clock cycle. false otherwise

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
        self.addr_abs += y;


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



    // Opcodes. These return true if they need another clock cycle. false otherwise
    fn ADC(&mut self) -> bool { false }
    fn AND(&mut self) -> bool { false }
    fn ASL(&mut self) -> bool { false }
    fn BCC(&mut self) -> bool { false }
    fn BCS(&mut self) -> bool { false }
    fn BEQ(&mut self) -> bool { false }
    fn BIT(&mut self) -> bool { false }
    fn BMI(&mut self) -> bool { false }
    fn BNE(&mut self) -> bool { false }
    fn BPL(&mut self) -> bool { false }
    fn BRK(&mut self) -> bool { false }
    fn BVC(&mut self) -> bool { false }
    fn BVS(&mut self) -> bool { false }
    fn CLC(&mut self) -> bool { false }
    fn CLD(&mut self) -> bool { false }
    fn CLI(&mut self) -> bool { false }
    fn CLV(&mut self) -> bool { false }
    fn CMP(&mut self) -> bool { false }
    fn CPX(&mut self) -> bool { false }
    fn CPY(&mut self) -> bool { false }
    fn DEC(&mut self) -> bool { false }
    fn DEX(&mut self) -> bool { false }
    fn DEY(&mut self) -> bool { false }
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

    fn fetch(&self) -> u8 { 0 }
}

struct Instruction{
    pub name: String,
    pub operate: fn(&mut Olc6502) -> bool,
    pub addrmode: fn(&mut Olc6502) -> bool,
    pub cycles: u8
}

impl Instruction {
    pub fn new(name: &str, operate: fn(&mut Olc6502) -> bool, addrmode: fn(&mut Olc6502) -> bool, cycles: u8) -> Self {
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
    use crate::olc6502::Olc6502;
    use crate::olc6502::Flags6502;

    #[test]
    fn flags_test() {
        let mut cpu = Olc6502::new();

        cpu.set_flag(Flags6502::C, true);
        assert_eq!(cpu.status, Flags6502::C);

        cpu.set_flag(Flags6502::I, true);
        assert_eq!(cpu.status, Flags6502::C | Flags6502::I);
    }
}