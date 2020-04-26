use minifb::{Window, WindowOptions, Scale, Key, KeyRepeat};
use image::{ImageBuffer, GenericImage};
use crate::gfx::utils::*;
use crate::gfx::utils::{create_char_sprites, image_to_vec};
use crate::{cpu6502, parse_program, bus};
use std::rc::Rc;
use std::cell::RefCell;
use crate::ppu2C02::Ppu2C02;
use crate::bus::Bus;
use crate::cartridge::Cartridge;

pub fn run(game: &str) {

}

pub fn test_run() {
    const WIDTH : usize = 640;
    const HEIGHT : usize = 480;

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
    let program = cpu6502::disassemble_program(parse_program(program)).join("\n");

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut canvas = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
        let text_img = compose_text(&program, &sprites);

        canvas.copy_from(&text_img, 90, 50).expect("Error copying to image buffer");

        let converted: Vec<u32> = image_to_vec(&canvas);
        window.update_with_buffer(&converted, WIDTH, HEIGHT).unwrap();
    }
}

pub fn test_run2() {

    const EDGE_OFFSET: u32 = 5;
    const WIDTH : usize = 450;
    const HEIGHT : usize = 250;

    let (mut window, bus, sprites) = setup(WIDTH, HEIGHT);

    bus.borrow_mut().insert_cartridge(Cartridge::new("nestest.nes"));
    bus.borrow_mut().cpu_mut().set_program_counter(0xC000);

    let mut emulation_run = false;

    let disassembly = bus.borrow().cpu().disassemble_range(0x0000, 0xFFFF);
    /*let mut temp = disassembly.iter().collect::<Vec<_>>();
    temp.sort_by(|(&a, _), (&b, _)| if (a as i32 - b as i32) > 0 { std::cmp::Ordering::Greater } else if (a as i32 - b as i32) < 0 { std::cmp::Ordering::Less } else { std::cmp::Ordering::Equal });

    temp.into_iter().for_each(|(addr, s)| println!("{:0>4X}: {}", addr, s));*/

    //println!("{}", bus.borrow().cpu().disassemble_instr_at(0xC000).0);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let mut canvas = ImageBuffer::new(WIDTH as u32, HEIGHT as u32);
        let cpu_state_img = draw_cpu_state(bus.borrow().cpu(), &sprites);
        let cpu_ops_img = draw_cpu_ops(bus.borrow().cpu(), &disassembly, 15, &sprites);
        canvas.copy_from(bus.borrow().ppu().get_screen(), EDGE_OFFSET, EDGE_OFFSET).expect("Error copying to image buffer");
        canvas.copy_from(&cpu_state_img, 256 + 2 * EDGE_OFFSET, EDGE_OFFSET).expect("Error copying to image buffer");

        // Add each line of ops
        cpu_ops_img.iter().enumerate().for_each(|(i, line)| {
            canvas.copy_from(
                line,
                256 + 2 * EDGE_OFFSET,
                cpu_ops_img[0].dimensions().1 * i as u32 + cpu_state_img.dimensions().1 + 2 * EDGE_OFFSET
            ).expect("Error copying to image buffer");
        });

        if emulation_run {
            bus.borrow().clock();
            while !bus.borrow().ppu().is_frame_complete() {
                bus.borrow().clock();
            }
            bus.borrow_mut().ppu_mut().set_frame_complete(false);
        } else {
            handle_input(&window, bus.clone());
        }
        if window.is_key_pressed(Key::Space, KeyRepeat::No) { emulation_run = !emulation_run; }

        let converted: Vec<u32> = image_to_vec(&canvas);
        window.update_with_buffer(&converted, WIDTH, HEIGHT).unwrap();
    }
}

fn handle_input(window: &Window, bus: Rc<RefCell<Bus>>) {

    // Code step by step
    if window.is_key_pressed(Key::C, KeyRepeat::Yes) {
        let bus = bus.borrow();
        bus.clock();
        while !bus.cpu().complete() {
            bus.clock();
        }

        // As the CPU runs slower than the system clock, "use up" the rest of the clocks for which the
        // current instruction is "completed"
        bus.clock();
        while bus.cpu().complete() {
            bus.clock();
        }
    }

    // Emulate entire frame
    if window.is_key_pressed(Key::F, KeyRepeat::No) {
        {
            let bus = bus.borrow();
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

/// Builds a test window with a bus and font sprite sheet included
fn setup(width: usize, height: usize) -> (Window, Rc<RefCell<Bus>>, CharacterSheet) {
    // Create the window options which are responsible for scale etc
    let mut options = WindowOptions::default();
    options.scale = Scale::X4;

    // Create the Window with the given options
    let mut window = Window::new(
        "Test",
        width,
        height,
        options
    ).expect("Error creating window");

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

    // Build the Sprite Sheet for the font and create the bus with cpu and ppu
    let sprites = create_char_sprites("res/font_scaled.png", 7, 9);
    let bus = {
        let cpu = cpu6502::Cpu6502::new();
        let ppu = Ppu2C02::new();
        bus::Bus::new(cpu, ppu)
    };

    (window, bus, sprites)
}
