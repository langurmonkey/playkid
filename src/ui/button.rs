use crate::canvas;
use crate::ui;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use ui::uimanager::UIManager;
use ui::uimanager::Widget;

/// A clickable button connected to a callback.
pub struct Button {
    visible: bool,
    text: String,
    font_size: usize,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    color: Color,
    bg_color: Color,
    pressed_color: Color,
    is_pressed: bool,
    is_hovered: bool,
    /// The callback closure executed on click.
    on_click: Box<dyn FnMut()>,
}

impl Widget for Button {
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
            (self.width as f32 * scale) as u32 + (padding * 2),
            (self.height as f32 * scale) as u32 + (padding * 2),
        );

        // Determine background color.
        let current_bg = if self.is_pressed {
            self.pressed_color
        } else {
            self.bg_color
        };
        canvas.sdl_canvas.set_draw_color(current_bg);
        canvas.sdl_canvas.fill_rect(rect).unwrap();

        // Determine border color (highlight on hover).
        let border_color = if self.is_pressed {
            Color::RGB(100, 100, 100)
        } else if self.is_hovered {
            Color::RGB(255, 255, 255)
        } else {
            Color::RGB(150, 150, 150)
        };

        canvas.sdl_canvas.set_draw_color(border_color);
        canvas.sdl_canvas.draw_rect(rect).unwrap();

        // Render Text.
        let text_surface = font.render(&self.text).blended(self.color).unwrap();
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

    fn handle_event(&mut self, event: &Event, canvas: &Canvas) -> bool {
        if !self.visible {
            return false;
        }

        let scale_factor = canvas.get_scale_factor();
        match event {
            // Track mouse movement for hover state
            Event::MouseMotion { mut x, mut y, .. } => {
                x = (x as f32 / scale_factor) as i32;
                y = (y as f32 / scale_factor) as i32;
                self.is_hovered = self.is_within_bounds(x, y);
                false // Return false so other widgets can also see motion
            }
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                mut x,
                mut y,
                ..
            } => {
                x = (x as f32 / scale_factor) as i32;
                y = (y as f32 / scale_factor) as i32;
                if self.is_within_bounds(x, y) {
                    self.is_pressed = true;
                    return true;
                }
                false
            }
            Event::MouseButtonUp {
                mouse_btn: MouseButton::Left,
                mut x,
                mut y,
                ..
            } => {
                if self.is_pressed {
                    self.is_pressed = false;
                    x = (x as f32 / scale_factor) as i32;
                    y = (y as f32 / scale_factor) as i32;
                    if self.is_within_bounds(x, y) {
                        (self.on_click)();
                    }
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn set_color(&mut self, color: sdl2::pixels::Color) {
        self.color = color;
    }

    fn update_size(&mut self, font: &Font) {
        let (w, h) = font.size_of(&self.text).unwrap_or((0, 0));
        self.width = w;
        self.height = h;
    }

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

impl Button {
    pub fn new<F>(
        text: &str,
        size: usize,
        col: Color,
        bg: Color,
        pressed: Color,
        on_click: F,
    ) -> Self
    where
        F: FnMut() + 'static,
    {
        Button {
            visible: true,
            text: text.to_string(),
            font_size: size,
            x: 0.0,
            y: 0.0,
            width: 0,
            height: 0,
            color: col,
            bg_color: bg,
            pressed_color: pressed,
            is_pressed: false,
            is_hovered: false,
            on_click: Box::new(on_click),
        }
    }

    fn is_within_bounds(&self, mx: i32, my: i32) -> bool {
        // Correcting for padding in the bounds check.
        let padding = 8;
        let full_width = self.width + (padding * 2);
        let full_height = self.height + (padding * 2);

        mx >= self.x as i32
            && mx <= (self.x as i32 + full_width as i32)
            && my >= self.y as i32
            && my <= (self.y as i32 + full_height as i32)
    }

    pub fn set_callback(&mut self, on_click: Box<dyn FnMut()>) {
        self.on_click = on_click;
    }

    /// Sets the text of this button.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}
