use crate::constants;
use crate::memory;
use crate::ppu;

use memory::Memory;
use ppu::PPU;
use sdl2;
use sdl2::rect::Rect;
use sdl2::{pixels::Color, EventPump};
use sdl2::{render::Canvas, video::Window};

/// Holds the display data, and renders the image.
pub struct Display {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub scale: u32,
    pub palette: [Color; 4],
}

impl Display {
    pub fn new(window_title: &str, scale: u32) -> Self {
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();
        let event_pump = sdl_context.event_pump().unwrap();
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
            Color::RGB(8, 24, 32),
            Color::RGB(52, 104, 86),
            Color::RGB(136, 192, 112),
            Color::RGB(224, 248, 208),
        ];

        Display {
            canvas,
            event_pump,
            scale,
            palette,
        }
    }

    // Clears the display to black.
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(self.palette[0]);
        self.canvas.clear();
        self.canvas.present();
    }

    // Renders the given buffer to the display.
    pub fn render(&mut self, mem: &Memory) {
        // Fill with buffer
        let scl = self.scale as usize;
        let ppu: &PPU = mem.ppu();

        if !ppu.is_ppu_enabled() {
            return;
        }

        // Get addresses of first and second tile blocks for Window and Background.
        let addr = ppu.get_bgwin_tiledata_addr();
        // BG tile map area.
        let bg_map_addr = ppu.get_bg_tilemap_addr();
        // Window tile map area.
        let win_map_addr = ppu.get_win_tilemap_addr();

        let mut i: u16 = 0;
        for x in 0..32 {
            for y in 0..32 {
                let tile_id = mem.read8(bg_map_addr + i) as u16;
                let tile_addr = addr + tile_id;
                // Each tile is 8 lines of 2 bytes each.
                for line in 0..8 {
                    let b0 = mem.read8(tile_addr + line);
                    let b1 = mem.read8(tile_addr + line + 1);
                    let ba0 = self.get_bits_of_byte(b0);
                    let ba1 = self.get_bits_of_byte(b1);
                    for pixel in 8..0 {
                        let col_id = (ba0[pixel] | (ba1[pixel] << 1)) as usize;
                        self.canvas.set_draw_color(self.palette[col_id]);
                        self.canvas
                            .fill_rect(Rect::new(
                                (x * scl) as i32,
                                (y * scl) as i32,
                                scl as u32,
                                scl as u32,
                            ))
                            .unwrap();
                    }
                }
                i += 2;
            }
        }

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
