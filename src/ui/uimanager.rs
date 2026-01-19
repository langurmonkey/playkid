use crate::canvas::Canvas;
use colored::Colorize;
use sdl2::event::Event;
use sdl2::ttf::Font;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;

/// User interface manager. Holds the widgets and renders them.
pub struct UIManager<'ttf> {
    /// List of widgets.
    widgets: Vec<Rc<RefCell<dyn Widget + 'ttf>>>,
    //// SDL 2 TTF context to load fonts on the fly.
    ttf: &'ttf sdl2::ttf::Sdl2TtfContext,
    /// Stores fonts indexed by their point size.
    fonts: RefCell<HashMap<usize, Arc<Font<'ttf, 'ttf>>>>,
}

impl<'ttf> UIManager<'ttf> {
    pub fn new(ttf: &'ttf sdl2::ttf::Sdl2TtfContext) -> Result<Self, String> {
        let fonts = HashMap::new();
        Ok(UIManager {
            widgets: vec![],
            ttf,
            fonts: RefCell::new(fonts),
        })
    }

    pub fn add_widget<'a, W>(&mut self, widget: W)
    where
        W: IntoWidgetPtr<'ttf>,
    {
        self.widgets.push(widget.into_widget_ptr());
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas
            .sdl_canvas
            .set_blend_mode(sdl2::render::BlendMode::Blend);
        for widget in &self.widgets {
            let w = widget.borrow();
            if w.is_visible() {
                w.render(canvas, &self);
            }
        }
    }

    /// Retrieves the requested font or falls back to the closest available size.
    pub fn font(&self, size: usize) -> Arc<Font<'ttf, 'ttf>> {
        // Check if font exists (Scope the borrow).
        {
            let cache = self.fonts.borrow();
            if let Some(font) = cache.get(&size) {
                return Arc::clone(font);
            }
        }

        // If not found, load it.
        println!("{}: Loading font size {}", "LD".magenta(), size);
        let mut new_font = self
            .ttf
            .load_font("assets/fnt/PressStart2P.ttf", size as u16)
            .expect("Failed to load font size at runtime");
        new_font.set_hinting(sdl2::ttf::Hinting::None);

        let shared_font = Arc::new(new_font);

        // Store it in the cache.
        self.fonts
            .borrow_mut()
            .insert(size, Arc::clone(&shared_font));

        shared_font
    }

    pub fn handle_event(&mut self, event: &Event) -> bool {
        let mut result = false;
        for widget in &mut self.widgets {
            result = result || widget.borrow_mut().handle_event(event);
        }
        result
    }
}

/// UI action queue.
pub struct UIState {
    pub reset_requested: bool,
    pub step_requested: bool,
    pub scanline_requested: bool,
    pub continue_requested: bool,
    pub br_add_requested: bool,
    pub br_remove_requested: bool,
    pub br_clear_requested: bool,
    pub br_addr: u16,
    pub exit_requested: bool,
    pub fps_requested: bool,
}

impl UIState {
    pub fn new() -> Self {
        Self {
            reset_requested: false,
            step_requested: false,
            scanline_requested: false,
            continue_requested: false,
            br_add_requested: false,
            br_remove_requested: false,
            br_clear_requested: false,
            br_addr: 0x00,
            exit_requested: false,
            fps_requested: false,
        }
    }
}

/// Widget trait.
pub trait Widget {
    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool;
    fn render(&self, canvas: &mut Canvas, ui: &UIManager);
    fn is_visible(&self) -> bool;
    fn set_visible(&mut self, visible: bool);
    fn set_color(&mut self, color: sdl2::pixels::Color);
    fn set_pos(&mut self, x: f32, y: f32);
    fn get_font_size(&self) -> usize;
    fn get_size(&self) -> (u32, u32);
    fn update_size(&mut self, font: &Font);
    fn layout(&mut self, ui: &UIManager, start_x: f32, start_y: f32);
}

// Helper trait for easy coercion.
pub trait IntoWidgetPtr<'ttf> {
    fn into_widget_ptr(self) -> Rc<RefCell<dyn Widget + 'ttf>>;
}
impl<'ttf, T: Widget + 'ttf> IntoWidgetPtr<'ttf> for Rc<RefCell<T>> {
    fn into_widget_ptr(self) -> Rc<RefCell<dyn Widget + 'ttf>> {
        self as Rc<RefCell<dyn Widget + 'ttf>>
    }
}
