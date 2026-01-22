use crate::eventhandler;
use colored::Colorize;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

/// Describes the current state of the Game Boy joypad.
pub struct Joypad {
    /// The P1/JOYP register.
    joyp: u8,
    /// Bits 0-4 contain the state of SsAB.
    select_buttons: bool,
    /// Bits 0-4 contain the state of the D-Pad.
    select_dpad: bool,
    /// Start button.
    pub start: bool,
    /// Select button.
    pub select: bool,
    /// A button.
    pub a: bool,
    /// B button.
    pub b: bool,
    /// D-pad down.
    pub down: bool,
    /// D-pad up.
    pub up: bool,
    /// D-pad left.
    pub left: bool,
    /// D-pad right.
    pub right: bool,
    /// JOY interrupt requested.
    request_interrupt: bool,
    /// Interrupt mask.
    pub i_mask: u8,
    /// Cycle counter,
    cycles: usize,
}

impl eventhandler::EventHandler for Joypad {
    fn handle_event(&mut self, event: &WinitInputHelper) -> bool {
        if event.key_pressed(KeyCode::ArrowDown) {
            self.down = true;
            true
        } else if event.key_released(KeyCode::ArrowDown) {
            self.down = false;
            true
        } else if event.key_pressed(KeyCode::ArrowUp) {
            self.up = true;
            true
        } else if event.key_released(KeyCode::ArrowUp) {
            self.up = false;
            true
        } else if event.key_pressed(KeyCode::ArrowRight) {
            self.right = true;
            true
        } else if event.key_released(KeyCode::ArrowRight) {
            self.right = false;
            true
        } else if event.key_pressed(KeyCode::ArrowLeft) {
            self.left = true;
            true
        } else if event.key_released(KeyCode::ArrowLeft) {
            self.left = false;
            true
        } else if event.key_pressed(KeyCode::Enter) {
            self.start = true;
            true
        } else if event.key_released(KeyCode::Enter) {
            self.start = false;
            true
        } else if event.key_pressed(KeyCode::Space) {
            self.select = true;
            true
        } else if event.key_released(KeyCode::Space) {
            self.select = false;
            true
        } else if event.key_pressed(KeyCode::KeyA) {
            self.a = true;
            true
        } else if event.key_released(KeyCode::KeyA) {
            self.a = false;
            true
        } else if event.key_pressed(KeyCode::KeyB) {
            self.b = true;
            true
        } else if event.key_released(KeyCode::KeyB) {
            self.b = false;
            true
        } else {
            false
        }
    }
}

impl Joypad {
    pub fn new() -> Self {
        let joypad = Joypad {
            joyp: 0xFF,
            select_buttons: false,
            select_dpad: false,
            start: false,
            select: false,
            a: false,
            b: false,
            down: false,
            up: false,
            left: false,
            right: false,
            request_interrupt: false,
            i_mask: 0,
            cycles: 0,
        };

        // Try to open the first available game controller
        // joypad.init_controller();

        joypad
    }

    /// Resets the state of the joypad.
    pub fn reset(&mut self) {
        self.joyp = 0xFF;
        self.select_buttons = false;
        self.select_dpad = false;
        self.cycles = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            0xFF00 => self.joyp,
            _ => panic!("Invalid Joypad address."),
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0xFF00 => {
                // Only write top nibble!
                self.joyp = (self.joyp & 0x0F) | (value & 0xF0);
                self.update_state();
            }
            _ => panic!("Invalid Joypad address."),
        }
    }

    /// Implements a Joypad cycle.
    /// Assumes `handle_event()` has been called previously and the state
    /// of the joypad is up to date.
    pub fn cycle(&mut self) {
        // Update state and raise interrupt if necessary.
        self.update_state();

        if self.request_interrupt {
            self.i_mask = 0b0001_0000;
            self.request_interrupt = false;
        }
    }

    /// Updates the flags in bits 5 and 4 (select buttons, select D-pad) of JOYP.
    fn update_state(&mut self) {
        // Start with all buttons unpressed (1 is unpressed in GB logic)
        // Keep the selection bits (4-5) as set by the CPU
        let mut res = self.joyp | 0x0F;

        self.select_buttons = (self.joyp & 0x20) == 0;
        self.select_dpad = (self.joyp & 0x10) == 0;

        // Use separate IF blocks because BOTH can be selected at once
        if self.select_buttons {
            if self.a {
                res &= 0xFE;
                self.request_interrupt = true;
            }
            if self.b {
                res &= 0xFD;
                self.request_interrupt = true;
            }
            if self.select {
                res &= 0xFB;
                self.request_interrupt = true;
            }
            if self.start {
                res &= 0xF7;
                self.request_interrupt = true;
            }
        }

        if self.select_dpad {
            if self.right {
                res &= 0xFE;
                self.request_interrupt = true;
            }
            if self.left {
                res &= 0xFD;
                self.request_interrupt = true;
            }
            if self.up {
                res &= 0xFB;
                self.request_interrupt = true;
            }
            if self.down {
                res &= 0xF7;
                self.request_interrupt = true;
            }
        }

        self.joyp = res;
    }
}
