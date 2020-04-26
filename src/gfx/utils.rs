use image::{SubImage, RgbaImage, ImageBuffer, GenericImage, Rgba, Pixel};
use std::rc::Rc;
use std::collections::HashMap;
use crate::cpu6502::Cpu6502;
use crate::cpu6502::Flags6502;
use std::time::Instant;

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);

// Todo make this more efficient
pub(crate) type CharacterSheet = HashMap<char, RgbaImage>;

/// Creates a Vector of u32 numbers containing the color of each pixel
pub fn image_to_vec(img: &RgbaImage) -> Vec<u32> {
    img.pixels().map(|pix| {
        // Extract the color information as an array of the RGBA values
        let pix = pix.0;
        // create the u32 representing the color as RGB
        ((pix[0] as u32) << 16) | ((pix[1] as u32) << 8) | (pix[2] as u32)
    }).collect()
}

/// Creates a hashmap of chars to their respective sprite from a sprite sheet
pub fn create_char_sprites(sheet: &str, char_width: usize, char_height: usize) -> CharacterSheet {
    let mut img = image::open(sheet).unwrap().into_rgba();
    // Remove Black
    img.pixels_mut().for_each(|pix| if *pix == BLACK { *pix = TRANSPARENT });

    let img = Rc::new(img);

    let (chars_x, chars_y) = {
        let (w, h) = img.dimensions();
        (w / char_width as u32, h / char_height as u32)
    };
    println!("x chars: {}; y chars: {}",  chars_x, chars_y);

    let mut sprites = HashMap::new();
    // The chars in the sprite sheet start at ' '
    let mut current = ' ';
    'outer: for y in 0u32..chars_y {
        for x in 0u32..chars_x {
            let sprite = SubImage::new(
                Rc::clone(&img),
                x * char_width as u32,
                y * char_height as u32,
                char_width as u32,
                char_height as u32
            );

            sprites.insert(current, sprite.to_image());


            // ~ is the last character in the font sprite sheet. If that has been handled, stop adding sprites
            if current == '~' { break 'outer; }

            // The characters in the sheet are in ascii order, so converting the char to a number,
            // increasing it by 1, and converting it back to a char will move to the next char in the sprite sheet
            current = (current as u8 + 1) as char;
        }
    }

    sprites
}

pub fn compose_text(text: &str, character_sheet: &CharacterSheet) -> RgbaImage {
    compose_text_with_tint(text, character_sheet, WHITE)
}

pub fn compose_text_with_tint<T: Pixel<Subpixel=u8>>(text: &str, character_sheet: &CharacterSheet, color: T) -> RgbaImage {
    let color = color.to_rgba();
    let max_line_length = text.split("\n").map(|segment| segment.len()).max().unwrap() as u32;
    // The line count is 1 plus the amount of newline characters in the string
    let line_count = 1 + text.chars().filter(|c| *c == '\n').count() as u32;
    // The dimensions of each character in the Sheet
    let (char_width, char_height) = character_sheet[&'a'].dimensions();
    // Create a new buffer that will hold the composed text
    let mut buffer = ImageBuffer::new(char_width * max_line_length, char_height * line_count);

    // The current line and column to be written to
    let mut line = 0;
    let mut column = 0;
    // Create the sprite for each character and copy the sprites into the buffer
    for current_char in text.chars() {
        // If a newline character is found, move to the next line and handle the next character
        if current_char == '\n' {
            line += 1;
            column = 0;
            continue
        }
        // Gets the sprite that corresponds to the character. If this sprite does not exist, it will return the '?' sprite
        let sprite = character_sheet.get(&current_char)
            .unwrap_or_else(|| &character_sheet[&'?']);

        sprite.enumerate_pixels()
            .for_each(|(x, y, pix)| {
                if *pix != TRANSPARENT {
                    buffer.put_pixel(column as u32 * char_width + x, line * char_height + y, color);
                } else {
                    buffer.put_pixel(column as u32 * char_width + x, line * char_height + y, *pix);
                }
            });
        column += 1;
    }

    buffer
}

pub fn draw_cpu_state<T: std::ops::Deref<Target=Cpu6502>>(cpu: T, character_sheet: &CharacterSheet) -> RgbaImage {

    let (char_w, char_h) = character_sheet[&'a'].dimensions();
    let mut registers: RgbaImage = RgbaImage::new(16 * char_w, 6 * char_h);
    registers.copy_from(
        &compose_text(format!
             ("STATUS:\nPC: ${:0>4X}\nA: ${:0>2X}\nX: ${:0>2X}\nY: ${:0>2X}\nSP: ${:0>4X}",
              cpu.get_program_counter(),
              cpu.get_acc(),
              cpu.get_x(),
              cpu.get_y(),
              cpu.get_stack_pointer()
             ).as_str(), &character_sheet),
        0, 0
    ).expect("Error copying to image buffer");

    let x_offset = 8 * char_w;

    macro_rules! add_flag_char {
        ($($flag:ident), *) => {
            {
                let mut i = 0;
                $(
                    let color = if !cpu.get_flag(Flags6502::$flag) {
                        Rgba([255, 0, 0, 255])
                    } else {
                        Rgba([0, 255, 0, 255])
                    };
                    let mut sprite = character_sheet[&stringify!($flag).chars().nth(0).unwrap()].clone();
                    sprite.enumerate_pixels()
                        .for_each(|(x, y, pix)| {
                            if *pix == Rgba([255, 255, 255, 255]) {
                                registers.put_pixel(x_offset + i * char_w + x, y, color);
                            } else {
                                registers.put_pixel(x_offset + i * char_w + x, y, *pix);
                            }
                        });
                    i += 1;
                )*
            }
        }
    }


    add_flag_char! { C, Z, I, D, B, U, V, N }

    registers
}

pub fn draw_cpu_ops<T: std::ops::Deref<Target=Cpu6502>>(cpu: T, disassembly: &HashMap<u16, String>, n: usize, character_sheet: &CharacterSheet) -> Vec<RgbaImage> {
    let mut lines = Vec::new();

    // Draw the instruction at the program counter
    if disassembly.contains_key(&cpu.get_program_counter()) {
        lines.push(
            compose_text_with_tint( format!("${:0>4X}: {}", cpu.get_program_counter(), disassembly[&cpu.get_program_counter()]).as_str(), &character_sheet, Rgba([0, 255, 255, 255]))
        );
    }

    // Draw instructions before the pc
    let mut count = 0;
    let mut current_addr = cpu.get_program_counter();
    while count < n as u16 / 2 {
        if current_addr == 0 { break; }
        current_addr -= 1;
        // Not every memory address contains the start of an instruction
        if disassembly.contains_key(&current_addr) {
            let instr = disassembly[&current_addr].as_str();
            lines.insert(0, compose_text(format!("${:0>4X}: {}", current_addr, disassembly[&current_addr]).as_str(), &character_sheet));
            count += 1;
        }
    }
    // Draw instructions after the pc
    let mut count = 0;
    let mut current_addr = cpu.get_program_counter();
    while count < n as u16 / 2 {
        if current_addr == 0xFFFF { break; }
        current_addr += 1;

        // Not every memory address contains the start of an instruction
        if disassembly.contains_key(&current_addr) {
            let instr = disassembly[&current_addr].as_str();
            lines.push( compose_text(format!("${:0>4X}: {}", current_addr, disassembly[&current_addr]).as_str(), &character_sheet));
            count += 1;
        }
    }

    lines
}