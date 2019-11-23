
pub mod mapper_000;


pub trait Mapper {

    // These return true if the address has been mapped successfully
    fn cpu_map_read(&mut self, addr: u16, mapped_addr : &mut u32) -> bool;
    fn cpu_map_write(&mut self, addr: u16, mapped_addr : &mut u32) -> bool;
    fn ppu_map_read(&mut self, addr: u16, mapped_addr : &mut u32) -> bool;
    fn ppu_map_write(&mut self, addr: u16, mapped_addr : &mut u32) -> bool;
}