use colored::Colorize;
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

/// Game Boy logo sequence.
const LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

impl Cartridge {
    pub fn new(rom: &str, skip_checksum: bool) -> Result<Self> {
        let mut file = File::open(rom)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Check Nintendo logo in ROM file.
        // In 0x104 - 0x133, with contents in LOGO.
        if !skip_checksum {
            //Cartridge::check_checksum(&buffer);
            let slice = &data[0x104..0x133];
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
            println!("{}: {}", "OK".green(), "Logo sequence");
        }

        // Get title.
        {
            let slice = &data[0x134..0x142];
            let title = str::from_utf8(slice).expect("Error getting ROM title");
            println!("Title: {}", title.bright_blue());
        }

        // Color or not color.
        {
            if data[0x143] == 0x80 {
                // Color GB.
                println!(" -> {}", "GB Color cartridge");
            } else {
                // Not color GB.
                println!(" -> {}", "Regular GB cartridge");
            }
        }

        // Super Game Boy.
        {
            let sgbf = data[0x146];
            if sgbf == 0x03 {
                println!(" -> {}", "Super Game Boy functions supported");
            }
        }

        // Cartridge type.
        let cart_type;
        {
            let t = data[0x147];
            let ct = Cartridge::cart_type_str(t);
            println!(" -> Cartridge type: {} ({})", ct.yellow(), t);

            cart_type = t;
        }

        // ROM size.
        {
            let rs: u8 = data[0x148];
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
                _ => &format!("Unknown ({:#04x})", rs),
            };
            println!(" -> ROM size: {}", size);
        }

        // RAM size.
        {
            let rs: u8 = data[0x149];
            let size = match rs {
                0 => "No RAM",
                1 => "Error, unused value!",
                2 => "8 KiB (1 bank)",
                3 => "32 KiB (4 banks of 8 KiB each)",
                4 => "128 KiB (16 banks of 8 KiB each)",
                5 => "64 KiB (8 banks of 8 KiB each)",
                _ => &format!("Unknown ({:#04x})", rs),
            };
            println!(" -> RAM size: {}", size);
        }

        // Destination code.
        {
            let dc: u8 = data[0x14A];
            match dc {
                0 => println!(" -> Destination code: Japan"),
                1 => println!(" -> Destination code: Overseas only"),
                _ => println!(" -> Destination code: Unknown ({:#04x})", dc),
            }
        }
        // Header checksum.
        if !skip_checksum {
            let hc: u8 = data[0x14D];
            let mut cs: i32 = 0;
            for address in 0x134..0x14D {
                cs = cs - data[address as usize] as i32 - 1
            }
            if hc != cs as u8 {
                panic!(
                    "Header checksum incorrect: [mem]{:#04x} != [cs]{:#04x}",
                    hc, cs as u8
                );
            } else {
                println!("{}: {}: {:#04x}", "OK".green(), "Header checksum", cs as u8);
            }
        }
        // Global checksum.
        if !skip_checksum {
            let gc: u16 = ((data[0x14E] as u16) << 8) | (data[0x14F] as u16);
            let mut cs: u32 = 0;
            for address in 0..data.len() {
                if address != 0x14E && address != 0x14F {
                    cs += data[address as usize] as u32;
                }
            }
            if gc != cs as u16 {
                panic!(
                    "{}: Global checksum incorrect: [mem]{:#06x} != [cs]{:#06X}",
                    "KO".red(),
                    gc,
                    cs as u16
                );
            } else {
                println!("{}: {}: {:#04x}", "OK".green(), "Global checksum", cs as u8);
            }
        }

        // Check supported modes.
        if cart_type > 0 {
            panic!(
                "Only ROM ONLY cartridges supported (current is {})",
                Cartridge::cart_type_str(cart_type)
            );
        }

        Ok(Self { data, cart_type })
    }

    pub fn cart_type_str(cart_type: u8) -> String {
        match cart_type {
            0x00 => "ROM ONLY".to_string(),
            0x01 => "MBC1".to_string(),
            0x02 => "MBC1+RAM".to_string(),
            0x03 => "MBC1+RAM+BATTERY".to_string(),
            0x05 => "MBC2".to_string(),
            0x06 => "MBC2+BATTERY".to_string(),
            0x08 => "ROM+RAM".to_string(),
            0x09 => "ROM+RAM+BATTERY".to_string(),
            0x0B => "MMM01".to_string(),
            0x0C => "MMM01+RAM".to_string(),
            0x0D => "MMM01+RAM+BATTERY".to_string(),
            0x0F => "MBC3+TIMER+BATTERY".to_string(),
            0x10 => "MBC3+TIMER+RAM+BATTERY".to_string(),
            0x11 => "MBC3".to_string(),
            0x12 => "MBC3+RAM".to_string(),
            0x13 => "MBC3+RAM+BATTERY".to_string(),
            0x19 => "MBC5".to_string(),
            0x1A => "MBC5+RAM".to_string(),
            0x1B => "MBC5+RAM+BATTERY".to_string(),
            0x1C => "MBC5+RUMBLE".to_string(),
            0x1D => "MBC5+RUMBLE+RAM".to_string(),
            0x1E => "MBC5+RUMBLE+RAM+BATTERY".to_string(),
            0x20 => "MBC6".to_string(),
            0x22 => "MBC7+SENSOR+RUMBLE+RAM+BATTERY".to_string(),
            0xFC => "POCKET CAMERA".to_string(),
            0xFD => "BANDAI TAMA5".to_string(),
            0xFE => "HuC3".to_string(),
            0xFF => "MuC1+RAM+BATTERY".to_string(),
            _ => format!("Unknown ({:#40x})", cart_type),
        }
    }
}
