use crate::canvas;

use canvas::Canvas;
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
    //// SDL 2 TTF context to load fonts on the fly..
    ttf: &'ttf sdl2::ttf::Sdl2TtfContext,
    /// Stores fonts indexed by their point size.
    fonts: RefCell<HashMap<usize, Arc<Font<'ttf, 'ttf>>>>,
}

impl<'ttf> UIManager<'ttf> {
    pub fn new(ttf: &'ttf sdl2::ttf::Sdl2TtfContext) -> Result<Self, String> {
        let mut fonts = HashMap::new();
        // Pre-load 10pt font..
        let default_font = ttf.load_font("assets/fnt/PixelatedElegance.ttf", 10)?;
        fonts.insert(10, Arc::new(default_font));

        Ok(UIManager {
            widgets: vec![],
            ttf,
            fonts: RefCell::new(fonts),
        })
    }

    pub fn add_widget<W: Widget + 'ttf>(&mut self, widget: Rc<RefCell<W>>) {
        self.widgets.push(widget as Rc<RefCell<dyn Widget + 'ttf>>);
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas
            .sdl_canvas
            .set_blend_mode(sdl2::render::BlendMode::Blend);
        for widget in &self.widgets {
            let w = widget.borrow();
            if w.is_visible() {
                let font = self.font(w.get_font_size());
                w.render(canvas, &font);
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
        println!(
            "{}: Loading font size {} on the fly...",
            "LD".magenta(),
            size
        );
        let new_font = self
            .ttf
            .load_font("assets/fnt/PixelatedElegance.ttf", size as u16)
            .expect("Failed to load font size at runtime");

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

pub trait Widget {
    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool;
    fn render(&self, canvas: &mut Canvas, font: &Arc<Font>);
    fn is_visible(&self) -> bool;
    fn visible(&mut self, visible: bool);
    fn set_pos(&mut self, x: f32, y: f32);
    fn get_pos(&self) -> (f32, f32);
    fn get_font_size(&self) -> usize;
    fn get_size(&self) -> (u32, u32);
    fn has_size(&self) -> bool;
    fn update_size(&mut self, font: &Font);
}
