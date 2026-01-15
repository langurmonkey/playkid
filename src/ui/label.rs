pub struct Label {
    id: usize,
    text: String,
    x: f32,
    y: f32,
    color: sdl2::pixels::Color,
    background_color: Option<sdl2::pixels::Color>,
    shadow: bool,
    shadow_offset: (i32, i32),
}

impl Label {
    pub fn new(
        id: usize,
        text: &str,
        x: f32,
        y: f32,
        color: sdl2::pixels::Color,
        background_color: Option<sdl2::pixels::Color>,
        shadow: bool,
    ) -> Self {
        Label {
            id,
            text: text.to_string(),
            x,
            y,
            color,
            background_color,
            shadow,
            shadow_offset: (2, 2), // Default shadow offset
        }
    }

    /// Renders the label to the canvas.
    pub fn render(&self, canvas: &mut Canvas) {
        // If background color is provided, draw the background.
        if let Some(bg_color) = self.background_color {
            // Render the background rectangle around the text
            let text_surface = self.create_text_surface(canvas);
            let text_width = text_surface.width();
            let text_height = text_surface.height();
            let padding = 5;

            let bg_rect = Rect::new(
                (self.x - padding as f32) as i32,
                (self.y - padding as f32) as i32,
                (text_width + padding * 2) as u32,
                (text_height + padding * 2) as u32,
            );
            canvas.sdl_canvas.set_draw_color(bg_color);
            canvas.sdl_canvas.fill_rect(bg_rect).unwrap();
        }

        // Draw text with optional shadow
        if self.shadow {
            canvas.draw_text_shadow(
                self.id,
                &self.text,
                self.x + self.shadow_offset.0 as f32,
                self.y + self.shadow_offset.1 as f32,
                sdl2::pixels::Color::RGB(0, 0, 0), // Shadow color
                true,
            );
        }

        // Draw the main text layer
        canvas.draw_text(self.id, &self.text, self.x, self.y, self.color);
    }

    /// Creates a surface for the text to measure its size.
    fn create_text_surface(&self, canvas: &Canvas) -> sdl2::surface::Surface<'static> {
        let font = &canvas.font;
        font.render(&self.text)
            .blended(self.color)
            .expect("Failed to render text")
    }
}

