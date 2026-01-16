/// User interface manager. Holds the widgets and renders them.
pub struct UIManager<'a> {
    widgets: Vec<Box<dyn Widget + 'a>>,
}

impl<'a> UIManager<'a> {
    pub fn new() -> Self {
        UIManager { widgets: vec![] }
    }

    pub fn add_widget(&mut self, widget: Box<dyn Widget + 'a>) {
        self.widgets.push(widget);
    }

    pub fn handle_event(&mut self, event: &sdl2::event::Event) {
        for w in &mut self.widgets {
            w.handle_event(event);
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        for w in &self.widgets {
            w.render(canvas);
        }
    }
}

pub trait Widget {
    fn handle_event(&mut self, event: &sdl2::event::Event);
    fn render(&self, canvas: &mut Canvas);
}
