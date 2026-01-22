mod mbc1;
mod mbc2;
mod mbc3;

use crate::eventhandler;
use colored::Colorize;
use mbc1::MBC1;
use mbc2::MBC2;
use mbc3::MBC3;
use std::fs::File;
use std::io::prelude::*;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::str;
use winit::keyboard::KeyCode;
use winit_input_helper::WinitInputHelper;

pub enum CartridgeType {
    RomOnly,
    MBC1(Box<MBC1>),
    MBC2(Box<MBC2>),
    MBC3(Box<MBC3>),
}

/// A representation of a Game Boy cartridge.
/// Checks for logo, header checksum, and detects Memory Bank Controller (MBC) type.
/// MBC1/2/3 implemented in dedicated files.
pub struct Cartridge {
    rom: String,
    pub cart_type: CartridgeType,
    /// Holds the ROM data in an array of bytes.
    data: Vec<u8>,
    /// Flag to keep track of dirty (unsaved) RAM.
    dirty: bool,
}

/// Game Boy logo sequence.
const LOGO: [u8; 48] = [
    0xCE, 0xED, 0x66, 0x66, 0xCC, 0x0D, 0x00, 0x0B, 0x03, 0x73, 0x00, 0x83, 0x00, 0x0C, 0x00, 0x0D,
    0x00, 0x08, 0x11, 0x1F, 0x88, 0x89, 0x00, 0x0E, 0xDC, 0xCC, 0x6E, 0xE6, 0xDD, 0xDD, 0xD9, 0x99,
    0xBB, 0xBB, 0x67, 0x63, 0x6E, 0x0E, 0xEC, 0xCC, 0xDD, 0xDC, 0x99, 0x9F, 0xBB, 0xB9, 0x33, 0x3E,
];

impl eventhandler::EventHandler for Cartridge {
    fn handle_event(&mut self, event: WinitInputHelper) -> bool {
        // Write SRAM file on `w`.
        if event.key_released(KeyCode::KeyW) {
            self.save_sram();
            true
        } else {
            false
        }
    }
}

impl Cartridge {
    pub fn new(rom: &str, skip_checksum: bool) -> Result<Self> {
        let mut file = File::open(rom)?;
        let mut data = Vec::new();
        file.read_to_end(&mut data)?;

        // Check Nintendo logo in ROM file.
        // In 0x104 - 0x133, with contents in LOGO.
        if !skip_checksum {
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
        let slice = &data[0x134..0x142];
        let title =
            str::from_utf8(slice).expect(&format!("{}: Error getting ROM title", "ERR".red()));
        println!("{}: Title: {}", "OK".green(), title.bright_blue());

        // Color or not color.
        {
            if data[0x143] == 0x80 {
                // Color GB.
                println!("{}: {}", "OK".green(), "GB Color cartridge");
            } else {
                // Not color GB.
                println!("{}: {}", "OK".green(), "Regular GB cartridge",);
            }
        }

        // Super Game Boy.
        {
            let sgbf = data[0x146];
            if sgbf == 0x03 {
                println!("{}: {}", "OK".green(), "Super Game Boy functions supported",);
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
                    "ERR".red(),
                    gc,
                    cs as u16
                );
            } else {
                println!("{}: {}: {:#04x}", "OK".green(), "Global checksum", cs as u8);
            }
        }

        // Check supported modes.
        let cart_type_enum = match cart_type {
            0x00 => {
                println!("{}: Using ROM ONLY mode", "OK".green());
                CartridgeType::RomOnly
            }
            0x01 | 0x02 | 0x03 => {
                println!("{}: Using MBC1 mode", "OK".green());
                let rom_size_code = data[0x148];
                let ram_size_code = data[0x149];
                CartridgeType::MBC1(Box::new(MBC1::new(
                    data.clone(),
                    rom_size_code,
                    ram_size_code,
                )))
            }
            0x05 | 0x06 => {
                println!("{}: Using MBC2 mode", "OK".green());
                let rom_size_code = data[0x148];
                CartridgeType::MBC2(Box::new(MBC2::new(data.clone(), rom_size_code)))
            }
            0x0F..=0x13 => {
                println!("{}: Using MBC3 mode", "OK".green());
                let rom_size_code = data[0x148];
                let ram_size_code = data[0x149];
                CartridgeType::MBC3(Box::new(MBC3::new(
                    data.clone(),
                    rom_size_code,
                    ram_size_code,
                )))
            }
            _ => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!(
                        "{}: Unsupported cartridge type: {}",
                        "ERR".red(),
                        Cartridge::cart_type_str(cart_type)
                    ),
                ));
            }
        };

        Ok(Self {
            rom: rom.to_string(),
            cart_type: cart_type_enum,
            data,
            dirty: false,
        })
    }

    /// Is the RAM dirty?
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    /// Consume the RAM so that it is clean again.
    pub fn consume_dirty(&mut self) {
        self.dirty = false;
    }

    /// ROM read.
    pub fn read(&self, address: u16) -> u8 {
        match &self.cart_type {
            CartridgeType::RomOnly => {
                if address < self.data.len() as u16 {
                    self.data[address as usize]
                } else {
                    0xFF
                }
            }
            CartridgeType::MBC1(mbc) => mbc.read(address),
            CartridgeType::MBC2(mbc) => mbc.read(address),
            CartridgeType::MBC3(mbc) => mbc.read(address),
        }
    }

    /// ROM write.
    pub fn write(&mut self, address: u16, value: u8) {
        match &mut self.cart_type {
            CartridgeType::RomOnly => {
                // ROM only has no banking, ignore writes
            }
            CartridgeType::MBC1(mbc) => mbc.write(address, value),
            CartridgeType::MBC2(mbc) => mbc.write(address, value),
            CartridgeType::MBC3(mbc) => mbc.write(address, value),
        }
    }

    /// Read from the given RAM address.
    pub fn read_ram(&self, address: u16) -> u8 {
        match &self.cart_type {
            CartridgeType::RomOnly => 0xFF, // No RAM for ROM only
            CartridgeType::MBC1(mbc) => mbc.read_ram(address),
            CartridgeType::MBC2(mbc) => mbc.read_ram(address),
            CartridgeType::MBC3(mbc) => mbc.read_ram(address),
        }
    }

    /// Write value to the given address of RAM.
    pub fn write_ram(&mut self, address: u16, value: u8) {
        match &mut self.cart_type {
            CartridgeType::RomOnly => {} // No RAM for ROM only
            CartridgeType::MBC1(mbc) => mbc.write_ram(address, value),
            CartridgeType::MBC2(mbc) => mbc.write_ram(address, value),
            CartridgeType::MBC3(mbc) => mbc.write_ram(address, value),
        }
        // Mark RAM dirty.
        self.dirty = true;
    }

    /// Get cartridge type as a descriptive string.
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

    /// Save SRAM of current cartridge to `.sav` file.
    pub fn save_sram(&self) {
        let rom_path = &self.rom;
        let save_path = Path::new(rom_path).with_extension("sav");

        // Only save if the mapper actually has RAM.
        let ram_data = match &self.cart_type {
            CartridgeType::MBC1(mbc) => &mbc.get_ram(),
            CartridgeType::MBC2(mbc) => &mbc.get_ram(),
            CartridgeType::MBC3(mbc) => &mbc.get_ram(),
            _ => return,
        };

        if !ram_data.is_empty() || matches!(self.cart_type, CartridgeType::MBC2(_)) {
            if let Ok(mut file) = File::create(&save_path) {
                let _ = file.write_all(ram_data);
                println!(
                    "{}: SRAM written to disk: {}",
                    "WR".magenta(),
                    save_path.display()
                );
            }
        }
    }

    /// Load `.sav` file into SRAM.
    pub fn load_sram(&mut self) {
        let rom_path = &self.rom;
        let save_path = Path::new(rom_path).with_extension("sav");

        if !save_path.exists() {
            return;
        }

        if let Ok(mut file) = File::open(&save_path) {
            let mut buffer = Vec::new();
            if file.read_to_end(&mut buffer).is_ok() {
                match &mut self.cart_type {
                    CartridgeType::MBC1(mbc) => {
                        if mbc.get_ram().len() == buffer.len() {
                            mbc.set_ram(buffer);
                        } else {
                            println!("{}: SRAM file size mismatch!", "WARN".yellow());
                        }
                    }
                    CartridgeType::MBC2(mbc) => mbc.set_ram(&buffer),
                    CartridgeType::MBC3(mbc) => {
                        if mbc.get_ram().len() == buffer.len() {
                            mbc.set_ram(buffer);
                        } else {
                            println!("{}: SRAM file size mismatch!", "WARN".yellow());
                        }
                    }
                    _ => (),
                }

                println!(
                    "{}: SRAM loaded from disk: {}",
                    "LD".magenta(),
                    save_path.display()
                );
            }
        }
    }
}
