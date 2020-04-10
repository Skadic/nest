use std::rc::Rc;
use crate::cartridge::Cartridge;
use std::cell::RefCell;

pub struct Ppu2C02 {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    name_table: [[u8; 1024]; 2],
    palette_table: [u8; 32],
    //pattern_table: Option[[u8; 4096]; 2], // Javid Future reminder
}

impl Ppu2C02 {

    pub fn new() -> Self {
        Ppu2C02 {
            cartridge: None,
            name_table: [[0; 1024]; 2],
            palette_table: [0; 32],
            //Debug information:
        }
    }

    /// Read from the main bus
    pub fn cpu_read(&self, addr: u16, _read_only: bool) -> u8 {
        match addr {
            0x0000 => 0, // Control
            0x0001 => 0, // Mask
            0x0002 => 0, // Status
            0x0003 => 0, // OAM Address
            0x0004 => 0, // OAM Data
            0x0005 => 0, // Scroll
            0x0006 => 0, // PPU Address
            0x0007 => 0, // PPU Data
            _ => 0
        }
    }

    /// Write to the main bus
    pub fn cpu_write(&mut self, addr: u16,   data: u8) {
        match addr {
            0x0000 => 0, // Control
            0x0001 => 0, // Mask
            0x0002 => 0, // Status
            0x0003 => 0, // OAM Address
            0x0004 => 0, // OAM Data
            0x0005 => 0, // Scroll
            0x0006 => 0, // PPU Address
            0x0007 => 0, // PPU Data
            _ => 0
        };
    }

    /// Read from the PPU bus
    pub fn ppu_read(&self, addr: u16, read_only: bool) -> u8 {
        let mut data = 0x00;
        let addr = addr & 0x3FFF;

        if let Some(cartridge) = self.cartridge.as_ref() {
            if cartridge.borrow_mut().ppu_read(addr, &mut data) {

            }
        }

        data
    }

    /// Write to the PPU bus
    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        let addr = addr & 0x3FFF;
        if let Some(cartridge) = self.cartridge.as_ref() {
            if cartridge.borrow_mut().ppu_write(addr, data) {

            }
        }
    }

    pub fn connect_cartridge(&mut self, cartridge: Rc<RefCell<Cartridge>>) {
        self.cartridge = Some(cartridge);
    }

    pub fn clock(&mut self) {

    }
}