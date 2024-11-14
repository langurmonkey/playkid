use crate::canvas;
use crate::constants;
use crate::memory;

use canvas::Canvas;
use memory::Memory;
use sdl2;
use sdl2::Sdl;

/// Holds the display objects and renders the image from the PPU data.
pub struct Display {
    /// The canvas itself.
    pub canvas: Canvas,
    /// The scale factor for the display.
    scale: usize,
    /// Holds the last rendered LY.
    last_ly: u8,
    /// Run in debug mode (present after every line).
    debug: bool,
}

impl Display {
    pub fn new(window_title: &str, scale: usize, sdl: &Sdl, debug: bool) -> Self {
        let w = constants::DISPLAY_WIDTH;
        let h = constants::DISPLAY_HEIGHT;

        let canvas = Canvas::new(window_title, w, h, scale).unwrap();

        Display {
            canvas,
            scale,
            debug,
            last_ly: 255,
        }
    }

    // Present the canvas.
    pub fn present(&mut self) {
        self.canvas.present();
    }

    pub fn clear(&mut self) {
        // TODO implement this.
    }

    // Renders the given buffer to the display.
    pub fn render(&mut self, mem: &Memory) {
        // Fill with buffer
        let ppu = mem.ppu();

        let y = ppu.ly % constants::DISPLAY_HEIGHT as u8;
        if ppu.data_available && self.last_ly != y {
            let pixels: &mut [u8; constants::DISPLAY_HEIGHT * constants::DISPLAY_WIDTH * 4] =
                &mut ppu.fb.clone();

            // Render line.
            for y in 0..constants::DISPLAY_HEIGHT {
                for x in 0..constants::DISPLAY_WIDTH {
                    self.canvas.draw_pixel_rgba(
                        x,
                        y,
                        pixels[(y * constants::DISPLAY_WIDTH + x) * 4],
                        pixels[(y * constants::DISPLAY_WIDTH + x) * 4 + 1],
                        pixels[(y * constants::DISPLAY_WIDTH + x) * 4 + 2],
                        pixels[(y * constants::DISPLAY_WIDTH + x) * 4 + 3],
                    );
                }
            }
            self.last_ly = y;
            // Only present when all screen lines are in the buffer (or debugging).
            if y == 143 || self.debug {
                self.canvas.flush();
            }
        }
    }
}
