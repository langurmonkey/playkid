use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};

/// A representation of a cartridge..
pub struct Cartridge {
    /// Holds the ROM data in an array of bytes.
    pub data: Vec<u8>,
}

const LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

impl Cartridge {
    pub fn new(rom: &str, check_logo: bool) -> Result<Self> {
        let mut file = File::open(rom)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check Nintendo logo in rom file.
        // In 0x104 - 0x133 (260 to 307), with contents in LOGO.
        if check_logo {
            //Cartridge::check_checksum(&buffer);
            let slice = &buffer[260..307];
            let mut i: usize = 0;
            for u in slice.iter().cloned() {
                if u != LOGO[i] {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Incorrect logo sequence!",
                    ));
                }
                i += 1;
            }
            println!("Correct logo sequence");
        }

        Ok(Self { data: buffer })
    }
}
