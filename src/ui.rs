pub mod button;
pub mod label;
pub mod textfield;

use crate::canvas;

use canvas::Canvas;
use sdl2::event::Event;
use sdl2::ttf::Font;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

/// User interface manager. Holds the widgets and renders them.
pub struct UIManager<'ttf> {
    widgets: Vec<Rc<RefCell<dyn Widget + 'ttf>>>,
    pub font: Arc<Font<'ttf, 'ttf>>,
}

impl<'ttf> UIManager<'ttf> {
    pub fn new(font: Font<'ttf, 'ttf>) -> Self {
        UIManager {
            widgets: vec![],
            font: Arc::new(font),
        }
    }

    pub fn add_widget<W: Widget + 'ttf>(&mut self, widget: Rc<RefCell<W>>) {
        self.widgets.push(widget as Rc<RefCell<dyn Widget + 'ttf>>);
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas
            .sdl_canvas
            .set_blend_mode(sdl2::render::BlendMode::Blend);
        for widget in &self.widgets {
            widget.borrow().render(canvas, &self.font);
        }
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
    fn visible(&mut self, visible: bool);
    fn set_position(&mut self, x: f32, y: f32);
}
