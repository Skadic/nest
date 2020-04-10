use crate::cpu6502::Cpu6502;
use std::cell::{RefCell, Ref};
use std::fmt::Debug;
use std::rc::Rc;
use crate::ppu2C02::Ppu2C02;
use crate::cartridge::Cartridge;
use bitflags::_core::cell::RefMut;

const RAM_SIZE: usize = 2048;

pub struct Bus {
    cpu: RefCell<Cpu6502>,
    ppu: RefCell<Ppu2C02>,
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    cpu_ram: RefCell<[u8; RAM_SIZE]>,
    system_clock_counter: RefCell<u64>
}

impl Bus {
    pub fn new(cpu: Cpu6502, ppu: Ppu2C02) -> Rc<RefCell<Self>> {
        let bus = Rc::new(RefCell::new(Bus {
            cpu: RefCell::new(cpu),
            ppu: RefCell::new(ppu),
            cartridge: None,
            cpu_ram: RefCell::new([0; RAM_SIZE]),
            system_clock_counter: RefCell::new(0),
        }));
        bus.borrow_mut().cpu.borrow_mut().connect_bus(bus.clone());

        bus
    }

    pub fn cpu(&self) -> Ref<Cpu6502> {
        self.cpu.borrow()
    }

    pub fn cpu_mut(&mut self) -> RefMut<Cpu6502> {
        self.cpu.borrow_mut()
    }

    pub fn ppu(&self) -> Ref<Ppu2C02> {
        self.ppu.borrow()
    }

    pub fn ppu_mut(&mut self) -> RefMut<Ppu2C02> {
        self.ppu.borrow_mut()
    }

    pub fn cpu_write(&self, addr: u16, data: u8) {

        if let Some(cartridge) = self.cartridge.as_ref() { // Cartridge gets "Priority access" to memory
            if cartridge.borrow_mut().cpu_write(addr, data) {
                return;
            }
        }

        if addr <= 0x1FFF { // Address range of the RAM
            self.cpu_ram.borrow_mut()[addr as usize & 0x07FF] = data; // As the actual 2kb of RAM are mirrored across an 8kb address range, the logic AND maps the given address to the address within the 2kb range
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
            self.cpu_ram.borrow()[addr as usize & 0x07FF] // As the actual 2kb of RAM are mirrored across an 8kb address range, the logic AND maps the given address to the address within the 2kb range
        } else if addr <= 0x3FFF { // Address range of the PPU
            self.ppu.borrow().cpu_read(addr & 0x0007, read_only) // Mirroring again. And yes, the ppu only has 8 bytes of memory
        } else {
            0x00
        }
    }

    pub fn insert_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge.clone());
        self.ppu.borrow_mut().connect_cartridge(cartridge);
    }

    pub fn reset(&self) {
        self.cpu.borrow_mut().reset();
        *self.system_clock_counter.borrow_mut() = 0;
    }

    pub fn clock(&self) {

        self.ppu.borrow_mut().clock();
        // The cpu clocks 3 times slower than the ppu
        if *self.system_clock_counter.borrow() % 3 == 0 {
            self.cpu.borrow_mut().clock();
        }
        *self.system_clock_counter.borrow_mut() += 1;


        //Todo THIS IS ONLY FOR TESTING PURPOSES, AS REPEATED CALLS OF BRK DECREMENT THE STACK POINTER AND RUST DOES NOT LIKE UNDERFLOW
        self.cpu.borrow_mut().set_stack_pointer(10);
    }
}

impl Debug for Bus {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(fmt, "bus")
    }
}
