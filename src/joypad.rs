use crate::eventhandler;
use gilrs::{Button, EventType};

/// # Joypad
/// This class manages the state of the Joypad of the Game Boy.
/// Contains the JOYP registers and the state variables for
/// every button.
pub struct Joypad {
    /// The P1/JOYP register.
    joyp: u8,
    /// Bit 5 - The state of Start/Select/A/B is in bits 0-3.
    select_buttons: bool,
    /// Bit 4 - The state of the D-Pad is in bits 0-3.
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
    fn handle_event(&mut self, i: &egui::InputState) -> bool {
        if i.key_pressed(egui::Key::ArrowDown) {
            self.down = true;
            return true;
        }
        if i.key_released(egui::Key::ArrowDown) {
            self.down = false;
            return true;
        }
        if i.key_pressed(egui::Key::ArrowUp) {
            self.up = true;
            return true;
        }
        if i.key_released(egui::Key::ArrowUp) {
            self.up = false;
            return true;
        }
        if i.key_pressed(egui::Key::ArrowRight) {
            self.right = true;
            return true;
        }
        if i.key_released(egui::Key::ArrowRight) {
            self.right = false;
            return true;
        }
        if i.key_pressed(egui::Key::ArrowLeft) {
            self.left = true;
            return true;
        }
        if i.key_released(egui::Key::ArrowLeft) {
            self.left = false;
            return true;
        }
        if i.key_pressed(egui::Key::A) {
            self.a = true;
            return true;
        }
        if i.key_released(egui::Key::A) {
            self.a = false;
            return true;
        }
        if i.key_pressed(egui::Key::B) {
            self.b = true;
            return true;
        }
        if i.key_released(egui::Key::B) {
            self.b = false;
            return true;
        }
        if i.key_pressed(egui::Key::Enter) {
            self.start = true;
            return true;
        }
        if i.key_released(egui::Key::Enter) {
            self.start = false;
            return true;
        }
        if i.key_pressed(egui::Key::Space) {
            self.select = true;
            return true;
        }
        if i.key_released(egui::Key::Space) {
            self.select = false;
            return true;
        }
        false
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
        let old_joyp = self.joyp;
        // Start with bits 0-3 as 1 (unpressed).
        let mut current_nibble = 0x0F;

        // Selection bits are active LOW.
        let select_buttons = (self.joyp & 0x20) == 0;
        let select_dpad = (self.joyp & 0x10) == 0;

        if select_buttons {
            if self.a {
                current_nibble &= !0x01;
            }
            if self.b {
                current_nibble &= !0x02;
            }
            if self.select {
                current_nibble &= !0x04;
            }
            if self.start {
                current_nibble &= !0x08;
            }
        }

        if select_dpad {
            if self.right {
                current_nibble &= !0x01;
            }
            if self.left {
                current_nibble &= !0x02;
            }
            if self.up {
                current_nibble &= !0x04;
            }
            if self.down {
                current_nibble &= !0x08;
            }
        }

        // Combine selection bits and new button nibble.
        self.joyp = (self.joyp & 0xF0) | current_nibble;

        // Interrupt triggers when any bit in lower nibble transitions from 1 to 0.
        // This happens if (old_bits & !new_bits) is non-zero.
        if (old_joyp & 0x0F) & !(self.joyp & 0x0F) != 0 {
            self.request_interrupt = true;
        }
    }

    /// Main game controller handler.
    pub fn handle_controller_input(&mut self, event: EventType) {
        match event {
            EventType::ButtonPressed(button, _) => self.update_button(button, true),
            EventType::ButtonReleased(button, _) => self.update_button(button, false),
            EventType::AxisChanged(axis, value, _) => {
                if axis == gilrs::Axis::LeftStickX {
                    self.left = value < -0.5;
                    self.right = value > 0.5;
                }
                if axis == gilrs::Axis::LeftStickY {
                    self.up = value > 0.5;
                    self.down = value < -0.5;
                }
            }
            _ => (),
        }
    }

    /// Game controller button handling.
    fn update_button(&mut self, button: Button, pressed: bool) {
        println!("Button: {:?} {}", button, pressed);
        match button {
            Button::South | Button::East => self.a = pressed,
            Button::North | Button::West => self.b = pressed,
            Button::Start => self.start = pressed,
            Button::Select => self.select = pressed,
            Button::DPadUp => self.up = pressed,
            Button::DPadDown => self.down = pressed,
            Button::DPadLeft => self.left = pressed,
            Button::DPadRight => self.right = pressed,
            _ => (),
        }
    }
}
