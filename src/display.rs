use crate::constants;

use sdl2;
use sdl2::rect::Rect;
use sdl2::{pixels::Color, EventPump};
use sdl2::{render::Canvas, video::Window};

/// Holds the display data, and renders the image.
pub struct Display {
    pub canvas: Canvas<Window>,
    pub event_pump: EventPump,
    pub scale: u32,
    pub fgcol: Color,
    pub bgcol: Color,
}

impl Display {
    pub fn new(window_title: &str, scale: u32, fg_col: (u8, u8, u8), bg_col: (u8, u8, u8)) -> Self {
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

        Display {
            canvas,
            event_pump,
            scale,
            fgcol: Color::RGB(fg_col.0, fg_col.1, fg_col.2),
            bgcol: Color::RGB(bg_col.0, bg_col.1, bg_col.2),
        }
    }

    // Clears the display to black
    pub fn clear(&mut self) {
        self.canvas.set_draw_color(self.bgcol);
        self.canvas.clear();
        self.canvas.present();
    }

    // Renders the given buffer to the display
    pub fn render(&mut self, buffer: [u8; 10]) {
        // Fill with buffer
        let scl = self.scale as usize;
        for x in 0..constants::DISPLAY_WIDTH {
            for y in 0..constants::DISPLAY_HEIGHT {
                if buffer[y * constants::DISPLAY_WIDTH + x] > 0 {
                    // Foreground
                    self.canvas.set_draw_color(self.fgcol);
                    self.canvas
                        .fill_rect(Rect::new(
                            (x * scl) as i32,
                            (y * scl) as i32,
                            scl as u32,
                            scl as u32,
                        ))
                        .unwrap();
                } else {
                    // Background
                    self.canvas.set_draw_color(self.bgcol);
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
        }

        self.canvas.present();
    }
}
