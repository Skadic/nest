use crate::cpu6502::Cpu6502;
use std::cell::RefCell;
use std::fmt::Debug;
use std::rc::Rc;
use crate::ppu2C02::Ppu2C02;
use crate::cartridge::Cartridge;

const RAM_SIZE: usize = 2048;

pub struct Bus {
    cpu: Rc<RefCell<Cpu6502>>,
    ppu: Rc<RefCell<Ppu2C02>>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    cpu_ram: [u8; RAM_SIZE],
    system_clock_counter: u64
}

impl Bus {
    pub fn new(cpu: Rc<RefCell<Cpu6502>>, ppu: Rc<RefCell<Ppu2C02>>) -> Rc<RefCell<Self>> {
        let bus = Rc::new(RefCell::new(Bus {
            cpu: cpu.clone(),
            ppu: ppu.clone(),
            cartridge: None,
            cpu_ram: [0; RAM_SIZE],
            system_clock_counter: 0,
        }));
        cpu.borrow_mut().connect_bus(bus.clone());

        bus
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {

        if let Some(cartridge) = self.cartridge.as_ref() { // Cartridge gets "Priority access" to memory
            if cartridge.borrow_mut().cpu_write(addr, data) {
                return;
            }
        }

        if addr <= 0x1FFF { // Address range of the RAM
            self.cpu_ram[addr as usize & 0x07FF] = data; // As the actual 2kb of RAM are mirrored across an 8kb address range, the logic AND maps the given address to the address within the 2kb range
        } else if addr <= 0x3FFF { // Address range of the PPU
            self.ppu.borrow_mut().cpu_write(addr & 0x0007, data); // Mirroring again. And yes, the ppu only has 8 bytes of memory
        }
    }

    pub fn cpu_read(&self, addr: u16, read_only: bool) -> u8 {
        let mut data = 0x00;
        if let Some(cartridge) = self.cartridge.as_ref() { // Cartridge gets "Priority access" to memory
            if cartridge.borrow_mut().cpu_read(addr, &mut data) {
                return data;
            }
        }

        if addr <= 0x1FFF {
            self.cpu_ram[addr as usize & 0x07FF] // As the actual 2kb of RAM are mirrored across an 8kb address range, the logic AND maps the given address to the address within the 2kb range
        } else if addr <= 0x3FFF { // Address range of the PPU
            self.ppu.borrow_mut().cpu_read(addr & 0x0007, read_only) // Mirroring again. And yes, the ppu only has 8 bytes of memory
        } else {
            0x00
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge.clone());
        self.ppu.borrow_mut().connect_cartridge(cartridge);
    }

    pub fn reset(&mut self) {
        self.cpu.borrow_mut().reset();
        self.system_clock_counter = 0;
    }

    pub fn clock(&mut self) {

    }
}

impl Debug for Bus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "bus")
    }
}
