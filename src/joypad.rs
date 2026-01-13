use crate::constants;

use crossterm::{execute, terminal::LeaveAlternateScreen};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::{EventPump, Sdl};
use std::io;
use std::process;

/// Describes the current state of the Game Boy joypad.
pub struct Joypad<'b> {
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
    /// Reference to the main SDL object.
    sdl: &'b Sdl,
}

impl<'b> Joypad<'b> {
    pub fn new(sdl: &'b Sdl) -> Self {
        Joypad {
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
            sdl,
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
        // Reset cycles.
        self.cycles = 0;

        // Poll.
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
                    // Raise deubg flag.
                    self.debug_flag = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::S),
                    ..
                } => {
                    // Lower debug flag..
                    self.debug_flag = false;
                }

                // DOWN
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // Set down.
                    self.down = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // Unset down.
                    self.down = false;
                }

                // UP
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // Set Up.
                    self.up = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // Unset Up.
                    self.up = false;
                }

                // LEFT
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Set left.
                    self.left = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Unset left.
                    self.left = false;
                }

                // RIGHT
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Set right.
                    self.right = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Unset right.
                    self.right = false;
                }

                // START (enter)
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    // Set Start.
                    self.start = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    // Unset Start.
                    self.start = false;
                }

                // SELECT (space)
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // Set Select.
                    self.select = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // Unset Select.
                    self.select = false;
                }

                // B
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    // Set B.
                    self.b = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    // Unset B.
                    self.b = false;
                }

                // A
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    // Set A.
                    self.a = true;
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    // Unset A.
                    self.a = false;
                }

                _ => {}
            }
        }

        // Raise interrupt if necessary.
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
            // Fixed the variable name here
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
