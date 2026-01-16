use crate::canvas;
use crate::eventhandler;
use crate::ui;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use std::sync::Arc;
use ui::Widget;

/// Button widget.
pub struct Button {
    visible: bool,
    label: String,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    callback: Option<Box<dyn Fn()>>,
}

impl Widget for Button {
    /// Renders the button to the canvas.
    fn render(&self, canvas: &mut Canvas, font: &Arc<Font>) {
        if !self.visible {
            return;
        }
        let scale_factor = canvas.get_scale_factor();
    }

    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        //self.handle_click()
        false
    }

    fn visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
}

impl Button {
    pub fn new(
        label: &str,
        x: f32,
        y: f32,
        width: u32,
        height: u32,
        callback: Option<Box<dyn Fn()>>,
    ) -> Self {
        Button {
            visible: true,
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
