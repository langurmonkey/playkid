use crate::eventhandler;

use colored::Colorize;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;

/// Manage the debug status and debug input events.
pub struct DebugManager {
    debugging: bool,
    step_instruction: bool,
    step_line: bool,
    breakpoints: Vec<u16>,
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
                self.request_step_instruction();
                true
            }
            // Step a scan line.
            Event::KeyDown {
                keycode: Some(Keycode::F8),
                ..
            } => {
                self.request_step_scanline();
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
            breakpoints: Vec::new(),
        }
    }

    pub fn set_debugging(&mut self, d: bool) {
        self.debugging = d;
    }

    pub fn debugging(&self) -> bool {
        self.debugging
    }

    pub fn get_breakpoints(&self) -> Vec<u16> {
        self.breakpoints.clone()
    }

    pub fn has_breakpoint(&self, addr: u16) -> bool {
        self.breakpoints.contains(&addr)
    }

    pub fn add_breakpoint(&mut self, addr: u16) {
        println!("{}: Add breakpoint: {:#04x}", "OK".green(), addr);
        self.breakpoints.push(addr);
    }

    pub fn delete_breakpoint(&mut self, addr: u16) {
        println!("{}: Add removed: {:#04x}", "OK".green(), addr);
        self.breakpoints.retain(|&x| x != addr);
    }

    pub fn clear_breakpoints(&mut self) {
        println!("{}: Breakpoints cleared", "OK".green());
        self.breakpoints.clear();
    }

    pub fn request_step_instruction(&mut self) {
        if self.debugging {
            self.step_instruction = true;
        }
    }

    pub fn request_step_scanline(&mut self) {
        if self.debugging {
            self.step_line = true;
        }
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
