use image::{SubImage, RgbaImage, ImageBuffer, GenericImage, RgbImage, Rgba, Pixel};
use std::rc::Rc;
use std::collections::HashMap;
use image::flat::NormalForm::ImagePacked;
use crate::cpu6502::Cpu6502;
use crate::cpu6502::Flags6502;
use image::imageops::FilterType;

type CharacterSheet = HashMap<char, SubImage<Rc<RgbaImage>>>;

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
    img.pixels_mut().for_each(|pix| if *pix == Rgba([0, 0, 0, 255]) { *pix = Rgba([0, 0, 0, 0]) });

    let mut img = Rc::new(img);

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

            sprites.insert(current, sprite);


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
    let max_line_length = text.split("\n").map(|segment| segment.len()).max().unwrap() as u32;
    // The line count is 1 plus the amount of newline characters in the string
    let line_count = 1 + text.chars().filter(|c| *c == '\n').count() as u32;
    // The dimensions of each character in the Sheet
    let (char_width, char_height) = character_sheet[&'a'].to_image().dimensions();
    // Create a new buffer that will hold the composed text
    let mut buffer = ImageBuffer::new(char_width * max_line_length, char_height * line_count);

    // The current line to be written to
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
            .unwrap_or_else(|| &character_sheet[&'?']).to_image();
        buffer.copy_from(&sprite, column as u32 * char_width, line * char_height);
        column += 1;
    }

    buffer
}

pub fn draw_cpu<T: std::ops::Deref<Target=Cpu6502>>(cpu: T, character_sheet: &CharacterSheet) -> RgbaImage {
    let (char_w, char_h) = character_sheet[&'a'].to_image().dimensions();
    let mut registers = RgbaImage::new(16 * char_w, 6 * char_h);
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
    );

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
                    let mut sprite : RgbaImage = character_sheet[&stringify!($flag).chars().nth(0).unwrap()].to_image();
                    sprite.pixels_mut()
                        .filter(|pix| **pix == Rgba([255, 255, 255, 255]))
                        .for_each(|pix| Pixel::blend(pix, &color));
                    registers.copy_from(&sprite, (8 + i) * char_w, 0);
                    i += 1;
                )*
            }
        }
    }

    add_flag_char! { C, Z, I, D, B, U, V, N }


    registers
}