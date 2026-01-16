use crate::canvas;
use crate::constants;
use crate::eventhandler;
use crate::memory;
use crate::ui;

use canvas::Canvas;
use memory::Memory;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::Sdl;
use std::cell::RefCell;
use std::rc::Rc;
use ui::label::Label;
use ui::UIManager;
use ui::Widget;

/// Background color.
const BG: Color = Color::RGBA(0, 0, 0, 180);

/// Holds the display objects and renders the image from the PPU data.
pub struct Display<'a> {
    /// The canvas itself.
    pub canvas: Canvas<'a>,
    /// User interface manager.
    pub ui: UIManager<'a>,
    /// FPS counter.
    fps: Rc<RefCell<Label>>,
    /// Debug title.
    debug_title: Rc<RefCell<Label>>,
    /// Holds the last rendered LY.
    last_ly: u8,
    /// Run in debug mode (present after every line).
    debug: bool,
}

impl<'a> eventhandler::EventHandler for Display<'a> {
    /// Handles a single event.
    fn handle_event(&mut self, event: &Event) -> bool {
        self.ui.handle_event(event);
        true
    }
}

impl<'a> Display<'a> {
    pub fn new(
        window_title: &str,
        scale: u8,
        sdl: &'a Sdl,
        ttf: &'a sdl2::ttf::Sdl2TtfContext,
        debug: bool,
    ) -> Result<Self, String> {
        let w = constants::DISPLAY_WIDTH;
        let h = constants::DISPLAY_HEIGHT;

        // 10pt font.
        let font10 = ttf
            .load_font("assets/fnt/PixelatedElegance.ttf", 10)
            .map_err(|e| format!("Failed to load font: {}", e))?;

        // 14pt font.
        let font14 = ttf
            .load_font("assets/fnt/PixelatedElegance.ttf", 14)
            .map_err(|e| format!("Failed to load font: {}", e))?;

        // Create canvas.
        let canvas = Canvas::new(sdl, window_title, w, h, scale as usize)
            .map_err(|e| format!("Failed to create canvas: {}", e))?;

        // Create UI manager.
        let mut ui = UIManager::new(font10, font14);

        // FPS label.
        let fps = Label::new("FPS: 0.0", 10, 10.0, 10.0, Color::RED, Some(BG), true);
        let fps = Rc::new(RefCell::new(fps));
        fps.borrow_mut().visible(false);
        // Add to UI manager.
        ui.add_widget(Rc::clone(&fps));

        // Debug title.
        let debug_title = Label::new("Debug interface", 14, 0.0, 0.0, Color::WHITE, None, false);
        let debug_title = Rc::new(RefCell::new(debug_title));
        debug_title.borrow_mut().visible(debug);
        // Add to UI manager.
        ui.add_widget(Rc::clone(&debug_title));

        Ok(Display {
            canvas,
            ui,
            fps,
            debug_title,
            debug,
            last_ly: 255,
        })
    }

    pub fn update_fps(&mut self, fps_value: f64) {
        self.fps
            .borrow_mut()
            .set_text(&format!("FPS: {:.1}", fps_value));
    }

    pub fn visible_fps(&mut self, visible: bool) {
        self.fps.borrow_mut().visible(visible);
    }

    /// Set the debug flag of this display.
    /// When true, the display is rendered at every line,
    /// not only at the end of the frame.
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
        self.debug_title.borrow_mut().visible(debug);
    }

    /// Present the canvas.
    pub fn present(&mut self) {
        self.canvas.present();
    }

    /// Clear the canvas.
    pub fn clear(&mut self) {
        self.canvas.clear();
    }

    /// Renders the given buffer to the display.
    pub fn render_lcd(&mut self, mem: &Memory) {
        let ppu = mem.ppu();
        let y = ppu.ly % constants::DISPLAY_HEIGHT as u8;

        if ppu.data_available && self.last_ly != y {
            let pixels = &ppu.fb;

            // Render line.
            {
                let y = y as usize;
                let offset = y * constants::DISPLAY_WIDTH * 4;
                let slice = &pixels[offset..offset + constants::DISPLAY_WIDTH * 4];
                self.canvas.draw_line_rgba(y, slice);
            }
            self.last_ly = y;
        }
    }

    /// Renders the user interface.
    pub fn render_ui(&mut self) {
        // Render UI at 60 FPS.
        self.update_debug_widget_posiitons();
        self.ui.render(&mut self.canvas);
    }

    /// The widgets that are part of the debug interface must float beside the LCD
    /// screen, so we need to update their positions.
    fn update_debug_widget_posiitons(&mut self) {
        let (x, y, w, _) = self.canvas.get_lcd_rect();
        let dx = x as f32 + w as f32;
        let dy = y as f32;

        // Title.
        self.debug_title
            .borrow_mut()
            .set_position(dx + 10.0, dy + 10.0);
    }
}
