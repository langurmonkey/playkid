use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::str;

/// A representation of a cartridge..
pub struct Cartridge {
    /// Holds the ROM data in an array of bytes.
    pub data: Vec<u8>,
    /// Cartridge type byte.
    pub cart_type: u8,
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
        // In 0x104 - 0x133, with contents in LOGO.
        if check_logo {
            //Cartridge::check_checksum(&buffer);
            let slice = &buffer[0x104..0x133];
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

        // Get title.
        {
            let slice = &buffer[0x134..0x142];
            let title = str::from_utf8(slice).expect("Error getting ROM title");
            println!("Title: {}", title);
        }

        // Color or not color.
        {
            if buffer[0x143] == 0x80 {
                // Color GB.
                println!(" -> GB Color cartridge");
            } else {
                // Not color GB.
                println!(" -> Regular GB cartridge");
            }
        }

        // Cartridge type.
        let mut cart_type;
        {
            let t = buffer[0x147];
            println!("Cartridge type: {}", t);

            cart_type = t;
        }

        // ROM size.
        {
            let rs: u8 = buffer[0x148];
            let size = match rs {
                0 => "256 kb",
                1 => "512 Kb",
                2 => "1 Mb",
                3 => "2 Mb",
                4 => "4 Mb",
                5 => "8 Mb",
                6 => "16 Mb",
                0x52 => "9 Mb",
                0x53 => "10 Mb",
                0x54 => "12 Mb",
                _ => "Unknown",
            };
            println!("ROM size: {}", size);
        }

        // RAM size.
        {
            let rs: u8 = buffer[0x149];
            let size = match rs {
                0 => "No RAM",
                1 => "16 kb",
                2 => "64 kb",
                3 => "256 kb",
                4 => "1 Mb",
                _ => "Unknown",
            };
            println!("RAM size: {}", size);
        }

        // Check supported modes.
        if cart_type > 0 {
            panic!("Only ROM ONLY cartridges supported (0)");
        }

        Ok(Self {
            data: buffer,
            cart_type,
        })
    }
}
