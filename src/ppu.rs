use crate::constants;
use std::collections::VecDeque;

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
    /// Current dot within a frame, in [0,4560).
    fdot: u32,
    /// Current dot within the line, in [0,456).
    ldot: u32,

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
    pub lx: u8,
    /// LY: LCD Y coordinate.
    pub ly: u8,
    /// LYC: LY compare.
    pub lyc: u8,
    /// Update LY.
    pub ly_update: bool,

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
    /// BGP: Background palette register.
    bgp: u8,
    /// OBP0: Object palette 0.
    obp0: u8,
    /// OBP1: Object palette 1.
    obp1: u8,

    /// LCD interrupt mask for registers IE and IF.
    pub i_mask: u8,

    /// Screen buffer with 8 bpp.
    pub scr: Vec<u8>,
    /// OAM sprite buffer.
    sprite_buf: Vec<Sprite>,
    /// Background pixel FIFO (for bg and window).
    pub fifo_bg: VecDeque<Pixel>,
    /// Sprite pixel FIFO.
    pub fifo_sprite: VecDeque<Pixel>,
    /// T-cycle accumulator, to trigger certain actions.
    tcycle_accum: u32,
    /// OAM pointer.
    oam_ptr: usize,
    /// Sprite fetcher step.
    sprite_step: u8,
    /// Background fetcher step.
    /// When this is 0, the background fetcher is paused.
    bg_step: u8,
    /// Current sprite index.
    curr_sprite_i: i16,
    /// Tile data (low).
    tile_low: u8,
    /// Tile data (high).
    tile_high: u8,
}

impl PPU {
    pub fn new(start_dot: u32) -> Self {
        PPU {
            oam: [0xFF; constants::OAM_SIZE],
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
            fdot: start_dot,
            ldot: start_dot % 456,
            lx: 0,
            ly: 0,
            lyc: 0,
            ly_update: false,
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
            bgp: 0,
            obp0: 0,
            obp1: 0,
            i_mask: 0,

            scr: Vec::with_capacity(144 * 160),
            sprite_buf: Vec::with_capacity(10),
            fifo_bg: VecDeque::with_capacity(16),
            fifo_sprite: VecDeque::with_capacity(16),
            tcycle_accum: 0,
            oam_ptr: 0,
            sprite_step: 0,
            bg_step: 0,
            curr_sprite_i: -1,
            tile_low: 0,
            tile_high: 0,
        }
    }

    pub fn reset(&mut self) {
        self.oam.fill(0xFF);
        self.vram.fill(0);
        self.mode = 0;
        self.lcdc = 0;
        self.fdot = self.start_dot;
        self.ldot = self.start_dot % 456;
        self.ly = 0;
        self.lx = 0;
        self.lyc = 0;
        self.stat = 0;
        self.scx = 0;
        self.scy = 0;
        self.wx = 0;
        self.wy = 0;
        self.bgp = 0;
        self.obp0 = 0;
        self.obp1 = 1;
        self.i_mask = 0;
        self.scr.fill(0);
        self.tcycle_accum = 0;
        self.oam_ptr = 0;
        self.sprite_step = 0;
        self.bg_step = 0;
        self.curr_sprite_i = -1;
        self.tile_low = 0;
        self.tile_high = 0;
    }

    pub fn read(&self, address: u16) -> u8 {
        match address {
            // VRAM.
            0x8000..=0x9FFF => {
                if self.mode != 3 {
                    self.vram[(address - 0x8000) as usize]
                } else {
                    // During mode 3 VRAM is inaccessible.
                    0xFF
                }
            }
            // OAM.
            0xFE00..=0xFE9F => {
                if self.mode & 0x02 == 0 {
                    self.oam[(address - 0xfe00) as usize]
                } else {
                    // During modes 2 and 3 OAM is inaccessible.
                    0xFF
                }
            }
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
            // BGP.
            0xFF47 => self.bgp,
            // OBP0.
            0xFF48 => self.obp0,
            // OBP1.
            0xFF49 => self.obp1,

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
                // VRAM only accessible when mode != 3
                if self.mode != 3 {
                    self.vram[(address - 0x8000) as usize] = value;
                }
            }
            0xFE00..=0xFE9F => {
                // OAM inaccessible in modes 2 and 3.
                if self.mode & 0x02 == 0 {
                    self.oam[(address - 0xFE00) as usize] = value;
                }
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
            // BGP.
            0xFF47 => self.bgp = value,
            // OBP0.
            0xFF48 => self.obp0 = value,
            // OBP1.
            0xFF49 => self.obp1 = value,

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

        // Update dot numbers.
        let last_ldot = self.ldot;
        self.fdot = (self.fdot + t_cycles) % 4560;
        self.ldot = self.fdot % 456;
        // Update mode if necessary.
        self.update_mode();

        // LY.
        if self.ly_update {
            self.ly = (self.ly + 1) % 154;
            self.ly_update = false;
            // Clear OAM object buffer.
            self.sprite_buf.clear();
            // Check LY==LYC condition.
            self.check_interrupt_lyc();
        }

        // LX.
        if self.ldot < last_ldot {
            // New line, update in next cycle.
            self.ly_update = true;
        }

        // Clear pixel FIFOs.
        self.fifo_bg.clear();
        self.fifo_sprite.clear();

        // Update T-cycle accumulator.
        self.tcycle_accum += t_cycles;
        match self.mode {
            0 => {
                // MODE 0: H-Blank (until dot % 456 == 0).
                // Only consume cycles.
                self.tcycle_accum -= t_cycles;
            }
            1 => {
                // MODE 1: V-Blank (10 lines * 456 T-cycles, 4560).
                // Only consume cycles.
                self.tcycle_accum -= t_cycles;
            }
            2 => {
                // MODE 2: OAM (80 T-cycles; check new OAM entry every 2 T-cycles, 40 in total).
                // Only 10 sprites per scanline are supported by the Game Boy.
                while self.tcycle_accum >= 2
                    && self.oam_ptr <= self.oam.len() - 4
                    && self.sprite_buf.len() < 10
                {
                    // Fetch new OAM entry.
                    let y = self.oam[self.oam_ptr].saturating_sub(15);
                    let x = self.oam[self.oam_ptr + 1].saturating_sub(7);
                    let tile = self.oam[self.oam_ptr + 2];
                    let flags = self.oam[self.oam_ptr + 3];

                    // Push to OAM buffer if sprite intersects with current scanline (LY).
                    if self.ly >= y && self.ly < y + 8 {
                        self.sprite_buf.push(Sprite::new(x, y, tile, flags));
                    }

                    // Advance pointer.
                    self.oam_ptr += 4;
                    // Consume cycles.
                    self.tcycle_accum -= 2;
                }
            }
            3 => {
                // MODE 3: DRAWING (172-289 T-cycles).
                // There are 4 steps, that take 2 cycles each to complete.
                while self.tcycle_accum >= 2 {
                    // Sprite fetcher check.
                    if self.sprite_step == 0 && self.check_sprite_fetch() {
                        // Reset background fetcher.
                        self.bg_step = 0;
                        self.sprite_step = 1;
                    }

                    // Sprite fetcher.
                    if self.sprite_step > 0 {
                        // Fetch indices of sprites that overlap pixel.
                        let mut sprite_indices: Vec<usize> = self
                            .sprite_buf
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| self.lx >= s.x && self.lx < s.x + 8)
                            .map(|(idx, _)| idx)
                            .collect::<Vec<_>>();

                        match self.sprite_step {
                            1 => {
                                // Read tile number from sprite buffer.
                                self.curr_sprite_i = sprite_indices.pop().unwrap() as i16;
                                self.sprite_step += 1;
                            }
                            2 => {
                                // Fetch tile data (low).
                                let s = self.sprite_buf[self.curr_sprite_i as usize];
                                // We are at line LY-SY, so the offset is the line times 2, as
                                // every line is 2 bytes.
                                let offset = (self.ly - s.y) * 2;
                                self.tile_low = self.read(0x8000 + (s.tile + offset) as u16);
                                self.sprite_step += 1;
                            }
                            3 => {
                                // Fetch tile data (high).
                                let s = self.sprite_buf[self.curr_sprite_i as usize];
                                // We are at line LY-SY, so the offset is the line times 2, as
                                // every line is 2 bytes.
                                let offset = (self.ly - s.y) * 2;
                                self.tile_high = self.read(0x8000 + (s.tile + offset + 1) as u16);
                                self.sprite_step += 1;
                            }
                            4 => {
                                // Decode tile_low and tile_high into pixels.
                                // Push pixels into sprite FIFO.
                                let s = self.sprite_buf[self.curr_sprite_i as usize];
                                let bits_low = self.get_bits_of_byte(self.tile_low);
                                let bits_high = self.get_bits_of_byte(self.tile_high);
                                for x_col in 0..8 {
                                    let col_id = (bits_low[x_col] | (bits_high[x_col] << 1)) as u8;
                                    let pix = Pixel::new(s.x + x_col as u8, self.ly, col_id, 0, 0);
                                    self.fifo_sprite.push_front(pix);
                                }
                                // Next pixel.
                                self.lx = (self.lx + 1) % 160;

                                // Back to step 0.
                                self.sprite_step = 0;
                                // Restore background fetcher for next cycle.
                                self.bg_step = 1;
                            }

                            _ => {}
                        }
                    } else if self.bg_step > 0 {
                        // Background and Window tiles.
                        if self.lcdc0 {
                            // Background fetcher.
                            {}

                            // Window fetcher.
                            if self.lcdc5 {}
                        }
                    }

                    // Consume cycles.
                    self.tcycle_accum -= 2;
                }
            }
            _ => {}
        }
    }

    /// Checks whether the PPU has pixels in the FIFOs.
    pub fn has_pixels(&self) -> bool {
        !self.fifo_sprite.is_empty()
    }

    pub fn consume_pixels(&self) -> Vec<Pixel> {
        self.fifo_sprite.iter().map(|p| p.clone()).collect()
    }

    /// Checks if there are any sprites in the sprite buffer that need to be fetched.
    /// The sprites X position is checked against the current LX position.
    /// The condition is LX >= S.X AND LX < S.X + 8.
    /// It also checks that LCDC1 is set (OBJ enable).
    fn check_sprite_fetch(&self) -> bool {
        self.lcdc1
            && self
                .sprite_buf
                .iter()
                .filter(|s| self.lx >= s.x && self.lx < s.x + 8)
                .count()
                != 0
    }

    /// Draws the sprite located at the given memory address at the given
    /// screen position [sx,sy] in the screen buffer.
    fn draw_sprite(&mut self, addr: u16, sx: usize, sy: usize) {
        let base = addr;
        // Sprites are 8x8 pixels, where each row of 8 pixels is 2 bytes.
        for row in 0..8 {
            let address = base + row * 2;
            let low = self.read(address);
            let high = self.read(address + 1);
            let bits_low = self.get_bits_of_byte(low);
            let bits_high = self.get_bits_of_byte(high);
            for col in 0..8 {
                let col_id = (bits_low[col] | (bits_high[col] << 1)) as u8;
                self.scr[sy * constants::DISPLAY_WIDTH + sx] = col_id;
            }
        }
    }

    /// Updates the mode given the current frame dot.
    fn update_mode(&mut self) {
        let new_mode = match self.ly % 154 {
            0..=143 => match self.ldot {
                // OAM search.
                0..=79 => 2,
                // Drawing.
                80..=252 => 3,
                // H-Blank.
                253..=455 => 0,
                // Not possible.
                _ => 10,
            },
            // V-Blank.
            144..=153 => 1,
            _ => 10,
        };

        // Update STAT bits 01 with PPU mode.
        self.stat = (self.stat & 0xF4) | new_mode;
        self.stat01 = new_mode;
        // Interrupts.
        if new_mode != self.mode {
            match new_mode {
                0 => {
                    // Reset sprite buffer.
                    self.sprite_buf.clear();

                    if self.stat3 {
                        // Request LCD STAT interrupt.
                        // H-Blank, raise LCD IF flag (bit 1).
                        self.i_mask |= 0b0000_0010;
                        // Clear sprite buffer for next line.
                    }
                }
                1 => {
                    if self.stat4 {
                        // Request LCD STAT interrupt.
                        // V-Blank, raise LCD IF flag (bit 1).
                        self.i_mask |= 0b0000_0010;
                    }
                    // Request V-Blank interrupt.
                    self.i_mask |= 0b0000_0001;
                }
                2 => {
                    // Reset OAM pointer.
                    self.oam_ptr = 0;

                    if self.stat5 {
                        // Request LCD STAT interrupt.
                        // OAM scanning, raise LCD IF flag (bit 1).
                        self.i_mask |= 0b0000_0010;
                    }
                }
                3 => {
                    // ??
                }
                _ => {}
            }
        }
        self.mode = new_mode;
    }

    /// Update STAT bit 2 (LYC==LY).
    fn check_interrupt_lyc(&mut self) {
        if self.ly == self.lyc {
            // Activate bit 2.
            self.stat = (self.stat & 0xFB) | 0x04;
            self.stat2 = true;
            if self.stat6 {
                self.i_mask |= 0b0000_0010;
            }
        } else {
            // Deactivate bit 2.
            self.stat = self.stat & 0xFB;
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

    /// Gets the bits of a byte as an array, with the most significant bit
    /// at index 0 and the least significant bit at index 7.
    /// For example, if the byte is 130, the array will be [1, 0, 0, 0, 0, 0, 1, 0]
    fn get_bits_of_byte(&self, byte: u8) -> [u8; 8] {
        let mut bits = [0u8; 8];
        for i in 0..=7 {
            let shifted_byte = byte >> i;
            // Get the rightmost bit of the shifted byte (least significant bit)
            let cur_bit = shifted_byte & 1;
            // For the first iteration, the cur_bit is the
            // least significant bit and therefore we place
            // that bit at index 7 of the array (rightmost bit)
            bits[7 - i] = cur_bit;
        }
        bits
    }
}

/// A pixel in either of the pixel FIFOs.
#[derive(Copy, Clone)]
pub struct Pixel {
    /// X LCD position.
    pub x: u8,
    /// Y LCD position.
    pub y: u8,
    /// Color ID.
    pub color: u8,
    /// Palette to use.
    pub palette: u8,
    // OBJ-to-BG priority.
    pub bg_prio: u8,
}

impl Pixel {
    /// Create a new pixel with the given data.
    fn new(x: u8, y: u8, color: u8, palette: u8, bg_prio: u8) -> Self {
        Pixel {
            x,
            y,
            color,
            palette,
            bg_prio,
        }
    }
}

/// Representation of a Sprite.
#[derive(Copy, Clone)]
struct Sprite {
    /// X position of the top-left pixel of this sprite in the LCD.
    pub x: u8,
    /// Y position of the top-left pixel of this sprite in the LCD.
    pub y: u8,
    /// Tile number. Sprites always use the $8000 addressing method.
    pub tile: u8,
    /// Flags.
    /// - 0: OBJ-to-BG priority.
    ///   - 0 (false): sprite rendered above bg.
    ///   - 1 (true): BG colors 1-3 overlay sprite, but sprite renders over 0.
    /// - 1: Y-flip.
    /// - 2: X-flip.
    /// - 3: Palette (false: OBP0, true: OBP1).
    pub flags: u8,
}

impl Sprite {
    fn new(x: u8, y: u8, tile: u8, flags: u8) -> Self {
        Sprite { x, y, tile, flags }
    }

    /// OBJ-to-BG priority.
    ///   - 0 (false): sprite rendered above bg.
    ///   - 1 (true): BG colors 1-3 overlay sprite, but sprite renders over 0.
    fn priority(&self) -> bool {
        self.flags & 0x80 > 0
    }

    /// Y-flip.
    fn x_flip(&self) -> bool {
        self.flags & 0x40 > 0
    }
    /// X-flip.
    fn y_flip(&self) -> bool {
        self.flags & 0x20 > 0
    }
    /// Palette (false: OBP0, true: OBP1).
    fn palette(&self) -> bool {
        self.flags & 0x10 > 0
    }
}
