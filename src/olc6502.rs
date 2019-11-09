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
            vec![
                Instruction::new("AAA", Self::ABS,  Self::ABS, 1)
            ]
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