use crate::eventhandler;

use colored::Colorize;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

/// Manage the debug status and debug input events.
pub struct DebugManager {
    /// The debug UI is visible.
    debugging: bool,
    /// The CPU is waiting for steps.
    paused: bool,
    /// Step request.
    step_instruction: bool,
    /// Step scanline request.
    step_line: bool,
    /// Breakpoints list.
    breakpoints: Vec<u16>,
}

impl eventhandler::EventHandler for DebugManager {
    /// Process keyboard inputs specifically for debugging.
    /// Returns true if the event was handled.
    fn handle_event(&mut self, event: &WinitInputHelper) -> bool {
        if event.key_pressed(KeyCode::F6) {
            self.request_step_instruction();
            true
        } else if event.key_released(KeyCode::F7) {
            self.request_step_scanline();
            true
        } else if event.key_released(KeyCode::F9) {
            self.toggle_paused();
            true
        } else {
            false
        }
    }
}

impl DebugManager {
    pub fn new(active: bool) -> Self {
        Self {
            debugging: active,
            paused: active,
            step_instruction: false,
            step_line: false,
            breakpoints: Vec::new(),
        }
    }

    pub fn toggle_debugging(&mut self) -> bool {
        self.debugging = !self.debugging;
        self.debugging
    }

    pub fn is_debugging(&self) -> bool {
        self.debugging
    }

    pub fn set_paused(&mut self, p: bool) {
        self.paused = p;
    }

    pub fn toggle_paused(&mut self) {
        self.paused = !self.paused;
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn get_breakpoints_vec(&self) -> &Vec<u16> {
        &self.breakpoints
    }

    pub fn has_breakpoint(&self, addr: u16) -> bool {
        self.breakpoints.contains(&addr)
    }

    pub fn add_breakpoint(&mut self, addr: u16) {
        if !self.has_breakpoint(addr) {
            println!("{}: Add breakpoint: {:#04x}", "OK".green(), addr);
            self.breakpoints.push(addr);
        }
    }
    pub fn delete_breakpoint(&mut self, addr: u16) {
        if self.has_breakpoint(addr) {
            println!("{}: Remove breakpoint: {:#04x}", "OK".green(), addr);
            self.breakpoints.retain(|&x| x != addr);
        }
    }

    pub fn toggle_breakpoint(&mut self, addr: u16) {
        if self.has_breakpoint(addr) {
            self.delete_breakpoint(addr);
        } else {
            self.add_breakpoint(addr);
        }
    }

    pub fn clear_breakpoints(&mut self) {
        if !self.breakpoints.is_empty() {
            println!("{}: Clear breakpoints", "OK".green());
            self.breakpoints.clear();
        }
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
