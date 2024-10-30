use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::Sdl;
use std::process;

/// Describes the current state of the Game Boy joypad.
pub struct Joypad<'b> {
    pub joyp: u8,
    pub select_buttons: bool,
    pub select_dpad: bool,
    pub start: bool,
    pub select: bool,
    pub a: bool,
    pub b: bool,
    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
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
            sdl,
        }
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
                self.update_flags();
            }
            _ => panic!("Invalid Joypad address."),
        }
    }

    pub fn update(&mut self) {
        self.update_flags();
        let mut event_pump = self.sdl.event_pump().unwrap();
        // Event loop
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                }
                | Event::KeyDown {
                    keycode: Some(Keycode::CapsLock),
                    ..
                } => process::exit(0),

                // DOWN
                Event::KeyDown {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // Set down.
                    if self.select_dpad {
                        self.joyp = self.joyp & 0xF7;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Down),
                    ..
                } => {
                    // Unset down.
                    if self.select_dpad {
                        self.joyp = self.joyp | 0x08;
                    }
                }

                // UP
                Event::KeyDown {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // Set Up.
                    if self.select_dpad {
                        self.joyp = self.joyp & 0xFB;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Up),
                    ..
                } => {
                    // Unset Up.
                    if self.select_dpad {
                        self.joyp = self.joyp | 0x04;
                    }
                }

                // LEFT
                Event::KeyDown {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Set left.
                    if self.select_dpad {
                        self.joyp = self.joyp & 0xFD;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Left),
                    ..
                } => {
                    // Unset left.
                    if self.select_dpad {
                        self.joyp = self.joyp | 0x02;
                    }
                }

                // RIGHT
                Event::KeyDown {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Set right.
                    if self.select_dpad {
                        self.joyp = self.joyp & 0xFE;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Right),
                    ..
                } => {
                    // Unset right.
                    if self.select_dpad {
                        self.joyp = self.joyp | 0x01;
                    }
                }

                // START (enter)
                Event::KeyDown {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    // Set Start.
                    if self.select_buttons {
                        self.joyp = self.joyp & 0xF7;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Return),
                    ..
                } => {
                    // Unset Start.
                    if self.select_buttons {
                        self.joyp = self.joyp | 0x08;
                    }
                }

                // SELECT (space)
                Event::KeyDown {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // Set Select.
                    if self.select_buttons {
                        self.joyp = self.joyp & 0xFB;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::Space),
                    ..
                } => {
                    // Unset Select.
                    if self.select_buttons {
                        self.joyp = self.joyp | 0x04;
                    }
                }

                // B
                Event::KeyDown {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    // Set B.
                    if self.select_buttons {
                        self.joyp = self.joyp & 0xFD;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::B),
                    ..
                } => {
                    // Unset left.
                    if self.select_buttons {
                        self.joyp = self.joyp | 0x02;
                    }
                }

                // A
                Event::KeyDown {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    // Set A.
                    if self.select_buttons {
                        self.joyp = self.joyp & 0xFE;
                    }
                }
                Event::KeyUp {
                    keycode: Some(Keycode::A),
                    ..
                } => {
                    // Unset A.
                    if self.select_buttons {
                        self.joyp = self.joyp | 0x01;
                    }
                }

                _ => {}
            }
        }
        self.update_buttons();
    }

    /// Updates the flags in bits 5 and 4 (select buttons, select D-pad) of JOYP.
    fn update_flags(&mut self) {
        // If bit 5 is zero, SsBA are in the lower nibble.
        self.select_buttons = (self.joyp & 0x20) == 0;
        // If bit 4 is zero, d-pad buttons are in the lower nibble.
        self.select_dpad = (self.joyp & 0x10) == 0;
    }

    /// Updates the state of the joypad buttons from the JOYP register.
    /// WARN: A button is pressed when the corresponding bit is 0!
    fn update_buttons(&mut self) {
        if self.select_buttons {
            self.start = (self.joyp & 0x08) == 0;
            self.select = (self.joyp & 0x04) == 0;
            self.b = (self.joyp & 0x02) == 0;
            self.a = (self.joyp & 0x01) == 0;
        }
        if self.select_dpad {
            self.down = (self.joyp & 0x08) == 0;
            self.up = (self.joyp & 0x04) == 0;
            self.left = (self.joyp & 0x02) == 0;
            self.right = (self.joyp & 0x01) == 0;
        }
    }
}
