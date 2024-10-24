use crate::cartridge;
use crate::constants;

use cartridge::Cartridge;

/// # Memory
/// The Game Boy uses a 2-byte address space (0x0000 to 0xFFFF) to map the different
/// types of memory (RAM, VRAM, Cartridge memory, etc.)

/// ## Memory map
/// 0x0000-0x3FFF: 16kB bank #0                 (cartridge)
/// 0x4000-0x7FFF: 16kB switchable ROM bank     (cartridge)
/// 0x8000-0x9FFF: 8kB video RAM                (VRAM)
/// 0xA000-0xBFFF: 8kB switchable RAM bank      (cartridge)
/// 0xC000-0xDFFF: 8kB work RAM                 (WRAM)
/// 0xE000-0xFDFF: Echo RAM                     (mirror of 0xC000-0xDFFF)
/// 0xFE00-0xFE9F: Object attribute memory      (OAM)
/// 0xFEA0-0xFEFF: Empty, not usable
/// 0xFF00-0xFF4B: I/O registers                (I/O)
/// 0xFF4C-0xFF7F: Empty (?)
/// 0xFF80-0xFFFE: High RAM                     (HRAM)
/// 0xFF80-0xFFFF: Interrupt Enable Register    (IER)

/// ## Video RAM
/// The Video RAM, or VRAM, are 8kB located in addresses 0x8000 to 0xA000.
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it  has two maps of 1024 bytes each.
/// In total, a bank has 8 KiB of memory.
///
/// - A **tile** has 8x8 pixels, with a color depth of 2 bpp. Each tile is 16 bytes.
///   Tiles in a bank are typically grouped into blocks.
/// - A **tile block** contains 128 tiles of 16 bytes each, so 2048 bytes.
/// - A **map** contains 32x32=1024 bytes.
pub struct Memory<'a> {
    /// Work RAM.
    pub wram: [u8; constants::WRAM_SIZE],
    // Video RAM.
    pub vram: [u8; constants::VRAM_SIZE],
    // High RAM.
    pub hram: [u8; constants::HRAM_SIZE],
    // Object Attribute Memory.
    pub oam: [u8; constants::OAM_SIZE],
    // I/O registers.
    pub io: [u8; constants::IO_SIZE],
    // IME flag.
    pub ime: bool,
    // Cartridge reference.
    pub cart: &'a Cartridge,
}

impl<'a> Memory<'a> {
    /// Create a new memory instance.
    pub fn new(cart: &'a Cartridge) -> Self {
        Memory {
            wram: [0; constants::WRAM_SIZE],
            vram: [0; constants::VRAM_SIZE],
            hram: [0; constants::HRAM_SIZE],
            oam: [0; constants::OAM_SIZE],
            io: [0; constants::IO_SIZE],
            ime: false,
            cart,
        }
    }

    /// Read a byte of memory at the given `address`.
    pub fn read8(&self, address: u16) -> u8 {
        match address {
            ..=0x3FFF => {
                // 16kB bank #0 (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            ..=0x7FFF => {
                // 16kB switchable ROM bank (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            ..=0x9FFF => {
                // 8kB VRAM.
                self.vram[(address - 0x8000) as usize]
            }
            ..=0xBFFF => {
                // 8kB switchable RAM bank (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            ..=0xDFFF => {
                // 8kB WRAM.
                self.wram[(address - 0xC000) as usize]
            }
            ..=0xFDFF => {
                // Echo of WRAM.
                self.wram[(address - 0xE000) as usize]
            }
            ..=0xFE9F => {
                // OAM.
                self.oam[(address - 0xFE00) as usize]
            }
            ..=0xFEFF => {
                // Empty, unusable.
                panic!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                )
            }
            ..=0xFF7F => {
                // I/O registers.
                self.io[(address - 0xFF00) as usize]
            }
            ..=0xFFFE => {
                // High RAM.
                self.hram[(address - 0xFF80) as usize]
            }
            ..=0xFFFF => {
                // IME flag.
                if self.ime {
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
    /// Read two bytes of memory at the given `address`.
    pub fn read16(&self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }
    /// Write the given byte `value` at the given `address`.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            ..=0x7FFF => {
                // Cartridge (ROM + switchable banks).
                panic!("Attempted to write to cartridge: {:#06X}", address);
            }
            ..=0x9FFF => {
                // 8kB VRAM.
                self.vram[(address - 0x8000) as usize] = value;
            }
            ..=0xBFFF => {
                // 8kB switchable RAM bank (cartridge).
                panic!("Attempted write to cartridge: {:#06X}", address);
            }
            ..=0xDFFF => {
                // 8kB WRAM.
                self.wram[(address - 0xC000) as usize] = value;
            }
            ..=0xFDFF => {
                // Echo of WRAM.
                self.wram[(address - 0xE000) as usize] = value;
            }
            ..=0xFE9F => {
                // OAM.
                self.oam[(address - 0xFE00) as usize] = value;
            }
            ..=0xFEFF => {
                // Empty, unusable.
                panic!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                )
            }
            ..=0xFF7F => {
                // I/O registers.
                self.io[(address - 0xFF00) as usize] = value;
            }
            ..=0xFFFE => {
                // High RAM.
                self.hram[(address - 0xFF80) as usize] = value;
            }
            ..=0xFFFF => {
                // IME flag.
                match value {
                    0 => self.ime = false,
                    1 => self.ime = true,
                    _ => panic!("Attempted IME write with non-0 or -1 value: {}", value),
                }
            }
        }
    }
    /// Write the given word `value` at the given `address`.
    pub fn write16(&mut self, address: u16, value: u16) {
        self.write(address, (value & 0xFF) as u8);
        self.write(address + 1, (value >> 8) as u8);
    }
}
