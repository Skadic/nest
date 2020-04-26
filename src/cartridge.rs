use std::fs::{File};
use std::io::{Seek, SeekFrom, Read, BufReader};
use crate::mappers::Mapper;
use std::rc::Rc;
use crate::mappers::mapper_000::Mapper000;
use bitflags::_core::cell::{RefCell};

pub struct Cartridge {
    program_memory: Vec<u8>,
    char_memory: Vec<u8>, // Pattern/Texture memory
    mapper_id: u8, // ID of the mapper currently in use
    program_banks: u8, // Amount of program memory banks
    char_banks: u8, // Amount of char memory banks
    mapper: Rc<RefCell<dyn Mapper>>
}

impl Cartridge {

    pub fn new(file_name: &str) -> Rc<RefCell<Self>> {
        let mut cartridge = Cartridge {
            program_memory: vec![],
            char_memory: vec![],
            mapper_id: 0,
            program_banks: 0,
            char_banks: 0,
            mapper: Rc::new(RefCell::new(Mapper000::new(0, 0))), // This is just a placeholder
        };

        // TODO All of this is pretty weird. If things don't work, I'll come back to this
        #[derive(Default, Debug)]
        struct NesHeader {
            name: String,
            program_rom_chunks: u8,
            char_rom_chunks: u8,
            mapper1: u8,
            mapper2: u8,
            program_ram_size: u8,
            tv_system1: u8,
            tv_system2: u8,
            unused: String,
        }

        let mut header = NesHeader::default();
        let file = File::open("roms/".to_owned() + file_name).expect("ROM does not exist");
        let mut reader = BufReader::new(file);

        header.name = {
            let mut buf = [0u8; 4];
            reader.read_exact(&mut buf).expect("Error reading name to buffer");
            buf.iter().map(|&n| n as char).collect()
        };

        macro_rules! read_u8 {
            ($x:ident) => {
                let mut buf = [0u8];
                reader.read_exact(&mut buf).expect("Error reading byte");
                header.$x = buf[0];
            };
        }

        read_u8!(program_rom_chunks);
        read_u8!(char_rom_chunks);
        read_u8!(mapper1);
        read_u8!(mapper2);
        read_u8!(program_ram_size);
        read_u8!(tv_system1);
        read_u8!(tv_system2);

        header.unused = {
            let mut buf = [0u8; 5];
            reader.read_exact(&mut buf).expect("Error reading name to buffer");
            buf.iter().map(|&n| n as char).collect()
        };

        if header.mapper1 & 0x04 > 0 {
            reader.seek(SeekFrom::Current(512)).expect("Error seeking"); // Skip training information
        }

        println!("{:?}", header);

        // Determine Mapper ID of the mapper used by the cartridge
        cartridge.mapper_id = ((header.mapper2 >> 4) << 4) | (header.mapper1 >> 4);

        // "Discover" File Format
        let file_type = 1;

        if file_type == 0 {

        }

        // Reads the amount of program/character memory banks to the cartridge fields,
        // resizes the memory vectors to the required size, and reads the memory from the ROM
        if file_type == 1 {
            cartridge.program_banks = header.program_rom_chunks;
            cartridge.program_memory.resize(cartridge.program_banks as usize * 16384, 0);
            reader.read_exact(&mut cartridge.program_memory[..]).expect("Error reading program memory");

            cartridge.char_banks = header.char_rom_chunks;
            cartridge.char_memory.resize(cartridge.char_banks as usize * 8192, 0);
            reader.read_exact(&mut cartridge.char_memory[..]).expect("Error reading char memory");
        }

        if file_type == 2 {

        }

        match cartridge.mapper_id {
            0 => cartridge.mapper = Rc::new(RefCell::new(Mapper000::new(cartridge.program_banks, cartridge.char_banks))),
            _ => unimplemented!("Mapper {} not implemented", cartridge.mapper_id)
        }

        Rc::new(RefCell::new(cartridge))
    }

    // These return true, if the cartridge is handling the read/write
    // The cartridge has priority access to memory, which is handled in the read and write methods of the Bus

    /// Read from the main bus
    pub fn cpu_read(&mut self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr = 0;
        // If the mapper says, that the cartridge should handle this read, read the data, otherwise do nothing
        if self.mapper.borrow_mut().cpu_map_read(addr, &mut mapped_addr) {
            *data = self.program_memory[mapped_addr as usize];
            true
        } else {
            false
        }
    }

    /// Write to the main bus
    pub fn cpu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr = 0;
        // If the mapper says, that the cartridge should handle this read, write the data, otherwise do nothing
        if self.mapper.borrow_mut().cpu_map_write(addr, &mut mapped_addr) {
            self.program_memory[mapped_addr as usize] = data;
            true
        } else {
            false
        }
    }

    /// Read from the PPU bus
    pub fn ppu_read(&self, addr: u16, data: &mut u8) -> bool {
        let mut mapped_addr = 0;
        if self.mapper.borrow_mut().ppu_map_read(addr, &mut mapped_addr) {
            *data = self.char_memory[mapped_addr as usize];
            true
        } else {
            false
        }
    }

    /// Write to the PPU bus
    pub fn ppu_write(&mut self, addr: u16, data: u8) -> bool {
        let mut mapped_addr = 0;
        if self.mapper.borrow_mut().ppu_map_write(addr, &mut mapped_addr) {
            self.char_memory[mapped_addr as usize] = data;
            true
        } else {
            false
        }
    }
}