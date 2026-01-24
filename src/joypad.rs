use crate::eventhandler;
use colored::Colorize;
use gilrs::{Button, Event, EventType, Gilrs};
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

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
    /// Game controller library.
    gilrs: Gilrs,
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
            gilrs: Gilrs::new().unwrap(),
        };

        // Print out detected gamepads.
        let gamepads = joypad.gilrs.gamepads();
        gamepads.for_each(move |(gid, g)| {
            println!(
                "{}: Gamepad {} detected: {} ",
                "OK".green(),
                gid,
                g.name().to_string().yellow()
            )
        });

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
    pub fn handle_controller_input(&mut self) {
        // Examine all events from the controller
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    let gamepad = self.gilrs.gamepad(id);
                    println!(
                        "{}: Gamepad connected: {} (VID: {:04x} PID: {:04x})",
                        "OK".green(),
                        gamepad.name().yellow(),
                        gamepad.vendor_id().unwrap_or(0),
                        gamepad.product_id().unwrap_or(0)
                    );
                }
                EventType::Disconnected => {
                    let gamepad = self.gilrs.gamepad(id);
                    println!(
                        "{}: Gamepad disconnected: {} (VID: {:04x} PID: {:04x})",
                        "WARN".yellow(),
                        gamepad.name().yellow(),
                        gamepad.vendor_id().unwrap_or(0),
                        gamepad.product_id().unwrap_or(0)
                    );
                }
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
    }

    /// Game controller button handling.
    fn update_button(&mut self, button: Button, pressed: bool) {
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
