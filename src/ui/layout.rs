use crate::canvas::Canvas;
use crate::ui::uimanager::UIManager;
use crate::ui::uimanager::Widget;

use sdl2::ttf::Font;
use std::cell::RefCell;
use std::rc::Rc;

pub enum Orientation {
    Horizontal,
    Vertical,
}

pub struct LayoutGroup<'ttf> {
    widgets: Vec<Rc<RefCell<dyn Widget + 'ttf>>>,
    orientation: Orientation,
    x: f32,
    y: f32,
    spacing: f32,
    visible: bool,
    // We store the calculated total size of the group
    width: u32,
    height: u32,
}

impl<'ttf> LayoutGroup<'ttf> {
    pub fn new(orientation: Orientation, spacing: f32) -> Self {
        Self {
            widgets: Vec::new(),
            orientation,
            x: 0.0,
            y: 0.0,
            spacing,
            visible: true,
            width: 0,
            height: 0,
        }
    }

    pub fn add(&mut self, widget: Rc<RefCell<dyn Widget + 'ttf>>) {
        self.widgets.push(widget);
    }

    /// This is the magic layout function.
    /// It calculates sizes recursively and sets positions.
    pub fn layout(&mut self, ui: &UIManager, start_x: f32, start_y: f32) {
        self.x = start_x;
        self.y = start_y;

        let mut curr_x = start_x;
        let mut curr_y = start_y;
        let mut max_w = 0u32;
        let mut max_h = 0u32;

        for widget_rc in &self.widgets {
            let mut w = widget_rc.borrow_mut();

            // 1. RECURSION: Tell child to layout its own internal children first
            w.layout(ui, curr_x, curr_y);

            // 2. SIZE: Update size based on text/font before we calculate the next gap
            let font = ui.font(w.get_font_size());
            w.update_size(&font);

            // 3. POSITION: Apply the current coordinates to the widget
            w.set_pos(curr_x, curr_y);

            // 4. OFFSET: Advance the cursor for the next widget in the group
            let (w_sz, h_sz) = w.get_size();
            match self.orientation {
                Orientation::Horizontal => {
                    curr_x += w_sz as f32 + self.spacing;
                    max_h = max_h.max(h_sz);
                    max_w += w_sz + self.spacing as u32;
                }
                Orientation::Vertical => {
                    curr_y += h_sz as f32 + self.spacing;
                    max_w = max_w.max(w_sz);
                    max_h += h_sz + self.spacing as u32;
                }
            }
        }
        self.width = max_w;
        self.height = max_h;
    }
}

impl<'ttf> Widget for LayoutGroup<'ttf> {
    fn render(&self, canvas: &mut Canvas, ui: &UIManager) {
        if !self.visible {
            return;
        }
        for widget_rc in &self.widgets {
            let w = widget_rc.borrow();
            if w.is_visible() {
                w.render(canvas, ui);
            }
        }
    }

    fn update_size(&mut self, font: &Font) {
        // Not used for groups, as they use the `layout()` method instead.
    }

    fn get_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }
    fn has_size(&self) -> bool {
        true
    }
    fn set_pos(&mut self, x: f32, y: f32) {
        self.x = x;
        self.y = y;
    }
    fn get_pos(&self) -> (f32, f32) {
        (self.x, self.y)
    }
    fn set_color(&mut self, color: sdl2::pixels::Color) {}
    fn is_visible(&self) -> bool {
        self.visible
    }
    fn set_visible(&mut self, v: bool) {
        self.visible = v;
        for widget_rc in &self.widgets {
            widget_rc.borrow_mut().set_visible(v);
        }
    }
    fn get_font_size(&self) -> usize {
        10
    }
    fn handle_event(&mut self, event: &sdl2::event::Event) -> bool {
        if !self.visible {
            return false;
        }
        let mut consumed = false;
        for widget_rc in &self.widgets {
            consumed = consumed || widget_rc.borrow_mut().handle_event(event);
        }
        consumed
    }
    fn layout(&mut self, ui: &UIManager, start_x: f32, start_y: f32) {
        self.layout(ui, start_x, start_y);
    }
}
