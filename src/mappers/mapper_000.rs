use crate::mappers::Mapper;

pub struct Mapper000 {
    program_banks: u8,
    char_banks: u8
}

impl Mapper000 {
    pub fn new(program_banks: u8, char_banks: u8) -> Self {
        Mapper000 {
            program_banks,
            char_banks
        }
    }
}

impl Mapper for Mapper000 {
    fn cpu_map_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            // For mapper 0, there can be either 1 or 2 program banks with 16kib each within the total 32kib memory
            // If there is only 1, then the remaining 16kib are mirrored to be the same as the first 16kib
            *mapped_addr = (addr & (if self.program_banks > 1 { 0x7FFF } else { 0x3FFF })) as u32;
            return true;
        }

        false
    }

    fn cpu_map_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            *mapped_addr = (addr & (if self.program_banks > 1 { 0x7FFF } else { 0x3FFF })) as u32;
            return true;
        }

        false
    }

    // The character memory is always 1 bank of 8kb memory for mapper 0,
    // so there is no mapping required for the PPU

    fn ppu_map_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            return true;
        }

        false
    }

    // The ppu reads from a rom, which can't be written to. So this always returns false
    fn ppu_map_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        false
    }
}