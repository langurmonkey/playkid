use crate::constants;
use crate::memory;
use crate::ppu;

use memory::Memory;
use ppu::PPU;
use sdl2;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::Sdl;
use sdl2::{render::Canvas, video::Window};

/// Holds the display data, and renders the image.
pub struct Display {
    pub canvas: Canvas<Window>,
    pub scale: u32,
    pub palette: [Color; 4],
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
    pub fn render(&mut self, m_cycle: u32, mem: &Memory) {
        // Fill with buffer
        let scl = self.scale as u32;
        let ppu: &PPU = mem.ppu();

        // Get addresses of first and second tile blocks for Window and Background.
        let addr = ppu.get_bgwin_tiledata_addr();
        // BG tile map area.
        let bg_map_addr = ppu.get_bg_tilemap_addr();
        // Window tile map area.
        let win_map_addr = ppu.get_win_tilemap_addr();

        let st = 0x8010;

        let mut x = 0;
        let mut y = 0;

        for sprite in 0..165 {
            let sx = x;
            let sy = y;
            // 8x8 sprites where each row of 8 pixels is 2 bytes.
            for row in 0..8 {
                let address = st + sprite * 16 + row * 2;
                let b0 = ppu.read(address);
                let b1 = ppu.read(address + 1);
                let ba0 = self.get_bits_of_byte(b0);
                let ba1 = self.get_bits_of_byte(b1);
                for col in 0..8 {
                    let col_id = (ba0[col] | (ba1[col] << 1)) as u8;
                    self.canvas.set_draw_color(self.palette[col_id as usize]);
                    self.canvas
                        .fill_rect(Rect::new(
                            ((sx + col) as u32 * scl) as i32,
                            ((sy + row) as u32 * scl) as i32,
                            scl as u32,
                            scl as u32,
                        ))
                        .unwrap();
                }
            }
            // Update [x,y] for the next sprite.
            x += 8;
            if x >= 160 {
                x = 0;
                y += 8;
            }
        }

        // Background rendering.
        // self.canvas.clear();
        // let mut col_idx = (m_cycle as usize) % self.palette.len();
        // for x in 0..constants::DISPLAY_WIDTH {
        //     for y in 0..constants::DISPLAY_HEIGHT {
        //         let color = self.palette[col_idx];
        //         self.canvas.set_draw_color(color);
        //         self.canvas
        //             .fill_rect(Rect::new(
        //                 (x * scl) as i32,
        //                 (y * scl) as i32,
        //                 scl as u32,
        //                 scl as u32,
        //             ))
        //             .unwrap();
        //         col_idx = (col_idx + 1) % self.palette.len();
        //     }
        //     col_idx = (col_idx + 1) % self.palette.len();
        // }

        self.canvas.present();
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
