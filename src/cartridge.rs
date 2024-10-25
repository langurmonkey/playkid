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
    pub fn new(rom: &str, skip_checksum: bool) -> Result<Self> {
        let mut file = File::open(rom)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check Nintendo logo in rom file.
        // In 0x104 - 0x133, with contents in LOGO.
        if !skip_checksum {
            //Cartridge::check_checksum(&buffer);
            let slice = &buffer[0x104..0x133];
            let mut i: usize = 0;
            for u in slice.iter().cloned() {
                if u != LOGO[i] {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "Incorrect Nintendo logo sequence in 0x104-0x133!",
                    ));
                }
                i += 1;
            }
            println!("OK: Logo sequence");
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

        // Super Game Boy.
        {
            let sgbf = buffer[0x146];
            if sgbf == 0x03 {
                println!(" -> Super Game Boy functions supported");
            }
        }

        // Cartridge type.
        let cart_type;
        {
            let t = buffer[0x147];
            println!(" -> Cartridge type: {}", t);

            cart_type = t;
        }

        // ROM size.
        {
            let rs: u8 = buffer[0x148];
            let size = match rs {
                0 => "32 KiB (2 banks)",
                1 => "64 KiB (4 banks)",
                2 => "128 KiB (8 banks)",
                3 => "256 KiB",
                4 => "512 KiB",
                5 => "1 MiB",
                6 => "2 MiB",
                7 => "4 MiB",
                8 => "8 MiB",
                0x52 => "1.1 MiB",
                0x53 => "1.2 MiB",
                0x54 => "1.5 MiB",
                _ => &format!("Unknown ({:#04X})", rs),
            };
            println!(" -> ROM size: {}", size);
        }

        // RAM size.
        {
            let rs: u8 = buffer[0x149];
            let size = match rs {
                0 => "No RAM",
                1 => "Error, unused value!",
                2 => "8 KiB (1 bank)",
                3 => "32 KiB (4 banks of 8 KiB each)",
                4 => "128 KiB (16 banks of 8 KiB each)",
                5 => "64 KiB (8 banks of 8 KiB each)",
                _ => &format!("Unknown ({:#04X})", rs),
            };
            println!(" -> RAM size: {}", size);
        }

        // Destination code.
        {
            let dc: u8 = buffer[0x14A];
            match dc {
                0 => println!(" -> Destination code: Japan"),
                1 => println!(" -> Destination code: Overseas only"),
                _ => println!(" -> Desination code: Unknown ({:#04X})", dc),
            }
        }
        // Header checksum.
        if !skip_checksum {
            let hc: u8 = buffer[0x14D];
            let mut cs: i32 = 0;
            for address in 0x134..0x14D {
                cs = cs - buffer[address as usize] as i32 - 1
            }
            if hc != cs as u8 {
                panic!(
                    "Header checksum incorrect: [mem]{:#04X} != [cs]{:#04X}",
                    hc, cs as u8
                );
            } else {
                println!("OK: Header checksum: {:#04X}", cs as u8);
            }
        }
        // Global checksum.
        if !skip_checksum {
            let gc: u16 = ((buffer[0x14E] as u16) << 8) | (buffer[0x14F] as u16);
            let mut cs: u32 = 0;
            for address in 0..buffer.len() {
                if address != 0x14E && address != 0x14F {
                    cs += buffer[address as usize] as u32;
                }
            }
            if gc != cs as u16 {
                panic!(
                    "Global checksum incorrect: [mem]{:#06X} != [cs]{:#06X}",
                    gc, cs as u16
                );
            } else {
                println!("OK: Global checksum: {:#06X}", cs as u16);
            }
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
