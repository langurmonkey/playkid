use crate::eventhandler;

use colored::Colorize;
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;

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
    // Keep controller subsystem alive to handle hot plug events.
    controller_subsystem: sdl2::GameControllerSubsystem,
    /// Connected game controller.
    controller: Option<GameController>,
}

impl eventhandler::EventHandler for Joypad {
    fn handle_event(&mut self, event: &Event) -> bool {
        match event {
            // Keyboard inputs.
            Event::KeyDown {
                keycode: Some(Keycode::Down),
                ..
            } => {
                self.down = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Down),
                ..
            } => {
                self.down = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::Up),
                ..
            } => {
                self.up = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Up),
                ..
            } => {
                self.up = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::Left),
                ..
            } => {
                self.left = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Left),
                ..
            } => {
                self.left = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::Right),
                ..
            } => {
                self.right = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Right),
                ..
            } => {
                self.right = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::Return),
                ..
            } => {
                self.start = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Return),
                ..
            } => {
                self.start = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::Space),
                ..
            } => {
                self.select = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::Space),
                ..
            } => {
                self.select = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::B),
                ..
            } => {
                self.b = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::B),
                ..
            } => {
                self.b = false;
                true
            }

            Event::KeyDown {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.a = true;
                true
            }
            Event::KeyUp {
                keycode: Some(Keycode::A),
                ..
            } => {
                self.a = false;
                true
            }

            // Controller button events.
            Event::ControllerButtonDown { button, .. } => match button {
                Button::A | Button::B => {
                    self.a = true;
                    true
                }
                Button::X | Button::Y => {
                    self.b = true;
                    true
                }
                Button::Start => {
                    self.start = true;
                    true
                }
                Button::Back => {
                    self.select = true;
                    true
                }
                Button::DPadUp => {
                    self.up = true;
                    true
                }
                Button::DPadDown => {
                    self.down = true;
                    true
                }
                Button::DPadLeft => {
                    self.left = true;
                    true
                }
                Button::DPadRight => {
                    self.right = true;
                    true
                }
                _ => false,
            },
            Event::ControllerButtonUp { button, .. } => match button {
                Button::A | Button::B => {
                    self.a = false;
                    true
                }
                Button::X | Button::Y => {
                    self.b = false;
                    true
                }
                Button::Start => {
                    self.start = false;
                    true
                }
                Button::Back => {
                    self.select = false;
                    true
                }
                Button::DPadUp => {
                    self.up = false;
                    true
                }
                Button::DPadDown => {
                    self.down = false;
                    true
                }
                Button::DPadLeft => {
                    self.left = false;
                    true
                }
                Button::DPadRight => {
                    self.right = false;
                    true
                }
                _ => false,
            },

            // Controller D-pad via axis (for controllers that use axes for D-pad).
            Event::ControllerAxisMotion { axis, value, .. } => {
                const DEADZONE: i16 = 10000;
                match axis {
                    Axis::LeftX => {
                        if value < &-DEADZONE {
                            self.left = true;
                            self.right = false;
                        } else if value > &DEADZONE {
                            self.right = true;
                            self.left = false;
                        } else {
                            self.left = false;
                            self.right = false;
                        }
                        true
                    }
                    Axis::LeftY => {
                        if value < &-DEADZONE {
                            self.up = true;
                            self.down = false;
                        } else if value > &DEADZONE {
                            self.down = true;
                            self.up = false;
                        } else {
                            self.up = false;
                            self.down = false;
                        }
                        true
                    }
                    _ => false,
                }
            }

            // Controller connected.
            Event::ControllerDeviceAdded { which, .. } => {
                if self.controller.is_none() {
                    // For 'Added' events, 'which' is the device index.
                    if self.controller_subsystem.is_game_controller(*which) {
                        match self.controller_subsystem.open(*which) {
                            Ok(c) => {
                                println!("{}: Controller connected: {}", "OK".green(), c.name());
                                self.controller = Some(c);
                            }
                            Err(e) => {
                                println!("{}: Failed to open controller: {}", "ERR".red(), e)
                            }
                        }
                        return true;
                    }
                }
                false
            }

            // Controller disconnected.
            Event::ControllerDeviceRemoved { which, .. } => {
                // For 'Removed' events, 'which' is the Instance ID.
                let is_current = self
                    .controller
                    .as_ref()
                    .map_or(false, |c| c.instance_id() == *which);

                if is_current {
                    println!(
                        "{}: Controller disconnected: {}",
                        "OK".green(),
                        self.controller.as_ref().unwrap().name()
                    );
                    self.controller = None;
                    return true;
                }
                false
            }

            _ => false,
        }
    }
}

impl Joypad {
    pub fn new(sdl: &Sdl) -> Self {
        let mut joypad = Joypad {
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
            controller_subsystem: sdl.game_controller().unwrap(),
            controller: None,
        };

        // Try to open the first available game controller
        joypad.init_controller();

        joypad
    }

    /// Initialize the first available game controller.
    fn init_controller(&mut self) {
        // Enable background events for controller connection/disconnection.
        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

        let available = self
            .controller_subsystem
            .num_joysticks()
            .map_err(|e| format!("can't enumerate joysticks: {}", e))
            .unwrap();

        println!("{}: Found {} joystick(s)", "OK".green(), available);

        // Try to open the first controller
        for id in 0..available {
            if self.controller_subsystem.is_game_controller(id) {
                match self.controller_subsystem.open(id) {
                    Ok(c) => {
                        println!(
                            "{}: Controller {} opened: {}",
                            "OK".green(),
                            id,
                            c.name().purple()
                        );
                        self.controller = Some(c);
                        break;
                    }
                    Err(e) => {
                        println!("{}: Failed to open controller {}: {}", "ERR".red(), id, e);
                    }
                }
            }
        }

        if self.controller.is_none() {
            println!(
                "{}: No game controllers found, using keyboard only",
                "OK".green()
            );
        }
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
    pub fn cycle(&mut self) -> bool {
        // Update state and raise interrupt if necessary.
        self.update_state();

        if self.request_interrupt {
            self.i_mask = 0b0001_0000;
            self.request_interrupt = false;
        }
        true
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
