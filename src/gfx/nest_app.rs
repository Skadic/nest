use minifb::{Window, WindowOptions, Scale, Key, KeyRepeat};
use image::{ImageBuffer, GenericImage};
use crate::gfx::utils::*;
use crate::gfx::utils::{create_char_sprites, image_to_vec};
use crate::{cpu6502, parse_program, bus};
use std::rc::Rc;
use std::cell::RefCell;
use crate::ppu2C02::Ppu2C02;
use crate::cpu6502::Flags6502;
use crate::bus::Bus;

pub fn run(game: &str) {

}

pub fn test_run() {
    const WIDTH : usize = 640;
    const HEIGHT : usize = 480;

    let mut buffer = vec![0u32; WIDTH * HEIGHT];
    let mut options = WindowOptions::default();
    options.scale = Scale::X2;
    let mut window = Window::new(
        "Test",
        WIDTH,
        HEIGHT,
        options
    ).expect("Error creating window");
    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    let sprites = create_char_sprites("res/font_scaled.png", 7, 9);

    let program = "A9 05 AA A9 06 8E 11 11 6D 11 11";
    let program = cpu6502::disassemble(parse_program(program)).join("\n");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut canvas = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
        let text_img = compose_text(&program, &sprites);

        canvas.copy_from(&text_img, 90, 50);

        let converted: Vec<u32> = image_to_vec(&canvas);
        window.update_with_buffer(&converted, WIDTH, HEIGHT).unwrap();
    }
}

pub fn test_run2() {

    const EDGE_OFFSET: u32 = 5;
    const WIDTH : usize = 400;
    const HEIGHT : usize = 250;

    // Create the window options which are responsible for scale etc
    let mut options = WindowOptions::default();
    options.scale = Scale::X4;

    // Create the Window with the given options
    let mut window = Window::new(
        "Test",
        WIDTH,
        HEIGHT,
        options
    ).expect("Error creating window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Build the Sprite Sheet for the font and create the bus with cpu and ppu
    let sprites = create_char_sprites("res/font_scaled.png", 7, 9);
    let mut bus = {
        let cpu = cpu6502::Cpu6502::new();
        let ppu = Ppu2C02::new();
        bus::Bus::new(cpu, ppu)
    };

    bus.borrow_mut().cpu_mut().set_flag(Flags6502::V, true);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut canvas = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
        let mut cpu_img = draw_cpu(bus.borrow_mut().cpu(), &sprites);
        canvas.copy_from(&cpu_img, 256 + 2 * EDGE_OFFSET, EDGE_OFFSET);
        canvas.copy_from(bus.borrow().ppu().get_screen(), EDGE_OFFSET, EDGE_OFFSET);

        handle_input(&window, bus.clone());

        let converted: Vec<u32> = image_to_vec(&canvas);
        window.update_with_buffer(&converted, WIDTH, HEIGHT).unwrap();
    }
}

fn handle_input(window: &Window, bus: Rc<RefCell<Bus>>) {
    // Code step by step
    if window.is_key_pressed(Key::C, KeyRepeat::No) {
        let bus = bus.borrow();
        bus.clock();
        while !bus.cpu().complete() {
            bus.clock();
        }

        // As the CPU runs slower than the system clock, "use up" the rest of the clocks for which the
        // current instruction is "completed"
        while bus.cpu().complete() {
            bus.clock();
        }
    }

    // Emulate entire frame
    if window.is_key_pressed(Key::F, KeyRepeat::No) {
        {
            let mut bus = bus.borrow();
            bus.clock();
            while !bus.ppu().is_frame_complete() {
                bus.clock();
            }

            // As the CPU runs slower than the system clock, "use up" the rest of the clocks for which the
            // current instruction is "completed"
            bus.clock();
            while !bus.cpu().complete() {
                bus.clock();
            }
        }
        let mut bus = bus.borrow_mut();
        bus.ppu_mut().set_frame_complete(false);
    }

    if window.is_key_pressed(Key::R, KeyRepeat::No) {
        bus.borrow_mut().reset();
    }
}
