/// Represents a basic text input field.
pub struct TextField {
    id: usize,
    text: String,
    x: f32,
    y: f32,
    width: u32,
    height: u32,
    focused: bool,
}

impl TextField {
    pub fn new(id: usize, x: f32, y: f32, width: u32, height: u32) -> Self {
        TextField {
            id,
            text: String::new(),
            x,
            y,
            width,
            height,
            focused: false,
        }
    }

    /// Renders the text field to the canvas.
    pub fn render(&self, canvas: &mut Canvas) {
        // Draw background
        canvas.sdl_canvas.set_draw_color(sdl2::pixels::Color::RGB(255, 255, 255));
        canvas.sdl_canvas.fill_rect(Rect::new(self.x as i32, self.y as i32, self.width, self.height)).unwrap();

        // Draw text (current text inside the field)
        let color = if self.focused { sdl2::pixels::Color::RGB(0, 255, 0) } else { sdl2::pixels::Color::RGB(0, 0, 0) };
        canvas.draw_text(self.id, &self.text, self.x, self.y, color);

        // Draw border (optional, to show focus state)
        canvas.sdl_canvas.set_draw_color(sdl2::pixels::Color::RGB(0, 0, 0));
        canvas.sdl_canvas.draw_rect(Rect::new(self.x as i32, self.y as i32, self.width, self.height)).unwrap();
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
                if keycode == Some(sdl2::keyboard::Keycode::Backspace) && !self.text.is_empty() {
                    self.text.pop();
                }
            }
            _ => {}
        }
    }

    /// Checks if the user clicked inside the text field to focus it.
    pub fn handle_click(&mut self, x: f32, y: f32) {
        if x >= self.x && x <= self.x + self.width as f32 && y >= self.y && y <= self.y + self.height as f32 {
            self.focused = true;
        } else {
            self.focused = false;
        }
    }
}

