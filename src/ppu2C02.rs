use std::rc::Rc;
use crate::cartridge::Cartridge;
use std::cell::RefCell;
use image::{Rgba, RgbaImage, ImageBuffer};
use rand::Rng;

pub struct Ppu2C02 {
    cartridge: Option<Rc<RefCell<Cartridge>>>,
    name_table: [[u8; 1024]; 2],
    palette_table: [u8; 32],
    //pattern_table: Option[[u8; 4096]; 2], // Javid Future reminder
    //Debug info:
    palette_screen: [Rgba<u8>; 64],
    sprite_screen: RgbaImage,
    sprite_name_table: [RgbaImage; 2],
    sprite_pattern_table: [RgbaImage; 2],
    frame_complete: bool,
    scan_line: i16,
    cycle: i16
}

impl Ppu2C02 {

    pub fn new() -> Self {
        let mut ppu = Ppu2C02 {
            cartridge: None,
            name_table: [[0; 1024]; 2],
            palette_table: [0; 32],
            //Debug information:
            palette_screen: [Rgba([0, 0, 0, 0]); 64],
            sprite_screen: RgbaImage::new(256, 240),
            sprite_name_table: [RgbaImage::new(256, 240), RgbaImage::new(256, 240)],
            sprite_pattern_table: [RgbaImage::new(128, 128), RgbaImage::new(128, 128)],
            frame_complete: false,
            // Basically which column and row the renderer is working on
            scan_line: 0,
            cycle: 0,
        };
        ppu.setup_palette_screen();
        ppu
    }

    /// This sets up the colors that the NES can use and stores them in palette_screen
    fn setup_palette_screen(&mut self) {
        self.palette_screen[0x00] = Rgba([84, 84, 84, 255]);
        self.palette_screen[0x01] = Rgba([0, 30, 116, 255]);
        self.palette_screen[0x02] = Rgba([8, 16, 144, 255]);
        self.palette_screen[0x03] = Rgba([48, 0, 136, 255]);
        self.palette_screen[0x04] = Rgba([68, 0, 100, 255]);
        self.palette_screen[0x05] = Rgba([92, 0, 48, 255]);
        self.palette_screen[0x06] = Rgba([84, 4, 0, 255]);
        self.palette_screen[0x07] = Rgba([60, 24, 0, 255]);
        self.palette_screen[0x08] = Rgba([32, 42, 0, 255]);
        self.palette_screen[0x09] = Rgba([8, 58, 0, 255]);
        self.palette_screen[0x0A] = Rgba([0, 64, 0, 255]);
        self.palette_screen[0x0B] = Rgba([0, 60, 0, 255]);
        self.palette_screen[0x0C] = Rgba([0, 50, 60, 255]);
        self.palette_screen[0x0D] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x0E] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x0F] = Rgba([0, 0, 0, 255]);

        self.palette_screen[0x10] = Rgba([152, 150, 152, 255]);
        self.palette_screen[0x11] = Rgba([8, 76, 196, 255]);
        self.palette_screen[0x12] = Rgba([48, 50, 236, 255]);
        self.palette_screen[0x13] = Rgba([92, 30, 228, 255]);
        self.palette_screen[0x14] = Rgba([136, 20, 176, 255]);
        self.palette_screen[0x15] = Rgba([160, 20, 100, 255]);
        self.palette_screen[0x16] = Rgba([152, 34, 32, 255]);
        self.palette_screen[0x17] = Rgba([120, 60, 0, 255]);
        self.palette_screen[0x18] = Rgba([84, 90, 0, 255]);
        self.palette_screen[0x19] = Rgba([40, 114, 0, 255]);
        self.palette_screen[0x1A] = Rgba([8, 124, 0, 255]);
        self.palette_screen[0x1B] = Rgba([0, 118, 40, 255]);
        self.palette_screen[0x1C] = Rgba([0, 102, 120, 255]);
        self.palette_screen[0x1D] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x1E] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x1F] = Rgba([0, 0, 0, 255]);

        self.palette_screen[0x20] = Rgba([236, 238, 236, 255]);
        self.palette_screen[0x21] = Rgba([76, 154, 236, 255]);
        self.palette_screen[0x22] = Rgba([120, 124, 236, 255]);
        self.palette_screen[0x23] = Rgba([176, 98, 236, 255]);
        self.palette_screen[0x24] = Rgba([228, 84, 236, 255]);
        self.palette_screen[0x25] = Rgba([236, 88, 180, 255]);
        self.palette_screen[0x26] = Rgba([236, 106, 100, 255]);
        self.palette_screen[0x27] = Rgba([212, 136, 32, 255]);
        self.palette_screen[0x28] = Rgba([160, 170, 0, 255]);
        self.palette_screen[0x29] = Rgba([116, 196, 0, 255]);
        self.palette_screen[0x2A] = Rgba([76, 208, 32, 255]);
        self.palette_screen[0x2B] = Rgba([56, 204, 108, 255]);
        self.palette_screen[0x2C] = Rgba([56, 180, 204, 255]);
        self.palette_screen[0x2D] = Rgba([60, 60, 60, 255]);
        self.palette_screen[0x2E] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x2F] = Rgba([0, 0, 0, 255]);

        self.palette_screen[0x30] = Rgba([236, 238, 236, 255]);
        self.palette_screen[0x31] = Rgba([168, 204, 236, 255]);
        self.palette_screen[0x32] = Rgba([188, 188, 236, 255]);
        self.palette_screen[0x33] = Rgba([212, 178, 236, 255]);
        self.palette_screen[0x34] = Rgba([236, 174, 236, 255]);
        self.palette_screen[0x35] = Rgba([236, 174, 212, 255]);
        self.palette_screen[0x36] = Rgba([236, 180, 176, 255]);
        self.palette_screen[0x37] = Rgba([228, 196, 144, 255]);
        self.palette_screen[0x38] = Rgba([204, 210, 120, 255]);
        self.palette_screen[0x39] = Rgba([180, 222, 120, 255]);
        self.palette_screen[0x3A] = Rgba([168, 226, 144, 255]);
        self.palette_screen[0x3B] = Rgba([152, 226, 180, 255]);
        self.palette_screen[0x3C] = Rgba([160, 214, 228, 255]);
        self.palette_screen[0x3D] = Rgba([160, 162, 160, 255]);
        self.palette_screen[0x3E] = Rgba([0, 0, 0, 255]);
        self.palette_screen[0x3F] = Rgba([0, 0, 0, 255]);
    }

    pub fn is_frame_complete(&self) -> bool {
        self.frame_complete
    }

    pub fn set_frame_complete(&mut self, b: bool) {
        self.frame_complete = b
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

        // Todo temporary fake noise
        let mut rng = rand::thread_rng();
        if ((self.cycle - 1) as u32) < 256 && (self.scan_line as u32) < 240 {
            self.sprite_screen.put_pixel((self.cycle - 1) as u32, self.scan_line as u32, self.palette_screen[if rng.gen_bool(0.5) { 0x3F } else { 0x30 }]);
        }
        self.cycle += 1;
        // Weird numbers are due to how the NES works
        if self.cycle >= 341 {
            self.cycle = 0;
            self.scan_line += 1;
            if self.scan_line >= 261 {
                self.scan_line = -1;
                self.frame_complete = true;
            }
        }
    }

    // --------------------- Debug Info -------------------------------

    pub fn get_screen(&self) -> &RgbaImage {
        &self.sprite_screen
    }

    pub fn get_name_table(&self, i: usize) -> &RgbaImage {
        &self.sprite_name_table[i]
    }

    pub fn get_pattern_table(&self, i: usize) -> &RgbaImage {
        &self.sprite_pattern_table[i]
    }
}