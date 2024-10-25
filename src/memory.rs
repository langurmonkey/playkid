use crate::cartridge;
use crate::constants;
use crate::ppu;

use cartridge::Cartridge;
use ppu::PPU;

/// # Memory
/// The Game Boy uses a 2-byte address space (0x0000 to 0xFFFF) to map the different
/// types of memory (RAM, VRAM, Cartridge memory, etc.)

/// ## Memory map
/// 0x0000-0x3FFF: 16 KiB bank #0                 (cartridge)
/// 0x4000-0x7FFF: 16 KiB switchable ROM bank     (cartridge)
/// 0x8000-0x9FFF: 8 KiB video RAM                (VRAM)
/// 0xA000-0xBFFF: 8 KiB switchable RAM bank      (cartridge)
/// 0xC000-0xDFFF: 8 KiB work RAM                 (WRAM)
/// 0xE000-0xFDFF: Echo RAM                     (mirror of 0xC000-0xDFFF)
/// 0xFE00-0xFE9F: Object attribute memory      (OAM)
/// 0xFEA0-0xFEFF: Empty, not usable
/// 0xFF00-0xFF4B: I/O registers                (I/O)
/// 0xFF4C-0xFF7F: Empty (?)
/// 0xFF80-0xFFFE: High RAM                     (HRAM)
/// 0xFF80-0xFFFF: Interrupt Enable Register    (IER)

pub struct Memory<'a> {
    /// Work RAM.
    pub wram: [u8; constants::WRAM_SIZE],
    // High RAM.
    pub hram: [u8; constants::HRAM_SIZE],
    // I/O registers.
    pub io: [u8; constants::IO_SIZE],
    // IME flag.
    pub ime: bool,
    // Cartridge reference.
    pub cart: &'a Cartridge,
    /// Our PPU, picture processing unit.
    pub ppu: PPU,
}

impl<'a> Memory<'a> {
    /// Create a new memory instance.
    pub fn new(cart: &'a Cartridge) -> Self {
        Memory {
            wram: [0; constants::WRAM_SIZE],
            hram: [0; constants::HRAM_SIZE],
            io: [0; constants::IO_SIZE],
            ime: false,
            cart,
            ppu: PPU::new(),
        }
    }

    pub fn initialize_hw_registers(&mut self) {
        // Initialize hardware registers in the I/O ports region.
        // P1
        self.write(0xFF00, 0xCF);
        // SB
        self.write(0xFF01, 0x00);
        // SC
        self.write(0xFF02, 0x7E);
        // DIV
        self.write(0xFF04, 0xAB);
        // TIMA
        self.write(0xFF05, 0x00);
        // TMA
        self.write(0xFF06, 0x00);
        // TAC
        self.write(0xFF07, 0xF8);
        // IF
        self.write(0xFF0F, 0xE1);
        // NR10
        self.write(0xFF10, 0x80);
        // NR11
        self.write(0xFF12, 0xBF);
        // NR12
        self.write(0xFF13, 0xF3);
        // NR13
        self.write(0xFF14, 0xFF);
        // NR14
        self.write(0xFF15, 0xBF);
        // NR21
        self.write(0xFF16, 0x3F);
        // NR22
        self.write(0xFF17, 0x00);
        // NR23
        self.write(0xFF18, 0xFF);
        // NR24
        self.write(0xFF19, 0xBF);
        // NR30
        self.write(0xFF1A, 0x7F);
        // NR31
        self.write(0xFF1B, 0xFF);
        // NR32
        self.write(0xFF1C, 0x9F);
        // NR33
        self.write(0xFF1D, 0xFF);
        // NR34
        self.write(0xFF1E, 0xBF);
        // NR41
        self.write(0xFF20, 0xFF);
        // NR42
        self.write(0xFF21, 0x00);
        // NR43
        self.write(0xFF22, 0x00);
        // NR44
        self.write(0xFF23, 0xBF);
        // NR50
        self.write(0xFF24, 0x77);
        // NR51
        self.write(0xFF25, 0xF3);
        // NR52
        self.write(0xFF26, 0xF1);
        // LCDC
        self.write(0xFF40, 0x91);
        // STAT
        self.write(0xFF41, 0x81);
        // SCY
        self.write(0xFF42, 0x00);
        // SCX
        self.write(0xFF43, 0x00);
        // LY
        self.write(0xFF44, 0x91);
        // LYC
        self.write(0xFF45, 0x00);
        // DMA
        self.write(0xFF46, 0xFF);
        // BGP
        self.write(0xFF47, 0xFC);
        // OBP0
        self.write(0xFF48, 0x00);
        // OBP1
        self.write(0xFF49, 0x00);
        // WY
        self.write(0xFF4A, 0x00);
        // WX
        self.write(0xFF4B, 0x00);
        // KEY1
        self.write(0xFF4D, 0x7E);
        // VBK
        self.write(0xFF4F, 0xFE);
        // HDMA1
        self.write(0xFF51, 0xFF);
        // HDMA2
        self.write(0xFF52, 0xFF);
        // HDMA3
        self.write(0xFF53, 0xFF);
        // HDMA4
        self.write(0xFF54, 0xFF);
        // HDMA5
        self.write(0xFF55, 0xFF);
        // RP
        self.write(0xFF56, 0x3E);
        // BCPS
        self.write(0xFF68, 0x00);
        // BCPD
        self.write(0xFF69, 0x00);
        // OCPS
        self.write(0xFF6A, 0x00);
        // OCPD
        self.write(0xFF6B, 0x00);
        // SVBK
        self.write(0xFF70, 0xF8);
        // IE
        self.write(0xFFFF, 0x00);
    }

    /// Gets the value of the IE register.
    pub fn get_ie(&self) -> u8 {
        self.read8(0xFFFF)
    }
    /// Gets the address of the Tile Data Table for the background,
    /// which is in register LCDC.
    pub fn get_lcdc(&self) -> u16 {
        self.read16(0xFF40)
    }

    /// Read a byte of memory at the given `address`.
    pub fn read8(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                // 16kB bank #0 (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            0x4000..=0x7FFF => {
                // 16kB switchable ROM bank (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            0x8000..=0x9FFF => {
                // VRAM.
                self.ppu.read(address)
            }
            0xA000..=0xBFFF => {
                // 8kB switchable RAM bank (cartridge).
                *self
                    .cart
                    .data
                    .get(address as usize)
                    .expect("Error getting cartridge data")
            }
            0xC000..=0xDFFF => {
                // 8kB WRAM.
                self.wram[(address - 0xC000) as usize]
            }
            0xE000..=0xFDFF => {
                // Echo of WRAM.
                self.wram[(address - 0xE000) as usize]
            }
            0xFE00..=0xFE9F => {
                // OAM.
                self.ppu.read(address)
            }
            0xFEA0..=0xFEFF => {
                // Empty, unusable.
                panic!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                )
            }
            // VRAM registers.
            0xFF40..=0xFF4F => self.ppu.read(address),
            0xFF00..=0xFF7F => {
                // I/O registers.
                self.io[(address - 0xFF00) as usize]
            }
            0xFF80..=0xFFFE => {
                // High RAM.
                self.hram[(address - 0xFF80) as usize]
            }
            0xFFFF => {
                // IME flag.
                if self.ime {
                    1
                } else {
                    0
                }
            }
            _ => 0xFF,
        }
    }
    /// Read two bytes of memory at the given `address`.
    pub fn read16(&self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }
    /// Write the given byte `value` at the given `address`.
    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => {
                // Cartridge (ROM + switchable banks).
                println!("Attempted to write to cartridge ROM: {:#06X}", address);
            }
            0x8000..=0x9FFF => {
                // VRAM.
                self.ppu.write(address, value);
            }
            0xA000..=0xBFFF => {
                // 8kB switchable RAM bank (cartridge).
                println!("Write to cartridge RAM: {:#06X}", address);
            }
            0xC000..=0xDFFF => {
                // 8kB WRAM.
                self.wram[(address - 0xC000) as usize] = value;
            }
            0xE000..=0xFDFF => {
                // Echo of WRAM.
                self.wram[(address - 0xE000) as usize] = value;
            }
            0xFE00..=0xFE9F => {
                // OAM.
                self.ppu.write(address, value);
            }
            0xFEA0..=0xFEFF => {
                // Empty, unusable.
                panic!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                )
            }
            // VRAM registers.
            0xFF40..=0xFF4F => self.ppu.write(address, value),
            0xFF00..=0xFF7F => {
                // I/O registers.
                self.io[(address - 0xFF00) as usize] = value;
            }
            0xFF80..=0xFFFE => {
                // High RAM.
                self.hram[(address - 0xFF80) as usize] = value;
            }
            0xFFFF => {
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
