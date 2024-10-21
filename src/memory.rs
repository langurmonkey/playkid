use crate::constants;

/// # Memory
/// The Game Boy uses a 2-byte address space (0x0000 to 0xFFFF) to map the different
/// types of memory (RAM, VRAM, Cartridge memory, etc.)
/// # VRAM
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it  has two maps of 1024 bytes each.
/// In total, a bank has 8 KiB of memory.
///
/// - A **tile** has 8x8 pixels, with a color depth of 2 bpp. Each tile is 16 bytes.
///   Tiles in a bank are typically grouped into blocks.
/// - A **tile block** contains 128 tiles of 16 bytes each, so 2048 bytes.
/// - A **map** contains 32x32=1024 bytes.
pub struct Memory {
    pub data: [u8; constants::MEM_SIZE],
}

impl Memory {
    /// Create a new memory instance.
    pub fn new() -> Self {
        Memory {
            data: [0; constants::MEM_SIZE],
        }
    }

    /// Read a byte of memory at the given `address`.
    pub fn read(&self, address: u16) -> u8 {
        self.data[address as usize]
    }
    /// Read two bytes of memory at the given `address`.
    pub fn read16(&self, address: u16) -> u16 {
        (self.read(address) as u16) | ((self.read(address + 1) as u16) << 8)
    }
    /// Write the given byte `value` at the given `address`.
    pub fn write(&mut self, address: u16, value: u8) {
        self.data[address as usize] = value;
    }
    /// Write the given word `value` at the given `address`.
    pub fn write16(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0xFF) as u8);
        self.write(address + 1, (value >> 8) as u8);
    }
}
