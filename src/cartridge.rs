use std::fs::File;
use std::io;
use std::io::prelude::*;

/// A representation of a cartridge..
pub struct Cartridge {
    /// Holds the ROM data in an array of bytes.
    data: Vec<u8>,
}

impl Cartridge {
    pub fn new(rom: &str, check: bool) -> io::Result<Self> {
        let mut file = File::open(rom)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Check Nintendo logo in rom file.
        // In 0x104 - 0x133 (260 to 307), with contents:
        // CE ED 66 66 CC 0D 00 0B 03 73 00 83 00 0C 00 0D
        // 00 08 11 1F 88 89 00 0E DC CC 6E E6 DD DD D9 99
        // BB BB 67 63 6E 0E EC CC DD DC 99 9F BB B9 33 3E
        if check {
            //Cartridge::check_checksum(&buffer);
        }

        Ok(Self { data: buffer })
    }
}
