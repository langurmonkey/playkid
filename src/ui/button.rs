use crate::canvas;
use crate::eventhandler;
use crate::ui;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::rect::Rect;
use ui::Widget;

/// Represents a basic button widget.
pub struct Button {
    id: usize,
    label: String,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    callback: Option<Box<dyn Fn()>>,
}

impl Widget for Button {
    /// Renders the button to the canvas.
    fn render(&self, canvas: &mut Canvas) {
        // Draw button background
        canvas
            .sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 255));
        canvas
            .sdl_canvas
            .fill_rect(Rect::new(
                self.x as i32,
                self.y as i32,
                self.width,
                self.height,
            ))
            .unwrap();

        // Draw label text
        canvas.draw_text(
            self.id,
            &self.label,
            self.x + 5.0,
            self.y + 5.0,
            sdl2::pixels::Color::RGB(255, 255, 255),
        );

        // Draw button border
        canvas
            .sdl_canvas
            .set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas
            .sdl_canvas
            .draw_rect(Rect::new(
                self.x as i32,
                self.y as i32,
                self.width,
                self.height,
            ))
            .unwrap();
    }

    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        //self.handle_click()
        false
    }
}

impl Button {
    pub fn new(
        id: usize,
        label: &str,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        callback: Option<Box<dyn Fn()>>,
    ) -> Self {
        Button {
            id,
            label: label.to_string(),
            x,
            y,
            width,
            height,
            callback,
        }
    }

    /// Handles click events on the button.
    pub fn handle_click(&self, x: f32, y: f32) {
        if x >= self.x
            && x <= self.x + self.width as f32
            && y >= self.y
            && y <= self.y + self.height as f32
        {
            // Trigger the callback
            if let Some(callback) = &self.callback {
                callback();
            }
        }
    }
}
