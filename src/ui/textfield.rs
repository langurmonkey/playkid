use crate::canvas;

use crate::ui::uimanager::{UIManager, Widget};
use canvas::Canvas;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;

pub struct TextField {
    visible: bool,
    text: String,
    font_size: usize,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    focused: bool,
    bg_color: Color,
    border_color: Color,
    cursor_timer: u32,
}

impl Widget for TextField {
    fn render(&self, canvas: &mut Canvas, ui: &UIManager) {
        if !self.visible {
            return;
        }

        let font = ui.font(self.font_size);
        let scale = canvas.get_scale_factor();
        let padding = (6.0 * scale) as u32;

        let rect = Rect::new(
            (self.x * scale) as i32,
            (self.y * scale) as i32,
            (self.width as f32 * scale).max(100.0) as u32 + (padding * 2),
            (self.height as f32 * scale) as u32 + (padding * 2),
        );

        // Draw Background.
        canvas.sdl_canvas.set_draw_color(self.bg_color);
        canvas.sdl_canvas.fill_rect(rect).unwrap();

        // Draw Border (Highlight if focused).
        let color = if self.focused {
            Color::RGB(66, 133, 244)
        } else {
            self.border_color
        };
        canvas.sdl_canvas.set_draw_color(color);
        canvas.sdl_canvas.draw_rect(rect).unwrap();

        // Render Text.
        if !self.text.is_empty() {
            let text_surface = font.render(&self.text).blended(Color::WHITE).unwrap();
            let texture = canvas
                .creator
                .create_texture_from_surface(&text_surface)
                .unwrap();
            canvas
                .sdl_canvas
                .copy(
                    &texture,
                    None,
                    Some(Rect::new(
                        (self.x * scale) as i32 + padding as i32,
                        (self.y * scale) as i32 + padding as i32,
                        (text_surface.width() as f32 * scale) as u32,
                        (text_surface.height() as f32 * scale) as u32,
                    )),
                )
                .unwrap();
        }

        // Draw Cursor (if focused and "blinking").
        if self.focused && (self.cursor_timer / 30) % 2 == 0 {
            let text_width = if self.text.is_empty() {
                0
            } else {
                font.size_of(&self.text).unwrap().0
            };
            let cursor_x =
                (self.x * scale) as i32 + padding as i32 + (text_width as f32 * scale) as i32;
            let cursor_rect = Rect::new(
                cursor_x,
                (self.y * scale) as i32 + padding as i32,
                2,
                (self.height as f32 * scale) as u32,
            );
            canvas.sdl_canvas.set_draw_color(Color::WHITE);
            canvas.sdl_canvas.fill_rect(cursor_rect).unwrap();
        }
    }

    fn handle_event(&mut self, event: &Event) -> bool {
        if !self.visible {
            return false;
        }

        match event {
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => {
                self.focused = self.is_within_bounds(*x, *y);
                self.focused
            }
            Event::KeyDown {
                keycode: Some(key), ..
            } if self.focused => {
                match key {
                    &Keycode::Backspace => {
                        self.text.pop();
                    }
                    _ => {}
                }
                true
            }
            Event::TextInput { text, .. } if self.focused => {
                self.text.push_str(text);
                true
            }
            _ => false,
        }
    }

    fn set_color(&mut self, _: sdl2::pixels::Color) {}

    fn update_size(&mut self, font: &Font) {
        let (_, h) = font.size_of("A").unwrap_or((0, 20)); // Base height on a char
        let text_w = font.size_of(&self.text).unwrap_or((0, 0)).0;
        self.width = text_w.max(100); // Give it a base width
        self.height = h;
        self.cursor_timer = self.cursor_timer.wrapping_add(1);
    }

    // Standard Boilerplate
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
    fn get_font_size(&self) -> usize {
        self.font_size
    }
    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn layout(&mut self, _ui: &UIManager, _sx: f32, _sy: f32) {}
}

impl TextField {
    pub fn new(size: usize) -> Self {
        TextField {
            visible: true,
            text: String::new(),
            font_size: size,
            x: 0.0,
            y: 0.0,
            width: 0,
            height: 0,
            focused: false,
            bg_color: Color::RGB(30, 30, 30),
            border_color: Color::RGB(100, 100, 100),
            cursor_timer: 0,
        }
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    fn is_within_bounds(&self, mx: i32, my: i32) -> bool {
        let padding = 6;
        mx >= self.x as i32
            && mx <= (self.x as i32 + self.width as i32 + padding * 2)
            && my >= self.y as i32
            && my <= (self.y as i32 + self.height as i32 + padding * 2)
    }
}
