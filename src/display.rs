use crate::canvas;
use crate::constants;
use crate::memory;

use canvas::Canvas;
use memory::Memory;
use sdl2;
use sdl2::pixels::Color;
use sdl2::Sdl;

/// ID of the FPS text.
const ID_FPS: usize = 0;

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
    pub fn new(
        window_title: &str,
        scale: u8,
        sdl: &'a Sdl,
        ttf: &'a sdl2::ttf::Sdl2TtfContext,
        debug: bool,
    ) -> Self {
        let w = constants::DISPLAY_WIDTH;
        let h = constants::DISPLAY_HEIGHT;

        let canvas = Canvas::new(sdl, ttf, window_title, w, h, scale as usize).unwrap();

        Display {
            canvas,
            debug,
            last_ly: 255,
        }
    }

    /// Set the debug flag of this display.
    /// When true, the display is rendered at every line,
    /// not only at the end of the frame.
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    /// Present the canvas.
    pub fn present(&mut self) {
        self.canvas.present();
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    // Renders the given buffer to the display.
    pub fn render(&mut self, mem: &Memory) {
        let ppu = mem.ppu();

        let y = ppu.ly % constants::DISPLAY_HEIGHT as u8;
        if ppu.data_available && self.last_ly != y {
            let pixels = &ppu.fb;

            // Render line.
            {
                let y = y as usize;
                let offset = y * constants::DISPLAY_WIDTH * 4;
                let slice = &pixels[offset..offset + constants::DISPLAY_WIDTH * 4];
                self.canvas.draw_line_rgba(y, slice);
            }
            self.last_ly = y;
            // Only present when all screen lines are in the buffer (or debugging).
            if y == 143 || self.debug {
                self.canvas.flush(false);
            }
        }
    }

    /// Draws the given FPS value to the canvas.
    pub fn draw_fps(&mut self, fps: f64, color: Color) {
        let fps_str = format!("FPS: {:.2}", fps);
        // Draw at coordinates (10, 10)
        self.canvas.draw_text(ID_FPS, &fps_str, 10.0, 10.0, color);
    }

    /// Remove the current FPS value.
    pub fn remove_fps(&mut self) {
        self.canvas.remove_text(ID_FPS);
    }
}
