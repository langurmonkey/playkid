use crate::constants;
use crate::memory;

use memory::Memory;
use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::{render::Canvas, video::Window};

/// Holds the display data, and renders the image.
pub struct Display {
    /// The canvas itself.
    pub canvas: Canvas<Window>,
    /// The scale factor for the display.
    pub scale: u32,
    /// The palette.
    pub palette: [Color; 4],
    /// Holds the last rendered LY.
    last_ly: u8,
}

impl Display {
    pub fn new(window_title: &str, scale: u32, sdl: &Sdl) -> Self {
        let video_subsystem = sdl.video().unwrap();
        let window = video_subsystem
            .window(
                window_title,
                constants::DISPLAY_WIDTH as u32 * scale,
                constants::DISPLAY_HEIGHT as u32 * scale,
            )
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().build().unwrap();

        // The default palette.
        let palette = [
            Color::RGB(224, 248, 208),
            Color::RGB(136, 192, 112),
            Color::RGB(52, 104, 86),
            Color::RGB(8, 24, 32),
        ];

        Display {
            canvas,
            scale,
            palette,
            last_ly: 255,
        }
    }

    // Present the canvas.
    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn clear(&mut self) {
        self.canvas.set_draw_color(self.palette[3]);
        self.canvas.clear();
    }

    // Renders the given buffer to the display.
    pub fn render(&mut self, mem: &Memory) {
        // Fill with buffer
        let scl = self.scale as u32;
        let ppu = mem.ppu();

        let y = ppu.ly % 144;
        if ppu.data_available && self.last_ly != y {
            let pixels: &Vec<u8> = &ppu.scr;
            let offset = y as usize * 160;

            // Render last line.
            for x in 0..160 {
                let color = pixels[offset + x];
                if color < 4 {
                    self.canvas.set_draw_color(self.palette[color as usize]);
                    self.canvas
                        .fill_rect(Rect::new(
                            (x as u32 * scl) as i32,
                            (y as u32 * scl) as i32,
                            scl as u32,
                            scl as u32,
                        ))
                        .unwrap();
                }
            }
            self.last_ly = y;
            self.canvas.present();
        }
    }

    /// Gets the bits of a byte as an array, with the most significant bit
    /// at index 0 and the least significant bit at index 7.
    /// For example, if the byte is 130, the array will be [1, 0, 0, 0, 0, 0, 1, 0]
    fn get_bits_of_byte(&self, byte: u8) -> [u8; 8] {
        let mut bits = [0u8; 8];
        for i in 0..=7 {
            let shifted_byte = byte >> i;
            // Get the rightmost bit of the shifted byte (least significant bit)
            let cur_bit = shifted_byte & 1;
            // For the first iteration, the cur_bit is the
            // least significant bit and therefore we place
            // that bit at index 7 of the array (rightmost bit)
            bits[7 - i] = cur_bit;
        }
        bits
    }
}
