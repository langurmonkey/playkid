mod button;
mod label;
mod textfield;

use crate::canvas;
use crate::eventhandler;

use canvas::Canvas;
use sdl2::event::Event;

/// User interface manager. Holds the widgets and renders them.
pub struct UIManager<'a> {
    widgets: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> eventhandler::EventHandler for UIManager<'a> {
    fn handle_event(&mut self, event: &Event) -> bool {
        let mut result = false;
        for w in &mut self.widgets {
            result = result || w.handle_event(event);
        }
        result
    }
}

impl<'a> UIManager<'a> {
    pub fn new() -> Self {
        UIManager { widgets: vec![] }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget + 'a>) {
        self.widgets.push(widget);
    }

    pub fn render(&self, canvas: &mut Canvas) {
        for w in &self.widgets {
            w.render(canvas);
        }
    }
}

pub trait Widget {
    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool;
    fn render(&self, canvas: &mut Canvas);
}
