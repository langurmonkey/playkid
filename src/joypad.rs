use crate::constants;

use crossterm::{execute, terminal::LeaveAlternateScreen};
use sdl2::controller::{Axis, Button, GameController};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{EventPump, Sdl};
use std::io;
use std::process;

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
    /// Debug flag. If this is on, a debug pause is requested.
    pub debug_flag: bool,
    /// Cycle counter,
    cycles: usize,
    /// The event pump.
    event_pump: EventPump,
    // Keep controller subsystem alive to handle hotplug events.
    controller_subsystem: sdl2::GameControllerSubsystem,
    /// Connected game controller.
    controller: Option<GameController>,
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
            debug_flag: false,
            cycles: 0,
            event_pump: sdl.event_pump().unwrap(),
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

        println!("Found {} joystick(s)", available);

        // Try to open the first controller
        for id in 0..available {
            if self.controller_subsystem.is_game_controller(id) {
                match self.controller_subsystem.open(id) {
                    Ok(c) => {
                        println!("Controller {} opened: {}", id, c.name());
                        self.controller = Some(c);
                        break;
                    }
                    Err(e) => {
                        println!("Failed to open controller {}: {}", id, e);
                    }
                }
            }
        }

        if self.controller.is_none() {
            println!("No game controllers found, using keyboard only");
        }
    }

    /// Resets the state of the joypad.
    pub fn reset(&mut self) {
        self.joyp = 0xFF;
        self.select_buttons = false;
        self.select_dpad = false;
        self.debug_flag = false;
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

    pub fn cycle(&mut self) {
        // Poll events twice per frame.
        // Polling once per frame results in too much latency.
        self.cycles += 1;
        if self.cycles * 2 < constants::CYCLES_PER_FRAME {
            return;
        }
        // Reset cycles.
        self.cycles = 0;

        // Poll events.
        for event in self.event_pump.poll_iter() {
            match event {
                // Quit.
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::CapsLock),
                    ..
                } => {
                    match execute!(io::stdout(), LeaveAlternateScreen) {
                        Err(error) => println!("{:?}", error),
                        _ => {}
                    }
                    println!("Bye bye!");
                    process::exit(0);
                }

                // Debug pause ('s' for stop).
                Event::KeyDown {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.debug_flag = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    self.debug_flag = false;
                }

                // Keyboard inputs
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => self.down = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => self.down = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => self.up = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => self.up = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => self.left = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => self.left = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => self.right = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => self.right = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => self.start = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => self.start = false,

                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => self.select = true,
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => self.select = false,

                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => self.b = true,
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => self.b = false,

                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => self.a = true,
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => self.a = false,

                // Controller button events
                Event::ControllerButtonDown { button, .. } => match button {
                    Button::A | Button::B => self.a = true,
                    Button::X | Button::Y => self.b = true,
                    Button::Start => self.start = true,
                    Button::Back => self.select = true,
                    Button::DPadUp => self.up = true,
                    Button::DPadDown => self.down = true,
                    Button::DPadLeft => self.left = true,
                    Button::DPadRight => self.right = true,
                    _ => {}
                },
                Event::ControllerButtonUp { button, .. } => match button {
                    Button::A | Button::B => self.a = false,
                    Button::X | Button::Y => self.b = false,
                    Button::Start => self.start = false,
                    Button::Back => self.select = false,
                    Button::DPadUp => self.up = false,
                    Button::DPadDown => self.down = false,
                    Button::DPadLeft => self.left = false,
                    Button::DPadRight => self.right = false,
                    _ => {}
                },

                // Controller D-pad via axis (for controllers that use axes for D-pad)
                Event::ControllerAxisMotion { axis, value, .. } => {
                    const DEADZONE: i16 = 10000;
                    match axis {
                        Axis::LeftX => {
                            if value < -DEADZONE {
                                self.left = true;
                                self.right = false;
                            } else if value > DEADZONE {
                                self.right = true;
                                self.left = false;
                            } else {
                                self.left = false;
                                self.right = false;
                            }
                        }
                        Axis::LeftY => {
                            if value < -DEADZONE {
                                self.up = true;
                                self.down = false;
                            } else if value > DEADZONE {
                                self.down = true;
                                self.up = false;
                            } else {
                                self.up = false;
                                self.down = false;
                            }
                        }
                        _ => {}
                    }
                }

                // Controller connected.
                Event::ControllerDeviceAdded { which, .. } => {
                    println!("Connected!");
                    if self.controller.is_none() {
                        // For 'Added' events, 'which' is the device index.
                        if self.controller_subsystem.is_game_controller(which) {
                            match self.controller_subsystem.open(which) {
                                Ok(c) => {
                                    println!("Controller connected: {}", c.name());
                                    self.controller = Some(c);
                                }
                                Err(e) => println!("Failed to open controller: {}", e),
                            }
                        }
                    }
                }

                // Controller disconnected.
                Event::ControllerDeviceRemoved { which, .. } => {
                    println!("Removed!");
                    // For 'Removed' events, 'which' is the Instance ID.
                    let is_current = self
                        .controller
                        .as_ref()
                        .map_or(false, |c| c.instance_id() == which);

                    if is_current {
                        println!(
                            "Controller disconnected: {}",
                            self.controller.as_ref().unwrap().name()
                        );
                        self.controller = None;
                    }
                }

                _ => {}
            }
        }

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
