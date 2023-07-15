use std::sync::mpsc::{Sender, Receiver};

use image::{RgbaImage, ImageFormat, GenericImageView};

use super::Device;

type Font = RgbaImage;

pub type STIOutput = Sender<u8>;
pub type STIInput = Receiver<u8>;

const ROWS: u32 = 8;
const COLS: u32 = 16;

const CHAR_WIDTH: u32 = 6;
const CHAR_HEIGHT: u32 = 12;

pub const WIDTH: u32 = COLS * CHAR_WIDTH;
pub const HEIGHT: u32 = ROWS * CHAR_HEIGHT;
pub const SCALE: u32 = 3;

const BACKGROUND_COLOR: [u8; 4] = [0x20, 0x20, 0x20, 0xff];
const FOREGROUND_COLOR: [u8; 4] = [0xff, 0xff, 0xff, 0xff];

fn load_font(bytes: &[u8]) -> Font {
    image::load_from_memory_with_format(bytes, ImageFormat::Png)
        .unwrap()
        .to_rgba8()
}

pub struct STIPrinter {
    buffer: [u8; (COLS * ROWS) as usize],
    input: STIInput,
    cursor: usize,
    font: Font
}

impl STIPrinter {
    pub fn new(input: STIInput) -> Self {
        Self {
            buffer: [0; (COLS * ROWS) as usize],
            input,
            cursor: 0,
            font: load_font(include_bytes!("../../../../ptes/font.png"))
        }
    }

    pub fn update(&mut self, framebuf: &mut [u8]) {
        for data in self.input.try_iter() {
            self.buffer[self.cursor] = data;
            self.cursor = (self.cursor + 1) % (COLS * ROWS) as usize;
        }

        self.draw(framebuf);
    }

    fn draw(&self, framebuf: &mut [u8]) {
        for (i, character) in self.buffer.iter().enumerate() {
            let i = i as u32;

            let col = i % COLS;
            let row = i / COLS;

            let x = col * CHAR_WIDTH;
            let y = row * CHAR_HEIGHT;

            let sprite = self.font.view(*character as u32 * CHAR_WIDTH, 0, CHAR_WIDTH, CHAR_HEIGHT);
            
            for (u, v, mask) in sprite.pixels() {
                let alpha = mask.0[3];

                let pixel = if alpha == 0 {
                    BACKGROUND_COLOR
                } else {
                    FOREGROUND_COLOR
                };

                let pixel_index = (WIDTH * (y + v) + (x + u)) as usize * 4;

                framebuf[pixel_index..pixel_index + 4].copy_from_slice(&pixel);
            }
        }
    }
}

#[derive(Debug)]
pub struct STI {
    output: STIOutput
}

impl STI {
    pub fn new(output: STIOutput) -> Self {
        Self { output }
    }
}

impl Device for STI {
    fn read(&mut self, address: u8) -> u8 {
        self.peak(address)
    }

    fn write(&mut self, _: u8, data: u8) {
        self.output.send(data).unwrap();
    }

    fn peak(&self, _: u8) -> u8 {
        0
    }
}