use crate::canvas;
use crate::eventhandler;
use crate::ui;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use std::sync::Arc;
use ui::uimanager::UIManager;
use ui::uimanager::Widget;

/// Button widget.
pub struct Button {
    visible: bool,
    label: String,
    size: usize,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    callback: Option<Box<dyn Fn()>>,
}

impl Widget for Button {
    /// Renders the button to the canvas.
    fn render(&self, canvas: &mut Canvas, ui: &UIManager) {
        if !self.visible {
            return;
        }
        let scale_factor = canvas.get_scale_factor();
    }

    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        //self.handle_click()
        false
    }

    fn set_color(&mut self, color: sdl2::pixels::Color) {}

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn is_visible(&self) -> bool {
        self.visible
    }

    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn get_pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn get_font_size(&self) -> usize {
        10
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn has_size(&self) -> bool {
        self.width <= 0 || self.height <= 0
    }

    fn update_size(&mut self, font: &Font) {
        let (w, h) = font.size_of(&self.label).unwrap_or((0, 0));
        self.width = w;
        self.height = h;
    }
    fn layout(&mut self, ui: &UIManager, start_x: f32, start_y: f32) {}
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
            size: 12,
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
