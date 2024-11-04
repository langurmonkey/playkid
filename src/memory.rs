use crate::cartridge;
use crate::constants;
use crate::joypad;
use crate::ppu;
use crate::timer;

use cartridge::Cartridge;
use joypad::Joypad;
use ppu::PPU;
use sdl2::Sdl;
use timer::Timer;

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

pub struct Memory<'a, 'b> {
    /// Work RAM.
    pub wram: [u8; constants::WRAM_SIZE],
    // High RAM.
    pub hram: [u8; constants::HRAM_SIZE],
    // I/O registers.
    pub io: [u8; constants::IO_SIZE],
    // IF: interrupt flag.
    pub iff: u8,
    // IE flag: interrupt enable.
    pub ie: u8,
    // Cartridge reference.
    pub cart: &'a Cartridge,
    /// Our PPU, picture processing unit.
    pub ppu: PPU,
    /// The timer.
    pub timer: Timer,
    /// The joypad.
    pub joypad: Joypad<'b>,
}

impl<'a, 'b> Memory<'a, 'b> {
    /// Create a new memory instance.
    pub fn new(cart: &'a Cartridge, sdl: &'b Sdl) -> Self {
        Memory {
            wram: [0; constants::WRAM_SIZE],
            hram: [0; constants::HRAM_SIZE],
            io: [0; constants::IO_SIZE],
            iff: 0,
            ie: 0,
            cart,
            ppu: PPU::new(),
            timer: Timer::new(),
            joypad: Joypad::new(sdl),
        }
    }

    pub fn ppu(&self) -> &PPU {
        &self.ppu
    }

    pub fn initialize_hw_registers(&mut self) {
        // Initialize hardware registers in the I/O ports region.
        // P1/JOYPAD
        self.write8(0xFF00, 0xCF);
        // SB
        self.write8(0xFF01, 0x00);
        // SC
        self.write8(0xFF02, 0x7E);
        // DIV
        self.write8(0xFF04, 0xAB);
        // TIMA
        self.write8(0xFF05, 0x00);
        // TMA
        self.write8(0xFF06, 0x00);
        // TAC
        self.write8(0xFF07, 0xF8);
        // IF
        self.write8(0xFF0F, 0xE1);
        // NR10
        self.write8(0xFF10, 0x80);
        // NR11
        self.write8(0xFF12, 0xBF);
        // NR12
        self.write8(0xFF13, 0xF3);
        // NR13
        self.write8(0xFF14, 0xFF);
        // NR14
        self.write8(0xFF15, 0xBF);
        // NR21
        self.write8(0xFF16, 0x3F);
        // NR22
        self.write8(0xFF17, 0x00);
        // NR23
        self.write8(0xFF18, 0xFF);
        // NR24
        self.write8(0xFF19, 0xBF);
        // NR30
        self.write8(0xFF1A, 0x7F);
        // NR31
        self.write8(0xFF1B, 0xFF);
        // NR32
        self.write8(0xFF1C, 0x9F);
        // NR33
        self.write8(0xFF1D, 0xFF);
        // NR34
        self.write8(0xFF1E, 0xBF);
        // NR41
        self.write8(0xFF20, 0xFF);
        // NR42
        self.write8(0xFF21, 0x00);
        // NR43
        self.write8(0xFF22, 0x00);
        // NR44
        self.write8(0xFF23, 0xBF);
        // NR50
        self.write8(0xFF24, 0x77);
        // NR51
        self.write8(0xFF25, 0xF3);
        // NR52
        self.write8(0xFF26, 0xF1);
        // LCDC
        self.write8(0xFF40, 0x91);
        // STAT
        self.write8(0xFF41, 0x81);
        // SCY
        self.write8(0xFF42, 0x00);
        // SCX
        self.write8(0xFF43, 0x00);
        // LY
        self.write8(0xFF44, 0x91);
        // LYC
        self.write8(0xFF45, 0x00);
        // DMA
        self.write8(0xFF46, 0xFF);
        // BGP
        self.write8(0xFF47, 0xFC);
        // OBP0
        self.write8(0xFF48, 0x00);
        // OBP1
        self.write8(0xFF49, 0x00);
        // WY
        self.write8(0xFF4A, 0x00);
        // WX
        self.write8(0xFF4B, 0x00);
        // KEY1
        self.write8(0xFF4D, 0x7E);
        // VBK
        self.write8(0xFF4F, 0xFE);
        // HDMA1
        self.write8(0xFF51, 0xFF);
        // HDMA2
        self.write8(0xFF52, 0xFF);
        // HDMA3
        self.write8(0xFF53, 0xFF);
        // HDMA4
        self.write8(0xFF54, 0xFF);
        // HDMA5
        self.write8(0xFF55, 0xFF);
        // RP
        self.write8(0xFF56, 0x3E);
        // BCPS
        self.write8(0xFF68, 0x00);
        // BCPD
        self.write8(0xFF69, 0x00);
        // OCPS
        self.write8(0xFF6A, 0x00);
        // OCPD
        self.write8(0xFF6B, 0x00);
        // SVBK
        self.write8(0xFF70, 0xF8);
        // IE
        self.write8(0xFFFF, 0x00);
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
                    .get((address - 0x4000) as usize)
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
                    .get((address - 0xA000) as usize)
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
                println!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                );
                0
            }
            // Joypad.
            0xFF00 => self.joypad.read(address),
            // Timer registers.
            0xFF04..=0xFF07 => self.timer.read(address),
            // Interrupt flag.
            0xFF0F => self.iff | 0b1110_0000,

            // VRAM registers.
            0xFF40..=0xFF4F => self.ppu.read(address),
            0xFF50..=0xFF7F => {
                // I/O registers.
                self.io[(address - 0xFF00) as usize]
            }
            0xFF80..=0xFFFE => {
                // High RAM.
                self.hram[(address - 0xFF80) as usize]
            }
            // IE
            0xFFFF => self.ie,
            _ => 0xFF,
        }
    }
    /// Read two bytes of memory at the given `address`.
    pub fn read16(&self, address: u16) -> u16 {
        (self.read8(address) as u16) | ((self.read8(address + 1) as u16) << 8)
    }
    /// Write the given byte `value` at the given `address`.
    pub fn write8(&mut self, address: u16, value: u8) {
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
                println!(
                    "Attempted usage of forbidden area 0xFEA0-0xFEFE: {:#06X}",
                    address
                )
            }
            // Joypad.
            0xFF00 => self.joypad.write(address, value),
            // Timer registers.
            0xFF04..=0xFF07 => self.timer.write(address, value),
            // IF: interrupt flag.
            0xFF0F => self.iff = value,

            // OAM DMA.
            0xFF46 => {
                // ROM/RAM to OAM.
                let src0 = (value as u16) << 8;
                let dest0 = 0xFE00;
                for i in 0..0xA0 {
                    let byte = self.read8(src0 + i);
                    self.write8(dest0 + i, byte);
                }
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
            // IE: interrupt enable.
            0xFFFF => self.ie = value,
        }
    }
    /// Write the given word `value` at the given `address`.
    pub fn write16(&mut self, address: u16, value: u16) {
        self.write8(address, (value & 0xFF) as u8);
        self.write8(address + 1, (value >> 8) as u8);
    }

    pub fn cycle(&mut self, t_cycles: u32) -> u32 {
        self.joypad.update();

        let vram_cycles = 0;
        let ppu_cycles = t_cycles + vram_cycles;

        // Time.
        self.timer.cycle(t_cycles);
        self.iff |= self.timer.i_mask;
        self.timer.i_mask = 0;

        // PPU
        self.ppu.cycle(t_cycles);
        self.iff |= self.ppu.i_mask;
        self.ppu.i_mask = 0;

        ppu_cycles
    }
}
