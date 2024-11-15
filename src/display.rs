use crate::canvas;
use crate::constants;
use crate::memory;

use canvas::Canvas;
use memory::Memory;
use sdl2;
use sdl2::Sdl;

/// Holds the display objects and renders the image from the PPU data.
pub struct Display<'a> {
    /// The canvas itself.
    pub canvas: Canvas<'a>,
    /// Holds the last rendered LY.
    last_ly: u8,
    /// Run in debug mode (present after every line).
    debug: bool,
}

impl<'a> Display<'a> {
    pub fn new(window_title: &str, scale: usize, sdl: &'a Sdl, debug: bool) -> Self {
        let w = constants::DISPLAY_WIDTH;
        let h = constants::DISPLAY_HEIGHT;

        let canvas = Canvas::new(sdl, window_title, w, h, scale).unwrap();

        Display {
            canvas,
            debug,
            last_ly: 255,
        }
    }

    // Present the canvas.
    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    // Renders the given buffer to the display.
    pub fn render(&mut self, mem: &Memory) {
        let ppu = mem.ppu();

        let y = ppu.ly % constants::DISPLAY_HEIGHT as u8;
        if ppu.data_available && self.last_ly != y {
            let pixels: &[u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4] = &ppu.fb;

            // Render line.
            {
                let y = y as usize;
                let slice = &pixels[(y * constants::DISPLAY_WIDTH) * 4
                    ..(y * constants::DISPLAY_WIDTH + 159) * 4 + 3];
                self.canvas.draw_line_rgba(y, slice);
            }
            self.last_ly = y;
            // Only present when all screen lines are in the buffer (or debugging).
            if y == 143 || self.debug {
                self.canvas.flush();
            }
        }
    }
}
