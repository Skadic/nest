use crate::cpu6502::Cpu6502;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;

const RAM_SIZE: usize = 2048;

pub struct Bus {
    cpu: Rc<RefCell<Cpu6502>>,
    ram: [u8; RAM_SIZE],
}

impl Bus {
    pub fn new(cpu: Rc<RefCell<Cpu6502>>) -> Rc<RefCell<Self>> {
        let bus = Rc::new(RefCell::new(Bus {
            cpu: cpu.clone(),
            ram: [0; RAM_SIZE],
        }));
        cpu.borrow_mut().connect_bus(bus.clone());

        bus
    }

    pub fn cpuWrite(&mut self, addr: u16, data: u8) {
        self.ram[addr as usize] = data;
    }

    pub fn cpuRead(&self, addr: u16, _read_only: bool) -> u8 {
        self.ram[addr as usize]
    }
}

impl Debug for Bus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "bus")
    }
}
