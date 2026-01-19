use crate::canvas::Canvas;
use crate::constants;
use crate::debugmanager::DebugManager;
use crate::eventhandler::EventHandler;
use crate::instruction::RunInstr;
use crate::memory::Memory;
use crate::registers::Registers;
use crate::ui::debugui::DebugUI;
use crate::ui::label::Label;
use crate::ui::uimanager::UIManager;
use crate::ui::uimanager::UIState;
use crate::ui::uimanager::Widget;
use colored::Colorize;
use sdl2;
use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::Sdl;
use std::cell::RefCell;
use std::rc::Rc;

/// Background color.
const BG: Color = Color::RGBA(0, 0, 0, 180);

/// Holds the display objects and renders the image from the PPU data.
pub struct Display<'a> {
    /// The canvas itself.
    pub canvas: Canvas<'a>,
    /// Holds the last rendered LY.
    last_ly: u8,
    /// Run in debug mode (present after every line).
    debug: bool,

    /// User interface manager.
    pub ui: UIManager<'a>,
    /// Debug UI.
    pub debug_ui: DebugUI<'a>,
    /// FPS counter.
    pub fps: Rc<RefCell<Label>>,
}

impl<'a> EventHandler for Display<'a> {
    /// Handles a single event.
    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            Event::Window {
                win_event: sdl2::event::WindowEvent::Resized(_, _),
                ..
            } => {
                // Update window constraints on resize.
                self.canvas.update_window_constraints(self.debug);
            }
            _ => {}
        }

        self.ui.handle_event(event, &self.canvas)
    }
}

impl<'a> Display<'a> {
    pub fn new(
        window_title: &str,
        sdl: &'a Sdl,
        ttf: &'a sdl2::ttf::Sdl2TtfContext,
        scale: u8,
        debug: bool,
        ui_state: Rc<RefCell<UIState>>,
    ) -> Result<Self, String> {
        let w = constants::DISPLAY_WIDTH;
        let h = constants::DISPLAY_HEIGHT;

        // Create canvas.
        let canvas = Canvas::new(sdl, window_title, w, h, scale as usize)
            .map_err(|e| format!("{}: Failed to create canvas: {}", "ERR".red(), e))?;

        // Create UI manager.
        let mut ui = UIManager::new(ttf)
            .map_err(|e| format!("{}: Failed to create UI manager: {}", "ERR".red(), e))?;

        // Debug UI.
        let mut debug_ui = DebugUI::new(&mut ui, Rc::clone(&ui_state));
        debug_ui.set_debug_visibility(debug);

        // FPS counter.
        let fps = Rc::new(RefCell::new(Label::new(
            "FPS: 0.0",
            10,
            10.0,
            10.0,
            Color::RED,
            Some(BG),
            false,
        )));
        ui.add_widget(Rc::clone(&fps));

        let display = Display {
            canvas,
            ui,
            debug_ui,
            debug,
            fps,
            last_ly: 255,
        };

        Ok(display)
    }

    pub fn update_fps(&mut self, fps_value: f64) {
        self.fps
            .borrow_mut()
            .set_text(&format!("FPS: {:.1}", fps_value));
    }

    /// Set FPS visibility.
    pub fn visible_fps(&mut self, visible: bool) {
        self.fps.borrow_mut().set_visible(visible);
    }

    /// Update the debug UI.
    pub fn machine_state_update(
        &mut self,
        pc: u16,
        reg: &Registers,
        mem: &Memory,
        run_instr: &RunInstr,
        debug: &DebugManager,
        opcode: u8,
        t_cycles: u32,
        m_cycles: u32,
        halted: bool,
    ) {
        self.debug_ui.machine_state_update(
            pc, reg, mem, run_instr, debug, opcode, t_cycles, m_cycles, halted,
        );
    }

    /// Set the debug flag of this display.
    /// When true, the display is rendered at every line,
    /// not only at the end of the frame.
    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
        self.debug_ui.set_debug_visibility(debug);

        // Force the window to respect the new minimum bounds.
        self.canvas.update_window_constraints(debug);
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
        // First, update positions of debug UI widgets.
        let (x, y, w, _) = self.canvas.get_lcd_rect();
        let sf = self.canvas.get_scale_factor();
        let dx = (x as f32 + w as f32 + 20.0 * sf) / sf;
        let dy = (y as f32) / sf;
        self.debug_ui.update_positions(&self.ui, dx, dy);

        // Actual render operation.
        self.ui.render(&mut self.canvas);
    }
}
