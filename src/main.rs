#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate lazy_static;

use std::ops::Shl;

mod olc6502;
mod bus;

fn main() {
    let i = 0u8;
    let j = (i as u16) << 8;
}
