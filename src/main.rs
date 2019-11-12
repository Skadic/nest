#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

use std::rc::Rc;
use std::cell::RefCell;

mod cpu6502;
mod bus;

fn main() {
    let cpu = Rc::new(RefCell::new(cpu6502::Cpu6502::new()));
    let bus = bus::Bus::new(cpu.clone());

    let program = "A9 05 AA A9 06 8E 11 11 6D 11 11";

    for (i, b) in parse_program(program).into_iter().enumerate() {
        bus.borrow_mut().write(i as u16, b);
    }
    for instr in cpu6502::disassemble(parse_program(program)).into_iter() {
        println!("{}", instr);
    }

    for _ in 0..cpu6502::disassemble(parse_program(program)).len() {
        cpu.borrow_mut().clock()    
    }
    
    println!("{}", bus.borrow().read(0x1111, false));
    println!("{:?}", cpu.borrow());
}

fn parse_program(program: &str) -> Vec<u8> {
    hex::decode(program.split_whitespace().collect::<String>()).expect("error parsing program")
}
