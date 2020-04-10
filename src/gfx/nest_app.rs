use minifb::{Window, WindowOptions, Scale, Key};
use image::{ImageBuffer, GenericImage};
use crate::gfx::utils::*;
use crate::gfx::utils::{create_char_sprites, image_to_vec};
use crate::{cpu6502, parse_program};

pub struct NestApp;

impl NestApp {
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
}

