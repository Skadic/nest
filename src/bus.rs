use crate::olc6502::Olc6502;
use std::cell::RefCell;
use std::rc::Rc;

const RAM_SIZE: usize = 64 * 1024;

pub struct Bus {
    cpu: Rc<RefCell<Olc6502>>,
    ram: [u8; RAM_SIZE]
}


impl Bus {

    pub fn new(cpu: Rc<RefCell<Olc6502>>) -> Rc<RefCell<Self>> {
        let mut bus = Rc::new(RefCell::new(
            Bus {
                cpu: cpu.clone(),
                ram: [0; RAM_SIZE]
            }
        ));
        cpu.borrow_mut().connect_bus(bus.clone());

        bus
    }


    pub fn write(&mut self, addr: u16, data: u8) {
        //if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize] = data;
        //}
    }

    pub fn read(&self, addr: u16, _read_only: bool) -> u8 {
        //if addr >= 0x0000 && addr <= 0xFFFF {
            self.ram[addr as usize]
        //} else {
        //    0x00
        //}
    }
}