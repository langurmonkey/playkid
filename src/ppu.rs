use crate::constants;

/// # PPU
/// The PPU is the picture processing unit of our machine.
///
/// ## Video RAM
/// The Video RAM, or VRAM, are 8 KiB located in addresses 0x8000 to 0x9FFF.
/// A **memory bank** contains 384 tiles, or 3 tile blocks, so 6 KiB of tile data.
/// After that, it  has two maps of 1024 bytes each (32 rows of 32 bytes each), the
/// Background Tile Map. Each byte contains the tile number to be displayed.
/// The tiles are taken from the Tile Data Table, which is at either 0x8000-0x8FFF,
/// or 0x8800-ox97FF. In the first case, tiles are numbered as unsigned bytes (u8).
/// In the second case, the numbers are signed (i8), and tile 0 lies at 0x9000.
/// The Tile Data Table address can be selected via the LCDC register.
///
/// In total, a bank has 8 KiB of memory.
///
/// - A **tile** has 8x8 pixels, with a color depth of 2 bpp. Each tile is 16 bytes.
///   Tiles in a bank are typically grouped into blocks.
/// - A **tile block** contains 128 tiles of 16 bytes each, so 2048 bytes.
/// - A **map** contains 32x32=1024 bytes.
pub struct PPU {
    /// Object Attribute Memory.
    pub oam: [u8; constants::OAM_SIZE],
    /// Video RAM.
    pub vram: [u8; constants::VRAM_SIZE],

    /// There are four modes:
    /// - 0: HBlank
    /// - 1: VBlank
    /// - 2: OAM scan
    /// - 3: HDraw
    mode: u8,
    /// Start dot of the PPU.
    start_dot: u32,
    /// Current dot in a frame.
    dot: u32,

    // The LCDC byte.
    pub lcdc: u8,
    /// LCD & PPU enable.
    lcdc7: bool,
    /// Window tile map.
    lcdc6: u16,
    /// Window enable.
    lcdc5: bool,
    /// BG & Window tile address.
    lcdc4: u16,
    /// BG tile map.
    lcdc3: u16,
    /// OBJ size.
    lcdc2: u32,
    /// OBJ enable.
    lcdc1: bool,
    /// BG & Window enable/priority.
    lcdc0: bool,

    /// LX: LCD X coordinate.
    pub lx: u32,
    /// LY: LCD Y coordinate.
    pub ly: u8,
    /// LYC: LY compare.
    pub lyc: u8,

    /// STAT: LCD status.
    pub stat: u8,
    /// STAT6: LYC int select.
    stat6: bool,
    /// STAT5: Mode2 int select.
    stat5: bool,
    /// STAT4: Mode1 int select.
    stat4: bool,
    /// STAT3: Mode0 int select.
    stat3: bool,
    /// STAT2: LYC == LY flag.
    stat2: bool,
    /// STAT10: PPU mode.
    stat01: u8,

    /// SCY: Scroll Y position. Top coordinate of the visible 160x144 area within the BG map.
    scy: u8,
    /// SCX: Scroll X position. Left coordinate of the visible 160x144 area within the BG map.
    scx: u8,

    /// WY: Window Y position.
    wy: u8,
    /// WX: Window X position plus 7.
    wx: u8,
    /// LCD interrupt mask for registers IE and IF.
    pub i_mask: u8,
}

impl PPU {
    pub fn new(start_dot: u32) -> Self {
        PPU {
            oam: [0; constants::OAM_SIZE],
            vram: [0; constants::VRAM_SIZE],
            mode: 0,
            lcdc: 0,
            lcdc7: true,
            lcdc6: 0,
            lcdc5: true,
            lcdc4: 0,
            lcdc3: 0,
            lcdc2: 0,
            lcdc1: true,
            lcdc0: true,
            start_dot,
            dot: start_dot,
            lx: start_dot,
            ly: 0,
            lyc: 0,
            stat: 0,
            stat6: false,
            stat5: false,
            stat4: false,
            stat3: false,
            stat2: false,
            stat01: 0,
            scy: 0,
            scx: 0,
            wy: 0,
            wx: 7,
            i_mask: 0,
        }
    }

    pub fn reset(&mut self) {
        self.oam.fill(0);
        self.vram.fill(0);
        self.mode = 0;
        self.lcdc = 0;
        self.dot = self.start_dot;
        self.ly = 0;
        self.lx = self.start_dot;
        self.lyc = 0;
        self.stat = 0;
        self.scx = 0;
        self.scy = 0;
        self.wx = 0;
        self.wy = 0;
        self.i_mask = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // VRAM.
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            // OAM.
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],
            // LCDC.
            0xFF40 => self.lcdc,
            // STAT.
            0xFF41 => self.stat,
            // SCY.
            0xFF42 => self.scy,
            // SCX.
            0xFF43 => self.scx,
            // LY.
            0xFF44 => self.ly,
            // LCY.
            0xFF45 => self.lyc,
            // DMA.
            0xFF46 => 0,

            // WX.
            0xFF4A => self.wx,
            // WY.
            0xFF4B => self.wy,

            _ => 0xFF,
        }
    }

    pub fn write(&mut self, address: u16, value: u8) {
        match address {
            0x8000..=0x9FFF => {
                // VRAM.
                self.vram[(address - 0x8000) as usize] = value;
            }
            0xFE00..=0xFE9F => {
                // OAM.
                self.oam[(address - 0xFE00) as usize] = value;
            }
            // LCDC.
            0xFF40 => {
                self.lcdc = value;
                self.update_lcdc_flags();
            }
            0xFF41 => {
                self.stat = value;
                self.update_stat_flags();
            }
            // SCY.
            0xFF42 => self.scy = value,
            // SCX.
            0xFF43 => self.scx = value,
            // LY.
            0xFF44 => self.ly = value,
            // LCY.
            0xFF45 => self.lyc = value,
            // DMA.
            0xFF46 => {
                // Writing to this register starts a DMA transfer from ROM/RAM to OAM.
                // The written value specifies the transfer source address divided by $100.
                // Source: $XX00-$XX9F (where XX is the written value).
                // Dest:   $FE00-FE9F
                // This is implemented in `memory.rs`.
            }

            // WX.
            0xFF4A => self.wx = value,
            // WY.
            0xFF4B => self.wy = value,
            _ => {}
        }
    }

    /// Performs a GPU cycle with the given number of t-cycles, or dots.
    /// 1 m-cycle has 4 dots, or t-cycles.
    /// Timing is divided between 154 lines, 144 during VDraw (modes 0, 2, 3),
    /// and 10 during VBlank. Each line takes 456 dots, and one frame takes
    /// 70224 dots.
    pub fn cycle(&mut self, t_cycles: u32) {
        if !self.is_ppu_enabled() {
            return;
        }

        self.dot += t_cycles;
        self.update_mode();

        self.lx = self.dot % 456;
        self.ly = (self.dot / 456) as u8;
        self.check_interrupt_lyc();
    }

    fn update_mode(&mut self) {
        let new_mode = match self.ly {
            0..=143 => match self.lx {
                0..=79 => 2,
                80..=252 => 3,
                253..=456 => 0,
                _ => 10,
            },
            144..=153 => 1,
            _ => 10,
        };
        // Interrupts.
        if new_mode != self.mode {
            match new_mode {
                0 => {
                    if self.stat3 {
                        // STAT mode 0.
                        self.i_mask |= 0b0000_0010;
                    }
                }
                1 => {
                    if self.stat4 {
                        // STAT mode 1.
                        self.i_mask |= 0b0000_0010;
                    }
                    // V-Blank.
                    self.i_mask |= 0b0000_0001;
                }
                2 => {
                    if self.stat5 {
                        // STAT mode 2.
                        self.i_mask |= 0b0000_0010;
                    }
                }
                _ => {}
            }
        }
        self.mode = new_mode;
    }

    /// Update STAT bit 2 (LYC==LY).
    fn check_interrupt_lyc(&mut self) {
        if self.ly == self.lyc {
            self.stat = self.stat | 0b0000_0100;
            self.stat2 = true;
            if self.stat6 {
                self.i_mask |= 0b0000_0010;
            }
        }
    }

    /// This method updates the LCDC flags from the current value
    /// in the byte `self.lcdc`.
    fn update_lcdc_flags(&mut self) {
        self.lcdc7 = self.lcdc & 0b1000_0000 != 0;
        self.lcdc6 = if self.lcdc & 0b0100_0000 != 0 {
            0x9800
        } else {
            0x9C00
        };
        self.lcdc5 = self.lcdc & 0b0010_0000 != 0;
        self.lcdc4 = if self.lcdc & 0b0001_0000 != 0 {
            // Signed access.
            0x9000
        } else {
            // Unsigned access.
            0x8000
        };
        self.lcdc3 = if self.lcdc & 0b0000_1000 != 0 {
            0x9800
        } else {
            0x9C00
        };
        self.lcdc2 = if self.lcdc & 0b0000_0100 != 0 {
            64
        } else {
            128
        };
        self.lcdc1 = self.lcdc & 0b0000_0010 != 0;
        self.lcdc0 = self.lcdc & 0b0000_0001 != 0;
    }

    /// This method updates the STAT flags from the current value
    /// in the byte `self.stat`.
    fn update_stat_flags(&mut self) {
        // LYC int select (rw).
        self.stat6 = self.stat & 0b0100_0000 != 0;
        // Mode 2 int select (rw).
        self.stat5 = self.stat & 0b0010_0000 != 0;
        // Mode 1 int select (rw).
        self.stat4 = self.stat & 0b0001_0000 != 0;
        // Mode 0 int select (rw).
        self.stat3 = self.stat & 0b0000_1000 != 0;
        // LYC == LY flag (read only).
        //self.stat2 = self.stat & 0b0000_0100 == 0;
        // PPU Mode (read only).
        //self.stat01 = self.stat & 0b0000_0011;
    }

    /// Gets the starting index of the tile data region. May be 0 (lcdc4 == 1, 0x8000-0x8FFF),
    /// or -128 (lcdc4 == 0, 0x8800-0x97FF, with 0 at 0x9000).
    pub fn get_bgwin_index(&self) -> i32 {
        if self.lcdc4 == 0x8000 {
            0
        } else if self.lcdc4 == 0x9000 {
            -128
        } else {
            panic!(
                "LCDC(4) does not contain a valid tile data address: {:#06X}",
                self.lcdc4
            );
        }
    }

    /// Gets the address in bit 4 of LCDC register.
    /// This is the background and window tile data address.
    pub fn get_bgwin_tiledata_addr(&self) -> u16 {
        self.lcdc4
    }
    /// Gets the address of the background tile map.
    pub fn get_bg_tilemap_addr(&self) -> u16 {
        self.lcdc3
    }
    /// Gets the address of the window tile map.
    pub fn get_win_tilemap_addr(&self) -> u16 {
        self.lcdc6
    }

    /// Are the LCD and the PPU enabled?
    pub fn is_ppu_enabled(&self) -> bool {
        self.lcdc7
    }
}
