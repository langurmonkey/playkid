use crate::eventhandler;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;

pub struct DebugManager {
    debugging: bool,
    step_instruction: bool,
    step_line: bool,
}
impl eventhandler::EventHandler for DebugManager {
    /// Process keyboard inputs specifically for debugging.
    /// Returns true if the event was handled.
    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            // Enable/disable debugging.
            Event::KeyDown {
                keycode: Some(Keycode::D),
                ..
            } => {
                self.debugging = !self.debugging;
                true
            }
            // Step single instruction.
            Event::KeyDown {
                keycode: Some(Keycode::F6),
                ..
            } => {
                if self.debugging {
                    self.step_instruction = true;
                }
                true
            }
            // Step a scan line.
            Event::KeyDown {
                keycode: Some(Keycode::F8),
                ..
            } => {
                if self.debugging {
                    self.step_line = true;
                }
                true
            }
            _ => false,
        }
    }
}

impl DebugManager {
    pub fn new(active: bool) -> Self {
        Self {
            debugging: active,
            step_instruction: false,
            step_line: false,
        }
    }

    pub fn debugging(&self) -> bool {
        self.debugging
    }

    /// Check if a single instruction step was requested and reset the flag.
    pub fn take_step_instruction(&mut self) -> bool {
        let val = self.step_instruction;
        self.step_instruction = false;
        val
    }

    /// Check if a scanline step was requested and reset the flag.
    pub fn take_step_line(&mut self) -> bool {
        let val = self.step_line;
        self.step_line = false;
        val
    }
}
