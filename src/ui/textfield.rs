use crate::canvas;
use crate::eventhandler;
use crate::ui;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use std::sync::Arc;
use ui::Widget;

/// Text field widget.
pub struct TextField {
    visible: bool,
    text: String,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    focused: bool,
}

impl Widget for TextField {
    /// Renders the text field to the canvas.
    fn render(&self, canvas: &mut Canvas, font: &Arc<Font>) {
        if !self.visible {
            return;
        }
        let scale_factor = canvas.get_scale_factor();
    }

    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        self.handle_input(event);
        /// Click
        false
    }

    fn visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn set_position(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }

    fn get_font_size(&self) -> usize {
        10
    }
}

impl TextField {
    pub fn new(x: f32, y: f32, width: u32, height: u32) -> Self {
        TextField {
            visible: true,
            text: String::new(),
            x,
            y,
            width,
            height,
            focused: false,
        }
    }

    /// Updates the text based on user input (keypress events).
    pub fn handle_input(&mut self, event: &sdl2::event::Event) {
        if !self.focused {
            return;
        }

        match event {
            sdl2::event::Event::TextInput { text, .. } => {
                self.text.push_str(text);
            }
            sdl2::event::Event::KeyDown { keycode, .. } => {
                if *keycode == Some(sdl2::keyboard::Keycode::Backspace) && !self.text.is_empty() {
                    self.text.pop();
                }
            }
            _ => {}
        }
    }

    /// Checks if the user clicked inside the text field to focus it.
    pub fn handle_click(&mut self, x: f32, y: f32) {
        if x >= self.x
            && x <= self.x + self.width as f32
            && y >= self.y
            && y <= self.y + self.height as f32
        {
            self.focused = true;
        } else {
            self.focused = false;
        }
    }
}
