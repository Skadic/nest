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
            *mapped_addr = (addr & (if self.program_banks > 1 { 0x7FFF } else { 0x3FFF })) as u32; // Mirror memory, if the rom contains more than 1 program bank
            return true;
        }

        false
    }

    fn cpu_map_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr >= 0x8000 {
            return true;
        }

        false
    }

    fn ppu_map_read(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        if addr <= 0x1FFF {
            *mapped_addr = addr as u32;
            return true;
        }

        false
    }

    fn ppu_map_write(&mut self, addr: u16, mapped_addr: &mut u32) -> bool {
        false
    }
}