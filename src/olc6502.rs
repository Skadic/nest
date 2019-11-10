use std::rc::Rc;
use std::cell::RefCell;
use crate::bus::Bus;

bitflags! {
    pub struct Flags6502: u8 {
        const C = (1 << 0); // Carry Bit
        const Z = (1 << 1); // Zero
        const I = (1 << 2); // Disable Interrupts
        const D = (1 << 3); // Decimal Mode (unused in this implementation)
        const B = (1 << 4); // Break
        const U = (1 << 5); // Unused
        const V = (1 << 6); // Overflow
        const N = (1 << 7); // Negative
    }
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
    lookup: Vec<Vec<Instruction>> // Lookup table for instructions

}

#[allow(non_snake_case)]
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
            lookup: vec![],
        };

        cpu.build_lookup();

        cpu
    }

    fn build_lookup(&mut self) {
        self.lookup = vec![
            vec![Instruction::new("BRK", Self::BRK, Self::IMM, 7), Instruction::new("ORA", Self::ORA, Self::IZX, 6), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 3), Instruction::new("ORA", Self::ORA, Self::ZP0, 3), Instruction::new("ASL", Self::ASL, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("PHP", Self::PHP, Self::IMP, 3), Instruction::new("ORA", Self::ORA, Self::IMM, 2), Instruction::new("ASL", Self::ASL, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("ORA", Self::ORA, Self::ABS, 4), Instruction::new("ASL", Self::ASL, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BPL", Self::BPL, Self::REL, 2), Instruction::new("ORA", Self::ORA, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("ORA", Self::ORA, Self::ZPX, 4), Instruction::new("ASL", Self::ASL, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("CLC", Self::CLC, Self::IMP, 2), Instruction::new("ORA", Self::ORA, Self::ABY, 4), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("ORA", Self::ORA, Self::ABX, 4), Instruction::new("ASL", Self::ASL, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
            vec![Instruction::new("JSR", Self::JSR, Self::ABS, 6), Instruction::new("AND", Self::AND, Self::IZX, 6), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("BIT", Self::BIT, Self::ZP0, 3), Instruction::new("AND", Self::AND, Self::ZP0, 3), Instruction::new("ROL", Self::ROL, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("PLP", Self::PLP, Self::IMP, 4), Instruction::new("AND", Self::AND, Self::IMM, 2), Instruction::new("ROL", Self::ROL, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("BIT", Self::BIT, Self::ABS, 4), Instruction::new("AND", Self::AND, Self::ABS, 4), Instruction::new("ROL", Self::ROL, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BMI", Self::BMI, Self::REL, 2), Instruction::new("AND", Self::AND, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("AND", Self::AND, Self::ZPX, 4), Instruction::new("ROL", Self::ROL, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("SEC", Self::SEC, Self::IMP, 2), Instruction::new("AND", Self::AND, Self::ABY, 4), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("AND", Self::AND, Self::ABX, 4), Instruction::new("ROL", Self::ROL, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
            vec![Instruction::new("RTI", Self::RTI, Self::IMP, 6), Instruction::new("EOR", Self::EOR, Self::IZX, 6), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 3), Instruction::new("EOR", Self::EOR, Self::ZP0, 3), Instruction::new("LSR", Self::LSR, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("PHA", Self::PHA, Self::IMP, 3), Instruction::new("EOR", Self::EOR, Self::IMM, 2), Instruction::new("LSR", Self::LSR, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("JMP", Self::JMP, Self::ABS, 3), Instruction::new("EOR", Self::EOR, Self::ABS, 4), Instruction::new("LSR", Self::LSR, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BVC", Self::BVC, Self::REL, 2), Instruction::new("EOR", Self::EOR, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("EOR", Self::EOR, Self::ZPX, 4), Instruction::new("LSR", Self::LSR, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("CLI", Self::CLI, Self::IMP, 2), Instruction::new("EOR", Self::EOR, Self::ABY, 4), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("EOR", Self::EOR, Self::ABX, 4), Instruction::new("LSR", Self::LSR, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
            vec![Instruction::new("RTS", Self::RTS, Self::IMP, 6), Instruction::new("ADC", Self::ADC, Self::IZX, 6), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 3), Instruction::new("ADC", Self::ADC, Self::ZP0, 3), Instruction::new("ROR", Self::ROR, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("PLA", Self::PLA, Self::IMP, 4), Instruction::new("ADC", Self::ADC, Self::IMM, 2), Instruction::new("ROR", Self::ROR, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("JMP", Self::JMP, Self::IND, 5), Instruction::new("ADC", Self::ADC, Self::ABS, 4), Instruction::new("ROR", Self::ROR, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BVS", Self::BVS, Self::REL, 2), Instruction::new("ADC", Self::ADC, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("ADC", Self::ADC, Self::ZPX, 4), Instruction::new("ROR", Self::ROR, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("SEI", Self::SEI, Self::IMP, 2), Instruction::new("ADC", Self::ADC, Self::ABY, 4), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("ADC", Self::ADC, Self::ABX, 4), Instruction::new("ROR", Self::ROR, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
            vec![Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("STA", Self::STA, Self::IZX, 6), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("STY", Self::STY, Self::ZP0, 3), Instruction::new("STA", Self::STA, Self::ZP0, 3), Instruction::new("STX", Self::STX, Self::ZP0, 3), Instruction::new("???", Self::XXX, Self::IMP, 3), Instruction::new("DEY", Self::DEY, Self::IMP, 2), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("TXA", Self::TXA, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("STY", Self::STY, Self::ABS, 4), Instruction::new("STA", Self::STA, Self::ABS, 4), Instruction::new("STX", Self::STX, Self::ABS, 4), Instruction::new("???", Self::XXX, Self::IMP, 4), ],
            vec![Instruction::new("BCC", Self::BCC, Self::REL, 2), Instruction::new("STA", Self::STA, Self::IZY, 6), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("STY", Self::STY, Self::ZPX, 4), Instruction::new("STA", Self::STA, Self::ZPX, 4), Instruction::new("STX", Self::STX, Self::ZPY, 4), Instruction::new("???", Self::XXX, Self::IMP, 4), Instruction::new("TYA", Self::TYA, Self::IMP, 2), Instruction::new("STA", Self::STA, Self::ABY, 5), Instruction::new("TXS", Self::TXS, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("???", Self::NOP, Self::IMP, 5), Instruction::new("STA", Self::STA, Self::ABX, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), ],
            vec![Instruction::new("LDY", Self::LDY, Self::IMM, 2), Instruction::new("LDA", Self::LDA, Self::IZX, 6), Instruction::new("LDX", Self::LDX, Self::IMM, 2), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("LDY", Self::LDY, Self::ZP0, 3), Instruction::new("LDA", Self::LDA, Self::ZP0, 3), Instruction::new("LDX", Self::LDX, Self::ZP0, 3), Instruction::new("???", Self::XXX, Self::IMP, 3), Instruction::new("TAY", Self::TAY, Self::IMP, 2), Instruction::new("LDA", Self::LDA, Self::IMM, 2), Instruction::new("TAX", Self::TAX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("LDY", Self::LDY, Self::ABS, 4), Instruction::new("LDA", Self::LDA, Self::ABS, 4), Instruction::new("LDX", Self::LDX, Self::ABS, 4), Instruction::new("???", Self::XXX, Self::IMP, 4), ],
            vec![Instruction::new("BCS", Self::BCS, Self::REL, 2), Instruction::new("LDA", Self::LDA, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("LDY", Self::LDY, Self::ZPX, 4), Instruction::new("LDA", Self::LDA, Self::ZPX, 4), Instruction::new("LDX", Self::LDX, Self::ZPY, 4), Instruction::new("???", Self::XXX, Self::IMP, 4), Instruction::new("CLV", Self::CLV, Self::IMP, 2), Instruction::new("LDA", Self::LDA, Self::ABY, 4), Instruction::new("TSX", Self::TSX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 4), Instruction::new("LDY", Self::LDY, Self::ABX, 4), Instruction::new("LDA", Self::LDA, Self::ABX, 4), Instruction::new("LDX", Self::LDX, Self::ABY, 4), Instruction::new("???", Self::XXX, Self::IMP, 4), ],
            vec![Instruction::new("CPY", Self::CPY, Self::IMM, 2), Instruction::new("CMP", Self::CMP, Self::IZX, 6), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("CPY", Self::CPY, Self::ZP0, 3), Instruction::new("CMP", Self::CMP, Self::ZP0, 3), Instruction::new("DEC", Self::DEC, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("INY", Self::INY, Self::IMP, 2), Instruction::new("CMP", Self::CMP, Self::IMM, 2), Instruction::new("DEX", Self::DEX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("CPY", Self::CPY, Self::ABS, 4), Instruction::new("CMP", Self::CMP, Self::ABS, 4), Instruction::new("DEC", Self::DEC, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BNE", Self::BNE, Self::REL, 2), Instruction::new("CMP", Self::CMP, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("CMP", Self::CMP, Self::ZPX, 4), Instruction::new("DEC", Self::DEC, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("CLD", Self::CLD, Self::IMP, 2), Instruction::new("CMP", Self::CMP, Self::ABY, 4), Instruction::new("NOP", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("CMP", Self::CMP, Self::ABX, 4), Instruction::new("DEC", Self::DEC, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
            vec![Instruction::new("CPX", Self::CPX, Self::IMM, 2), Instruction::new("SBC", Self::SBC, Self::IZX, 6), Instruction::new("???", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("CPX", Self::CPX, Self::ZP0, 3), Instruction::new("SBC", Self::SBC, Self::ZP0, 3), Instruction::new("INC", Self::INC, Self::ZP0, 5), Instruction::new("???", Self::XXX, Self::IMP, 5), Instruction::new("INX", Self::INX, Self::IMP, 2), Instruction::new("SBC", Self::SBC, Self::IMM, 2), Instruction::new("NOP", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::SBC, Self::IMP, 2), Instruction::new("CPX", Self::CPX, Self::ABS, 4), Instruction::new("SBC", Self::SBC, Self::ABS, 4), Instruction::new("INC", Self::INC, Self::ABS, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), ],
            vec![Instruction::new("BEQ", Self::BEQ, Self::REL, 2), Instruction::new("SBC", Self::SBC, Self::IZY, 5), Instruction::new("???", Self::XXX, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 8), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("SBC", Self::SBC, Self::ZPX, 4), Instruction::new("INC", Self::INC, Self::ZPX, 6), Instruction::new("???", Self::XXX, Self::IMP, 6), Instruction::new("SED", Self::SED, Self::IMP, 2), Instruction::new("SBC", Self::SBC, Self::ABY, 4), Instruction::new("NOP", Self::NOP, Self::IMP, 2), Instruction::new("???", Self::XXX, Self::IMP, 7), Instruction::new("???", Self::NOP, Self::IMP, 4), Instruction::new("SBC", Self::SBC, Self::ABX, 4), Instruction::new("INC", Self::INC, Self::ABX, 7), Instruction::new("???", Self::XXX, Self::IMP, 7), ],
        ];
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
        self.status = match set {
            true    => self.status | flag,
            false   => if self.get_flag(flag) { self.status ^ flag } else { flag }
        };
    }


    // Addressing Modes

    pub fn IMP(&self) -> u8 { 0 }
    pub fn ZP0(&self) -> u8 { 0 }
    pub fn ZPY(&self) -> u8 { 0 }
    pub fn ABS(&self) -> u8 { 0 }
    pub fn ABY(&self) -> u8 { 0 }
    pub fn IZX(&self) -> u8 { 0 }
    pub fn IMM(&self) -> u8 { 0 }
    pub fn ZPX(&self) -> u8 { 0 }
    pub fn REL(&self) -> u8 { 0 }
    pub fn ABX(&self) -> u8 { 0 }
    pub fn IND(&self) -> u8 { 0 }
    pub fn IZY(&self) -> u8 { 0 }

    // Opcodes
    fn ADC(&self) -> u8 { 0 }
    fn AND(&self) -> u8 { 0 }
    fn ASL(&self) -> u8 { 0 }
    fn BCC(&self) -> u8 { 0 }
    fn BCS(&self) -> u8 { 0 }
    fn BEQ(&self) -> u8 { 0 }
    fn BIT(&self) -> u8 { 0 }
    fn BMI(&self) -> u8 { 0 }
    fn BNE(&self) -> u8 { 0 }
    fn BPL(&self) -> u8 { 0 }
    fn BRK(&self) -> u8 { 0 }
    fn BVC(&self) -> u8 { 0 }
    fn BVS(&self) -> u8 { 0 }
    fn CLC(&self) -> u8 { 0 }
    fn CLD(&self) -> u8 { 0 }
    fn CLI(&self) -> u8 { 0 }
    fn CLV(&self) -> u8 { 0 }
    fn CMP(&self) -> u8 { 0 }
    fn CPX(&self) -> u8 { 0 }
    fn CPY(&self) -> u8 { 0 }
    fn DEC(&self) -> u8 { 0 }
    fn DEX(&self) -> u8 { 0 }
    fn DEY(&self) -> u8 { 0 }
    fn EOR(&self) -> u8 { 0 }
    fn INC(&self) -> u8 { 0 }
    fn INX(&self) -> u8 { 0 }
    fn INY(&self) -> u8 { 0 }
    fn JMP(&self) -> u8 { 0 }
    fn JSR(&self) -> u8 { 0 }
    fn LDA(&self) -> u8 { 0 }
    fn LDX(&self) -> u8 { 0 }
    fn LDY(&self) -> u8 { 0 }
    fn LSR(&self) -> u8 { 0 }
    fn NOP(&self) -> u8 { 0 }
    fn ORA(&self) -> u8 { 0 }
    fn PHA(&self) -> u8 { 0 }
    fn PHP(&self) -> u8 { 0 }
    fn PLA(&self) -> u8 { 0 }
    fn PLP(&self) -> u8 { 0 }
    fn ROL(&self) -> u8 { 0 }
    fn ROR(&self) -> u8 { 0 }
    fn RTI(&self) -> u8 { 0 }
    fn RTS(&self) -> u8 { 0 }
    fn SBC(&self) -> u8 { 0 }
    fn SEC(&self) -> u8 { 0 }
    fn SED(&self) -> u8 { 0 }
    fn SEI(&self) -> u8 { 0 }
    fn STA(&self) -> u8 { 0 }
    fn STX(&self) -> u8 { 0 }
    fn STY(&self) -> u8 { 0 }
    fn TAX(&self) -> u8 { 0 }
    fn TAY(&self) -> u8 { 0 }
    fn TSX(&self) -> u8 { 0 }
    fn TXA(&self) -> u8 { 0 }
    fn TXS(&self) -> u8 { 0 }
    fn TYA(&self) -> u8 { 0 }

    // Illegal Opcode
    fn XXX(&self) -> u8 { 0 }


    fn clock(&self) {}
    fn reset(&self) {}
    /// Interrupt request signal
    fn irq(&self) {}
    /// Non-maskable interrupt request signal
    fn nmi(&self) {}

    fn fetch(&self) -> u8 { 0 }
}

struct Instruction{
    name: String,
    operate: fn(&Olc6502) -> u8,
    addrmode: fn(&Olc6502) -> u8,
    cycles: u8
}

impl Instruction {
    pub fn new(name: &str, operate: fn(&Olc6502) -> u8, addrmode: fn(&Olc6502) -> u8, cycles: u8) -> Self {
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