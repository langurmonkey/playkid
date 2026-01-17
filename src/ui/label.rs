use crate::canvas;
use crate::ui;

use canvas::Canvas;
use sdl2::rect::Rect;
use sdl2::ttf::Font;
use ui::uimanager::UIManager;
use ui::uimanager::Widget;

/// A label widget.
pub struct Label {
    /// Is this label visible?
    visible: bool,
    /// Label text.
    text: String,
    /// Font size.
    font_size: usize,
    /// X position.
    x: f32,
    /// Y position.
    y: f32,
    /// Width.
    width: u32,
    /// Height.
    height: u32,
    /// Main color.
    color: sdl2::pixels::Color,
    /// Background color, if any.
    background_color: Option<sdl2::pixels::Color>,
    /// Cast shadow.
    shadow: bool,
    /// Shadow offset.
    shadow_offset: (i32, i32),
}

impl Widget for Label {
    fn render(&self, canvas: &mut Canvas, ui: &UIManager) {
        if !self.visible {
            return;
        }
        let font = ui.font(self.get_font_size());
        let scale_factor = canvas.get_scale_factor();

        // Render background color if set.
        if let Some(bg_color) = self.background_color {
            let text_surface = font
                .render(&self.text)
                .blended(self.color)
                .expect("Failed to render text");
            let text_width = text_surface.width();
            let text_height = text_surface.height();
            let padding = (5.0 * scale_factor) as u32;

            let bg_rect = Rect::new(
                ((self.x * scale_factor) - padding as f32) as i32,
                ((self.y * scale_factor) - padding as f32) as i32,
                (text_width * scale_factor as u32) + (padding * 2),
                (text_height * scale_factor as u32) + (padding * 2),
            );
            canvas.sdl_canvas.set_draw_color(bg_color);
            canvas.sdl_canvas.fill_rect(bg_rect).unwrap();
        }

        // Draw text with shadow if enabled
        if self.shadow {
            let text_surface = font
                .render(&self.text)
                .blended(sdl2::pixels::Color::RGB(0, 0, 0)) // Shadow color
                .expect("Failed to render shadow text");
            canvas
                .sdl_canvas
                .copy(
                    &canvas
                        .creator
                        .create_texture_from_surface(&text_surface)
                        .unwrap(),
                    None,
                    Some(Rect::new(
                        ((self.x + self.shadow_offset.0 as f32) * scale_factor) as i32,
                        ((self.y + self.shadow_offset.1 as f32) * scale_factor) as i32,
                        (text_surface.width() as f32 * scale_factor) as u32,
                        (text_surface.height() as f32 * scale_factor) as u32,
                    )),
                )
                .unwrap();
        }

        // Render the actual text
        let text_surface = font
            .render(&self.text)
            .blended(self.color)
            .expect("Failed to render text");
        canvas
            .sdl_canvas
            .copy(
                &canvas
                    .creator
                    .create_texture_from_surface(&text_surface)
                    .unwrap(),
                None,
                Some(Rect::new(
                    (self.x * scale_factor) as i32,
                    (self.y * scale_factor) as i32,
                    (text_surface.width() as f32 * scale_factor) as u32,
                    (text_surface.height() as f32 * scale_factor) as u32,
                )),
            )
            .unwrap();
    }

    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        false
    }

    fn set_color(&mut self, color: sdl2::pixels::Color) {
        self.color = color;
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

    fn get_pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }

    fn get_font_size(&self) -> usize {
        self.font_size
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn has_size(&self) -> bool {
        self.width > 0 && self.height > 0
    }

    fn update_size(&mut self, font: &Font) {
        let t = if self.text.trim().is_empty() {
            // Enxure size can be computed.
            "D"
        } else {
            &self.text
        };
        let (w, h) = font.size_of(t).unwrap_or((0, 0));
        self.width = w;
        self.height = h;
    }

    fn layout(&mut self, ui: &UIManager, start_x: f32, start_y: f32) {}
}

impl Label {
    pub fn new(
        text: &str,
        size: usize,
        x: f32,
        y: f32,
        color: sdl2::pixels::Color,
        background_color: Option<sdl2::pixels::Color>,
        shadow: bool,
    ) -> Self {
        Label {
            visible: true,
            text: text.to_string(),
            font_size: size,
            x,
            y,
            width: 0,
            height: 0,
            color,
            background_color,
            shadow,
            shadow_offset: (2, 2), // Default shadow offset
        }
    }

    /// Sets the text of this label.
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }
}
