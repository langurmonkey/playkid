use crate::constants;

/// # PPU
/// The PPU is the picture processing unit of our machine.
///
/// ## Video RAM
/// The Video RAM, or VRAM, are 8 KiB located in addresses 0x8000 to 0xA000.
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

    mode: u8,

    // The LCDC byte.
    /// LCD & PPU enable.
    lcdc7: bool,
    /// Window tile map.
    lcdc6: u16,
    /// Window enable.
    lcdc5: bool,
    /// BG & Window tiles.
    lcdc4: u16,
    /// BG tile map.
    lcdc3: u16,
    /// OBJ size.
    lcdc2: u32,
    /// OBJ enable.
    lcdc1: bool,
    /// BG & Window enable/priority.
    lcdc0: bool,

    /// LY: LCD Y coordinate.
    ly: u8,
    /// LYC: LY compare.
    lyc: u8,

    /// STAT: LCD status.
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
    /// SCX: Scroll X posiiton. Left coordinate of the visible 160x144 area within the BG map.
    scx: u8,

    /// WY: Window Y position.
    wy: u8,
    /// WX: Window X position plus 7.
    wx: u8,
}

impl PPU {
    pub fn new() -> Self {
        PPU {
            oam: [0; constants::OAM_SIZE],
            vram: [0; constants::VRAM_SIZE],
            mode: 0,
            lcdc7: true,
            lcdc6: 0,
            lcdc5: true,
            lcdc4: 0,
            lcdc3: 0,
            lcdc2: 0,
            lcdc1: true,
            lcdc0: true,
            ly: 0,
            lyc: 0,
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
        }
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // VRAM.
            0x8000..=0x9FFF => self.vram[(address - 0x8000) as usize],
            // OAM.
            0xFE00..=0xFE9F => self.oam[(address - 0xFE00) as usize],

            // Bank select.
            0xFF40 => 0xFF,
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
            _ => {}
        }
    }
}
